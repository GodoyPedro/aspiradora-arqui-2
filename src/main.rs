use std::sync::Arc;

use robot_vacuum_firmware::api::{create_router, SharedSimulationController};
use robot_vacuum_firmware::simulation::{create_simulation_controller, SimulationConfig};
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[tokio::main]
async fn main() {
    let controller = create_simulation_controller(SimulationConfig::default());
    let shared: SharedSimulationController = Arc::new(Mutex::new(controller));
    let app = create_router(shared);

    let listener = TcpListener::bind("127.0.0.1:3000")
        .await
        .expect("failed to bind HTTP listener");

    println!("Robot vacuum firmware simulator listening on http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .expect("axum server failed");
}
