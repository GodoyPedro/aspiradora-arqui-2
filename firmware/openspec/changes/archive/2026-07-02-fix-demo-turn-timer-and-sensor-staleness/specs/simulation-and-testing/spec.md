## MODIFIED Requirements

### Requirement: Firmware shall include the minimum required unit coverage
The test suite SHALL cover starting cleaning from `Standby`, stopping cleaning from `Cleaning`, pausing cleaning from `Cleaning`, manual movement changing wheel speeds, low battery triggering `ReturningToDock`, dock detection transitioning `ReturningToDock` to `Charging`, each required critical sensor fault transitioning to `Error` and stopping all actuators, clearing an error when safe, rejecting unsupported cleaning modes, and rejecting or safely ignoring invalid command/state combinations.

#### Scenario: AUTO turn expiry regression test
- **WHEN** a unit test starts an `AUTO` turn via obstacle or bumper, advances simulated time beyond the turn window, and clears the blocker
- **THEN** the test SHALL verify that the robot resumes forward movement instead of spinning indefinitely

#### Scenario: Manual deadline expiry regression test
- **WHEN** a unit test starts a manual move and then advances simulated time beyond the manual deadline
- **THEN** the test SHALL verify transition back to `Standby` with stopped wheels

### Requirement: Firmware shall include the minimum required HTTP integration coverage
The test suite SHALL include HTTP integration tests that exercise the in-memory `axum` router directly, use isolated application state per test, and validate status codes, JSON bodies, state transitions, actuator state, and structured error responses for the documented API.

#### Scenario: Tick delta integration coverage
- **WHEN** the HTTP integration suite sends `POST /simulation/tick` with a valid `delta_ms`
- **THEN** it SHALL verify that simulated time advances and the endpoint returns `RobotStatus`

#### Scenario: Invalid tick delta integration coverage
- **WHEN** the HTTP integration suite sends `POST /simulation/tick` with invalid `delta_ms`
- **THEN** it SHALL verify HTTP `400` and unchanged timing behavior

#### Scenario: Tick compatibility integration coverage
- **WHEN** the HTTP integration suite sends `POST /simulation/tick` without a payload
- **THEN** it SHALL verify that the endpoint remains backward-compatible and still returns `RobotStatus`

## ADDED Requirements

### Requirement: Demo verification shall cover finite turning and clean event logs
Manual verification of the plain JavaScript demo SHALL confirm that turning is finite, forward movement resumes, stale sensors clear on stop/reset, and the resulting log dump no longer shows fake anti-loop events every frame.

#### Scenario: Frontend manual verification checklist
- **WHEN** the implementation is reviewed before merge
- **THEN** the verification notes SHALL confirm that the robot turns for finite time, resumes movement with changing `x/y`, Stop clears stale obstacle/bumper flags, and the log shows bounded `TURN_START` plus ordinary turning frames rather than endless repeated fake escape events
