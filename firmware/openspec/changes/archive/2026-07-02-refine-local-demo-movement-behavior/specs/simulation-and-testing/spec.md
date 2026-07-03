## MODIFIED Requirements

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

#### Scenario: AUTO movement recovery regression test
- **WHEN** a unit or HTTP-visible test clears `obstacle_detected` and `bumper_pressed` after an avoidance maneuver and then triggers `tick()`
- **THEN** the test SHALL verify that the robot remains in `Cleaning` and that forward AUTO wheel speeds are restored without changing unrelated command behavior

#### Scenario: Dust container full regression test
- **WHEN** a unit test activates `dust_container_full` while the robot is in `Cleaning`
- **THEN** the test SHALL verify transition to `Error`, `current_error = DUST_CONTAINER_FULL`, and that wheels, suction, and brushes are all stopped

#### Scenario: Invalid manual speed regression test
- **WHEN** a unit or HTTP test sends `MANUAL_MOVE` with invalid speed or an invalid direction and zero-speed combination
- **THEN** the test SHALL verify a structured bad-request failure and unchanged robot state
