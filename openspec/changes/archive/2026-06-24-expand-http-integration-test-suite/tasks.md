## 1. HTTP Test Foundation

- [x] 1.1 Add a reusable HTTP integration test helper module that builds a fresh in-memory `axum` app with isolated simulated state
- [x] 1.2 Add helper utilities for sending JSON and empty-body requests, parsing `RobotStatus`, and reading structured error responses
- [x] 1.3 Add test-only helpers for injecting simulated sensor, battery, and docking state and for triggering `tick()` without exposing public simulation endpoints

## 2. Status And Success-Path Endpoint Tests

- [x] 2.1 Add `GET /status` integration tests for HTTP `200`, initial `Standby`, required telemetry fields, error-state readability, and non-mutation
- [x] 2.2 Add success-path integration tests for `POST /commands/start`, `POST /commands/stop`, and `POST /commands/pause`, including the documented HTTP `409` behavior for start-while-cleaning and stop-from-standby
- [x] 2.3 Add success-path integration tests for `POST /commands/return-to-dock`, `POST /commands/manual-move`, `POST /commands/mode`, and `POST /commands/clear-error`, including the documented standby return-to-dock success path and non-error clear-error conflict path

## 3. Validation And Conflict Coverage

- [x] 3.1 Add HTTP validation tests for missing manual-move fields, `duration_ms = 0`, invalid speed values, invalid direction values, malformed JSON, and low-battery start returning HTTP `409`
- [x] 3.2 Add HTTP conflict tests for invalid-state operations such as invalid pause, manual movement while `Charging` or `Error`, and other documented state conflicts
- [x] 3.3 Add structured error response assertions for HTTP `400` and HTTP `409`, including the fields `code` and `message`

## 4. Safety, Battery, And Docking Visibility

- [x] 4.1 Add HTTP-visible safety tests that inject critical sensor conditions, trigger `tick()`, and verify `/status` reflects `Error`, the correct `current_error`, and stopped actuators
- [x] 4.2 Add HTTP-visible battery and docking tests for low-battery return-to-dock, dock-detected charging, and charging-to-standby behavior where documented in the current spec
- [x] 4.3 Add HTTP tests that lock down the documented edge-case behavior for low-battery start conflict, stop from `Standby` conflict, return-to-dock from `Standby` success, and clear-error from non-error states conflict

## 5. Final Verification

- [x] 5.1 Organize the HTTP integration tests by endpoint or behavior category with descriptive names and independent state per test
- [ ] 5.2 Run `cargo test` and fix any deterministic test issues uncovered by the expanded HTTP integration suite
