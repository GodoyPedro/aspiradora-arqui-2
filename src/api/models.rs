use serde::{Deserialize, Serialize};

use crate::domain::{CleaningMode, ManualDirection, RobotStatus};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CleaningModeRequest {
    Auto,
    #[serde(alias = "ZIGZAG")]
    ZigZag,
    WallFollowing,
    Spot,
}

impl From<CleaningModeRequest> for CleaningMode {
    fn from(value: CleaningModeRequest) -> Self {
        match value {
            CleaningModeRequest::Auto => CleaningMode::Auto,
            CleaningModeRequest::ZigZag => CleaningMode::ZigZag,
            CleaningModeRequest::WallFollowing => CleaningMode::WallFollowing,
            CleaningModeRequest::Spot => CleaningMode::Spot,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ManualDirectionRequest {
    Forward,
    Backward,
    Left,
    Right,
    Stop,
}

impl From<ManualDirectionRequest> for ManualDirection {
    fn from(value: ManualDirectionRequest) -> Self {
        match value {
            ManualDirectionRequest::Forward => ManualDirection::Forward,
            ManualDirectionRequest::Backward => ManualDirection::Backward,
            ManualDirectionRequest::Left => ManualDirection::Left,
            ManualDirectionRequest::Right => ManualDirection::Right,
            ManualDirectionRequest::Stop => ManualDirection::Stop,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct ManualMoveRequest {
    pub direction: ManualDirectionRequest,
    pub speed: u8,
    pub duration_ms: u64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SetModeRequest {
    pub mode: CleaningModeRequest,
}

#[derive(Clone, Debug, Serialize)]
pub struct ErrorResponse {
    pub code: String,
    pub message: String,
    pub current_state: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SensorSnapshotResponse {
    pub obstacle_detected: bool,
    pub drop_off_detected: bool,
    pub bumper_pressed: bool,
    pub dust_container_full: bool,
    pub wheel_stuck: bool,
    pub brush_stuck: bool,
    pub top_cover_open: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct RobotStatusResponse {
    pub state: String,
    pub cleaning_mode: String,
    pub battery_percent: u8,
    pub is_charging: bool,
    pub suction_enabled: bool,
    pub brushes_enabled: bool,
    pub left_wheel_speed: i16,
    pub right_wheel_speed: i16,
    pub current_error: Option<String>,
    pub sensors: SensorSnapshotResponse,
}

impl From<RobotStatus> for RobotStatusResponse {
    fn from(value: RobotStatus) -> Self {
        Self {
            state: value.state.to_string(),
            cleaning_mode: value.cleaning_mode.to_string(),
            battery_percent: value.battery_percent,
            is_charging: value.is_charging,
            suction_enabled: value.suction_enabled,
            brushes_enabled: value.brushes_enabled,
            left_wheel_speed: value.left_wheel_speed,
            right_wheel_speed: value.right_wheel_speed,
            current_error: value.current_error.map(|error| error.to_string()),
            sensors: SensorSnapshotResponse {
                obstacle_detected: value.sensors.obstacle_detected,
                drop_off_detected: value.sensors.drop_off_detected,
                bumper_pressed: value.sensors.bumper_pressed,
                dust_container_full: value.sensors.dust_container_full,
                wheel_stuck: value.sensors.wheel_stuck,
                brush_stuck: value.sensors.brush_stuck,
                top_cover_open: value.sensors.top_cover_open,
            },
        }
    }
}
