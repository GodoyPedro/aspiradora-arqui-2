## MODIFIED Requirements

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

#### Scenario: AUTO movement is restored after prior avoidance clears
- **WHEN** `tick()` runs while the robot is in `Cleaning`, both `obstacle_detected` and `bumper_pressed` are false, and the robot is recovering from the prior avoidance maneuver rather than executing another explicit movement behavior
- **THEN** the firmware SHALL restore normal AUTO forward wheel speeds such as `left_wheel_speed = 60` and `right_wheel_speed = 60`
