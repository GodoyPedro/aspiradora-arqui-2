use crate::domain::{ContactType, SensorSnapshot, WallSide};
use crate::hal::{
    BatteryDriver, BatteryStatus, BrushDriver, Clock, DockingDriver, DockingStatus, SensorReader,
    SuctionDriver, WheelMotorDriver,
};

#[derive(Clone, Debug, Default)]
pub struct SimulatedWheelMotorDriver {
    pub left_speed: i16,
    pub right_speed: i16,
}

impl WheelMotorDriver for SimulatedWheelMotorDriver {
    fn set_wheel_speeds(&mut self, left_speed: i16, right_speed: i16) {
        self.left_speed = left_speed;
        self.right_speed = right_speed;
    }

    fn wheel_speeds(&self) -> (i16, i16) {
        (self.left_speed, self.right_speed)
    }
}

#[derive(Clone, Debug, Default)]
pub struct SimulatedSuctionDriver {
    pub running: bool,
}

impl SuctionDriver for SimulatedSuctionDriver {
    fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

#[derive(Clone, Debug, Default)]
pub struct SimulatedBrushDriver {
    pub running: bool,
}

impl BrushDriver for SimulatedBrushDriver {
    fn set_running(&mut self, running: bool) {
        self.running = running;
    }

    fn is_running(&self) -> bool {
        self.running
    }
}

#[derive(Clone, Debug, Default)]
pub struct SimulatedSensorReader {
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

impl SimulatedSensorReader {
    pub fn set_snapshot(&mut self, snapshot: SensorSnapshot) {
        self.obstacle_detected = snapshot.obstacle_detected;
        self.drop_off_detected = snapshot.drop_off_detected;
        self.bumper_pressed = snapshot.bumper_pressed;
        self.proximity_contact = snapshot.proximity_contact;
        self.contact_type = snapshot.contact_type;
        self.wall_side = snapshot.wall_side;
        self.forward_clearance_blocked = snapshot.forward_clearance_blocked;
        self.dust_container_full = snapshot.dust_container_full;
        self.wheel_stuck = snapshot.wheel_stuck;
        self.brush_stuck = snapshot.brush_stuck;
        self.top_cover_open = snapshot.top_cover_open;
    }
}

impl SensorReader for SimulatedSensorReader {
    fn read_sensors(&self) -> SensorSnapshot {
        SensorSnapshot {
            obstacle_detected: self.obstacle_detected,
            drop_off_detected: self.drop_off_detected,
            bumper_pressed: self.bumper_pressed,
            proximity_contact: self.proximity_contact,
            contact_type: self.contact_type,
            wall_side: self.wall_side,
            forward_clearance_blocked: self.forward_clearance_blocked,
            dust_container_full: self.dust_container_full,
            wheel_stuck: self.wheel_stuck,
            brush_stuck: self.brush_stuck,
            top_cover_open: self.top_cover_open,
        }
    }
}

#[derive(Clone, Debug)]
pub struct SimulatedBatteryDriver {
    pub percentage: u8,
    pub charging: bool,
}

impl SimulatedBatteryDriver {
    pub fn new(percentage: u8) -> Self {
        Self {
            percentage,
            charging: false,
        }
    }

    pub fn set_percentage(&mut self, percentage: u8) {
        self.percentage = percentage.min(100);
    }
}

impl BatteryDriver for SimulatedBatteryDriver {
    fn read_battery(&self) -> BatteryStatus {
        BatteryStatus {
            percentage: self.percentage,
            charging: self.charging,
        }
    }

    fn set_charging(&mut self, charging: bool) {
        self.charging = charging;
    }
}

#[derive(Clone, Debug)]
pub struct SimulatedDockingDriver {
    pub dock_available: bool,
    pub dock_detected: bool,
}

impl SimulatedDockingDriver {
    pub fn new(dock_available: bool) -> Self {
        Self {
            dock_available,
            dock_detected: false,
        }
    }

    pub fn set_dock_detected(&mut self, dock_detected: bool) {
        self.dock_detected = dock_detected;
    }

    pub fn set_dock_available(&mut self, dock_available: bool) {
        self.dock_available = dock_available;
    }
}

impl DockingDriver for SimulatedDockingDriver {
    fn read_docking(&self) -> DockingStatus {
        DockingStatus {
            dock_available: self.dock_available,
            dock_detected: self.dock_detected,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SimulatedClock {
    pub current_time_ms: u64,
}

impl SimulatedClock {
    pub fn advance_ms(&mut self, delta_ms: u64) {
        self.current_time_ms = self.current_time_ms.saturating_add(delta_ms);
    }

    pub fn set_time_ms(&mut self, value: u64) {
        self.current_time_ms = value;
    }
}

impl Clock for SimulatedClock {
    fn now_millis(&self) -> u64 {
        self.current_time_ms
    }
}
