## MODIFIED Requirements

### Requirement: Simulated defaults and helpers shall support deterministic tests
Simulated battery initialization SHALL be configurable with a default suitable for tests. Simulation mutation helpers MAY be provided for tests or demos, but they SHALL remain separate from the main command protocol. The firmware MAY expose demo/testing-only HTTP helper endpoints for mutating simulated sensors, battery, docking, and for triggering `tick()`, provided those endpoints operate only on the in-memory simulation state.

#### Scenario: Test configures startup battery
- **WHEN** a test initializes the simulated battery driver with an explicit percentage
- **THEN** the first reported `RobotStatus` SHALL reflect that configured battery percentage

#### Scenario: HTTP test mutates simulated state through the allowed helper mechanisms
- **WHEN** an HTTP integration test needs to inject sensor, battery, or docking conditions
- **THEN** the test MAY use either in-memory test helpers or the documented demo/testing-only `/simulation/*` endpoints, while keeping `/simulation/*` outside the primary `/commands/*` and `/status` command API

#### Scenario: Demo helper endpoints mutate in-memory simulation state
- **WHEN** the local demo calls `/simulation/sensors`, `/simulation/battery`, or `/simulation/docking`
- **THEN** those endpoints SHALL mutate only the shared simulated backend state rather than introducing a second source of truth

### Requirement: Firmware shall include the minimum required HTTP integration coverage
The test suite SHALL include HTTP integration tests that exercise the in-memory `axum` router directly, use isolated application state per test, and validate status codes, JSON bodies, state transitions, actuator state, and structured error responses for the documented API. The suite SHALL also validate the demo/testing helper endpoints under `/simulation/*`.

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

#### Scenario: Simulation helper integration coverage
- **WHEN** the HTTP integration suite exercises `/simulation/sensors`, `/simulation/battery`, `/simulation/docking`, and `/simulation/tick`
- **THEN** it SHALL verify those endpoints mutate visible simulated state and that `POST /simulation/tick` returns a `RobotStatus`

#### Scenario: Simulation helper validation coverage
- **WHEN** the HTTP integration suite sends malformed JSON, invalid payload shapes, or invalid battery values to `/simulation/*`
- **THEN** it SHALL verify HTTP `400` responses use the existing structured error body format and do not mutate simulation state unexpectedly

#### Scenario: Cargo test execution target
- **WHEN** the full Rust test suite is run with `cargo test`
- **THEN** the HTTP integration tests SHALL run deterministically without requiring a real network port
