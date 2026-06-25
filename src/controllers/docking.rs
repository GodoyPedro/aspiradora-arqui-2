use crate::hal::{DockingDriver, DockingStatus, WheelMotorDriver};

#[derive(Debug, Default)]
pub struct DockingManager;

impl DockingManager {
    pub fn start_return_to_dock<W: WheelMotorDriver>(&self, wheel_driver: &mut W) {
        wheel_driver.set_wheel_speeds(30, 30);
    }

    pub fn read<D: DockingDriver>(&self, docking_driver: &D) -> DockingStatus {
        docking_driver.read_docking()
    }
}
