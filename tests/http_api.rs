mod http_helpers;

use axum::http::StatusCode;
use http_helpers::{read_json, ErrorBody, RobotStatusBody, TestApp};
use robot_vacuum_firmware::domain::SensorSnapshot;
use serde_json::json;

#[tokio::test]
async fn status_returns_http_200_and_initial_standby_state() {
    let app = TestApp::new();

    let (status, body) = app.get_status().await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.state, "STANDBY");
    assert_eq!(body.cleaning_mode, "AUTO");
    assert_eq!(body.battery_percent, 80);
}

#[tokio::test]
async fn status_includes_required_telemetry_fields() {
    let app = TestApp::new();

    let (_, body) = app.get_status().await;

    assert!(!body.is_charging);
    assert!(!body.suction_enabled);
    assert!(!body.brushes_enabled);
    assert_eq!(body.left_wheel_speed, 0);
    assert_eq!(body.right_wheel_speed, 0);
    assert!(body.current_error.is_none());
    assert!(!body.sensors.drop_off_detected);
    assert!(!body.sensors.bumper_pressed);
    assert!(!body.sensors.top_cover_open);
}

#[tokio::test]
async fn status_returns_http_200_in_error_and_does_not_mutate_state() {
    let app = TestApp::new();
    app.post_empty("/commands/start").await;
    app.set_sensors(SensorSnapshot {
        drop_off_detected: true,
        ..SensorSnapshot::default()
    })
    .await;
    app.tick().await;

    let (first_status_code, first_body) = app.get_status().await;
    let (second_status_code, second_body) = app.get_status().await;

    assert_eq!(first_status_code, StatusCode::OK);
    assert_eq!(second_status_code, StatusCode::OK);
    assert_eq!(first_body.state, "ERROR");
    assert_eq!(first_body.current_error.as_deref(), Some("DROP_OFF_DETECTED"));
    assert_eq!(first_body, second_body);
}

#[tokio::test]
async fn start_from_standby_returns_http_200_and_enables_cleaning_outputs() {
    let app = TestApp::new();

    let response = app.post_empty("/commands/start").await;
    let status = response.status();
    let body: RobotStatusBody = read_json(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.state, "CLEANING");
    assert!(body.suction_enabled);
    assert!(body.brushes_enabled);
    assert_ne!(body.left_wheel_speed, 0);
    assert_ne!(body.right_wheel_speed, 0);
}

#[tokio::test]
async fn start_while_already_cleaning_returns_http_409_without_state_mutation() {
    let app = TestApp::new();
    app.post_empty("/commands/start").await;
    let (_, before) = app.get_status().await;

    let response = app.post_empty("/commands/start").await;
    let status = response.status();
    let error: ErrorBody = read_json(response).await;
    let (_, after) = app.get_status().await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error.code, "COMMAND_NOT_ALLOWED");
    assert_eq!(before, after);
}

#[tokio::test]
async fn start_with_low_battery_returns_http_409() {
    let app = TestApp::with_battery(15);

    let response = app.post_empty("/commands/start").await;
    let status = response.status();
    let error: ErrorBody = read_json(response).await;
    let (_, body) = app.get_status().await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error.code, "BATTERY_TOO_LOW");
    assert_eq!(body.state, "STANDBY");
}

#[tokio::test]
async fn stop_while_cleaning_returns_http_200_and_standby() {
    let app = TestApp::new();
    app.post_empty("/commands/start").await;

    let response = app.post_empty("/commands/stop").await;
    let status = response.status();
    let body: RobotStatusBody = read_json(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.state, "STANDBY");
    assert_eq!(body.left_wheel_speed, 0);
    assert_eq!(body.right_wheel_speed, 0);
    assert!(!body.suction_enabled);
    assert!(!body.brushes_enabled);
}

#[tokio::test]
async fn stop_from_standby_returns_http_409() {
    let app = TestApp::new();

    let response = app.post_empty("/commands/stop").await;
    let status = response.status();
    let error: ErrorBody = read_json(response).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error.code, "COMMAND_NOT_ALLOWED");
}

#[tokio::test]
async fn pause_while_cleaning_returns_http_200_and_paused() {
    let app = TestApp::new();
    app.post_empty("/commands/start").await;

    let response = app.post_empty("/commands/pause").await;
    let status = response.status();
    let body: RobotStatusBody = read_json(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.state, "PAUSED");
    assert_eq!(body.left_wheel_speed, 0);
    assert_eq!(body.right_wheel_speed, 0);
    assert!(!body.suction_enabled);
    assert!(!body.brushes_enabled);
}

#[tokio::test]
async fn pause_from_invalid_state_returns_http_409_with_structured_error() {
    let app = TestApp::new();

    let response = app.post_empty("/commands/pause").await;
    let status = response.status();
    let error: ErrorBody = read_json(response).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error.code, "COMMAND_NOT_ALLOWED");
    assert!(!error.message.is_empty());
}

#[tokio::test]
async fn return_to_dock_while_cleaning_returns_http_200() {
    let app = TestApp::new();
    app.post_empty("/commands/start").await;

    let response = app.post_empty("/commands/return-to-dock").await;
    let status = response.status();
    let body: RobotStatusBody = read_json(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.state, "RETURNING_TO_DOCK");
    assert!(!body.suction_enabled);
    assert!(!body.brushes_enabled);
    assert_eq!(body.left_wheel_speed, 30);
    assert_eq!(body.right_wheel_speed, 30);
}

#[tokio::test]
async fn return_to_dock_from_standby_returns_http_200() {
    let app = TestApp::new();

    let response = app.post_empty("/commands/return-to-dock").await;
    let status = response.status();
    let body: RobotStatusBody = read_json(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.state, "RETURNING_TO_DOCK");
}

#[tokio::test]
async fn manual_move_forward_returns_http_200_and_positive_wheel_speeds() {
    let app = TestApp::new();

    let response = app
        .post_json(
            "/commands/manual-move",
            json!({"direction":"FORWARD","speed":40,"duration_ms":1500}),
        )
        .await;
    let status = response.status();
    let body: RobotStatusBody = read_json(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.state, "MANUAL_CONTROL");
    assert!(body.left_wheel_speed > 0);
    assert!(body.right_wheel_speed > 0);
}

#[tokio::test]
async fn manual_move_backward_returns_negative_wheel_speeds() {
    let app = TestApp::new();

    let response = app
        .post_json(
            "/commands/manual-move",
            json!({"direction":"BACKWARD","speed":40,"duration_ms":1500}),
        )
        .await;
    let body: RobotStatusBody = read_json(response).await;

    assert!(body.left_wheel_speed < 0);
    assert!(body.right_wheel_speed < 0);
}

#[tokio::test]
async fn manual_move_left_and_right_turn_as_expected() {
    let app_left = TestApp::new();
    let left_response = app_left
        .post_json(
            "/commands/manual-move",
            json!({"direction":"LEFT","speed":40,"duration_ms":1500}),
        )
        .await;
    let left_body: RobotStatusBody = read_json(left_response).await;
    assert!(left_body.left_wheel_speed < 0);
    assert!(left_body.right_wheel_speed > 0);

    let app_right = TestApp::new();
    let right_response = app_right
        .post_json(
            "/commands/manual-move",
            json!({"direction":"RIGHT","speed":40,"duration_ms":1500}),
        )
        .await;
    let right_body: RobotStatusBody = read_json(right_response).await;
    assert!(right_body.left_wheel_speed > 0);
    assert!(right_body.right_wheel_speed < 0);
}

#[tokio::test]
async fn manual_move_stop_sets_zero_wheel_speeds() {
    let app = TestApp::new();

    let response = app
        .post_json(
            "/commands/manual-move",
            json!({"direction":"STOP","speed":0,"duration_ms":1500}),
        )
        .await;
    let status = response.status();
    let body: RobotStatusBody = read_json(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.left_wheel_speed, 0);
    assert_eq!(body.right_wheel_speed, 0);
}

#[tokio::test]
async fn manual_move_while_charging_returns_http_409() {
    let app = TestApp::new();
    app.post_empty("/commands/start").await;
    app.set_battery_percent(15).await;
    app.tick().await;
    app.set_dock_detected(true).await;
    app.tick().await;

    let response = app
        .post_json(
            "/commands/manual-move",
            json!({"direction":"FORWARD","speed":40,"duration_ms":1500}),
        )
        .await;
    let status = response.status();
    let error: ErrorBody = read_json(response).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error.code, "COMMAND_NOT_ALLOWED");
}

#[tokio::test]
async fn manual_move_while_error_returns_http_409() {
    let app = TestApp::new();
    app.post_empty("/commands/start").await;
    app.set_sensors(SensorSnapshot {
        drop_off_detected: true,
        ..SensorSnapshot::default()
    })
    .await;
    app.tick().await;
    app.set_sensors(SensorSnapshot::default()).await;

    let response = app
        .post_json(
            "/commands/manual-move",
            json!({"direction":"FORWARD","speed":40,"duration_ms":1500}),
        )
        .await;
    let status = response.status();
    let error: ErrorBody = read_json(response).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error.code, "COMMAND_NOT_ALLOWED");
}

#[tokio::test]
async fn manual_move_validation_errors_return_http_400() {
    let app = TestApp::new();

    let invalid_requests = [
        json!({"speed":40,"duration_ms":1500}),
        json!({"direction":"FORWARD","duration_ms":1500}),
        json!({"direction":"FORWARD","speed":40}),
        json!({"direction":"FORWARD","speed":40,"duration_ms":0}),
        json!({"direction":"FORWARD","speed":101,"duration_ms":1500}),
        json!({"direction":"FORWARD","speed":0,"duration_ms":1500}),
    ];

    for payload in invalid_requests {
        let response = app.post_json("/commands/manual-move", payload).await;
        let status = response.status();
        let error: ErrorBody = read_json(response).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert!(!error.code.is_empty());
        assert!(!error.message.is_empty());
    }
}

#[tokio::test]
async fn invalid_direction_and_malformed_json_return_http_400() {
    let app = TestApp::new();

    let invalid_direction = app
        .post_json(
            "/commands/manual-move",
            json!({"direction":"UPSIDE_DOWN","speed":40,"duration_ms":1500}),
        )
        .await;
    let invalid_direction_status = invalid_direction.status();
    let invalid_direction_body: ErrorBody = read_json(invalid_direction).await;
    assert_eq!(invalid_direction_status, StatusCode::BAD_REQUEST);
    assert!(!invalid_direction_body.code.is_empty());
    assert!(!invalid_direction_body.message.is_empty());

    let malformed = app.post_raw_json("/commands/manual-move", "{").await;
    let malformed_status = malformed.status();
    let malformed_body: ErrorBody = read_json(malformed).await;
    assert_eq!(malformed_status, StatusCode::BAD_REQUEST);
    assert!(!malformed_body.code.is_empty());
    assert!(!malformed_body.message.is_empty());
}

#[tokio::test]
async fn mode_endpoint_accepts_auto_and_rejects_unsupported_modes_without_mutation() {
    let app = TestApp::new();
    let (_, before) = app.get_status().await;

    let auto_response = app.post_json("/commands/mode", json!({"mode":"AUTO"})).await;
    let auto_status = auto_response.status();
    let auto_body: RobotStatusBody = read_json(auto_response).await;
    assert_eq!(auto_status, StatusCode::OK);
    assert_eq!(auto_body.cleaning_mode, "AUTO");

    for mode in ["ZIGZAG", "WALL_FOLLOWING", "SPOT"] {
        let response = app.post_json("/commands/mode", json!({ "mode": mode })).await;
        let status = response.status();
        let error: ErrorBody = read_json(response).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(error.code, "UNSUPPORTED_MODE");
    }

    let (_, after) = app.get_status().await;
    assert_eq!(before.state, after.state);
    assert_eq!(after.cleaning_mode, "AUTO");
}

#[tokio::test]
async fn clear_error_success_and_non_error_conflict_are_visible_over_http() {
    let app = TestApp::new();
    app.post_empty("/commands/start").await;
    app.set_sensors(SensorSnapshot {
        drop_off_detected: true,
        ..SensorSnapshot::default()
    })
    .await;
    app.tick().await;
    app.set_sensors(SensorSnapshot::default()).await;

    let success = app.post_empty("/commands/clear-error").await;
    let success_status = success.status();
    let success_body: RobotStatusBody = read_json(success).await;
    assert_eq!(success_status, StatusCode::OK);
    assert_eq!(success_body.state, "STANDBY");

    let conflict = app.post_empty("/commands/clear-error").await;
    let conflict_status = conflict.status();
    let conflict_body: ErrorBody = read_json(conflict).await;
    assert_eq!(conflict_status, StatusCode::CONFLICT);
    assert_eq!(conflict_body.code, "COMMAND_NOT_ALLOWED");
}

#[tokio::test]
async fn clear_error_while_safety_condition_is_still_active_returns_http_409() {
    let app = TestApp::new();
    app.post_empty("/commands/start").await;
    app.set_sensors(SensorSnapshot {
        drop_off_detected: true,
        ..SensorSnapshot::default()
    })
    .await;
    app.tick().await;

    let response = app.post_empty("/commands/clear-error").await;
    let status = response.status();
    let error: ErrorBody = read_json(response).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error.code, "SAFETY_CONDITION_ACTIVE");
}

#[tokio::test]
async fn safety_conditions_are_visible_through_http_status_and_stop_actuators() {
    let cases = [
        (
            SensorSnapshot {
                drop_off_detected: true,
                ..SensorSnapshot::default()
            },
            "DROP_OFF_DETECTED",
        ),
        (
            SensorSnapshot {
                wheel_stuck: true,
                ..SensorSnapshot::default()
            },
            "WHEEL_STUCK",
        ),
        (
            SensorSnapshot {
                brush_stuck: true,
                ..SensorSnapshot::default()
            },
            "BRUSH_STUCK",
        ),
        (
            SensorSnapshot {
                top_cover_open: true,
                ..SensorSnapshot::default()
            },
            "TOP_COVER_OPEN",
        ),
        (
            SensorSnapshot {
                dust_container_full: true,
                ..SensorSnapshot::default()
            },
            "DUST_CONTAINER_FULL",
        ),
    ];

    for (snapshot, expected_error) in cases {
        let app = TestApp::new();
        app.post_empty("/commands/start").await;
        app.set_sensors(snapshot).await;
        app.tick().await;

        let (status, body) = app.get_status().await;

        assert_eq!(status, StatusCode::OK);
        assert_eq!(body.state, "ERROR");
        assert_eq!(body.current_error.as_deref(), Some(expected_error));
        assert_eq!(body.left_wheel_speed, 0);
        assert_eq!(body.right_wheel_speed, 0);
        assert!(!body.suction_enabled);
        assert!(!body.brushes_enabled);
    }
}

#[tokio::test]
async fn battery_and_docking_transitions_are_visible_through_http_status() {
    let app = TestApp::new();
    app.post_empty("/commands/start").await;

    let (_, cleaning) = app.get_status().await;
    assert_eq!(cleaning.state, "CLEANING");

    app.set_battery_percent(15).await;
    app.tick().await;
    let (_, returning) = app.get_status().await;
    assert_eq!(returning.state, "RETURNING_TO_DOCK");

    app.set_dock_detected(true).await;
    app.tick().await;
    let (_, charging) = app.get_status().await;
    assert_eq!(charging.state, "CHARGING");

    app.set_battery_percent(100).await;
    app.tick().await;
    let (_, standby) = app.get_status().await;
    assert_eq!(standby.state, "STANDBY");
}

#[tokio::test]
async fn error_responses_include_code_and_message() {
    let app = TestApp::new();

    let bad_request = app
        .post_json(
            "/commands/manual-move",
            json!({"direction":"FORWARD","speed":0,"duration_ms":1500}),
        )
        .await;
    let bad_request_status = bad_request.status();
    let bad_request_body: ErrorBody = read_json(bad_request).await;
    assert_eq!(bad_request_status, StatusCode::BAD_REQUEST);
    assert!(!bad_request_body.code.is_empty());
    assert!(!bad_request_body.message.is_empty());

    let conflict = app.post_empty("/commands/pause").await;
    let conflict_status = conflict.status();
    let conflict_body: ErrorBody = read_json(conflict).await;
    assert_eq!(conflict_status, StatusCode::CONFLICT);
    assert!(!conflict_body.code.is_empty());
    assert!(!conflict_body.message.is_empty());
}
