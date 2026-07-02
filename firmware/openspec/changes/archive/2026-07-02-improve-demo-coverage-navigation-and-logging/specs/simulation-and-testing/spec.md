## MODIFIED Requirements

### Requirement: Firmware shall include the minimum required unit coverage
The test suite SHALL cover starting cleaning from `Standby`, stopping cleaning from `Cleaning`, pausing cleaning from `Cleaning`, manual movement changing wheel speeds, low battery triggering `ReturningToDock`, dock detection transitioning `ReturningToDock` to `Charging`, each required critical sensor fault transitioning to `Error` and stopping all actuators, clearing an error when safe, rejecting unsupported cleaning modes, and rejecting or safely ignoring invalid command/state combinations.

#### Scenario: AUTO bumper turn regression test
- **WHEN** a unit test activates `bumper_pressed` while the robot is in `Cleaning/AUTO`
- **THEN** the test SHALL verify that the controller performs a turn maneuver while remaining in `Cleaning` and without entering `Error`

#### Scenario: AUTO resumes forward after turn regression test
- **WHEN** a unit test clears the blocking condition after the configured turn period
- **THEN** the test SHALL verify that the controller resumes forward AUTO movement

#### Scenario: Repeated bumper events do not cause permanent spinning
- **WHEN** a unit test injects repeated collisions within the anti-loop window
- **THEN** the test SHALL verify that the controller eventually performs the deterministic escape behavior rather than spinning forever with one unchanged turn pattern

#### Scenario: Normal obstacle events do not leave AUTO stopped
- **WHEN** a unit test injects a normal obstacle or bumper event during `Cleaning/AUTO`
- **THEN** the test SHALL verify that the robot does not remain stopped indefinitely and resumes or continues movement according to the simple turn behavior

### Requirement: Firmware shall include the minimum required HTTP integration coverage
The test suite SHALL include HTTP integration tests that exercise the in-memory `axum` router directly, use isolated application state per test, and validate status codes, JSON bodies, state transitions, actuator state, and structured error responses for the documented API.

#### Scenario: Reset endpoint integration coverage
- **WHEN** the HTTP integration suite sends `POST /simulation/reset`
- **THEN** it SHALL verify HTTP `200`, the initial safe `RobotStatus`, and that prior demo-only simulated state is cleared

#### Scenario: Log dump success integration coverage
- **WHEN** the HTTP integration suite sends `POST /simulation/log-dump` with a valid payload
- **THEN** it SHALL verify a success response containing `session_id` and a persisted stable path inside the demo log directory for that session

#### Scenario: Log dump validation integration coverage
- **WHEN** the HTTP integration suite sends malformed JSON or an invalid log dump payload
- **THEN** it SHALL verify HTTP `400` and that no file is created

## ADDED Requirements

### Requirement: Demo verification shall cover local simulator controls
Manual verification of the plain JavaScript demo SHALL confirm coverage painting, speed control, reset behavior, new map generation, session rotation, and that generated log dumps include map, movement timeline, and coverage data.

#### Scenario: Frontend manual verification checklist
- **WHEN** the implementation is reviewed before merge
- **THEN** the verification notes SHALL confirm that speed changes affect visual movement, reset clears coverage and movement history while keeping the same map/session, generating a new map creates a new session id, the current session id stays visible in the UI, coverage updates while moving, and generated dumps contain the expected reconstruction and analysis fields
