export type RobotState =
  | 'OFF'
  | 'STANDBY'
  | 'CLEANING'
  | 'PAUSED'
  | 'MANUAL_CONTROL'
  | 'RETURNING_TO_DOCK'
  | 'CHARGING'
  | 'ERROR'
  | string;

export interface SensorSnapshot {
  obstacle_detected: boolean;
  drop_off_detected: boolean;
  bumper_pressed: boolean;
  dust_container_full: boolean;
  wheel_stuck: boolean;
  brush_stuck: boolean;
  top_cover_open: boolean;
}

export interface RobotStatus {
  state: RobotState;
  cleaning_mode: string;
  battery_percent: number;
  is_charging: boolean;
  suction_enabled: boolean;
  brushes_enabled: boolean;
  left_wheel_speed: number;
  right_wheel_speed: number;
  current_error: string | null;
  sensors: SensorSnapshot;
  position?: Partial<RobotPosition> | null;
  pose?: Partial<RobotPosition> | null;
  x?: number;
  y?: number;
  heading?: number;
  heading_degrees?: number;
  orientation?: number;
  [key: string]: unknown;
}

export interface RobotPosition {
  x: number;
  y: number;
  heading?: number;
}

export interface ApiErrorBody {
  code: string;
  message: string;
  current_state?: string | null;
}

export type ManualDirection = 'FORWARD' | 'BACKWARD' | 'LEFT' | 'RIGHT' | 'STOP';

export interface ManualMoveRequest {
  direction: ManualDirection;
  speed: number;
  duration_ms: number;
}
