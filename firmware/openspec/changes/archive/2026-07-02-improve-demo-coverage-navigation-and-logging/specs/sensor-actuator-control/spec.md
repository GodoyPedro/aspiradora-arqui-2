## MODIFIED Requirements

### Requirement: Firmware shall evaluate control rules in a periodic tick loop
The firmware SHALL provide a `tick()` method or equivalent periodic update function that reads the current `SensorSnapshot`, evaluates safety conditions, evaluates battery and docking conditions, and updates `RobotState` and `RobotStatus` accordingly. Docking evaluation in `tick()` SHALL use `DockingDriver` rather than `SensorSnapshot`.

#### Scenario: Tick drives charging transition
- **WHEN** `tick()` runs while the robot is in `ReturningToDock` and `DockingDriver` reports docking has been detected
- **THEN** the firmware SHALL stop wheel motion, transition to `Charging`, and return status reflecting active charging behavior

#### Scenario: AUTO forward is the default movement
- **WHEN** `tick()` runs while the robot is in `Cleaning`, `cleaning_mode = AUTO`, and no obstacle, bumper, or critical safety condition is active
- **THEN** the firmware SHALL keep the robot trying to move forward as the default cleaning behavior

#### Scenario: AUTO turns in place on frontal obstacle detection
- **WHEN** `tick()` runs while the robot is in `Cleaning`, `cleaning_mode = AUTO`, and a very close frontal obstacle is detected
- **THEN** the firmware SHALL keep the robot in `Cleaning`, MAY keep moving until the immediate turn threshold is reached, and when reacting SHALL start or continue an in-place turn without backing up

#### Scenario: AUTO turns in place on bumper contact
- **WHEN** `tick()` runs while the robot is in `Cleaning`, `cleaning_mode = AUTO`, and `bumper_pressed` is true
- **THEN** the firmware SHALL treat the bumper contact as normal navigation feedback, keep the robot in `Cleaning`, and perform an in-place turn rather than entering `Error`

#### Scenario: AUTO resumes forward after turn completion
- **WHEN** the configured turn window expires and the robot is no longer blocked
- **THEN** the firmware SHALL resume default forward AUTO movement

#### Scenario: Normal obstacle events never leave AUTO stopped indefinitely
- **WHEN** the robot experiences a normal bumper event or very close frontal obstacle event during `Cleaning/AUTO`
- **THEN** the firmware SHALL NOT remain stopped indefinitely and SHALL either continue forward toward contact or resume forward automatically after the in-place turn window

#### Scenario: Default turn direction is consistent
- **WHEN** the controller performs its normal obstacle reaction
- **THEN** it SHALL use a consistent default turn direction, such as clockwise/right, unless the anti-loop heuristic temporarily overrides it

#### Scenario: Repeated collisions trigger a simple escape maneuver
- **WHEN** repeated obstacle or bumper events occur within a short recent window while the robot remains in `Cleaning/AUTO`
- **THEN** the firmware SHALL apply a simple deterministic anti-loop response such as a longer turn or a one-off opposite-direction escape turn so it does not spin forever around the same obstacle, without introducing map memory or path planning
