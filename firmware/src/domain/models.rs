use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotState {
    Off,
    Standby,
    Cleaning,
    Paused,
    ManualControl,
    ReturningToDock,
    Charging,
    Error,
}

impl fmt::Display for RobotState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Off => "OFF",
            Self::Standby => "STANDBY",
            Self::Cleaning => "CLEANING",
            Self::Paused => "PAUSED",
            Self::ManualControl => "MANUAL_CONTROL",
            Self::ReturningToDock => "RETURNING_TO_DOCK",
            Self::Charging => "CHARGING",
            Self::Error => "ERROR",
        };

        write!(f, "{value}")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CleaningMode {
    Auto,
    Manual,
    ZigZag,
    WallFollowing,
    Spot,
}

impl fmt::Display for CleaningMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Auto => "AUTO",
            Self::Manual => "MANUAL",
            Self::ZigZag => "ZIG_ZAG",
            Self::WallFollowing => "WALL_FOLLOWING",
            Self::Spot => "SPOT",
        };

        write!(f, "{value}")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManualDirection {
    Forward,
    Backward,
    Left,
    Right,
    Stop,
}

impl fmt::Display for ManualDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Forward => "FORWARD",
            Self::Backward => "BACKWARD",
            Self::Left => "LEFT",
            Self::Right => "RIGHT",
            Self::Stop => "STOP",
        };

        write!(f, "{value}")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RobotError {
    DropOffDetected,
    WheelStuck,
    BrushStuck,
    TopCoverOpen,
    DustContainerFull,
}

impl fmt::Display for RobotError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::DropOffDetected => "DROP_OFF_DETECTED",
            Self::WheelStuck => "WHEEL_STUCK",
            Self::BrushStuck => "BRUSH_STUCK",
            Self::TopCoverOpen => "TOP_COVER_OPEN",
            Self::DustContainerFull => "DUST_CONTAINER_FULL",
        };

        write!(f, "{value}")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AutoNavigationPhase {
    Idle,
    RoomCrossing,
    ClearanceBackup,
    ClearanceTurnClear,
    ClearanceExitForward,
    WallBackup,
    WallTurnAway,
    WallExitForward,
    ContactBackup,
    WallAlign,
    WallFollow,
    CornerBackup,
    CornerTurnClear,
    CornerExitForward,
    WallRelease,
    ObstacleBackup,
    ObstacleTurnAway,
    ObstacleEscapeForward,
    AntiLoopBackup,
    AntiLoopTurn,
    AntiLoopForward,
}

impl fmt::Display for AutoNavigationPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Idle => "IDLE",
            Self::RoomCrossing => "ROOM_CROSSING",
            Self::ClearanceBackup => "CLEARANCE_BACKUP",
            Self::ClearanceTurnClear => "CLEARANCE_TURN_CLEAR",
            Self::ClearanceExitForward => "CLEARANCE_EXIT_FORWARD",
            Self::WallBackup => "WALL_BACKUP",
            Self::WallTurnAway => "WALL_TURN_AWAY",
            Self::WallExitForward => "WALL_EXIT_FORWARD",
            Self::ContactBackup => "CONTACT_BACKUP",
            Self::WallAlign => "WALL_ALIGN",
            Self::WallFollow => "WALL_FOLLOW",
            Self::CornerBackup => "CORNER_BACKUP",
            Self::CornerTurnClear => "CORNER_TURN_CLEAR",
            Self::CornerExitForward => "CORNER_EXIT_FORWARD",
            Self::WallRelease => "WALL_RELEASE",
            Self::ObstacleBackup => "OBSTACLE_BACKUP",
            Self::ObstacleTurnAway => "OBSTACLE_TURN_AWAY",
            Self::ObstacleEscapeForward => "OBSTACLE_ESCAPE_FORWARD",
            Self::AntiLoopBackup => "ANTI_LOOP_BACKUP",
            Self::AntiLoopTurn => "ANTI_LOOP_TURN",
            Self::AntiLoopForward => "ANTI_LOOP_FORWARD",
        };

        write!(f, "{value}")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ContactType {
    Wall,
    Obstacle,
    Unknown,
}

impl fmt::Display for ContactType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Wall => "WALL",
            Self::Obstacle => "OBSTACLE",
            Self::Unknown => "UNKNOWN",
        };

        write!(f, "{value}")
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WallSide {
    Left,
    Right,
    Top,
    Bottom,
}

impl fmt::Display for WallSide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            Self::Left => "LEFT",
            Self::Right => "RIGHT",
            Self::Top => "TOP",
            Self::Bottom => "BOTTOM",
        };

        write!(f, "{value}")
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct SensorSnapshot {
    pub obstacle_detected: bool,
    pub drop_off_detected: bool,
    pub bumper_pressed: bool,
    pub proximity_contact: bool,
    pub contact_type: Option<ContactType>,
    pub wall_side: Option<WallSide>,
    pub forward_clearance_blocked: bool,
    pub dust_container_full: bool,
    pub wheel_stuck: bool,
    pub brush_stuck: bool,
    pub top_cover_open: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RobotStatus {
    pub state: RobotState,
    pub cleaning_mode: CleaningMode,
    pub battery_percent: u8,
    pub is_charging: bool,
    pub suction_enabled: bool,
    pub brushes_enabled: bool,
    pub left_wheel_speed: i16,
    pub right_wheel_speed: i16,
    pub current_error: Option<RobotError>,
    pub auto_navigation_phase: Option<AutoNavigationPhase>,
    pub sensors: SensorSnapshot,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RobotCommand {
    StartCleaning,
    StopCleaning,
    PauseCleaning,
    ReturnToDock,
    ManualMove {
        direction: ManualDirection,
        speed: u8,
        duration_ms: u64,
    },
    SetCleaningMode(CleaningMode),
    ClearError,
    GetStatus,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ErrorKind {
    BadRequest,
    Conflict,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DomainError {
    pub kind: ErrorKind,
    pub code: &'static str,
    pub message: String,
    pub current_state: Option<RobotState>,
}

impl DomainError {
    pub fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind: ErrorKind::BadRequest,
            code,
            message: message.into(),
            current_state: None,
        }
    }

    pub fn conflict(
        code: &'static str,
        message: impl Into<String>,
        current_state: RobotState,
    ) -> Self {
        Self {
            kind: ErrorKind::Conflict,
            code,
            message: message.into(),
            current_state: Some(current_state),
        }
    }
}
