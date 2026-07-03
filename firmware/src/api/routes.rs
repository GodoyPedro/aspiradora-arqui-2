use std::fs;
use std::path::{Path, PathBuf};

use axum::body::to_bytes;
use axum::extract::{rejection::JsonRejection, Request, State};
use axum::response::{Html, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde_json::Value;

use crate::api::{
    demo::{demo_css, demo_js, demo_page},
    ApiError, AppState, ManualMoveRequest, RobotStatusResponse, SavedPathResponse, SetModeRequest,
    SimulationBatchLogDumpRequest, SimulationBatchSummaryRequest, SimulationBatteryRequest,
    SimulationDockingRequest, SimulationLogDumpPayload, SimulationSensorsRequest,
    SimulationTickRequest,
};
use crate::domain::{ContactType, DomainError, RobotCommand, SensorSnapshot, WallSide};
use crate::simulation::reset_simulation_controller;

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/demo", get(show_demo))
        .route("/static/demo.css", get(show_demo_css))
        .route("/static/demo.js", get(show_demo_js))
        .route("/commands/start", post(start_cleaning))
        .route("/commands/stop", post(stop_cleaning))
        .route("/commands/pause", post(pause_cleaning))
        .route("/commands/return-to-dock", post(return_to_dock))
        .route("/commands/manual-move", post(manual_move))
        .route("/commands/mode", post(set_mode))
        .route("/commands/clear-error", post(clear_error))
        .route("/simulation/sensors", post(update_simulation_sensors))
        .route("/simulation/battery", post(update_simulation_battery))
        .route("/simulation/docking", post(update_simulation_docking))
        .route("/simulation/reset", post(simulation_reset))
        .route("/simulation/batch-summary", post(simulation_batch_summary))
        .route("/simulation/log-dump", post(simulation_log_dump))
        .route("/simulation/tick", post(simulation_tick))
        .route("/status", get(get_status))
        .with_state(state)
}

async fn show_demo() -> Html<&'static str> {
    demo_page().await
}

async fn show_demo_css() -> Response {
    demo_css().await
}

async fn show_demo_js() -> Response {
    demo_js().await
}

async fn start_cleaning(
    State(state): State<AppState>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let mut controller = state.controller.lock().await;
    Ok(Json(
        controller
            .handle_command(RobotCommand::StartCleaning)?
            .into(),
    ))
}

async fn stop_cleaning(
    State(state): State<AppState>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let mut controller = state.controller.lock().await;
    Ok(Json(
        controller
            .handle_command(RobotCommand::StopCleaning)?
            .into(),
    ))
}

async fn pause_cleaning(
    State(state): State<AppState>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let mut controller = state.controller.lock().await;
    Ok(Json(
        controller
            .handle_command(RobotCommand::PauseCleaning)?
            .into(),
    ))
}

async fn return_to_dock(
    State(state): State<AppState>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let mut controller = state.controller.lock().await;
    Ok(Json(
        controller
            .handle_command(RobotCommand::ReturnToDock)?
            .into(),
    ))
}

async fn manual_move(
    State(state): State<AppState>,
    payload: Result<Json<ManualMoveRequest>, JsonRejection>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let payload = payload.map_err(ApiError::from_json_rejection)?.0;
    let mut controller = state.controller.lock().await;

    Ok(Json(
        controller
            .handle_command(RobotCommand::ManualMove {
                direction: payload.direction.into(),
                speed: payload.speed,
                duration_ms: payload.duration_ms,
            })?
            .into(),
    ))
}

async fn set_mode(
    State(state): State<AppState>,
    payload: Result<Json<SetModeRequest>, JsonRejection>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let payload = payload.map_err(ApiError::from_json_rejection)?.0;
    let mut controller = state.controller.lock().await;

    Ok(Json(
        controller
            .handle_command(RobotCommand::SetCleaningMode(payload.mode.into()))?
            .into(),
    ))
}

async fn clear_error(State(state): State<AppState>) -> Result<Json<RobotStatusResponse>, ApiError> {
    let mut controller = state.controller.lock().await;
    Ok(Json(
        controller.handle_command(RobotCommand::ClearError)?.into(),
    ))
}

async fn get_status(State(state): State<AppState>) -> Result<Json<RobotStatusResponse>, ApiError> {
    let controller = state.controller.lock().await;
    Ok(Json(controller.current_status().into()))
}

async fn update_simulation_sensors(
    State(state): State<AppState>,
    payload: Result<Json<SimulationSensorsRequest>, JsonRejection>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let payload = payload.map_err(ApiError::from_json_rejection)?.0;
    let mut controller = state.controller.lock().await;
    let current = controller.current_status().sensors;
    let proximity_contact = payload
        .proximity_contact
        .or(payload.bumper_pressed)
        .unwrap_or(false);
    let bumper_pressed = proximity_contact;
    let parsed_wall_side = match payload.wall_side.as_deref() {
        Some(value) => Some(parse_wall_side(value)?),
        None => None,
    };
    let parsed_contact_type = match payload.contact_type.as_deref() {
        Some(value) => Some(parse_contact_type(value)?),
        None => None,
    };
    let wall_side = if proximity_contact {
        parsed_wall_side
    } else {
        None
    };
    let contact_type = if !proximity_contact {
        None
    } else if wall_side.is_some() {
        Some(ContactType::Wall)
    } else {
        match parsed_contact_type {
            Some(ContactType::Wall) => Some(ContactType::Wall),
            Some(ContactType::Obstacle) => Some(ContactType::Obstacle),
            Some(ContactType::Unknown) | None => Some(ContactType::Unknown),
        }
    };
    let next = SensorSnapshot {
        obstacle_detected: payload
            .obstacle_detected
            .unwrap_or(current.obstacle_detected),
        drop_off_detected: payload
            .drop_off_detected
            .unwrap_or(current.drop_off_detected),
        bumper_pressed,
        proximity_contact,
        contact_type,
        wall_side,
        forward_clearance_blocked: payload
            .forward_clearance_blocked
            .unwrap_or(false),
        dust_container_full: payload
            .dust_container_full
            .unwrap_or(current.dust_container_full),
        wheel_stuck: payload.wheel_stuck.unwrap_or(current.wheel_stuck),
        brush_stuck: payload.brush_stuck.unwrap_or(current.brush_stuck),
        top_cover_open: payload.top_cover_open.unwrap_or(current.top_cover_open),
    };
    controller.sensor_reader_mut().set_snapshot(next);

    Ok(Json(controller.current_status().into()))
}

fn parse_contact_type(value: &str) -> Result<ContactType, ApiError> {
    match value {
        "wall" => Ok(ContactType::Wall),
        "obstacle" => Ok(ContactType::Obstacle),
        "unknown" => Ok(ContactType::Unknown),
        _ => Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_CONTACT_TYPE",
            "contact_type must be one of: wall, obstacle, unknown",
        ))),
    }
}

fn parse_wall_side(value: &str) -> Result<WallSide, ApiError> {
    match value {
        "left" => Ok(WallSide::Left),
        "right" => Ok(WallSide::Right),
        "top" => Ok(WallSide::Top),
        "bottom" => Ok(WallSide::Bottom),
        _ => Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_WALL_SIDE",
            "wall_side must be one of: left, right, top, bottom",
        ))),
    }
}

async fn update_simulation_battery(
    State(state): State<AppState>,
    payload: Result<Json<SimulationBatteryRequest>, JsonRejection>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let payload = payload.map_err(ApiError::from_json_rejection)?.0;
    if payload.battery_percent > 100 {
        return Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_BATTERY_PERCENT",
            "battery_percent must be in the range 0..=100",
        )));
    }

    let mut controller = state.controller.lock().await;
    controller
        .battery_driver_mut()
        .set_percentage(payload.battery_percent);

    Ok(Json(controller.current_status().into()))
}

async fn update_simulation_docking(
    State(state): State<AppState>,
    payload: Result<Json<SimulationDockingRequest>, JsonRejection>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let payload = payload.map_err(ApiError::from_json_rejection)?.0;
    let mut controller = state.controller.lock().await;
    let docking = controller.docking_driver_mut();
    if let Some(dock_available) = payload.dock_available {
        docking.set_dock_available(dock_available);
    }
    if let Some(dock_detected) = payload.dock_detected {
        docking.set_dock_detected(dock_detected);
    }

    Ok(Json(controller.current_status().into()))
}

async fn simulation_tick(
    State(state): State<AppState>,
    request: Request,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let mut controller = state.controller.lock().await;
    let payload = parse_simulation_tick_request(request).await?;
    let delta_ms = payload.delta_ms.unwrap_or(100);
    validate_tick_delta_ms(delta_ms)?;
    controller.clock_mut().advance_ms(delta_ms);
    Ok(Json(controller.tick().into()))
}

async fn simulation_reset(
    State(state): State<AppState>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let mut controller = state.controller.lock().await;
    reset_simulation_controller(&mut controller, state.simulation_config);
    Ok(Json(controller.current_status().into()))
}

async fn simulation_log_dump(
    State(state): State<AppState>,
    payload: Result<Json<Value>, JsonRejection>,
) -> Result<Json<SavedPathResponse>, ApiError> {
    let value = payload.map_err(ApiError::from_json_rejection)?.0;

    if value.get("log").is_some() {
        let batch: SimulationBatchLogDumpRequest =
            serde_json::from_value(value).map_err(|error| {
                ApiError::Domain(DomainError::bad_request(
                    "INVALID_LOG_DUMP",
                    format!("invalid batch log dump payload: {error}"),
                ))
            })?;
        persist_batch_log_dump(&state, &batch)
    } else {
        let manual: SimulationLogDumpPayload = serde_json::from_value(value).map_err(|error| {
            ApiError::Domain(DomainError::bad_request(
                "INVALID_LOG_DUMP",
                format!("invalid manual log dump payload: {error}"),
            ))
        })?;
        persist_manual_log_dump(&state, &manual)
    }
}

async fn simulation_batch_summary(
    State(state): State<AppState>,
    payload: Result<Json<SimulationBatchSummaryRequest>, JsonRejection>,
) -> Result<Json<SavedPathResponse>, ApiError> {
    let payload = payload.map_err(ApiError::from_json_rejection)?.0;
    validate_batch_summary_payload(&payload)?;

    let safe_batch_id = sanitize_identifier(&payload.batch_id, "batch_id").ok_or_else(|| {
        ApiError::Domain(DomainError::bad_request(
            "INVALID_BATCH_ID",
            "batch_id must contain only letters, digits, hyphens, or underscores",
        ))
    })?;
    let batch_dir = state.demo_log_root.join(&safe_batch_id);
    let summary_path = batch_dir.join("batch-summary.json");
    ensure_within_root(&state.demo_log_root, &summary_path)?;
    fs::create_dir_all(&batch_dir).map_err(io_to_api_error)?;
    write_pretty_json(&summary_path, &payload.summary, "INVALID_BATCH_SUMMARY")?;

    Ok(Json(SavedPathResponse {
        saved_path: format!("demo-logs/{safe_batch_id}/batch-summary.json"),
    }))
}

fn persist_manual_log_dump(
    state: &AppState,
    payload: &SimulationLogDumpPayload,
) -> Result<Json<SavedPathResponse>, ApiError> {
    validate_manual_log_dump_payload(payload)?;

    let safe_session_id =
        sanitize_identifier(&payload.session_id, "session_id").ok_or_else(|| {
            ApiError::Domain(DomainError::bad_request(
                "INVALID_SESSION_ID",
                "session_id must contain only letters, digits, hyphens, or underscores",
            ))
        })?;
    let session_dir = state.demo_log_root.join(&safe_session_id);
    let log_path = session_dir.join("log.json");
    ensure_within_root(&state.demo_log_root, &log_path)?;
    fs::create_dir_all(&session_dir).map_err(io_to_api_error)?;
    write_pretty_json(&log_path, payload, "INVALID_LOG_DUMP")?;

    Ok(Json(SavedPathResponse {
        saved_path: format!("demo-logs/{safe_session_id}/log.json"),
    }))
}

fn persist_batch_log_dump(
    state: &AppState,
    payload: &SimulationBatchLogDumpRequest,
) -> Result<Json<SavedPathResponse>, ApiError> {
    validate_batch_log_dump_payload(payload)?;

    let safe_batch_id = sanitize_identifier(&payload.batch_id, "batch_id").ok_or_else(|| {
        ApiError::Domain(DomainError::bad_request(
            "INVALID_BATCH_ID",
            "batch_id must contain only letters, digits, hyphens, or underscores",
        ))
    })?;
    let safe_session_id =
        sanitize_identifier(&payload.session_id, "session_id").ok_or_else(|| {
            ApiError::Domain(DomainError::bad_request(
                "INVALID_SESSION_ID",
                "session_id must contain only letters, digits, hyphens, or underscores",
            ))
        })?;

    let session_dir = state
        .demo_log_root
        .join(&safe_batch_id)
        .join(&safe_session_id);
    let log_path = session_dir.join("log.json");
    ensure_within_root(&state.demo_log_root, &log_path)?;
    fs::create_dir_all(&session_dir).map_err(io_to_api_error)?;
    write_pretty_json(&log_path, &payload.log, "INVALID_LOG_DUMP")?;

    Ok(Json(SavedPathResponse {
        saved_path: format!("demo-logs/{safe_batch_id}/{safe_session_id}/log.json"),
    }))
}

fn validate_manual_log_dump_payload(payload: &SimulationLogDumpPayload) -> Result<(), ApiError> {
    if payload.session_id.trim().is_empty() {
        return Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_LOG_DUMP",
            "session_id is required",
        )));
    }

    if payload.created_at.trim().is_empty()
        || payload.map.room_width == 0
        || payload.map.room_height == 0
        || payload.map.robot_radius == 0
        || payload.frontend_config.coverage_grid.cols == 0
        || payload.frontend_config.coverage_grid.rows == 0
        || payload.frontend_config.simulation_speed <= 0.0
        || payload.frontend_config.sensor_thresholds.obstacle_distance <= 0.0
        || payload.timeline.is_empty()
    {
        return Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_LOG_DUMP",
            "log dump is missing required reconstruction or timeline data",
        )));
    }

    if !(0.0..=100.0).contains(&payload.coverage.coverage_percentage)
        || !(0.0..=100.0).contains(&payload.summary.coverage_percentage)
    {
        return Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_LOG_DUMP",
            "coverage percentages must be in the range 0..=100",
        )));
    }

    if payload.timeline.iter().any(|frame| {
        frame.battery_percent > 100 || !(0.0..=100.0).contains(&frame.coverage_percentage)
    }) {
        return Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_LOG_DUMP",
            "timeline contains invalid battery or coverage values",
        )));
    }

    validate_timeline_timestamps(&payload.timeline)?;

    Ok(())
}

fn validate_batch_log_dump_payload(
    payload: &SimulationBatchLogDumpRequest,
) -> Result<(), ApiError> {
    if payload.batch_id.trim().is_empty() || payload.session_id.trim().is_empty() {
        return Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_LOG_DUMP",
            "batch_id and session_id are required for batch log dumps",
        )));
    }

    validate_manual_log_dump_payload(&payload.log)?;

    let embedded_batch_id = payload.log.batch_id.as_deref().ok_or_else(|| {
        ApiError::Domain(DomainError::bad_request(
            "INVALID_LOG_DUMP",
            "log.batch_id is required for batch log dumps",
        ))
    })?;
    let embedded_run_index = payload.log.run_index.ok_or_else(|| {
        ApiError::Domain(DomainError::bad_request(
            "INVALID_LOG_DUMP",
            "log.run_index is required for batch log dumps",
        ))
    })?;
    let embedded_total_runs = payload.log.batch_total_runs.ok_or_else(|| {
        ApiError::Domain(DomainError::bad_request(
            "INVALID_LOG_DUMP",
            "log.batch_total_runs is required for batch log dumps",
        ))
    })?;
    let batch_mode = payload.log.batch_mode.ok_or_else(|| {
        ApiError::Domain(DomainError::bad_request(
            "INVALID_LOG_DUMP",
            "log.batch_mode is required for batch log dumps",
        ))
    })?;
    if payload.log.run_config.is_none() {
        return Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_LOG_DUMP",
            "log.run_config is required for batch log dumps",
        )));
    }
    if embedded_batch_id != payload.batch_id || payload.log.session_id != payload.session_id {
        return Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_LOG_DUMP",
            "top-level and embedded batch/session identifiers must match",
        )));
    }
    if !batch_mode {
        return Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_LOG_DUMP",
            "log.batch_mode must be true for batch log dumps",
        )));
    }
    if embedded_total_runs == 0
        || embedded_run_index == 0
        || embedded_run_index > embedded_total_runs
    {
        return Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_LOG_DUMP",
            "batch run metadata is inconsistent",
        )));
    }

    Ok(())
}

fn validate_batch_summary_payload(payload: &SimulationBatchSummaryRequest) -> Result<(), ApiError> {
    if payload.batch_id.trim().is_empty() {
        return Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_BATCH_SUMMARY",
            "batch_id is required",
        )));
    }
    if payload.summary.batch_id.trim().is_empty() {
        return Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_BATCH_SUMMARY",
            "summary.batch_id is required",
        )));
    }
    if payload.summary.batch_id != payload.batch_id {
        return Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_BATCH_SUMMARY",
            "summary.batch_id must match top-level batch_id",
        )));
    }
    Ok(())
}

fn validate_timeline_timestamps(
    timeline: &[crate::api::SimulationFrameRequest],
) -> Result<(), ApiError> {
    let mut last_timestamp = None;
    for frame in timeline {
        if let Some(timestamp_ms) = frame.timestamp_ms {
            if let Some(previous) = last_timestamp {
                if timestamp_ms < previous {
                    return Err(ApiError::Domain(DomainError::bad_request(
                        "INVALID_LOG_DUMP",
                        "timeline timestamp_ms values must be non-decreasing",
                    )));
                }
            }
            last_timestamp = Some(timestamp_ms);
        }
    }
    Ok(())
}

async fn parse_simulation_tick_request(
    request: Request,
) -> Result<SimulationTickRequest, ApiError> {
    let bytes = to_bytes(request.into_body(), usize::MAX)
        .await
        .map_err(|error| ApiError::BadRequest(format!("Failed to read request body: {error}")))?;

    if bytes.is_empty() {
        return Ok(SimulationTickRequest::default());
    }

    serde_json::from_slice::<SimulationTickRequest>(&bytes)
        .map_err(|error| ApiError::BadRequest(format!("Invalid JSON body: {error}")))
}

fn validate_tick_delta_ms(delta_ms: u64) -> Result<(), ApiError> {
    if delta_ms == 0 {
        return Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_TICK_DELTA",
            "delta_ms must be greater than zero",
        )));
    }

    if delta_ms > 1_000 {
        return Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_TICK_DELTA",
            "delta_ms must be less than or equal to 1000",
        )));
    }

    Ok(())
}

fn sanitize_identifier(raw: &str, _field_name: &str) -> Option<String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    let sanitized: String = trimmed
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
        .collect();

    if sanitized.is_empty() {
        None
    } else {
        Some(sanitized)
    }
}

fn write_pretty_json<T: serde::Serialize>(
    path: &Path,
    payload: &T,
    error_code: &'static str,
) -> Result<(), ApiError> {
    let serialized = serde_json::to_vec_pretty(payload).map_err(|error| {
        ApiError::Domain(DomainError::bad_request(
            error_code,
            format!("Failed to serialize JSON payload: {error}"),
        ))
    })?;
    fs::write(path, serialized).map_err(io_to_api_error)
}

fn ensure_within_root(root: &Path, path: &Path) -> Result<(), ApiError> {
    let root = absolute_like(root);
    let path = absolute_like(path);
    if path.starts_with(&root) {
        Ok(())
    } else {
        Err(ApiError::Domain(DomainError::bad_request(
            "INVALID_LOG_DUMP",
            "log path must remain inside the demo log directory",
        )))
    }
}

fn absolute_like(path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}

fn io_to_api_error(error: std::io::Error) -> ApiError {
    ApiError::Domain(DomainError::bad_request(
        "LOG_WRITE_FAILED",
        format!("Failed to persist demo log dump: {error}"),
    ))
}
