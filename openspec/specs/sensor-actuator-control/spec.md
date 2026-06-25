## Purpose

Define controller behavior for actuation, safety shutdown, battery recovery, docking, and the periodic firmware loop.

## Requirements

### Requirement: Motion and cleaning controllers shall manage actuators through the HAL
The firmware SHALL provide a `MotionController` for wheel movement and a `CleaningController` for suction and side brushes. These controllers SHALL expose the required actions for movement, stopping, wheel speed changes, suction control, brush control, and dust-container-full reactions through hardware abstractions.

#### Scenario: Manual movement updates wheel speeds
- **WHEN** the robot accepts a `MANUAL_MOVE` command with direction, speed, and `duration_ms`
- **THEN** the motion controller SHALL set the left and right wheel outputs needed for that movement and report the resulting wheel speeds through `RobotStatus`

#### Scenario: Stop cleaning disables actuators
- **WHEN** the robot processes `STOP_CLEANING`
- **THEN** the cleaning controller SHALL stop suction and side brushes and the motion controller SHALL stop the wheels before the robot returns to `Standby`

#### Scenario: Dust container full stops all actuators
- **WHEN** `dust_container_full` becomes true while the robot is in `Cleaning`
- **THEN** the firmware SHALL stop wheels, suction, and brushes, set `current_error = DUST_CONTAINER_FULL`, and transition the robot to `Error`

### Requirement: Safety manager shall fail fast on critical conditions
The firmware SHALL provide a `SafetyManager` that evaluates critical sensor conditions including `drop_off_detected`, `wheel_stuck`, `brush_stuck`, and `top_cover_open`. On any critical condition, it SHALL stop all actuators, set the current `RobotError`, and transition the robot to `Error`.

#### Scenario: Drop-off safety trigger
- **WHEN** `drop_off_detected` becomes true during operation
- **THEN** the safety manager SHALL stop wheels, suction, and brushes and move the robot to `Error`

#### Scenario: Wheel stuck safety trigger
- **WHEN** `wheel_stuck` becomes true during operation
- **THEN** the safety manager SHALL stop wheels, suction, and brushes and move the robot to `Error`

### Requirement: Battery and docking managers shall coordinate autonomous recovery
The firmware SHALL provide a `BatteryManager` that reads battery percentage, detects low and full battery states, and identifies charging activity, and a `DockingManager` that initiates return-to-dock behavior, reacts to docking detection, and transitions the robot to `Charging`.

#### Scenario: Low battery starts docking flow
- **WHEN** the battery manager reports a low-battery condition while the robot is cleaning
- **THEN** the docking manager SHALL begin return-to-dock behavior and the robot SHALL transition to `ReturningToDock`

### Requirement: Firmware shall evaluate control rules in a periodic tick loop
The firmware SHALL provide a `tick()` method or equivalent periodic update function that reads the current `SensorSnapshot`, evaluates safety conditions, evaluates battery and docking conditions, and updates `RobotState` and `RobotStatus` accordingly. Docking evaluation in `tick()` SHALL use `DockingDriver` rather than `SensorSnapshot`.

#### Scenario: Tick drives charging transition
- **WHEN** `tick()` runs while the robot is in `ReturningToDock` and `DockingDriver` reports docking has been detected
- **THEN** the firmware SHALL stop wheel motion, transition to `Charging`, and return status reflecting active charging behavior

#### Scenario: AUTO obstacle reaction
- **WHEN** `tick()` runs while the robot is in `Cleaning` and `obstacle_detected` is true
- **THEN** the firmware SHALL keep the robot in `Cleaning` and apply a simple avoidance maneuver such as `left_wheel_speed = 30` and `right_wheel_speed = -30`

#### Scenario: AUTO bumper reaction
- **WHEN** `tick()` runs while the robot is in `Cleaning` and `bumper_pressed` is true
- **THEN** the firmware SHALL keep the robot in `Cleaning` and apply a simple avoidance maneuver such as `left_wheel_speed = 30` and `right_wheel_speed = -30`

### Requirement: Implemented navigation modes shall remain intentionally narrow
The firmware SHALL implement `AUTO` cleaning behavior and `MANUAL` movement behavior in this change. `/commands/mode` SHALL accept only `AUTO`. `ZigZag`, `WallFollowing`, and `Spot` SHALL be documented only as future extensions and SHALL NOT be exposed as implemented behaviors.

#### Scenario: Unsupported future mode request
- **WHEN** a client requests a cleaning mode other than `AUTO` in the first implementation
- **THEN** the firmware SHALL reject the request as unsupported without changing the active mode
