## MODIFIED Requirements

### Requirement: Simulated defaults and helpers shall support deterministic tests
Simulated battery initialization SHALL be configurable with a default suitable for tests. Simulation mutation helpers MAY be provided for tests or demos, but they SHALL remain separate from the main command protocol.

#### Scenario: Test configures startup battery
- **WHEN** a test initializes the simulated battery driver with an explicit percentage
- **THEN** the first reported `RobotStatus` SHALL reflect that configured battery percentage

#### Scenario: HTTP test mutates simulated state without public endpoints
- **WHEN** an HTTP integration test needs to inject sensor, battery, or docking conditions
- **THEN** the test SHALL do so through test-only access to the in-memory simulation state rather than through public HTTP endpoints

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
