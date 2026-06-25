## Purpose

Define the in-memory simulation model, deterministic helper behavior, and minimum test coverage expectations for the firmware.

## Requirements

### Requirement: Firmware shall provide in-memory simulated drivers
The firmware SHALL implement `SimulatedWheelMotorDriver`, `SimulatedSuctionDriver`, `SimulatedBrushDriver`, `SimulatedSensorReader`, `SimulatedBatteryDriver`, and `SimulatedDockingDriver` using in-memory state that can be inspected and mutated during tests.

#### Scenario: Simulator stores actuator state
- **WHEN** the controller commands wheel, suction, or brush changes
- **THEN** the corresponding simulated driver SHALL retain the new state in memory for later status reporting and test assertions

### Requirement: Simulated drivers shall expose the required state surface
The simulated drivers SHALL model at least `left_speed`, `right_speed`, `running` for suction, `running` for brushes, sensor flags for obstacle, drop-off, bumper, dust container full, wheel stuck, brush stuck, and top cover open, battery `percentage`, charging state, and docking availability or detection state through `SimulatedDockingDriver`.

#### Scenario: Test injects sensor fault
- **WHEN** a unit test sets `brush_stuck` to true in the simulated sensor reader
- **THEN** the next firmware evaluation SHALL observe that flag through the normal `SensorReader` interface

### Requirement: Status reporting shall expose required telemetry
`RobotStatus` SHALL include at least `state`, `cleaning_mode`, `battery_percent`, `is_charging`, `suction_enabled`, `brushes_enabled`, `left_wheel_speed`, `right_wheel_speed`, `current_error`, and the relevant sensor flags needed to understand the robot's current condition.

#### Scenario: Status reflects simulated state
- **WHEN** the robot is queried after command execution or a periodic tick
- **THEN** the returned `RobotStatus` SHALL reflect the current driver state, battery state, robot state, and active error consistently

#### Scenario: Status remains readable in error conditions
- **WHEN** the robot is already in `Error` or a critical condition is active
- **THEN** reading status SHALL still return the current `RobotStatus` instead of failing with a conflict

### Requirement: Simulated defaults and helpers shall support deterministic tests
Simulated battery initialization SHALL be configurable with a default suitable for tests. Simulation mutation helpers MAY be provided for tests or demos, but they SHALL remain separate from the main command protocol.

#### Scenario: Test configures startup battery
- **WHEN** a test initializes the simulated battery driver with an explicit percentage
- **THEN** the first reported `RobotStatus` SHALL reflect that configured battery percentage

#### Scenario: HTTP test mutates simulated state without public endpoints
- **WHEN** an HTTP integration test needs to inject sensor, battery, or docking conditions
- **THEN** the test SHALL do so through test-only access to the in-memory simulation state rather than through public HTTP endpoints

### Requirement: Firmware shall include the minimum required unit coverage
The test suite SHALL cover starting cleaning from `Standby`, stopping cleaning from `Cleaning`, pausing cleaning from `Cleaning`, manual movement changing wheel speeds, low battery triggering `ReturningToDock`, dock detection transitioning `ReturningToDock` to `Charging`, each required critical sensor fault transitioning to `Error` and stopping all actuators, clearing an error when safe, rejecting unsupported cleaning modes, and rejecting or safely ignoring invalid command/state combinations.

#### Scenario: Safety regression test
- **WHEN** a unit test activates `top_cover_open` while the robot is operating
- **THEN** the test SHALL verify transition to `Error` and that wheels, suction, and brushes are all stopped

#### Scenario: Status read regression test
- **WHEN** a unit or HTTP test requests `GET /status` while the robot is already in `Error`
- **THEN** the test SHALL verify that the request succeeds and returns the current `RobotStatus`

#### Scenario: AUTO obstacle reaction regression test
- **WHEN** a unit test activates `obstacle_detected` or `bumper_pressed` while the robot is in `Cleaning`
- **THEN** the test SHALL verify that wheel speeds change to the avoidance maneuver while `state = Cleaning`

#### Scenario: Dust container full regression test
- **WHEN** a unit test activates `dust_container_full` while the robot is in `Cleaning`
- **THEN** the test SHALL verify transition to `Error`, `current_error = DUST_CONTAINER_FULL`, and that wheels, suction, and brushes are all stopped

#### Scenario: Invalid manual speed regression test
- **WHEN** a unit or HTTP test sends `MANUAL_MOVE` with invalid speed or an invalid direction and zero-speed combination
- **THEN** the test SHALL verify a structured bad-request failure and unchanged robot state

### Requirement: Firmware shall include the minimum required HTTP integration coverage
The test suite SHALL include HTTP integration tests that exercise the in-memory `axum` router directly, use isolated application state per test, and validate status codes, JSON bodies, state transitions, actuator state, and structured error responses for the documented API.

#### Scenario: Status endpoint integration coverage
- **WHEN** the HTTP integration suite runs
- **THEN** it SHALL verify that `GET /status` returns HTTP `200`, exposes the initial `Standby` state and required telemetry fields, remains readable in `Error`, and does not mutate robot state

#### Scenario: Command success-path integration coverage
- **WHEN** the HTTP integration suite runs
- **THEN** it SHALL verify documented success-path behavior for `start`, `stop`, `pause`, `return-to-dock`, `manual-move`, `mode`, and `clear-error`

#### Scenario: Command validation integration coverage
- **WHEN** the HTTP integration suite runs
- **THEN** it SHALL verify malformed JSON, missing fields, invalid manual speed inputs, invalid direction values, and unsupported mode requests return the documented HTTP `400` behavior

#### Scenario: Invalid-state integration coverage
- **WHEN** the HTTP integration suite runs
- **THEN** it SHALL verify disallowed commands such as invalid pause, manual movement while `Charging` or `Error`, and other documented state conflicts return HTTP `409` with structured error payloads

#### Scenario: Safety and runtime visibility integration coverage
- **WHEN** the HTTP integration suite injects critical sensor conditions and triggers `tick()`
- **THEN** it SHALL verify `/status` reflects `Error`, the correct `current_error`, and stopped actuators for the documented safety cases

#### Scenario: Battery and docking visibility integration coverage
- **WHEN** the HTTP integration suite injects low battery or docking conditions and triggers `tick()`
- **THEN** it SHALL verify `/status` reflects the documented `Cleaning`, `ReturningToDock`, `Charging`, and `Standby` transitions visible to an external API client

#### Scenario: Cargo test execution target
- **WHEN** the full Rust test suite is run with `cargo test`
- **THEN** the HTTP integration tests SHALL run deterministically without requiring a real network port
