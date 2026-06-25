use crate::domain::SensorSnapshot;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BatteryStatus {
    pub percentage: u8,
    pub charging: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DockingStatus {
    pub dock_available: bool,
    pub dock_detected: bool,
}

pub trait WheelMotorDriver {
    fn set_wheel_speeds(&mut self, left_speed: i16, right_speed: i16);
    fn wheel_speeds(&self) -> (i16, i16);

    fn stop(&mut self) {
        self.set_wheel_speeds(0, 0);
    }
}

pub trait SuctionDriver {
    fn set_running(&mut self, running: bool);
    fn is_running(&self) -> bool;
}

pub trait BrushDriver {
    fn set_running(&mut self, running: bool);
    fn is_running(&self) -> bool;
}

pub trait SensorReader {
    fn read_sensors(&self) -> SensorSnapshot;
}

pub trait BatteryDriver {
    fn read_battery(&self) -> BatteryStatus;
    fn set_charging(&mut self, charging: bool);
}

pub trait DockingDriver {
    fn read_docking(&self) -> DockingStatus;
}

pub trait Clock {
    fn now_millis(&self) -> u64;
}
