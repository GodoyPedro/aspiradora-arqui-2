## ADDED Requirements

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

### Requirement: Simulated defaults and helpers shall support deterministic tests
Simulated battery initialization SHALL be configurable with a default suitable for tests. Simulation mutation helpers MAY be provided for tests or demos, but they SHALL remain separate from the main command protocol.

#### Scenario: Test configures startup battery
- **WHEN** a test initializes the simulated battery driver with an explicit percentage
- **THEN** the first reported `RobotStatus` SHALL reflect that configured battery percentage

### Requirement: Firmware shall include the minimum required unit coverage
The test suite SHALL cover starting cleaning from `Standby`, stopping cleaning from `Cleaning`, pausing cleaning from `Cleaning`, manual movement changing wheel speeds, low battery triggering `ReturningToDock`, dock detection transitioning `ReturningToDock` to `Charging`, each required critical sensor fault transitioning to `Error` and stopping all actuators, clearing an error when safe, rejecting unsupported cleaning modes, and rejecting or safely ignoring invalid command/state combinations.

#### Scenario: Safety regression test
- **WHEN** a unit test activates `top_cover_open` while the robot is operating
- **THEN** the test SHALL verify transition to `Error` and that wheels, suction, and brushes are all stopped
