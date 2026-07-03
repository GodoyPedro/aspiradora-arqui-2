## MODIFIED Requirements

### Requirement: Firmware shall evaluate control rules in a periodic tick loop
The firmware SHALL provide a `tick()` method or equivalent periodic update function that reads the current `SensorSnapshot`, evaluates safety conditions, evaluates battery and docking conditions, and updates `RobotState` and `RobotStatus` accordingly. Docking evaluation in `tick()` SHALL use `DockingDriver` rather than `SensorSnapshot`.

#### Scenario: Tick drives charging transition
- **WHEN** `tick()` runs while the robot is in `ReturningToDock` and `DockingDriver` reports docking has been detected
- **THEN** the firmware SHALL stop wheel motion, transition to `Charging`, and return status reflecting active charging behavior

#### Scenario: AUTO obstacle reaction
- **WHEN** `tick()` runs while the robot is in `Cleaning` and `obstacle_detected` is true
- **THEN** the firmware SHALL keep the robot in `Cleaning` unless a documented safety rule requires `Error`, and it SHALL NOT require backend knowledge of lane indexes, lane shifts, or obstacle-bypass geometry

#### Scenario: AUTO bumper reaction
- **WHEN** `tick()` runs while the robot is in `Cleaning` and `bumper_pressed` is true
- **THEN** the firmware SHALL keep the robot in `Cleaning` unless a documented safety rule requires `Error`, and it SHALL continue reporting high-level actuator/status state without implying frontend spatial path ownership

### Requirement: Implemented navigation modes shall remain intentionally narrow
The firmware SHALL implement `AUTO` cleaning behavior and `MANUAL` movement behavior in this change. `/commands/mode` SHALL accept only `AUTO`. `ZigZag`, `WallFollowing`, and `Spot` SHALL be documented only as future extensions and SHALL NOT be exposed as implemented behaviors.

#### Scenario: Unsupported future mode request
- **WHEN** a client requests a cleaning mode other than `AUTO` in the first implementation
- **THEN** the firmware SHALL reject the request as unsupported without changing the active mode

## ADDED Requirements

### Requirement: Backend cleaning behavior shall remain spatially generic
While the robot is in `Cleaning`, the backend SHALL remain responsible for high-level state, actuator/status reporting, safety stop and error transitions, and docking transition readiness. It SHALL NOT be specified as owning lane geometry, lane indexes, lane shifts, obstacle-bypass paths, or visual `x/y/heading` localization for the browser demo.

#### Scenario: Charging start stops actuators
- **WHEN** docking is detected while the robot is in `ReturningToDock`
- **THEN** the backend SHALL transition to `Charging` and report stopped wheel motion

#### Scenario: Critical fault still preempts cleaning
- **WHEN** a documented critical safety condition occurs during `Cleaning`
- **THEN** the backend SHALL stop actuators and enter `Error` regardless of any demo-owned visual navigation behavior
