pub mod drivers;

use crate::application::RobotController;

pub use drivers::*;

#[derive(Clone, Copy, Debug)]
pub struct SimulationConfig {
    pub initial_battery_percent: u8,
    pub dock_available: bool,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            initial_battery_percent: 80,
            dock_available: true,
        }
    }
}

pub type SimulationRobotController = RobotController<
    SimulatedWheelMotorDriver,
    SimulatedSuctionDriver,
    SimulatedBrushDriver,
    SimulatedSensorReader,
    SimulatedBatteryDriver,
    SimulatedDockingDriver,
    SimulatedClock,
>;

pub fn create_simulation_controller(config: SimulationConfig) -> SimulationRobotController {
    RobotController::new(
        SimulatedWheelMotorDriver::default(),
        SimulatedSuctionDriver::default(),
        SimulatedBrushDriver::default(),
        SimulatedSensorReader::default(),
        SimulatedBatteryDriver::new(config.initial_battery_percent),
        SimulatedDockingDriver::new(config.dock_available),
        SimulatedClock::default(),
    )
}
