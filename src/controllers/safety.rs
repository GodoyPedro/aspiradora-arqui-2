use crate::domain::{RobotError, SensorSnapshot};
use crate::hal::{BrushDriver, SuctionDriver, WheelMotorDriver};

#[derive(Debug, Default)]
pub struct SafetyManager;

impl SafetyManager {
    pub fn detect_critical_error(&self, sensors: &SensorSnapshot) -> Option<RobotError> {
        if sensors.drop_off_detected {
            Some(RobotError::DropOffDetected)
        } else if sensors.wheel_stuck {
            Some(RobotError::WheelStuck)
        } else if sensors.brush_stuck {
            Some(RobotError::BrushStuck)
        } else if sensors.top_cover_open {
            Some(RobotError::TopCoverOpen)
        } else {
            None
        }
    }

    pub fn stop_all<W: WheelMotorDriver, S: SuctionDriver, B: BrushDriver>(
        &self,
        wheel_driver: &mut W,
        suction_driver: &mut S,
        brush_driver: &mut B,
    ) {
        wheel_driver.stop();
        suction_driver.set_running(false);
        brush_driver.set_running(false);
    }
}
