use crate::hal::{BatteryDriver, BatteryStatus};

#[derive(Debug, Default)]
pub struct BatteryManager;

impl BatteryManager {
    pub const LOW_BATTERY_THRESHOLD: u8 = 15;

    pub fn read<D: BatteryDriver>(&self, driver: &D) -> BatteryStatus {
        driver.read_battery()
    }

    pub fn is_low(&self, status: BatteryStatus) -> bool {
        status.percentage <= Self::LOW_BATTERY_THRESHOLD
    }

    pub fn is_full(&self, status: BatteryStatus) -> bool {
        status.percentage >= 100
    }

    pub fn is_charging(&self, status: BatteryStatus) -> bool {
        status.charging
    }
}
