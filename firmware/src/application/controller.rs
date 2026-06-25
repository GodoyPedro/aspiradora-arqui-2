use crate::controllers::{
    BatteryManager, CleaningController, DockingManager, MotionController, SafetyManager,
};
use crate::domain::{
    CleaningMode, DomainError, ManualDirection, RobotCommand, RobotError, RobotState,
    RobotStateMachine, RobotStatus,
};
use crate::hal::{
    BatteryDriver, BrushDriver, Clock, DockingDriver, SensorReader, SuctionDriver,
    WheelMotorDriver,
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
}

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

        if self.state() == RobotState::Cleaning && self.sensor_reader.read_sensors().dust_container_full
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
            let _ = self.state_machine.transition_to(RobotState::ReturningToDock);
        }

        if self.state() == RobotState::Cleaning
            && (sensors.obstacle_detected || sensors.bumper_pressed)
        {
            self.wheel_driver.set_wheel_speeds(30, -30);
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
        self.motion_controller
            .move_direction(&mut self.wheel_driver, ManualDirection::Forward, 60);
        self.cleaning_mode = CleaningMode::Auto;
        self.manual_move_deadline_ms = None;
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
        self.state_machine.transition_to(RobotState::ReturningToDock)
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
        self.safety_manager.stop_all(
            &mut self.wheel_driver,
            &mut self.suction_driver,
            &mut self.brush_driver,
        );
        self.battery_driver.set_charging(false);
        self.state_machine.transition_to(RobotState::Standby)
    }
}
