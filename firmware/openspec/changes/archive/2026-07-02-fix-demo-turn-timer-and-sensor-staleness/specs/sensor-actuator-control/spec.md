## MODIFIED Requirements

### Requirement: Firmware shall evaluate control rules in a periodic tick loop
The firmware SHALL provide a `tick()` method or equivalent periodic update function that reads the current `SensorSnapshot`, evaluates safety conditions, evaluates battery and docking conditions, and updates `RobotState` and `RobotStatus` accordingly. Docking evaluation in `tick()` SHALL use `DockingDriver` rather than `SensorSnapshot`.

#### Scenario: AUTO timed turn expires and movement resumes
- **WHEN** the robot is in `Cleaning/AUTO`, a blocker started a timed turn, simulated time advances past the configured turn deadline, and the blocker is no longer active
- **THEN** the firmware SHALL end the turn window and restore forward wheel speeds such as `60/60`

#### Scenario: Stale obstacle does not restart turn every frame
- **WHEN** a timed `AUTO` turn is already active and the same stale `obstacle_detected` input is observed again before the turn window expires
- **THEN** the firmware SHALL continue the current turn window and SHALL NOT schedule a fresh turn deadline on every frame

#### Scenario: Blocker still active after turn expiry
- **WHEN** a timed `AUTO` turn expires, sensors are reevaluated, and the robot is still blocked
- **THEN** the firmware MAY start one new turn window, but only after the previous one actually expired and reevaluation confirmed the blocker remains

#### Scenario: Manual movement deadline expires with simulated time
- **WHEN** the robot is in `MANUAL_CONTROL` and simulated time advances past the configured manual movement deadline
- **THEN** the firmware SHALL stop wheel motion and transition back to `Standby`
