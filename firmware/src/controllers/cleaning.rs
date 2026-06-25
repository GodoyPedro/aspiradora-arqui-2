use crate::domain::SensorSnapshot;
use crate::hal::{BrushDriver, SuctionDriver};

#[derive(Debug, Default)]
pub struct CleaningController;

impl CleaningController {
    pub fn start_cleaning<S: SuctionDriver, B: BrushDriver>(
        &self,
        suction_driver: &mut S,
        brush_driver: &mut B,
    ) {
        suction_driver.set_running(true);
        brush_driver.set_running(true);
    }

    pub fn stop_cleaning<S: SuctionDriver, B: BrushDriver>(
        &self,
        suction_driver: &mut S,
        brush_driver: &mut B,
    ) {
        suction_driver.set_running(false);
        brush_driver.set_running(false);
    }

    pub fn react_to_dust_container_full<S: SuctionDriver, B: BrushDriver>(
        &self,
        sensors: &SensorSnapshot,
        suction_driver: &mut S,
        brush_driver: &mut B,
    ) {
        if sensors.dust_container_full {
            self.stop_cleaning(suction_driver, brush_driver);
        }
    }
}
