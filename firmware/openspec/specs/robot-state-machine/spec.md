## Purpose

Define the robot lifecycle states, explicit transitions, and automatic transition rules driven by safety, battery, and docking conditions.
## Requirements
### Requirement: Firmware shall implement the required robot states
The robot state machine SHALL support the states `Off`, `Standby`, `Cleaning`, `Paused`, `ManualControl`, `ReturningToDock`, `Charging`, and `Error`. In this implementation, the runtime SHALL initialize in `Standby`, and `Off` SHALL remain a future embedded power-management state that is not reachable through the HTTP command API.

#### Scenario: Initial firmware boot state
- **WHEN** the firmware initializes its state model
- **THEN** it SHALL start in `Standby` and represent that state using the supported `RobotState` values

#### Scenario: Off state is not exposed through HTTP
- **WHEN** a client uses the implemented HTTP command API
- **THEN** no supported command SHALL transition the runtime into `Off`

### Requirement: State transitions shall be explicit and validated
The state machine SHALL define the allowed transitions between states and SHALL prevent invalid transitions from being applied by commands or periodic updates.

#### Scenario: Pause from cleaning
- **WHEN** the robot is in `Cleaning` and receives `PAUSE_CLEANING`
- **THEN** the state machine SHALL allow a transition to `Paused`

#### Scenario: Invalid transition attempt
- **WHEN** the robot is in `Charging` and receives a command that requires movement before leaving the dock safely
- **THEN** the state machine SHALL reject or safely defer that transition according to controller policy

#### Scenario: Dust container full during cleaning
- **WHEN** the robot is in `Cleaning` and `dust_container_full` becomes true
- **THEN** the state machine SHALL transition the robot to `Error`

### Requirement: Error recovery shall be safety-gated
The robot SHALL remain in `Error` until the current critical condition has cleared and a `CLEAR_ERROR` command is accepted, after which it SHALL transition to `Standby`.

#### Scenario: Safe error recovery
- **WHEN** the robot is in `Error`, no critical sensor condition remains active, and `CLEAR_ERROR` is received
- **THEN** the state machine SHALL clear the active error and transition the robot to `Standby`

#### Scenario: Dust container full sets persistent error state
- **WHEN** `dust_container_full` causes the robot to stop during `Cleaning`
- **THEN** the robot SHALL remain in `Error` with `current_error = DUST_CONTAINER_FULL` until the condition is cleared and `CLEAR_ERROR` is accepted

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

