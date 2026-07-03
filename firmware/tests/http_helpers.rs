use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::http::{header, Method, Request, StatusCode};
use http_body_util::BodyExt;
use robot_vacuum_firmware::api::{create_router, AppState};
use robot_vacuum_firmware::domain::SensorSnapshot;
use robot_vacuum_firmware::simulation::SimulationConfig;
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use tower::ServiceExt;

#[derive(Debug, Deserialize)]
pub struct ErrorBody {
    pub code: String,
    pub message: String,
    pub current_state: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct SensorSnapshotBody {
    pub obstacle_detected: bool,
    pub drop_off_detected: bool,
    pub bumper_pressed: bool,
    pub proximity_contact: bool,
    pub contact_type: Option<String>,
    pub wall_side: Option<String>,
    #[serde(default)]
    pub forward_clearance_blocked: bool,
    pub dust_container_full: bool,
    pub wheel_stuck: bool,
    pub brush_stuck: bool,
    pub top_cover_open: bool,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct RobotStatusBody {
    pub state: String,
    pub cleaning_mode: String,
    pub battery_percent: u8,
    pub is_charging: bool,
    pub suction_enabled: bool,
    pub brushes_enabled: bool,
    pub left_wheel_speed: i16,
    pub right_wheel_speed: i16,
    pub current_error: Option<String>,
    pub auto_navigation_phase: Option<String>,
    pub sensors: SensorSnapshotBody,
}

#[derive(Debug, Deserialize)]
pub struct SavedPathBody {
    pub saved_path: String,
}

pub struct TestApp {
    state: AppState,
}

impl TestApp {
    pub fn new() -> Self {
        Self::with_config(SimulationConfig::default())
    }

    pub fn with_battery(initial_battery_percent: u8) -> Self {
        Self::with_config(SimulationConfig {
            initial_battery_percent,
            ..SimulationConfig::default()
        })
    }

    pub fn with_config(config: SimulationConfig) -> Self {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after epoch")
            .as_nanos();
        let log_root = std::env::temp_dir().join(format!("robot-vacuum-demo-logs-{unique}"));
        Self {
            state: AppState::with_log_root(config, PathBuf::from(log_root)),
        }
    }

    pub async fn get_status(&self) -> (StatusCode, RobotStatusBody) {
        let response = self.request(Method::GET, "/status", None).await;
        let status = response.status();
        let body = read_json::<RobotStatusBody>(response).await;
        (status, body)
    }

    pub async fn get(&self, path: &str) -> axum::response::Response {
        self.request(Method::GET, path, None).await
    }

    pub async fn post_empty(&self, path: &str) -> axum::response::Response {
        self.request(Method::POST, path, None).await
    }

    pub async fn post_json(&self, path: &str, body: Value) -> axum::response::Response {
        self.request(Method::POST, path, Some(body)).await
    }

    pub async fn post_raw_json(&self, path: &str, body: &str) -> axum::response::Response {
        self.request_raw(Method::POST, path, Some(body)).await
    }

    pub async fn set_sensors(&self, sensors: SensorSnapshot) {
        let mut controller = self.state.controller.lock().await;
        controller.sensor_reader_mut().set_snapshot(sensors);
    }

    pub async fn set_battery_percent(&self, percentage: u8) {
        let mut controller = self.state.controller.lock().await;
        controller.battery_driver_mut().set_percentage(percentage);
    }

    pub async fn set_dock_detected(&self, dock_detected: bool) {
        let mut controller = self.state.controller.lock().await;
        controller
            .docking_driver_mut()
            .set_dock_detected(dock_detected);
    }

    pub async fn tick(&self) {
        let mut controller = self.state.controller.lock().await;
        controller.tick();
    }

    pub fn log_root(&self) -> PathBuf {
        (*self.state.demo_log_root).clone()
    }

    async fn request(
        &self,
        method: Method,
        path: &str,
        body: Option<Value>,
    ) -> axum::response::Response {
        let body = body.map(|value| value.to_string());
        self.request_raw(method, path, body.as_deref()).await
    }

    async fn request_raw(
        &self,
        method: Method,
        path: &str,
        body: Option<&str>,
    ) -> axum::response::Response {
        let router = create_router(self.state.clone());
        let request = if let Some(body) = body {
            Request::builder()
                .method(method)
                .uri(path)
                .header("content-type", "application/json")
                .body(Body::from(body.to_owned()))
                .expect("request should build")
        } else {
            Request::builder()
                .method(method)
                .uri(path)
                .body(Body::empty())
                .expect("request should build")
        };

        router
            .oneshot(request)
            .await
            .expect("request should succeed")
    }
}

pub async fn read_json<T: DeserializeOwned>(response: axum::response::Response) -> T {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body should be readable")
        .to_bytes();

    serde_json::from_slice(&bytes).expect("body should be valid JSON")
}

pub async fn read_text(response: axum::response::Response) -> String {
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body should be readable")
        .to_bytes();

    String::from_utf8(bytes.to_vec()).expect("body should be utf-8 text")
}

pub fn content_type(response: &axum::response::Response) -> Option<&str> {
    response
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
}
