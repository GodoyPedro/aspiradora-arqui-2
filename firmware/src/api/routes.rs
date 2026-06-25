use axum::extract::{rejection::JsonRejection, State};
use axum::routing::{get, post};
use axum::{Json, Router};

use crate::api::{
    ApiError, ManualMoveRequest, RobotStatusResponse, SetModeRequest, SharedSimulationController,
};
use crate::domain::RobotCommand;

pub fn create_router(state: SharedSimulationController) -> Router {
    Router::new()
        .route("/commands/start", post(start_cleaning))
        .route("/commands/stop", post(stop_cleaning))
        .route("/commands/pause", post(pause_cleaning))
        .route("/commands/return-to-dock", post(return_to_dock))
        .route("/commands/manual-move", post(manual_move))
        .route("/commands/mode", post(set_mode))
        .route("/commands/clear-error", post(clear_error))
        .route("/status", get(get_status))
        .with_state(state)
}

async fn start_cleaning(
    State(state): State<SharedSimulationController>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let mut controller = state.lock().await;
    Ok(Json(
        controller
            .handle_command(RobotCommand::StartCleaning)?
            .into(),
    ))
}

async fn stop_cleaning(
    State(state): State<SharedSimulationController>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let mut controller = state.lock().await;
    Ok(Json(
        controller
            .handle_command(RobotCommand::StopCleaning)?
            .into(),
    ))
}

async fn pause_cleaning(
    State(state): State<SharedSimulationController>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let mut controller = state.lock().await;
    Ok(Json(
        controller
            .handle_command(RobotCommand::PauseCleaning)?
            .into(),
    ))
}

async fn return_to_dock(
    State(state): State<SharedSimulationController>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let mut controller = state.lock().await;
    Ok(Json(
        controller
            .handle_command(RobotCommand::ReturnToDock)?
            .into(),
    ))
}

async fn manual_move(
    State(state): State<SharedSimulationController>,
    payload: Result<Json<ManualMoveRequest>, JsonRejection>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let payload = payload.map_err(ApiError::from_json_rejection)?.0;
    let mut controller = state.lock().await;

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
    State(state): State<SharedSimulationController>,
    payload: Result<Json<SetModeRequest>, JsonRejection>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let payload = payload.map_err(ApiError::from_json_rejection)?.0;
    let mut controller = state.lock().await;

    Ok(Json(
        controller
            .handle_command(RobotCommand::SetCleaningMode(payload.mode.into()))?
            .into(),
    ))
}

async fn clear_error(
    State(state): State<SharedSimulationController>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let mut controller = state.lock().await;
    Ok(Json(
        controller
            .handle_command(RobotCommand::ClearError)?
            .into(),
    ))
}

async fn get_status(
    State(state): State<SharedSimulationController>,
) -> Result<Json<RobotStatusResponse>, ApiError> {
    let controller = state.lock().await;
    Ok(Json(controller.current_status().into()))
}
