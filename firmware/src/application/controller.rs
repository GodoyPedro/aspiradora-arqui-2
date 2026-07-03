use crate::controllers::{
    BatteryManager, CleaningController, DockingManager, MotionController, SafetyManager,
};
use crate::domain::{
    AutoNavigationPhase, CleaningMode, ContactType, DomainError, ManualDirection, RobotCommand,
    RobotError, RobotState, RobotStateMachine, RobotStatus, SensorSnapshot, WallSide,
};
use crate::hal::{
    BatteryDriver, BrushDriver, Clock, DockingDriver, SensorReader, SuctionDriver, WheelMotorDriver,
};

#[derive(Debug)]
pub struct RobotController<W, S, B, Se, Bd, D, C>
where
    W: WheelMotorDriver,
    S: SuctionDriver,
    B: BrushDriver,
    Se: SensorReader,
    Bd: BatteryDriver,
    D: DockingDriver,
    C: Clock,
{
    state_machine: RobotStateMachine,
    motion_controller: MotionController,
    cleaning_controller: CleaningController,
    battery_manager: BatteryManager,
    docking_manager: DockingManager,
    safety_manager: SafetyManager,
    wheel_driver: W,
    suction_driver: S,
    brush_driver: B,
    sensor_reader: Se,
    battery_driver: Bd,
    docking_driver: D,
    clock: C,
    cleaning_mode: CleaningMode,
    current_error: Option<RobotError>,
    manual_move_deadline_ms: Option<u64>,
    auto_phase: AutoNavigationPhase,
    auto_phase_started_ms: u64,
    auto_escape_right: bool,
    auto_clearance_turn_right: bool,
    auto_wall_turn_right: bool,
    auto_clear_sensor_ticks: u8,
    auto_wall_clear_sensor_ticks: u8,
    auto_wall_recovery_count: u8,
    auto_wall_contact_streak: u8,
    auto_recovery_attempts: u8,
    auto_recent_contact_times: Vec<u64>,
    auto_last_progress_ms: u64,
    auto_contact_type: Option<ContactType>,
    auto_wall_side: Option<WallSide>,
    auto_last_contact_type: Option<ContactType>,
    auto_last_wall_side: Option<WallSide>,
    auto_last_wall_recovery_side: Option<WallSide>,
}

const ROOM_CROSSING_SPEED: i16 = 36;
const WALL_FOLLOW_SPEED: i16 = 30;
const BACKUP_SPEED: i16 = -24;
const TURN_SPEED: i16 = 22;
const CLEARANCE_TURN_SPEED: i16 = 6;
const ESCAPE_FORWARD_SPEED: i16 = 34;
const ANTI_LOOP_FORWARD_SPEED: i16 = 38;

const BACKUP_MS: u64 = 900;
const WALL_ALIGN_MS: u64 = 900;
const WALL_RELEASE_MS: u64 = 900;
const OBSTACLE_TURN_MS: u64 = 1_000;
const OBSTACLE_ESCAPE_FORWARD_MS: u64 = 2_500;
const ANTI_LOOP_BACKUP_MS: u64 = 1_200;
const ANTI_LOOP_TURN_MS: u64 = 1_500;
const ANTI_LOOP_FORWARD_MS: u64 = 3_500;

const CLEARANCE_BACKUP_MS: u64 = 900;
const CLEARANCE_EXIT_FORWARD_MS: u64 = 1_200;
const CLEAR_SENSOR_TICKS_REQUIRED: u8 = 2;

const WALL_BACKUP_MS: u64 = 900;
const WALL_TURN_AWAY_BASE_MIN_MS: u64 = 5_000;
const WALL_TURN_AWAY_EXTRA_MS: u64 = 1_000;
const WALL_TURN_AWAY_MAX_MIN_MS: u64 = 8_000;
const WALL_STREAK_RESET_MS: u64 = 12_000;
const WALL_EXIT_FORWARD_BASE_MS: u64 = 1_600;
const WALL_EXIT_FORWARD_EXTRA_MS: u64 = 500;
const WALL_EXIT_FORWARD_MAX_MS: u64 = 2_600;
const WALL_CLEAR_SENSOR_TICKS_REQUIRED: u8 = 2;

const ANTI_LOOP_CONTACT_WINDOW_MS: u64 = 8_000;

impl<W, S, B, Se, Bd, D, C> RobotController<W, S, B, Se, Bd, D, C>
where
    W: WheelMotorDriver,
    S: SuctionDriver,
    B: BrushDriver,
    Se: SensorReader,
    Bd: BatteryDriver,
    D: DockingDriver,
    C: Clock,
{
    pub fn new(
        mut wheel_driver: W,
        mut suction_driver: S,
        mut brush_driver: B,
        sensor_reader: Se,
        mut battery_driver: Bd,
        docking_driver: D,
        clock: C,
    ) -> Self {
        wheel_driver.stop();
        suction_driver.set_running(false);
        brush_driver.set_running(false);
        battery_driver.set_charging(false);

        Self {
            state_machine: RobotStateMachine::new(),
            motion_controller: MotionController,
            cleaning_controller: CleaningController,
            battery_manager: BatteryManager,
            docking_manager: DockingManager,
            safety_manager: SafetyManager,
            wheel_driver,
            suction_driver,
            brush_driver,
            sensor_reader,
            battery_driver,
            docking_driver,
            clock,
            cleaning_mode: CleaningMode::Auto,
            current_error: None,
            manual_move_deadline_ms: None,
            auto_phase: AutoNavigationPhase::Idle,
            auto_phase_started_ms: 0,
            auto_escape_right: true,
            auto_clearance_turn_right: true,
            auto_wall_turn_right: true,
            auto_clear_sensor_ticks: 0,
            auto_wall_clear_sensor_ticks: 0,
            auto_wall_recovery_count: 0,
            auto_wall_contact_streak: 0,
            auto_recovery_attempts: 0,
            auto_recent_contact_times: Vec::with_capacity(8),
            auto_last_progress_ms: 0,
            auto_contact_type: None,
            auto_wall_side: None,
            auto_last_contact_type: None,
            auto_last_wall_side: None,
            auto_last_wall_recovery_side: None,
        }
    }

    pub fn handle_command(&mut self, command: RobotCommand) -> Result<RobotStatus, DomainError> {
        if let Some(error) = self.read_critical_error() {
            self.fail_safe(error)?;
            return Err(DomainError::conflict(
                "SAFETY_CONDITION_ACTIVE",
                format!("Cannot process command while safety condition {error} is active"),
                self.state(),
            ));
        }

        if self.state() == RobotState::Cleaning
            && self.sensor_reader.read_sensors().dust_container_full
        {
            self.fail_safe(RobotError::DustContainerFull)?;
            return Err(DomainError::conflict(
                "DUST_CONTAINER_FULL",
                "Cannot process command while the dust container is full",
                self.state(),
            ));
        }

        match command {
            RobotCommand::StartCleaning => self.start_cleaning()?,
            RobotCommand::StopCleaning => self.stop_cleaning()?,
            RobotCommand::PauseCleaning => self.pause_cleaning()?,
            RobotCommand::ReturnToDock => self.return_to_dock()?,
            RobotCommand::ManualMove {
                direction,
                speed,
                duration_ms,
            } => self.manual_move(direction, speed, duration_ms)?,
            RobotCommand::SetCleaningMode(mode) => self.set_cleaning_mode(mode)?,
            RobotCommand::ClearError => self.clear_error()?,
            RobotCommand::GetStatus => {}
        }

        Ok(self.current_status())
    }

    pub fn tick(&mut self) -> RobotStatus {
        let sensors = self.sensor_reader.read_sensors();

        if let Some(error) = self.safety_manager.detect_critical_error(&sensors) {
            let _ = self.fail_safe(error);
            return self.current_status();
        }

        if self.state() == RobotState::Cleaning && sensors.dust_container_full {
            let _ = self.fail_safe(RobotError::DustContainerFull);
            return self.current_status();
        }

        if self.state() == RobotState::ManualControl {
            if let Some(deadline) = self.manual_move_deadline_ms {
                if self.clock.now_millis() >= deadline {
                    self.motion_controller.stop(&mut self.wheel_driver);
                    self.manual_move_deadline_ms = None;
                    let _ = self.state_machine.transition_to(RobotState::Standby);
                    self.cleaning_mode = CleaningMode::Auto;
                }
            }
        }

        let battery = self.battery_manager.read(&self.battery_driver);
        if self.state() == RobotState::Cleaning && self.battery_manager.is_low(battery) {
            self.cleaning_controller
                .stop_cleaning(&mut self.suction_driver, &mut self.brush_driver);
            self.docking_manager
                .start_return_to_dock(&mut self.wheel_driver);
            self.manual_move_deadline_ms = None;
            let _ = self
                .state_machine
                .transition_to(RobotState::ReturningToDock);
        }

        if self.state() == RobotState::Cleaning && self.cleaning_mode == CleaningMode::Auto {
            self.tick_auto_navigation(&sensors);
        }

        if self.state() == RobotState::ReturningToDock {
            let docking = self.docking_manager.read(&self.docking_driver);
            if docking.dock_detected {
                self.motion_controller.stop(&mut self.wheel_driver);
                self.battery_driver.set_charging(true);
                let _ = self.state_machine.transition_to(RobotState::Charging);
            }
        }

        let battery = self.battery_manager.read(&self.battery_driver);
        if self.state() == RobotState::Charging && self.battery_manager.is_full(battery) {
            self.battery_driver.set_charging(false);
            let _ = self.state_machine.transition_to(RobotState::Standby);
            self.cleaning_mode = CleaningMode::Auto;
        }

        self.current_status()
    }

    pub fn current_status(&self) -> RobotStatus {
        let (left_wheel_speed, right_wheel_speed) = self.wheel_driver.wheel_speeds();
        let battery = self.battery_driver.read_battery();

        RobotStatus {
            state: self.state(),
            cleaning_mode: self.cleaning_mode,
            battery_percent: battery.percentage,
            is_charging: self.battery_manager.is_charging(battery),
            suction_enabled: self.suction_driver.is_running(),
            brushes_enabled: self.brush_driver.is_running(),
            left_wheel_speed,
            right_wheel_speed,
            current_error: self.current_error,
            auto_navigation_phase: if self.state() == RobotState::Cleaning
                && self.cleaning_mode == CleaningMode::Auto
            {
                Some(self.auto_phase)
            } else {
                None
            },
            sensors: self.sensor_reader.read_sensors(),
        }
    }

    pub fn state(&self) -> RobotState {
        self.state_machine.state()
    }

    pub fn wheel_driver(&self) -> &W {
        &self.wheel_driver
    }

    pub fn wheel_driver_mut(&mut self) -> &mut W {
        &mut self.wheel_driver
    }

    pub fn suction_driver(&self) -> &S {
        &self.suction_driver
    }

    pub fn suction_driver_mut(&mut self) -> &mut S {
        &mut self.suction_driver
    }

    pub fn brush_driver(&self) -> &B {
        &self.brush_driver
    }

    pub fn brush_driver_mut(&mut self) -> &mut B {
        &mut self.brush_driver
    }

    pub fn sensor_reader_mut(&mut self) -> &mut Se {
        &mut self.sensor_reader
    }

    pub fn battery_driver_mut(&mut self) -> &mut Bd {
        &mut self.battery_driver
    }

    pub fn docking_driver_mut(&mut self) -> &mut D {
        &mut self.docking_driver
    }

    pub fn clock_mut(&mut self) -> &mut C {
        &mut self.clock
    }

    fn read_critical_error(&self) -> Option<RobotError> {
        let sensors = self.sensor_reader.read_sensors();
        self.safety_manager.detect_critical_error(&sensors)
    }

    fn fail_safe(&mut self, error: RobotError) -> Result<(), DomainError> {
        self.safety_manager.stop_all(
            &mut self.wheel_driver,
            &mut self.suction_driver,
            &mut self.brush_driver,
        );
        self.battery_driver.set_charging(false);
        self.current_error = Some(error);
        self.manual_move_deadline_ms = None;
        self.cleaning_mode = CleaningMode::Auto;
        self.reset_auto_navigation_state();

        if self.state() != RobotState::Error {
            self.state_machine.transition_to(RobotState::Error)?;
        }

        Ok(())
    }

    fn start_cleaning(&mut self) -> Result<(), DomainError> {
        match self.state() {
            RobotState::Standby | RobotState::Paused => {}
            state => {
                return Err(DomainError::conflict(
                    "COMMAND_NOT_ALLOWED",
                    "START_CLEANING is only allowed from STANDBY or PAUSED",
                    state,
                ))
            }
        }

        let battery = self.battery_manager.read(&self.battery_driver);
        if self.battery_manager.is_low(battery) {
            return Err(DomainError::conflict(
                "BATTERY_TOO_LOW",
                "Battery is too low to start cleaning",
                self.state(),
            ));
        }

        self.battery_driver.set_charging(false);
        self.cleaning_controller
            .start_cleaning(&mut self.suction_driver, &mut self.brush_driver);
        self.cleaning_mode = CleaningMode::Auto;
        self.manual_move_deadline_ms = None;
        self.reset_auto_navigation_state();
        self.set_auto_phase(AutoNavigationPhase::RoomCrossing);
        self.apply_auto_phase_motion();
        self.current_error = None;
        self.state_machine.transition_to(RobotState::Cleaning)
    }

    fn stop_cleaning(&mut self) -> Result<(), DomainError> {
        match self.state() {
            RobotState::Cleaning
            | RobotState::Paused
            | RobotState::ManualControl
            | RobotState::ReturningToDock => {}
            state => {
                return Err(DomainError::conflict(
                    "COMMAND_NOT_ALLOWED",
                    "STOP_CLEANING is not allowed in the current state",
                    state,
                ))
            }
        }

        self.safety_manager.stop_all(
            &mut self.wheel_driver,
            &mut self.suction_driver,
            &mut self.brush_driver,
        );
        self.battery_driver.set_charging(false);
        self.manual_move_deadline_ms = None;
        self.cleaning_mode = CleaningMode::Auto;
        self.reset_auto_navigation_state();
        self.state_machine.transition_to(RobotState::Standby)
    }

    fn pause_cleaning(&mut self) -> Result<(), DomainError> {
        match self.state() {
            RobotState::Cleaning | RobotState::ManualControl => {}
            state => {
                return Err(DomainError::conflict(
                    "COMMAND_NOT_ALLOWED",
                    "PAUSE_CLEANING is only allowed while actively moving",
                    state,
                ))
            }
        }

        self.safety_manager.stop_all(
            &mut self.wheel_driver,
            &mut self.suction_driver,
            &mut self.brush_driver,
        );
        self.battery_driver.set_charging(false);
        self.manual_move_deadline_ms = None;
        self.reset_auto_navigation_state();
        self.state_machine.transition_to(RobotState::Paused)
    }

    fn return_to_dock(&mut self) -> Result<(), DomainError> {
        match self.state() {
            RobotState::Standby
            | RobotState::Cleaning
            | RobotState::Paused
            | RobotState::ManualControl => {}
            state => {
                return Err(DomainError::conflict(
                    "COMMAND_NOT_ALLOWED",
                    "RETURN_TO_DOCK is not allowed in the current state",
                    state,
                ))
            }
        }

        let docking = self.docking_manager.read(&self.docking_driver);
        if !docking.dock_available {
            return Err(DomainError::conflict(
                "DOCK_UNAVAILABLE",
                "Docking station is not available",
                self.state(),
            ));
        }

        self.cleaning_controller
            .stop_cleaning(&mut self.suction_driver, &mut self.brush_driver);
        self.docking_manager
            .start_return_to_dock(&mut self.wheel_driver);
        self.manual_move_deadline_ms = None;
        self.reset_auto_navigation_state();
        self.state_machine
            .transition_to(RobotState::ReturningToDock)
    }

    fn manual_move(
        &mut self,
        direction: ManualDirection,
        speed: u8,
        duration_ms: u64,
    ) -> Result<(), DomainError> {
        if duration_ms == 0 {
            return Err(DomainError::bad_request(
                "INVALID_DURATION",
                "MANUAL_MOVE requires duration_ms greater than zero",
            ));
        }

        if speed > 100 {
            return Err(DomainError::bad_request(
                "INVALID_SPEED",
                "MANUAL_MOVE speed must be in the range 0..=100",
            ));
        }

        if direction != ManualDirection::Stop && speed == 0 {
            return Err(DomainError::bad_request(
                "INVALID_SPEED",
                "MANUAL_MOVE speed must be greater than zero for movement directions",
            ));
        }

        match self.state() {
            RobotState::Standby | RobotState::Paused | RobotState::ManualControl => {}
            state => {
                return Err(DomainError::conflict(
                    "COMMAND_NOT_ALLOWED",
                    "MANUAL_MOVE is only allowed from STANDBY, PAUSED, or MANUAL_CONTROL",
                    state,
                ))
            }
        }

        self.cleaning_controller
            .stop_cleaning(&mut self.suction_driver, &mut self.brush_driver);
        self.motion_controller
            .move_direction(&mut self.wheel_driver, direction, speed);
        self.cleaning_mode = CleaningMode::Manual;
        self.manual_move_deadline_ms = Some(self.clock.now_millis().saturating_add(duration_ms));
        self.reset_auto_navigation_state();
        self.state_machine.transition_to(RobotState::ManualControl)
    }

    fn set_cleaning_mode(&mut self, mode: CleaningMode) -> Result<(), DomainError> {
        if mode != CleaningMode::Auto {
            return Err(DomainError::bad_request(
                "UNSUPPORTED_MODE",
                format!("{mode} is not supported in this implementation"),
            ));
        }

        self.cleaning_mode = CleaningMode::Auto;
        Ok(())
    }

    fn clear_error(&mut self) -> Result<(), DomainError> {
        if self.state() != RobotState::Error {
            return Err(DomainError::conflict(
                "COMMAND_NOT_ALLOWED",
                "CLEAR_ERROR is only allowed from ERROR",
                self.state(),
            ));
        }

        if let Some(error) = self.read_critical_error() {
            return Err(DomainError::conflict(
                "SAFETY_CONDITION_ACTIVE",
                format!("Cannot clear error while {error} is still active"),
                self.state(),
            ));
        }

        if self.current_error == Some(RobotError::DustContainerFull)
            && self.sensor_reader.read_sensors().dust_container_full
        {
            return Err(DomainError::conflict(
                "DUST_CONTAINER_FULL",
                "Cannot clear error while the dust container is still full",
                self.state(),
            ));
        }

        self.current_error = None;
        self.cleaning_mode = CleaningMode::Auto;
        self.manual_move_deadline_ms = None;
        self.reset_auto_navigation_state();
        self.safety_manager.stop_all(
            &mut self.wheel_driver,
            &mut self.suction_driver,
            &mut self.brush_driver,
        );
        self.battery_driver.set_charging(false);
        self.state_machine.transition_to(RobotState::Standby)
    }

    fn tick_auto_navigation(&mut self, sensors: &SensorSnapshot) {
        self.prune_recent_contacts();

        match self.auto_phase {
            AutoNavigationPhase::Idle => {
                self.set_auto_phase(AutoNavigationPhase::RoomCrossing);
            }
            AutoNavigationPhase::RoomCrossing => {
                if self.has_proximity_contact(sensors) {
                    self.begin_contact_recovery(sensors);
                    self.apply_auto_phase_motion();
                    return;
                } else if self.phase_elapsed_ms() >= WALL_STREAK_RESET_MS {
                    self.auto_wall_contact_streak = 0;
                    self.auto_wall_recovery_count = 0;
                    self.auto_last_wall_recovery_side = None;
                }
            }
            AutoNavigationPhase::ClearanceBackup => {
                if self.phase_elapsed_ms() >= CLEARANCE_BACKUP_MS {
                    self.auto_clear_sensor_ticks = 0;
                    self.set_auto_phase(AutoNavigationPhase::ClearanceTurnClear);
                }
            }
            AutoNavigationPhase::CornerBackup => {
                self.set_auto_phase(AutoNavigationPhase::ClearanceBackup);
            }
            AutoNavigationPhase::ClearanceTurnClear => {
                if self.clearance_path_is_clear(sensors) {
                    self.auto_clear_sensor_ticks = self.auto_clear_sensor_ticks.saturating_add(1);
                } else {
                    self.auto_clear_sensor_ticks = 0;
                }

                if self.auto_clear_sensor_ticks >= CLEAR_SENSOR_TICKS_REQUIRED {
                    self.set_auto_phase(AutoNavigationPhase::ClearanceExitForward);
                }
            }
            AutoNavigationPhase::CornerTurnClear => {
                self.set_auto_phase(AutoNavigationPhase::ClearanceTurnClear);
            }
            AutoNavigationPhase::ClearanceExitForward => {
                if self.has_proximity_contact(sensors) {
                    self.begin_contact_recovery(sensors);
                    self.apply_auto_phase_motion();
                    return;
                } else if sensors.obstacle_detected || sensors.forward_clearance_blocked {
                    self.auto_clear_sensor_ticks = 0;
                    self.set_auto_phase(AutoNavigationPhase::ClearanceTurnClear);
                } else if self.phase_elapsed_ms() >= CLEARANCE_EXIT_FORWARD_MS {
                    self.auto_recovery_attempts = 0;
                    self.auto_last_contact_type = None;
                    self.auto_last_wall_side = None;
                    self.auto_contact_type = None;
                    self.auto_wall_side = None;
                    self.set_auto_phase(AutoNavigationPhase::RoomCrossing);
                } else {
                    self.auto_last_progress_ms = self.clock.now_millis();
                }
            }
            AutoNavigationPhase::CornerExitForward => {
                self.set_auto_phase(AutoNavigationPhase::ClearanceExitForward);
            }
            AutoNavigationPhase::WallBackup => {
                if self.phase_elapsed_ms() >= WALL_BACKUP_MS {
                    self.auto_wall_clear_sensor_ticks = 0;
                    self.set_auto_phase(AutoNavigationPhase::WallTurnAway);
                }
            }
            AutoNavigationPhase::WallTurnAway => {
                let turn_minimum_elapsed =
                    self.phase_elapsed_ms() >= self.current_wall_turn_away_min_ms();

                if turn_minimum_elapsed && self.wall_recovery_path_is_clear(sensors) {
                    self.auto_wall_clear_sensor_ticks = self.auto_wall_clear_sensor_ticks.saturating_add(1);
                } else {
                    self.auto_wall_clear_sensor_ticks = 0;
                }

                if self.auto_wall_clear_sensor_ticks >= WALL_CLEAR_SENSOR_TICKS_REQUIRED {
                    self.set_auto_phase(AutoNavigationPhase::WallExitForward);
                }
            }
            AutoNavigationPhase::WallExitForward => {
                if self.has_proximity_contact(sensors) {
                    self.begin_contact_recovery(sensors);
                    self.apply_auto_phase_motion();
                    return;
                } else if sensors.obstacle_detected || sensors.forward_clearance_blocked {
                    self.auto_wall_clear_sensor_ticks = 0;
                    self.set_auto_phase(AutoNavigationPhase::WallTurnAway);
                } else if self.phase_elapsed_ms() >= self.current_wall_exit_forward_ms() {
                    self.auto_recovery_attempts = 0;
                    self.auto_contact_type = None;
                    self.auto_wall_side = None;
                    self.set_auto_phase(AutoNavigationPhase::RoomCrossing);
                } else {
                    self.auto_last_progress_ms = self.clock.now_millis();
                }
            }
            AutoNavigationPhase::ContactBackup => {
                if self.phase_elapsed_ms() >= BACKUP_MS {
                    if self.auto_contact_type == Some(ContactType::Wall) {
                        self.set_auto_phase(AutoNavigationPhase::WallAlign);
                    } else {
                        self.set_auto_phase(AutoNavigationPhase::ObstacleTurnAway);
                    }
                }
            }
            AutoNavigationPhase::WallAlign => {
                if self.phase_elapsed_ms() >= WALL_ALIGN_MS {
                    self.begin_wall_follow();
                }
            }
            AutoNavigationPhase::WallFollow => {
                if self.has_proximity_contact(sensors) {
                    self.begin_contact_recovery(sensors);
                    self.apply_auto_phase_motion();
                    return;
                } else {
                    self.auto_last_progress_ms = self.clock.now_millis();
                }
            }
            AutoNavigationPhase::WallRelease => {
                if self.has_proximity_contact(sensors) {
                    self.begin_contact_recovery(sensors);
                    self.apply_auto_phase_motion();
                    return;
                } else if self.phase_elapsed_ms() >= WALL_RELEASE_MS {
                    self.auto_contact_type = None;
                    self.auto_wall_side = None;
                    self.auto_recovery_attempts = 0;
                    self.set_auto_phase(AutoNavigationPhase::RoomCrossing);
                }
            }
            AutoNavigationPhase::ObstacleBackup => {
                if self.phase_elapsed_ms() >= BACKUP_MS {
                    self.set_auto_phase(AutoNavigationPhase::ObstacleTurnAway);
                }
            }
            AutoNavigationPhase::ObstacleTurnAway => {
                if self.phase_elapsed_ms() >= OBSTACLE_TURN_MS {
                    self.set_auto_phase(AutoNavigationPhase::ObstacleEscapeForward);
                }
            }
            AutoNavigationPhase::ObstacleEscapeForward => {
                if self.has_proximity_contact(sensors) {
                    self.begin_contact_recovery(sensors);
                    self.apply_auto_phase_motion();
                    return;
                } else if self.phase_elapsed_ms() >= OBSTACLE_ESCAPE_FORWARD_MS {
                    self.auto_contact_type = None;
                    self.auto_wall_side = None;
                    self.auto_recovery_attempts = 0;
                    self.set_auto_phase(AutoNavigationPhase::RoomCrossing);
                } else {
                    self.auto_last_progress_ms = self.clock.now_millis();
                }
            }
            AutoNavigationPhase::AntiLoopBackup => {
                if self.phase_elapsed_ms() >= ANTI_LOOP_BACKUP_MS {
                    self.set_auto_phase(AutoNavigationPhase::AntiLoopTurn);
                }
            }
            AutoNavigationPhase::AntiLoopTurn => {
                if self.phase_elapsed_ms() >= ANTI_LOOP_TURN_MS {
                    self.set_auto_phase(AutoNavigationPhase::AntiLoopForward);
                }
            }
            AutoNavigationPhase::AntiLoopForward => {
                if self.has_proximity_contact(sensors) {
                    self.begin_contact_recovery(sensors);
                    self.apply_auto_phase_motion();
                    return;
                } else if self.phase_elapsed_ms() >= ANTI_LOOP_FORWARD_MS {
                    self.auto_contact_type = None;
                    self.auto_wall_side = None;
                    self.auto_recovery_attempts = 0;
                    self.set_auto_phase(AutoNavigationPhase::RoomCrossing);
                } else {
                    self.auto_last_progress_ms = self.clock.now_millis();
                }
            }
        }

        self.apply_auto_phase_motion();
    }

    fn reset_auto_navigation_state(&mut self) {
        self.auto_phase = AutoNavigationPhase::Idle;
        self.auto_phase_started_ms = self.clock.now_millis();
        self.auto_escape_right = true;
        self.auto_clearance_turn_right = true;
        self.auto_wall_turn_right = true;
        self.auto_clear_sensor_ticks = 0;
        self.auto_wall_clear_sensor_ticks = 0;
        self.auto_wall_recovery_count = 0;
        self.auto_wall_contact_streak = 0;
        self.auto_recovery_attempts = 0;
        self.auto_recent_contact_times.clear();
        self.auto_last_progress_ms = self.clock.now_millis();
        self.auto_contact_type = None;
        self.auto_wall_side = None;
        self.auto_last_contact_type = None;
        self.auto_last_wall_side = None;
        self.auto_last_wall_recovery_side = None;
    }

    fn set_auto_phase(&mut self, phase: AutoNavigationPhase) {
        self.auto_phase = phase;
        self.auto_phase_started_ms = self.clock.now_millis();
    }

    fn begin_wall_follow(&mut self) {
        self.auto_recovery_attempts = 0;
        self.set_auto_phase(AutoNavigationPhase::WallFollow);
    }

    fn begin_contact_recovery(&mut self, sensors: &SensorSnapshot) {
        if self.is_wall_contact(sensors) {
            self.begin_wall_recovery(sensors);
        } else {
            self.begin_clearance_escape(sensors);
        }
    }

    fn begin_wall_recovery(&mut self, sensors: &SensorSnapshot) {
        self.record_contact(sensors);
        self.auto_wall_contact_streak = self.auto_wall_contact_streak.saturating_add(1);

        if self.auto_last_wall_recovery_side == self.auto_wall_side {
            self.auto_wall_recovery_count = self.auto_wall_recovery_count.saturating_add(1);
        } else {
            self.auto_wall_recovery_count = 0;
            self.auto_last_wall_recovery_side = self.auto_wall_side;
        }

        self.auto_wall_turn_right = choose_wall_recovery_turn_right(
            self.auto_wall_side,
            self.auto_escape_right,
            self.auto_wall_recovery_count,
        );
        self.auto_escape_right = !self.auto_escape_right;
        self.auto_wall_clear_sensor_ticks = 0;
        self.set_auto_phase(AutoNavigationPhase::WallBackup);
    }

    fn begin_clearance_escape(&mut self, sensors: &SensorSnapshot) {
        self.record_contact(sensors);
        self.auto_wall_contact_streak = 0;
        self.auto_wall_recovery_count = 0;
        self.auto_last_wall_recovery_side = None;
        self.auto_last_contact_type = self.auto_contact_type;
        self.auto_last_wall_side = self.auto_wall_side;
        self.auto_clearance_turn_right = choose_clearance_turn_right(
            self.auto_wall_side,
            self.auto_contact_type,
            self.auto_escape_right,
        );
        self.auto_escape_right = !self.auto_escape_right;
        self.auto_clear_sensor_ticks = 0;
        self.set_auto_phase(AutoNavigationPhase::ClearanceBackup);
    }

    fn phase_elapsed_ms(&self) -> u64 {
        self.clock
            .now_millis()
            .saturating_sub(self.auto_phase_started_ms)
    }

    fn apply_auto_phase_motion(&mut self) {
        match self.auto_phase {
            AutoNavigationPhase::Idle => self.wheel_driver.set_wheel_speeds(0, 0),
            AutoNavigationPhase::RoomCrossing => self
                .wheel_driver
                .set_wheel_speeds(ROOM_CROSSING_SPEED, ROOM_CROSSING_SPEED),
            AutoNavigationPhase::ClearanceBackup => self
                .wheel_driver
                .set_wheel_speeds(BACKUP_SPEED, BACKUP_SPEED),
            AutoNavigationPhase::CornerBackup => self
                .wheel_driver
                .set_wheel_speeds(BACKUP_SPEED, BACKUP_SPEED),
            AutoNavigationPhase::ClearanceTurnClear => {
                self.apply_clearance_turn(self.auto_clearance_turn_right);
            }
            AutoNavigationPhase::CornerTurnClear => {
                self.apply_clearance_turn(self.auto_clearance_turn_right);
            }
            AutoNavigationPhase::ClearanceExitForward => self
                .wheel_driver
                .set_wheel_speeds(ROOM_CROSSING_SPEED, ROOM_CROSSING_SPEED),
            AutoNavigationPhase::WallBackup => self
                .wheel_driver
                .set_wheel_speeds(BACKUP_SPEED, BACKUP_SPEED),
            AutoNavigationPhase::WallTurnAway => {
                self.apply_clearance_turn(self.auto_wall_turn_right);
            }
            AutoNavigationPhase::WallExitForward => self
                .wheel_driver
                .set_wheel_speeds(ROOM_CROSSING_SPEED, ROOM_CROSSING_SPEED),
            AutoNavigationPhase::CornerExitForward => self
                .wheel_driver
                .set_wheel_speeds(ROOM_CROSSING_SPEED, ROOM_CROSSING_SPEED),
            AutoNavigationPhase::ContactBackup
            | AutoNavigationPhase::ObstacleBackup
            | AutoNavigationPhase::AntiLoopBackup => self
                .wheel_driver
                .set_wheel_speeds(BACKUP_SPEED, BACKUP_SPEED),
            AutoNavigationPhase::WallAlign => {
                self.apply_turn(self.auto_escape_right);
            }
            AutoNavigationPhase::WallFollow => self
                .wheel_driver
                .set_wheel_speeds(WALL_FOLLOW_SPEED, WALL_FOLLOW_SPEED),
            AutoNavigationPhase::WallRelease => {
                self.apply_turn(!self.auto_escape_right);
            }
            AutoNavigationPhase::ObstacleTurnAway => {
                self.apply_turn(self.auto_escape_right);
            }
            AutoNavigationPhase::ObstacleEscapeForward => self
                .wheel_driver
                .set_wheel_speeds(ESCAPE_FORWARD_SPEED, ESCAPE_FORWARD_SPEED),
            AutoNavigationPhase::AntiLoopTurn => {
                self.apply_clearance_turn(self.auto_escape_right);
            }
            AutoNavigationPhase::AntiLoopForward => self
                .wheel_driver
                .set_wheel_speeds(ANTI_LOOP_FORWARD_SPEED, ANTI_LOOP_FORWARD_SPEED),
        }
    }

    fn apply_turn(&mut self, turn_right: bool) {
        let (left, right) = if turn_right {
            (TURN_SPEED, -TURN_SPEED)
        } else {
            (-TURN_SPEED, TURN_SPEED)
        };
        self.wheel_driver.set_wheel_speeds(left, right);
    }

    fn apply_clearance_turn(&mut self, turn_right: bool) {
        let (left, right) = if turn_right {
            (CLEARANCE_TURN_SPEED, -CLEARANCE_TURN_SPEED)
        } else {
            (-CLEARANCE_TURN_SPEED, CLEARANCE_TURN_SPEED)
        };

        self.wheel_driver.set_wheel_speeds(left, right);
    }

    fn has_proximity_contact(&self, sensors: &SensorSnapshot) -> bool {
        sensors.proximity_contact || sensors.bumper_pressed
    }

    fn is_wall_contact(&self, sensors: &SensorSnapshot) -> bool {
        sensors.wall_side.is_some() || sensors.contact_type == Some(ContactType::Wall)
    }

    fn wall_recovery_path_is_clear(&self, sensors: &SensorSnapshot) -> bool {
        !self.has_proximity_contact(sensors)
            && !sensors.obstacle_detected
            && !sensors.forward_clearance_blocked
    }

    fn current_wall_turn_away_min_ms(&self) -> u64 {
        let extra = u64::from(self.auto_wall_contact_streak.saturating_sub(1))
            .saturating_mul(WALL_TURN_AWAY_EXTRA_MS);

        WALL_TURN_AWAY_BASE_MIN_MS
            .saturating_add(extra)
            .min(WALL_TURN_AWAY_MAX_MIN_MS)
    }

    fn current_wall_exit_forward_ms(&self) -> u64 {
        let repeated_wall_extra = u64::from(self.auto_wall_recovery_count)
            .saturating_mul(WALL_EXIT_FORWARD_EXTRA_MS);
        let streak_extra = u64::from(self.auto_wall_contact_streak.saturating_sub(1))
            .saturating_mul(WALL_EXIT_FORWARD_EXTRA_MS);

        WALL_EXIT_FORWARD_BASE_MS
            .saturating_add(repeated_wall_extra)
            .saturating_add(streak_extra)
            .min(WALL_EXIT_FORWARD_MAX_MS)
    }

    fn clearance_path_is_clear(&self, sensors: &SensorSnapshot) -> bool {
        !self.has_proximity_contact(sensors)
            && !sensors.obstacle_detected
            && !sensors.forward_clearance_blocked
    }

    fn record_contact(&mut self, sensors: &SensorSnapshot) {
        let now = self.clock.now_millis();
        self.auto_recent_contact_times.push(now);
        self.prune_recent_contacts();
        self.auto_wall_side = sensors.wall_side;
        self.auto_contact_type = Some(if sensors.wall_side.is_some() {
            ContactType::Wall
        } else {
            sensors.contact_type.unwrap_or(ContactType::Unknown)
        });
    }

    fn prune_recent_contacts(&mut self) {
        let now = self.clock.now_millis();
        self.auto_recent_contact_times
            .retain(|timestamp| now.saturating_sub(*timestamp) <= ANTI_LOOP_CONTACT_WINDOW_MS);
    }

}

fn choose_clearance_turn_right(
    wall_side: Option<WallSide>,
    contact_type: Option<ContactType>,
    fallback_turn_right: bool,
) -> bool {
    match wall_side {
        Some(WallSide::Right) => false,
        Some(WallSide::Left) => true,
        Some(WallSide::Top) | Some(WallSide::Bottom) => fallback_turn_right,
        None => match contact_type {
            Some(ContactType::Wall) => fallback_turn_right,
            Some(ContactType::Obstacle) => fallback_turn_right,
            Some(ContactType::Unknown) | None => fallback_turn_right,
        },
    }
}

fn choose_wall_recovery_turn_right(
    wall_side: Option<WallSide>,
    fallback_turn_right: bool,
    repeated_same_wall_count: u8,
) -> bool {
    let preferred = match wall_side {
        Some(WallSide::Right) => false,
        Some(WallSide::Left) => true,
        Some(WallSide::Top) | Some(WallSide::Bottom) | None => fallback_turn_right,
    };

    if repeated_same_wall_count >= 2 {
        !preferred
    } else {
        preferred
    }
}
