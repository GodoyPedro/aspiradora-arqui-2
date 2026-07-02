## MODIFIED Requirements

### Requirement: Battery and docking rules shall drive automatic transitions
The robot SHALL transition from `Cleaning` to `ReturningToDock` when battery percentage is less than or equal to `15`. The robot SHALL transition from `ReturningToDock` to `Charging` when docking is detected through `DockingDriver`. The robot MAY transition from `Charging` to `Standby` when battery percentage reaches `100`.

#### Scenario: Return-to-dock command enters returning state from allowed states
- **WHEN** a client sends `POST /commands/return-to-dock` from a controller state that the implementation allows for docking
- **THEN** the backend SHALL transition the robot to `ReturningToDock`

#### Scenario: Return-to-dock command is rejected clearly from refused states
- **WHEN** a client sends `POST /commands/return-to-dock` from a controller state that the implementation does not allow for docking
- **THEN** the backend SHALL return a clear error response and SHALL NOT change the current state

#### Scenario: Dock detection completes charging transition through simulation tick
- **WHEN** the robot is in `ReturningToDock`, `/simulation/docking` sets `dock_detected = true`, and `/simulation/tick` runs
- **THEN** the backend SHALL transition the robot to `Charging`

#### Scenario: Charging stops wheel motion
- **WHEN** the robot has transitioned to `Charging`
- **THEN** the reported wheel speeds SHALL be `0/0`
