## 1. Rust Project Setup

- [x] 1.1 Create the Rust crate structure and top-level modules for API, application, domain, HAL, simulation, and tests
- [x] 1.2 Add `axum`, serialization, and testing dependencies needed for the first firmware implementation
- [x] 1.3 Add a bootstrap entrypoint that wires the controller with simulated drivers for local execution

## 2. Core Robot Models

- [x] 2.1 Define `RobotState`, `RobotCommand`, `RobotError`, `CleaningMode`, `ManualDirection`, `SensorSnapshot`, and `RobotStatus`, including `Standby` runtime initialization and future-only `Off`
- [x] 2.2 Define validation rules and shared result/error types for command handling, structured HTTP error bodies, and state transition outcomes
- [x] 2.3 Add serialization contracts for the HTTP request and response models without leaking transport types into the domain layer

## 3. Hardware Abstraction Traits

- [x] 3.1 Define the `WheelMotorDriver`, `SuctionDriver`, `BrushDriver`, `SensorReader`, `BatteryDriver`, `DockingDriver`, and `Clock` traits
- [x] 3.2 Define trait-friendly data types and method signatures for wheel speeds, battery readings, docking state, and time access, with dock detection owned only by `DockingDriver`
- [x] 3.3 Add unit tests or compile-time checks that confirm controllers can depend on traits rather than simulator concretes

## 4. Simulated Drivers

- [x] 4.1 Implement `SimulatedWheelMotorDriver`, `SimulatedSuctionDriver`, and `SimulatedBrushDriver` with in-memory actuator state
- [x] 4.2 Implement `SimulatedSensorReader`, `SimulatedBatteryDriver`, and `SimulatedDockingDriver` with mutable in-memory readings and configurable default battery state
- [x] 4.3 Implement a simulated or controllable `Clock` for deterministic time-based tests
- [x] 4.4 Add simulation mutation helpers for tests or demos without extending the main command protocol

## 5. RobotController And State Machine

- [x] 5.1 Implement the robot state machine with the required states, explicit legal transitions, `Standby` initialization, and unreachable `Off` through HTTP
- [x] 5.2 Implement `RobotController` command handling for `START_CLEANING`, `STOP_CLEANING`, `PAUSE_CLEANING`, `RETURN_TO_DOCK`, `MANUAL_MOVE`, `SET_CLEANING_MODE`, `CLEAR_ERROR`, and `GET_STATUS`
- [x] 5.3 Enforce invalid command/state handling so valid-but-disallowed operations produce structured conflicts and unsupported modes produce `400` responses

## 6. Domain Controllers

- [x] 6.1 Implement `MotionController` for directional movement, stop behavior, and wheel speed management
- [x] 6.2 Implement `CleaningController` for suction, side brushes, and dust-container-full reactions
- [x] 6.3 Implement `BatteryManager`, `DockingManager`, and `SafetyManager` for battery evaluation, dock transitions, and critical fault shutdown

## 7. Tick Firmware Loop

- [x] 7.1 Add the periodic `tick()` flow that reads the latest `SensorSnapshot`, docking state, and current hardware state
- [x] 7.2 Apply safety checks in `tick()` for drop-off, wheel stuck, brush stuck, and top-cover-open conditions
- [x] 7.3 Apply battery and docking checks in `tick()` for low-battery return, dock detection, charging, and full-battery standby behavior

## 8. HTTP And JSON Command Interface

- [x] 8.1 Implement the required `axum` HTTP endpoints and request models for all command routes and `GET /status`
- [x] 8.2 Map validated HTTP requests to internal `RobotCommand` values, require `duration_ms` for `MANUAL_MOVE`, and serialize `RobotStatus` responses consistently
- [x] 8.3 Return HTTP `400` for malformed JSON or unsupported payload values and HTTP `409` with a structured error body for valid but disallowed operations

## 9. Unit Tests

- [x] 9.1 Add tests for start, stop, pause, and manual movement behavior
- [x] 9.2 Add tests for low battery, return-to-dock, dock detection, charging, and safe clear-error recovery
- [x] 9.3 Add tests for drop-off, wheel stuck, brush stuck, top cover open, unsupported mode rejection, and invalid command/state combinations

## 10. README And Usage Examples

- [x] 10.1 Document the project architecture, scope, and module responsibilities in the README
- [x] 10.2 Document example HTTP requests and JSON responses for each supported endpoint
- [x] 10.3 Document simulator usage, supported cleaning modes, and explicitly out-of-scope future extensions
