## MODIFIED Requirements

### Requirement: Status reporting shall expose required telemetry
`RobotStatus` SHALL include at least `state`, `cleaning_mode`, `battery_percent`, `is_charging`, `suction_enabled`, `brushes_enabled`, `left_wheel_speed`, `right_wheel_speed`, `current_error`, and the relevant sensor flags needed to understand the robot's current condition.

#### Scenario: Status reflects simulated state
- **WHEN** the robot is queried after command execution or a periodic tick
- **THEN** the returned `RobotStatus` SHALL reflect the current driver state, battery state, robot state, and active error consistently

#### Scenario: Status remains readable in error conditions
- **WHEN** the robot is already in `Error` or a critical condition is active
- **THEN** reading status SHALL still return the current `RobotStatus` instead of failing with a conflict

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
