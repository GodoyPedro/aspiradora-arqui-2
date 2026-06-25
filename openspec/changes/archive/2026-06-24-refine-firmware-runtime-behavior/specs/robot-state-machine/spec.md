## MODIFIED Requirements

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
