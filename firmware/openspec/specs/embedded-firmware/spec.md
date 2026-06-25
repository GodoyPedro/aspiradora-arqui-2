## Purpose

Define the overall firmware architecture, runtime initialization, and scope boundaries for the robot vacuum firmware layer.

## Requirements

### Requirement: Firmware architecture shall use explicit layers
The firmware SHALL separate command transport, application orchestration, robot state management, domain controllers, and hardware abstractions into distinct Rust modules with clear ownership boundaries.

#### Scenario: HTTP layer delegates to firmware controller
- **WHEN** an external command is received through the HTTP API
- **THEN** the transport layer translates it into a `RobotCommand` and delegates execution to the application controller without directly mutating robot hardware state

### Requirement: RobotController shall coordinate firmware behavior
The firmware SHALL expose a central `RobotController` that receives `RobotCommand` values, validates command applicability against the current `RobotState`, coordinates domain controllers, applies legal state transitions, and produces the current `RobotStatus`.

#### Scenario: Command accepted in valid state
- **WHEN** `RobotController` receives `START_CLEANING` while the robot is in `Standby` and battery conditions are sufficient
- **THEN** it SHALL start the relevant controllers, transition the robot to `Cleaning`, and return an updated `RobotStatus`

#### Scenario: Command rejected in invalid state
- **WHEN** `RobotController` receives a command that is not allowed in the current `RobotState`
- **THEN** it SHALL reject or safely ignore the command without violating safety guarantees or corrupting internal state

### Requirement: Firmware runtime shall start in a deterministic standby configuration
The implemented runtime SHALL initialize in `Standby` with all actuators stopped, `current_error` set to `None`, `cleaning_mode` set to `AUTO`, and a configurable simulated battery percentage that defaults to a test-friendly value.

#### Scenario: Initial runtime status
- **WHEN** the firmware starts in the simulated environment
- **THEN** the first observable `RobotStatus` SHALL report `Standby`, stopped actuators, no active error, `AUTO` mode, and the configured initial battery percentage

### Requirement: Firmware scope shall exclude unsupported platform concerns
The firmware specification SHALL exclude Android client implementation, Bluetooth transport, real GPIO or board-specific drivers, SLAM, room mapping, camera or lidar integration, and advanced navigation algorithms from the first implementation.

#### Scenario: Unsupported feature request
- **WHEN** a feature outside the defined firmware scope is considered during implementation planning
- **THEN** the change SHALL document it as out of scope or a future extension rather than implementing it in the first milestone
