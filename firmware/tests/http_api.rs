#![recursion_limit = "256"]

mod http_helpers;

use axum::http::StatusCode;
use http_helpers::{
    content_type, read_json, read_text, ErrorBody, RobotStatusBody, SavedPathBody, TestApp,
};
use robot_vacuum_firmware::domain::SensorSnapshot;
use serde_json::json;
use std::fs;

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
    assert_eq!(
        first_body.current_error.as_deref(),
        Some("DROP_OFF_DETECTED")
    );
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
    assert_eq!(error.current_state.as_deref(), Some("STANDBY"));
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
async fn return_to_dock_from_paused_and_manual_control_returns_http_200() {
    let paused_app = TestApp::new();
    paused_app.post_empty("/commands/start").await;
    paused_app.post_empty("/commands/pause").await;
    let paused_response = paused_app.post_empty("/commands/return-to-dock").await;
    let paused_status = paused_response.status();
    let paused_body: RobotStatusBody = read_json(paused_response).await;
    assert_eq!(paused_status, StatusCode::OK);
    assert_eq!(paused_body.state, "RETURNING_TO_DOCK");

    let manual_app = TestApp::new();
    manual_app
        .post_json(
            "/commands/manual-move",
            json!({"direction":"FORWARD","speed":40,"duration_ms":1500}),
        )
        .await;
    let manual_response = manual_app.post_empty("/commands/return-to-dock").await;
    let manual_status = manual_response.status();
    let manual_body: RobotStatusBody = read_json(manual_response).await;
    assert_eq!(manual_status, StatusCode::OK);
    assert_eq!(manual_body.state, "RETURNING_TO_DOCK");
}

#[tokio::test]
async fn return_to_dock_from_charging_returns_clear_http_409_error() {
    let app = TestApp::new();
    app.post_empty("/commands/start").await;
    app.post_empty("/commands/return-to-dock").await;
    app.set_dock_detected(true).await;
    app.tick().await;

    let response = app.post_empty("/commands/return-to-dock").await;
    let status = response.status();
    let error: ErrorBody = read_json(response).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(error.code, "COMMAND_NOT_ALLOWED");
    assert_eq!(error.current_state.as_deref(), Some("CHARGING"));
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

    let auto_response = app
        .post_json("/commands/mode", json!({"mode":"AUTO"}))
        .await;
    let auto_status = auto_response.status();
    let auto_body: RobotStatusBody = read_json(auto_response).await;
    assert_eq!(auto_status, StatusCode::OK);
    assert_eq!(auto_body.cleaning_mode, "AUTO");

    for mode in ["ZIGZAG", "WALL_FOLLOWING", "SPOT"] {
        let response = app
            .post_json("/commands/mode", json!({ "mode": mode }))
            .await;
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
    assert_eq!(charging.left_wheel_speed, 0);
    assert_eq!(charging.right_wheel_speed, 0);

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

#[tokio::test]
async fn simulation_sensors_endpoint_updates_status_and_preserves_unspecified_fields() {
    let app = TestApp::new();

    let first = app
        .post_json(
            "/simulation/sensors",
            json!({"obstacle_detected": true, "wheel_stuck": true}),
        )
        .await;
    let first_status = first.status();
    let first_body: RobotStatusBody = read_json(first).await;
    assert_eq!(first_status, StatusCode::OK);
    assert!(first_body.sensors.obstacle_detected);
    assert!(first_body.sensors.wheel_stuck);
    assert!(!first_body.sensors.bumper_pressed);
    assert!(!first_body.sensors.proximity_contact);

    let second = app
        .post_json("/simulation/sensors", json!({"bumper_pressed": true}))
        .await;
    let second_body: RobotStatusBody = read_json(second).await;
    assert!(second_body.sensors.obstacle_detected);
    assert!(second_body.sensors.wheel_stuck);
    assert!(second_body.sensors.bumper_pressed);
    assert!(second_body.sensors.proximity_contact);
}

#[tokio::test]
async fn simulation_sensor_clear_and_tick_restore_auto_forward_motion() {
    let app = TestApp::new();
    app.post_empty("/commands/start").await;

    let detected = app
        .post_json(
            "/simulation/sensors",
            json!({"proximity_contact": true, "contact_type": "wall", "wall_side": "left"}),
        )
        .await;
    let detected_status = detected.status();
    let detected_body: RobotStatusBody = read_json(detected).await;
    assert_eq!(detected_status, StatusCode::OK);
    assert!(detected_body.sensors.proximity_contact);
    assert_eq!(
        detected_body.auto_navigation_phase.as_deref(),
        Some("ROOM_CROSSING")
    );

    let avoidance_tick = app
        .post_json("/simulation/tick", json!({"delta_ms": 100}))
        .await;
    let avoidance_status = avoidance_tick.status();
    let avoidance_body: RobotStatusBody = read_json(avoidance_tick).await;
    assert_eq!(avoidance_status, StatusCode::OK);
    assert_eq!(avoidance_body.state, "CLEANING");
    assert_eq!(avoidance_body.left_wheel_speed, -24);
    assert_eq!(avoidance_body.right_wheel_speed, -24);
    assert_eq!(
        avoidance_body.auto_navigation_phase.as_deref(),
        Some("CLEARANCE_BACKUP")
    );

    let cleared = app
        .post_json(
            "/simulation/sensors",
            json!({"proximity_contact": false, "bumper_pressed": false}),
        )
        .await;
    let cleared_status = cleared.status();
    let cleared_body: RobotStatusBody = read_json(cleared).await;
    assert_eq!(cleared_status, StatusCode::OK);
    assert!(!cleared_body.sensors.proximity_contact);
    assert!(!cleared_body.sensors.bumper_pressed);

    let recovery_tick = app
        .post_json("/simulation/tick", json!({"delta_ms": 901}))
        .await;
    let recovery_status = recovery_tick.status();
    let recovery_body: RobotStatusBody = read_json(recovery_tick).await;
    assert_eq!(recovery_status, StatusCode::OK);
    assert_eq!(recovery_body.state, "CLEANING");
    assert_eq!(
        recovery_body.auto_navigation_phase.as_deref(),
        Some("CLEARANCE_TURN_CLEAR")
    );
}

#[tokio::test]
async fn simulation_sensors_reject_invalid_contact_type() {
    let app = TestApp::new();

    let response = app
        .post_json("/simulation/sensors", json!({"contact_type": "bad-value"}))
        .await;
    let status = response.status();
    let body: ErrorBody = read_json(response).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body.code, "INVALID_CONTACT_TYPE");
}

#[tokio::test]
async fn simulation_battery_endpoint_updates_status() {
    let app = TestApp::new();

    let response = app
        .post_json("/simulation/battery", json!({"battery_percent": 42}))
        .await;
    let status = response.status();
    let body: RobotStatusBody = read_json(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.battery_percent, 42);
}

#[tokio::test]
async fn simulation_docking_endpoint_updates_status_and_supports_charging_transition() {
    let app = TestApp::new();

    let first = app
        .post_json("/simulation/docking", json!({"dock_available": false}))
        .await;
    let first_status = first.status();
    let first_body: RobotStatusBody = read_json(first).await;
    assert_eq!(first_status, StatusCode::OK);
    assert_eq!(first_body.state, "STANDBY");

    let second = app
        .post_json("/simulation/docking", json!({"dock_available": true}))
        .await;
    let second_status = second.status();
    let second_body: RobotStatusBody = read_json(second).await;
    assert_eq!(second_status, StatusCode::OK);
    assert_eq!(second_body.state, "STANDBY");

    app.post_empty("/commands/start").await;
    app.post_empty("/commands/return-to-dock").await;

    let docking_update = app
        .post_json("/simulation/docking", json!({"dock_detected": true}))
        .await;
    let docking_status = docking_update.status();
    let docking_body: RobotStatusBody = read_json(docking_update).await;
    assert_eq!(docking_status, StatusCode::OK);
    assert_eq!(docking_body.state, "RETURNING_TO_DOCK");

    let tick = app
        .post_json("/simulation/tick", json!({"delta_ms": 100}))
        .await;
    let tick_status = tick.status();
    let tick_body: RobotStatusBody = read_json(tick).await;
    assert_eq!(tick_status, StatusCode::OK);
    assert_eq!(tick_body.state, "CHARGING");
    assert_eq!(tick_body.left_wheel_speed, 0);
    assert_eq!(tick_body.right_wheel_speed, 0);
}

#[tokio::test]
async fn simulation_tick_returns_robot_status() {
    let app = TestApp::new();
    app.post_empty("/commands/start").await;

    let response = app.post_empty("/simulation/tick").await;
    let status = response.status();
    let body: RobotStatusBody = read_json(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.state, "CLEANING");
}

#[tokio::test]
async fn simulation_tick_with_delta_ms_advances_simulated_time() {
    let app = TestApp::new();
    app.post_json(
        "/commands/manual-move",
        json!({"direction":"FORWARD","speed":40,"duration_ms":150}),
    )
    .await;

    let response = app
        .post_json("/simulation/tick", json!({"delta_ms": 200}))
        .await;
    let status = response.status();
    let body: RobotStatusBody = read_json(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.state, "STANDBY");
    assert_eq!(body.left_wheel_speed, 0);
    assert_eq!(body.right_wheel_speed, 0);
}

#[tokio::test]
async fn simulation_tick_rejects_invalid_delta_ms() {
    let app = TestApp::new();

    for payload in [json!({"delta_ms": 0}), json!({"delta_ms": 1001})] {
        let response = app.post_json("/simulation/tick", payload).await;
        let status = response.status();
        let body: ErrorBody = read_json(response).await;

        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(body.code, "INVALID_TICK_DELTA");
    }
}

#[tokio::test]
async fn simulation_tick_allows_manual_move_to_expire_via_http() {
    let app = TestApp::new();
    app.post_json(
        "/commands/manual-move",
        json!({"direction":"FORWARD","speed":45,"duration_ms":250}),
    )
    .await;

    let response = app
        .post_json("/simulation/tick", json!({"delta_ms": 300}))
        .await;
    let status = response.status();
    let body: RobotStatusBody = read_json(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.state, "STANDBY");
    assert_eq!(body.left_wheel_speed, 0);
    assert_eq!(body.right_wheel_speed, 0);
}

#[tokio::test]
async fn simulation_reset_returns_initial_standby_status() {
    let app = TestApp::new();
    app.post_empty("/commands/start").await;
    app.post_json("/simulation/battery", json!({"battery_percent": 42}))
        .await;

    let response = app.post_empty("/simulation/reset").await;
    let status = response.status();
    let body: RobotStatusBody = read_json(response).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.state, "STANDBY");
    assert_eq!(body.cleaning_mode, "AUTO");
    assert_eq!(body.battery_percent, 80);
    assert_eq!(body.left_wheel_speed, 0);
    assert_eq!(body.right_wheel_speed, 0);
}

#[tokio::test]
async fn simulation_log_dump_persists_valid_payload() {
    let app = TestApp::new();
    let payload = json!({
        "session_id": "demo-map-20260628-01",
        "map_id": "map-001",
        "created_at": "2026-06-28T15:00:00Z",
        "map": {
            "room_width": 860,
            "room_height": 560,
            "robot_radius": 29,
            "initial_robot_pose": { "x": 120.0, "y": 120.0, "heading": 0.0 },
            "docking_station": { "x": 84.0, "y": 523.0 },
            "obstacles": [
                { "id": "obstacle-1", "x": 220.0, "y": 140.0, "width": 120.0, "height": 80.0 }
            ]
        },
        "frontend_config": {
            "simulation_speed": 5.0,
            "sensor_thresholds": {
                "obstacle_distance": 42.0,
                "obstacle_half_angle_rad": 0.45,
                "bumper_contact_distance": 0.0
            },
            "coverage_grid": {
                "cols": 40,
                "rows": 28,
                "cell_width": 21.5,
                "cell_height": 20.0
            }
        },
        "timeline": [
            {
                "frame_index": 0,
                "timestamp_ms": 0,
                "x": 120.0,
                "y": 120.0,
                "heading": 0.0,
                "left_wheel_speed": 60,
                "right_wheel_speed": 60,
                "state": "CLEANING",
                "cleaning_mode": "AUTO",
                "current_error": null,
                "battery_percent": 80,
                "is_charging": false,
                "sensors": {
                    "obstacle_detected": false,
                    "drop_off_detected": false,
                    "bumper_pressed": false,
                    "dust_container_full": false,
                    "wheel_stuck": false,
                    "brush_stuck": false,
                    "top_cover_open": false
                },
                "coverage_percentage": 1.2,
                "event": "BUMPER_CONTACT",
                "event_detail": {
                    "collision_target": "obstacle",
                    "obstacle_id": "obstacle-1"
                }
            }
        ],
        "coverage": {
            "covered_cells": [{ "col": 1, "row": 1 }],
            "coverage_percentage": 1.2
        },
        "summary": {
            "total_frames": 1,
            "total_simulated_time_ms": 0,
            "collisions": 0,
            "obstacle_detections": 0,
            "turns": 0,
            "coverage_percentage": 1.2
        }
    });

    let response = app.post_json("/simulation/log-dump", payload).await;
    let status = response.status();
    assert_eq!(status, StatusCode::OK);
    let body: SavedPathBody = read_json(response).await;

    assert_eq!(body.saved_path, "demo-logs/demo-map-20260628-01/log.json");
    assert!(fs::metadata(app.log_root().join("demo-map-20260628-01").join("log.json")).is_ok());
}

#[tokio::test]
async fn simulation_log_dump_rejects_invalid_payload() {
    let app = TestApp::new();
    let response = app
        .post_json(
            "/simulation/log-dump",
            json!({
                "session_id": "",
                "created_at": "",
                "map": { "room_width": 0, "room_height": 0, "robot_radius": 0, "initial_robot_pose": { "x": 0.0, "y": 0.0, "heading": 0.0 }, "docking_station": { "x": 0.0, "y": 0.0 }, "obstacles": [] },
                "frontend_config": {
                    "simulation_speed": 0.0,
                    "sensor_thresholds": { "obstacle_distance": 0.0, "obstacle_half_angle_rad": 0.0, "bumper_contact_distance": 0.0 },
                    "coverage_grid": { "cols": 0, "rows": 0, "cell_width": 0.0, "cell_height": 0.0 }
                },
                "timeline": [],
                "coverage": { "covered_cells": [], "coverage_percentage": 0.0 },
                "summary": { "total_frames": 0, "total_simulated_time_ms": 0, "collisions": 0, "obstacle_detections": 0, "turns": 0, "coverage_percentage": 0.0 }
            }),
        )
        .await;
    let status = response.status();
    let error: ErrorBody = read_json(response).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error.code, "INVALID_LOG_DUMP");
}

#[tokio::test]
async fn simulation_batch_summary_persists_summary_object_under_batch_folder() {
    let app = TestApp::new();
    let payload = json!({
        "batch_id": "batch-20260628-001",
        "summary": {
            "batch_id": "batch-20260628-001",
            "created_at": "2026-06-28T15:00:00Z",
            "finished_at": "2026-06-28T15:10:00Z",
            "requested_runs": 2,
            "completed_runs": 2,
            "cancelled": false,
            "config": { "run_count": 2 },
            "runs": [],
            "aggregate_metrics": {
                "average_coverage": 12.5,
                "min_coverage": 10.0,
                "max_coverage": 15.0,
                "average_simulated_time": 12000,
                "docking_success_rate": null,
                "most_common_finish_reason": "TIME_LIMIT_REACHED",
                "total_bumper_contacts": 1,
                "total_wall_contacts": 1,
                "total_obstacle_contacts": 0,
                "total_lane_starts": 2,
                "total_lane_ends": 2,
                "total_lane_shifts": 1,
                "total_lane_blocked_events": 0,
                "total_obstacle_bypass_attempts": 0,
                "total_obstacle_bypass_failures": 0,
                "total_pattern_recoveries": 0,
                "total_long_turn_guards": 0,
                "total_escape_turns": 0,
                "total_stuck_diagnostics": 0,
                "total_dock_blocked_events": 0,
                "total_dock_stuck_diagnostics": 0
            },
            "worst_runs": {
                "lowest_coverage": [],
                "highest_wall_contacts": [],
                "highest_obstacle_contacts": [],
                "most_long_turn_guards": [],
                "most_stuck_diagnostics": [],
                "failed_docking": [],
                "longest_no_movement_streak": [],
                "longest_turning_without_movement_streak": []
            }
        }
    });

    let response = app.post_json("/simulation/batch-summary", payload).await;
    let status = response.status();
    assert_eq!(status, StatusCode::OK);
    let body: SavedPathBody = read_json(response).await;
    assert_eq!(
        body.saved_path,
        "demo-logs/batch-20260628-001/batch-summary.json"
    );

    let persisted_path = app
        .log_root()
        .join("batch-20260628-001")
        .join("batch-summary.json");
    let persisted: serde_json::Value =
        serde_json::from_slice(&fs::read(&persisted_path).expect("summary file should exist"))
            .expect("summary file should parse");
    assert!(persisted.get("summary").is_none());
    assert_eq!(persisted["batch_id"], "batch-20260628-001");
}

#[tokio::test]
async fn simulation_batch_summary_rejects_missing_or_mismatched_batch_ids() {
    let app = TestApp::new();

    for payload in [
        json!({
            "batch_id": "batch-20260628-001",
            "summary": {
                "batch_id": "batch-20260628-999",
                "created_at": "2026-06-28T15:00:00Z",
                "finished_at": "2026-06-28T15:10:00Z",
                "requested_runs": 1,
                "completed_runs": 1,
                "cancelled": false,
                "config": {},
                "runs": [],
                "aggregate_metrics": {
                    "average_coverage": 0.0,
                    "min_coverage": 0.0,
                    "max_coverage": 0.0,
                    "average_simulated_time": 0,
                    "docking_success_rate": null,
                    "most_common_finish_reason": null,
                    "total_bumper_contacts": 0,
                    "total_wall_contacts": 0,
                    "total_obstacle_contacts": 0,
                    "total_lane_starts": 0,
                    "total_lane_ends": 0,
                    "total_lane_shifts": 0,
                    "total_lane_blocked_events": 0,
                    "total_obstacle_bypass_attempts": 0,
                    "total_obstacle_bypass_failures": 0,
                    "total_pattern_recoveries": 0,
                    "total_long_turn_guards": 0,
                    "total_escape_turns": 0,
                    "total_stuck_diagnostics": 0,
                    "total_dock_blocked_events": 0,
                    "total_dock_stuck_diagnostics": 0
                },
                "worst_runs": {
                    "lowest_coverage": [],
                    "highest_wall_contacts": [],
                    "highest_obstacle_contacts": [],
                    "most_long_turn_guards": [],
                    "most_stuck_diagnostics": [],
                    "failed_docking": [],
                    "longest_no_movement_streak": [],
                    "longest_turning_without_movement_streak": []
                }
            }
        }),
        json!({
            "batch_id": "",
            "summary": {
                "batch_id": "",
                "created_at": "2026-06-28T15:00:00Z",
                "finished_at": "2026-06-28T15:10:00Z",
                "requested_runs": 1,
                "completed_runs": 1,
                "cancelled": false,
                "config": {},
                "runs": [],
                "aggregate_metrics": {
                    "average_coverage": 0.0,
                    "min_coverage": 0.0,
                    "max_coverage": 0.0,
                    "average_simulated_time": 0,
                    "docking_success_rate": null,
                    "most_common_finish_reason": null,
                    "total_bumper_contacts": 0,
                    "total_wall_contacts": 0,
                    "total_obstacle_contacts": 0,
                    "total_lane_starts": 0,
                    "total_lane_ends": 0,
                    "total_lane_shifts": 0,
                    "total_lane_blocked_events": 0,
                    "total_obstacle_bypass_attempts": 0,
                    "total_obstacle_bypass_failures": 0,
                    "total_pattern_recoveries": 0,
                    "total_long_turn_guards": 0,
                    "total_escape_turns": 0,
                    "total_stuck_diagnostics": 0,
                    "total_dock_blocked_events": 0,
                    "total_dock_stuck_diagnostics": 0
                },
                "worst_runs": {
                    "lowest_coverage": [],
                    "highest_wall_contacts": [],
                    "highest_obstacle_contacts": [],
                    "most_long_turn_guards": [],
                    "most_stuck_diagnostics": [],
                    "failed_docking": [],
                    "longest_no_movement_streak": [],
                    "longest_turning_without_movement_streak": []
                }
            }
        }),
    ] {
        let response = app.post_json("/simulation/batch-summary", payload).await;
        let status = response.status();
        let error: ErrorBody = read_json(response).await;
        assert_eq!(status, StatusCode::BAD_REQUEST);
        assert_eq!(error.code, "INVALID_BATCH_SUMMARY");
    }
}

#[tokio::test]
async fn simulation_batch_log_dump_persists_under_batch_folder_and_uses_saved_path() {
    let app = TestApp::new();
    let payload = json!({
        "batch_id": "batch-20260628-001",
        "session_id": "demo-map-20260628-001-run-001",
        "log": {
            "session_id": "demo-map-20260628-001-run-001",
            "map_id": "map-001",
            "created_at": "2026-06-28T15:00:00Z",
            "batch_id": "batch-20260628-001",
            "run_index": 1,
            "batch_total_runs": 3,
            "batch_mode": true,
            "run_config": {
                "max_simulated_time": 12000,
                "target_coverage": 30.0,
                "simulation_speed": 10.0,
                "return_to_dock_after_run": false,
                "enabled_stop_conditions": {
                    "time_limit": true,
                    "coverage_threshold": true,
                    "successful_docking": true,
                    "stuck_diagnostic": true
                },
                "obstacle_count": 4
            },
            "map": {
                "room_width": 860,
                "room_height": 560,
                "robot_radius": 29,
                "initial_robot_pose": { "x": 120.0, "y": 120.0, "heading": 0.0 },
                "docking_station": { "x": 84.0, "y": 523.0 },
                "obstacles": []
            },
            "frontend_config": {
                "simulation_speed": 10.0,
                "sensor_thresholds": {
                    "obstacle_distance": 42.0,
                    "obstacle_half_angle_rad": 0.45,
                    "bumper_contact_distance": 0.0
                },
                "coverage_grid": {
                    "cols": 40,
                    "rows": 28,
                    "cell_width": 21.5,
                    "cell_height": 20.0
                }
            },
            "timeline": [
                {
                    "frame_index": 0,
                    "timestamp_ms": 0,
                    "x": 120.0,
                    "y": 120.0,
                    "heading": 0.0,
                    "left_wheel_speed": 60,
                    "right_wheel_speed": 60,
                    "backend_left_wheel_speed": 60,
                    "backend_right_wheel_speed": 60,
                    "demo_left_wheel_speed": 44,
                    "demo_right_wheel_speed": 44,
                    "demo_navigation_phase": "LANE_DRIVE",
                    "state": "CLEANING",
                    "cleaning_mode": "AUTO",
                    "current_error": null,
                    "battery_percent": 80,
                    "is_charging": false,
                    "sensors": {
                        "obstacle_detected": false,
                        "drop_off_detected": false,
                        "bumper_pressed": false,
                        "dust_container_full": false,
                        "wheel_stuck": false,
                        "brush_stuck": false,
                        "top_cover_open": false
                    },
                    "coverage_percentage": 1.2,
                    "event": "LANE_START",
                    "event_detail": {
                        "demo_navigation_phase": "LANE_DRIVE",
                        "lane_index": 0,
                        "lane_direction": "forward",
                        "lane_heading": 0.0,
                        "lane_shift_side": "down",
                        "lane_spacing_px": 46,
                        "target_heading": 0.0,
                        "blocked_target": null,
                        "contact_type": null,
                        "obstacle_id": null
                    }
                }
            ],
            "coverage": {
                "covered_cells": [{ "col": 1, "row": 1 }],
                "coverage_percentage": 1.2
            },
            "summary": {
                "total_frames": 1,
                "total_simulated_time_ms": 0,
                "collisions": 0,
                "obstacle_detections": 0,
                "turns": 0,
                "coverage_percentage": 1.2
            }
        }
    });

    let response = app.post_json("/simulation/log-dump", payload).await;
    let status = response.status();
    assert_eq!(status, StatusCode::OK);
    let body: SavedPathBody = read_json(response).await;
    assert_eq!(
        body.saved_path,
        "demo-logs/batch-20260628-001/demo-map-20260628-001-run-001/log.json"
    );

    let persisted_path = app
        .log_root()
        .join("batch-20260628-001")
        .join("demo-map-20260628-001-run-001")
        .join("log.json");
    assert!(fs::metadata(&persisted_path).is_ok());
    let persisted: serde_json::Value =
        serde_json::from_slice(&fs::read(&persisted_path).expect("log file should exist"))
            .expect("log file should parse");
    assert_eq!(persisted["batch_mode"], true);
    assert_eq!(
        persisted["timeline"][0]["demo_navigation_phase"],
        "LANE_DRIVE"
    );
    assert!(persisted.get("log").is_none());
}

#[tokio::test]
async fn simulation_batch_log_dump_rejects_mismatched_identifiers() {
    let app = TestApp::new();
    let response = app
        .post_json(
            "/simulation/log-dump",
            json!({
                "batch_id": "batch-20260628-001",
                "session_id": "demo-map-20260628-001-run-001",
                "log": {
                    "session_id": "different-session",
                    "map_id": "map-001",
                    "created_at": "2026-06-28T15:00:00Z",
                    "batch_id": "other-batch",
                    "run_index": 1,
                    "batch_total_runs": 3,
                    "batch_mode": true,
                    "run_config": {
                        "max_simulated_time": 12000,
                        "target_coverage": 30.0,
                        "simulation_speed": 10.0,
                        "return_to_dock_after_run": false,
                        "enabled_stop_conditions": {
                            "time_limit": true,
                            "coverage_threshold": true,
                            "successful_docking": true,
                            "stuck_diagnostic": true
                        }
                    },
                    "map": {
                        "room_width": 860,
                        "room_height": 560,
                        "robot_radius": 29,
                        "initial_robot_pose": { "x": 120.0, "y": 120.0, "heading": 0.0 },
                        "docking_station": { "x": 84.0, "y": 523.0 },
                        "obstacles": []
                    },
                    "frontend_config": {
                        "simulation_speed": 10.0,
                        "sensor_thresholds": {
                            "obstacle_distance": 42.0,
                            "obstacle_half_angle_rad": 0.45,
                            "bumper_contact_distance": 0.0
                        },
                        "coverage_grid": {
                            "cols": 40,
                            "rows": 28,
                            "cell_width": 21.5,
                            "cell_height": 20.0
                        }
                    },
                    "timeline": [{
                        "frame_index": 0,
                        "timestamp_ms": 0,
                        "x": 120.0,
                        "y": 120.0,
                        "heading": 0.0,
                        "left_wheel_speed": 60,
                        "right_wheel_speed": 60,
                        "backend_left_wheel_speed": 60,
                        "backend_right_wheel_speed": 60,
                        "demo_left_wheel_speed": 44,
                        "demo_right_wheel_speed": 44,
                        "demo_navigation_phase": "LANE_DRIVE",
                        "state": "CLEANING",
                        "cleaning_mode": "AUTO",
                        "current_error": null,
                        "battery_percent": 80,
                        "is_charging": false,
                        "sensors": {
                            "obstacle_detected": false,
                            "drop_off_detected": false,
                            "bumper_pressed": false,
                            "dust_container_full": false,
                            "wheel_stuck": false,
                            "brush_stuck": false,
                            "top_cover_open": false
                        },
                        "coverage_percentage": 1.2,
                        "event": "LANE_START",
                        "event_detail": {
                            "demo_navigation_phase": "LANE_DRIVE",
                            "lane_index": 0,
                            "lane_direction": "forward",
                            "lane_heading": 0.0,
                            "lane_shift_side": "down",
                            "lane_spacing_px": 46,
                            "target_heading": 0.0,
                            "blocked_target": null,
                            "contact_type": null,
                            "obstacle_id": null
                        }
                    }],
                    "coverage": { "covered_cells": [{ "col": 1, "row": 1 }], "coverage_percentage": 1.2 },
                    "summary": {
                        "total_frames": 1,
                        "total_simulated_time_ms": 0,
                        "collisions": 0,
                        "obstacle_detections": 0,
                        "turns": 0,
                        "coverage_percentage": 1.2
                    }
                }
            }),
        )
        .await;

    let status = response.status();
    let error: ErrorBody = read_json(response).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error.code, "INVALID_LOG_DUMP");
}

#[tokio::test]
async fn simulation_endpoints_reject_invalid_payloads() {
    let app = TestApp::new();

    let malformed = app.post_raw_json("/simulation/sensors", "{").await;
    let malformed_status = malformed.status();
    let malformed_body: ErrorBody = read_json(malformed).await;
    assert_eq!(malformed_status, StatusCode::BAD_REQUEST);
    assert_eq!(malformed_body.code, "BAD_REQUEST");

    let invalid_shape = app
        .post_json("/simulation/docking", json!({"dock_detected": "yes"}))
        .await;
    let invalid_shape_status = invalid_shape.status();
    let invalid_shape_body: ErrorBody = read_json(invalid_shape).await;
    assert_eq!(invalid_shape_status, StatusCode::BAD_REQUEST);
    assert_eq!(invalid_shape_body.code, "BAD_REQUEST");

    let invalid_battery = app
        .post_json("/simulation/battery", json!({"battery_percent": 101}))
        .await;
    let invalid_battery_status = invalid_battery.status();
    let invalid_battery_body: ErrorBody = read_json(invalid_battery).await;
    assert_eq!(invalid_battery_status, StatusCode::BAD_REQUEST);
    assert_eq!(invalid_battery_body.code, "INVALID_BATTERY_PERCENT");

    let (_, status_body) = app.get_status().await;
    assert_eq!(status_body.battery_percent, 80);
}

#[tokio::test]
async fn demo_and_static_assets_are_served() {
    let app = TestApp::new();

    let demo = app.get("/demo").await;
    let demo_status = demo.status();
    let demo_content_type = content_type(&demo).map(str::to_owned);
    let demo_text = read_text(demo).await;
    assert_eq!(demo_status, StatusCode::OK);
    assert_eq!(
        demo_content_type.as_deref(),
        Some("text/html; charset=utf-8")
    );
    assert!(demo_text.contains("Demo local del firmware"));

    let css = app.get("/static/demo.css").await;
    let css_status = css.status();
    let css_content_type = content_type(&css).map(str::to_owned);
    let css_text = read_text(css).await;
    assert_eq!(css_status, StatusCode::OK);
    assert_eq!(css_content_type.as_deref(), Some("text/css; charset=utf-8"));
    assert!(css_text.contains(".room"));

    let js = app.get("/static/demo.js").await;
    let js_status = js.status();
    let js_content_type = content_type(&js).map(str::to_owned);
    let js_text = read_text(js).await;
    assert_eq!(js_status, StatusCode::OK);
    assert_eq!(
        js_content_type.as_deref(),
        Some("application/javascript; charset=utf-8")
    );
    assert!(js_text.contains("isTicking"));
}
