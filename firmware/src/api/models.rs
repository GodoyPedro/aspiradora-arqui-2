use serde::{Deserialize, Serialize};
use serde_json::Value;

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

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SimulationSensorsRequest {
    pub obstacle_detected: Option<bool>,
    pub drop_off_detected: Option<bool>,
    pub bumper_pressed: Option<bool>,
    pub proximity_contact: Option<bool>,
    pub contact_type: Option<String>,
    pub wall_side: Option<String>,
    pub forward_clearance_blocked: Option<bool>,
    pub dust_container_full: Option<bool>,
    pub wheel_stuck: Option<bool>,
    pub brush_stuck: Option<bool>,
    pub top_cover_open: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SimulationBatteryRequest {
    pub battery_percent: u8,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SimulationDockingRequest {
    pub dock_available: Option<bool>,
    pub dock_detected: Option<bool>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SimulationResetRequest {}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SimulationTickRequest {
    pub delta_ms: Option<u64>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SimulationLogDumpPayload {
    pub session_id: String,
    pub map_id: Option<String>,
    pub created_at: String,
    pub map: SimulationMapRequest,
    pub frontend_config: FrontendConfigRequest,
    pub timeline: Vec<SimulationFrameRequest>,
    pub coverage: CoverageExportRequest,
    pub summary: SimulationSummaryRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_index: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_total_runs: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_mode: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_config: Option<BatchRunConfigRequest>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SimulationBatchLogDumpRequest {
    pub batch_id: String,
    pub session_id: String,
    pub log: SimulationLogDumpPayload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum SimulationLogDumpRequest {
    Manual(SimulationLogDumpPayload),
    Batch(SimulationBatchLogDumpRequest),
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SimulationMapRequest {
    pub room_width: u32,
    pub room_height: u32,
    pub robot_radius: u32,
    pub initial_robot_pose: PoseRequest,
    pub docking_station: PointRequest,
    pub obstacles: Vec<ObstacleRequest>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PoseRequest {
    pub x: f64,
    pub y: f64,
    pub heading: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PointRequest {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ObstacleRequest {
    pub id: String,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FrontendConfigRequest {
    pub simulation_speed: f64,
    pub sensor_thresholds: SensorThresholdsRequest,
    pub coverage_grid: CoverageGridConfigRequest,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BatchRunConfigRequest {
    pub max_simulated_time: u64,
    pub target_coverage: f64,
    pub simulation_speed: f64,
    pub return_to_dock_after_run: bool,
    pub enabled_stop_conditions: BatchStopConditionsRequest,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub obstacle_count: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BatchStopConditionsRequest {
    pub time_limit: bool,
    pub coverage_threshold: bool,
    pub successful_docking: bool,
    pub stuck_diagnostic: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SensorThresholdsRequest {
    pub obstacle_distance: f64,
    pub obstacle_half_angle_rad: f64,
    pub bumper_contact_distance: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CoverageGridConfigRequest {
    pub cols: u32,
    pub rows: u32,
    pub cell_width: f64,
    pub cell_height: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SimulationFrameRequest {
    pub frame_index: u64,
    pub timestamp_ms: Option<u64>,
    pub x: f64,
    pub y: f64,
    pub heading: f64,
    pub left_wheel_speed: i16,
    pub right_wheel_speed: i16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_left_wheel_speed: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_right_wheel_speed: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demo_left_wheel_speed: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demo_right_wheel_speed: Option<i16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub demo_navigation_phase: Option<String>,
    pub state: String,
    pub cleaning_mode: String,
    pub current_error: Option<String>,
    pub battery_percent: u8,
    pub is_charging: bool,
    pub sensors: SensorSnapshotResponseInput,
    pub coverage_percentage: f64,
    pub event: Option<String>,
    pub event_detail: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SensorSnapshotResponseInput {
    pub obstacle_detected: bool,
    pub drop_off_detected: bool,
    pub bumper_pressed: bool,
    #[serde(default)]
    pub proximity_contact: bool,
    #[serde(default)]
    pub contact_type: Option<String>,
    #[serde(default)]
    pub wall_side: Option<String>,
    #[serde(default)]
    pub forward_clearance_blocked: bool,
    pub dust_container_full: bool,
    pub wheel_stuck: bool,
    pub brush_stuck: bool,
    pub top_cover_open: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CoverageExportRequest {
    pub covered_cells: Vec<CoverageCellRequest>,
    pub coverage_percentage: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CoverageCellRequest {
    pub col: u32,
    pub row: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SimulationSummaryRequest {
    pub total_frames: u64,
    pub total_simulated_time_ms: u64,
    pub collisions: u64,
    pub obstacle_detections: u64,
    pub turns: u64,
    pub coverage_percentage: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SimulationBatchSummaryRequest {
    pub batch_id: String,
    pub summary: BatchSummaryPayload,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BatchSummaryPayload {
    pub batch_id: String,
    pub created_at: String,
    pub finished_at: String,
    pub requested_runs: u64,
    pub completed_runs: u64,
    pub cancelled: bool,
    pub config: Value,
    pub runs: Vec<BatchRunSummaryRequest>,
    pub aggregate_metrics: BatchAggregateMetricsRequest,
    pub worst_runs: BatchWorstRunsRequest,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BatchRunSummaryRequest {
    pub batch_id: String,
    pub run_index: u64,
    pub session_id: String,
    pub map_id: String,
    pub log_path: Option<String>,
    pub created_at: String,
    pub finished_at: String,
    pub finish_reason: String,
    pub final_state: String,
    pub final_coverage_percentage: f64,
    pub total_frames: u64,
    pub total_simulated_time_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_real_time_ms: Option<u64>,
    pub obstacle_count: u64,
    pub obstacle_detections: u64,
    pub bumper_contacts: u64,
    #[serde(default)]
    pub proximity_contacts: u64,
    pub wall_contacts: u64,
    pub obstacle_contacts: u64,
    pub turns: u64,
    #[serde(default)]
    pub room_crossing_segments: u64,
    #[serde(default)]
    pub wall_follow_segments: u64,
    #[serde(default)]
    pub obstacle_escape_attempts: u64,
    #[serde(default)]
    pub obstacle_escape_successes: u64,
    #[serde(default)]
    pub obstacle_escape_failures: u64,
    #[serde(default)]
    pub anti_loop_escapes: u64,
    #[serde(default)]
    pub backup_blocked_events: u64,
    #[serde(default)]
    pub lane_starts: u64,
    #[serde(default)]
    pub lane_ends: u64,
    #[serde(default)]
    pub lane_shifts: u64,
    #[serde(default)]
    pub lane_blocked_events: u64,
    #[serde(default)]
    pub obstacle_bypass_attempts: u64,
    #[serde(default)]
    pub obstacle_bypass_failures: u64,
    #[serde(default)]
    pub pattern_recoveries: u64,
    pub long_turn_guards: u64,
    pub escape_turns: u64,
    pub dock_attempted: bool,
    pub docking_succeeded: bool,
    pub reached_charging: bool,
    pub dock_blocked_events: u64,
    pub dock_stuck_diagnostics: u64,
    pub stuck_diagnostics: u64,
    pub max_consecutive_no_movement_frames_with_nonzero_wheels: u64,
    pub max_consecutive_turning_frames_without_xy_change: u64,
    #[serde(default)]
    pub max_no_movement_ms: u64,
    #[serde(default)]
    pub coverage_per_real_second: f64,
    #[serde(default)]
    pub coverage_per_simulated_second: f64,
    pub final_x: f64,
    pub final_y: f64,
    pub final_heading: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_save_error: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BatchAggregateMetricsRequest {
    pub average_coverage: f64,
    pub min_coverage: f64,
    pub max_coverage: f64,
    pub average_simulated_time: u64,
    pub docking_success_rate: Option<f64>,
    pub most_common_finish_reason: Option<String>,
    pub total_bumper_contacts: u64,
    #[serde(default)]
    pub total_proximity_contacts: u64,
    pub total_wall_contacts: u64,
    pub total_obstacle_contacts: u64,
    #[serde(default)]
    pub total_room_crossing_segments: u64,
    #[serde(default)]
    pub total_wall_follow_segments: u64,
    #[serde(default)]
    pub total_obstacle_escape_attempts: u64,
    #[serde(default)]
    pub total_obstacle_escape_successes: u64,
    #[serde(default)]
    pub total_obstacle_escape_failures: u64,
    #[serde(default)]
    pub total_anti_loop_escapes: u64,
    #[serde(default)]
    pub total_backup_blocked_events: u64,
    #[serde(default)]
    pub total_lane_starts: u64,
    #[serde(default)]
    pub total_lane_ends: u64,
    #[serde(default)]
    pub total_lane_shifts: u64,
    #[serde(default)]
    pub total_lane_blocked_events: u64,
    #[serde(default)]
    pub total_obstacle_bypass_attempts: u64,
    #[serde(default)]
    pub total_obstacle_bypass_failures: u64,
    #[serde(default)]
    pub total_pattern_recoveries: u64,
    pub total_long_turn_guards: u64,
    pub total_escape_turns: u64,
    pub total_stuck_diagnostics: u64,
    pub total_dock_blocked_events: u64,
    pub total_dock_stuck_diagnostics: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct BatchWorstRunsRequest {
    #[serde(default)]
    pub lowest_coverage: Vec<Value>,
    #[serde(default)]
    pub highest_wall_contacts: Vec<Value>,
    #[serde(default)]
    pub highest_obstacle_contacts: Vec<Value>,
    #[serde(default)]
    pub most_anti_loop_escapes: Vec<Value>,
    #[serde(default)]
    pub most_long_turn_guards: Vec<Value>,
    #[serde(default)]
    pub most_stuck_diagnostics: Vec<Value>,
    #[serde(default)]
    pub failed_docking: Vec<Value>,
    #[serde(default)]
    pub longest_no_movement_streak: Vec<Value>,
    #[serde(default)]
    pub longest_turning_without_movement_streak: Vec<Value>,
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
    pub proximity_contact: bool,
    pub contact_type: Option<String>,
    pub wall_side: Option<String>,
    pub forward_clearance_blocked: bool,
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
    pub auto_navigation_phase: Option<String>,
    pub sensors: SensorSnapshotResponse,
}

#[derive(Clone, Debug, Serialize)]
pub struct SavedPathResponse {
    pub saved_path: String,
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
            auto_navigation_phase: value.auto_navigation_phase.map(|phase| phase.to_string()),
            sensors: SensorSnapshotResponse {
                obstacle_detected: value.sensors.obstacle_detected,
                drop_off_detected: value.sensors.drop_off_detected,
                bumper_pressed: value.sensors.bumper_pressed,
                proximity_contact: value.sensors.proximity_contact,
                contact_type: value.sensors.contact_type.map(|value| value.to_string()),
                wall_side: value.sensors.wall_side.map(|value| value.to_string()),
                forward_clearance_blocked: value.sensors.forward_clearance_blocked,
                dust_container_full: value.sensors.dust_container_full,
                wheel_stuck: value.sensors.wheel_stuck,
                brush_stuck: value.sensors.brush_stuck,
                top_cover_open: value.sensors.top_cover_open,
            },
        }
    }
}
