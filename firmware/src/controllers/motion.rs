use crate::domain::ManualDirection;
use crate::hal::WheelMotorDriver;

#[derive(Debug, Default)]
pub struct MotionController;

impl MotionController {
    pub fn move_direction<D: WheelMotorDriver>(
        &self,
        driver: &mut D,
        direction: ManualDirection,
        speed: u8,
    ) {
        let speed = speed as i16;
        let (left, right) = match direction {
            ManualDirection::Forward => (speed, speed),
            ManualDirection::Backward => (-speed, -speed),
            ManualDirection::Left => (-speed, speed),
            ManualDirection::Right => (speed, -speed),
            ManualDirection::Stop => (0, 0),
        };

        driver.set_wheel_speeds(left, right);
    }

    pub fn stop<D: WheelMotorDriver>(&self, driver: &mut D) {
        driver.stop();
    }
}
