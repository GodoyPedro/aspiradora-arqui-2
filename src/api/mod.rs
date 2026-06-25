pub mod error;
pub mod models;
pub mod routes;

use std::sync::Arc;

use tokio::sync::Mutex;

pub use error::*;
pub use models::*;
pub use routes::*;

use crate::simulation::SimulationRobotController;

pub type SharedSimulationController = Arc<Mutex<SimulationRobotController>>;
