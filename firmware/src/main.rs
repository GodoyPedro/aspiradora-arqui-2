use robot_vacuum_firmware::api::{create_router, AppState};
use robot_vacuum_firmware::simulation::SimulationConfig;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
    let app = create_router(AppState::new(SimulationConfig::default()));

    let listener = TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("failed to bind HTTP listener");

    println!("Robot vacuum firmware simulator listening on http://127.0.0.1:3000");

    axum::serve(listener, app)
        .await
        .expect("axum server failed");
}
