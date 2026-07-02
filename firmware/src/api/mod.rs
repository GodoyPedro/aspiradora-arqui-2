pub mod demo;
pub mod error;
pub mod models;
pub mod routes;

use std::path::PathBuf;
use std::sync::Arc;

use tokio::sync::Mutex;

pub use error::*;
pub use models::*;
pub use routes::*;

use crate::simulation::{
    create_simulation_controller, SimulationConfig, SimulationRobotController,
};

pub type SharedSimulationController = Arc<Mutex<SimulationRobotController>>;

#[derive(Clone)]
pub struct AppState {
    pub controller: SharedSimulationController,
    pub simulation_config: SimulationConfig,
    pub demo_log_root: Arc<PathBuf>,
}

impl AppState {
    pub fn new(simulation_config: SimulationConfig) -> Self {
        Self::with_log_root(simulation_config, PathBuf::from("demo-logs"))
    }

    pub fn with_log_root(simulation_config: SimulationConfig, demo_log_root: PathBuf) -> Self {
        let controller = create_simulation_controller(simulation_config);
        Self {
            controller: Arc::new(Mutex::new(controller)),
            simulation_config,
            demo_log_root: Arc::new(demo_log_root),
        }
    }
}
