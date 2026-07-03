## MODIFIED Requirements

### Requirement: Demo frontend shall feed computed local sensor signals back into the simulation
On each animation interval, the demo frontend SHALL poll `GET /status`, update local visual movement from wheel speeds, compute obstacle proximity or collision in the local room, send the resulting sensor flags to `/simulation/sensors`, invoke `/simulation/tick`, and then render the updated status. The frontend SHALL avoid concurrent simulation cycles by waiting for the previous cycle to finish or by using an `isTicking` guard. The demo SHALL recompute obstacle proximity on every cycle and SHALL clear `obstacle_detected` and `bumper_pressed` when the robot is no longer blocked.

#### Scenario: Demo sends simulated time progression
- **WHEN** the demo loop runs a new simulation cycle
- **THEN** the frontend SHALL send a meaningful `delta_ms` value to `POST /simulation/tick` derived from elapsed frame time and the selected speed factor

#### Scenario: Tick delta is clamped for stability
- **WHEN** the browser frame delay or speed factor would produce an excessively large simulated delta
- **THEN** the frontend SHALL clamp `delta_ms` to the documented safe maximum before calling `/simulation/tick`

## ADDED Requirements

### Requirement: Demo frontend shall provide accurate turning events and stale-sensor cleanup
The demo frontend SHALL distinguish between entering a turn, continuing a normal turning frame, and triggering an explicit anti-loop escape maneuver. It SHALL also clear stale obstacle and bumper flags when the user stops, resets, or generates a new map.

#### Scenario: TURN_START is emitted only on transition into turning
- **WHEN** wheel-speed/state data changes from forward or straight motion into a turn
- **THEN** the frontend SHALL record `TURN_START` once for that transition rather than on every turning frame

#### Scenario: ANTI_LOOP_ESCAPE is emitted only for real escape behavior
- **WHEN** the backend/frontend identifies an explicit anti-loop escape maneuver rather than an ordinary turn frame
- **THEN** the frontend SHALL record `ANTI_LOOP_ESCAPE`

#### Scenario: Stop and reset clear stale sensors
- **WHEN** the user clicks Stop, Reset, or Generate New Map
- **THEN** the frontend SHALL clear `obstacle_detected`, clear `bumper_pressed`, clear any local collision state, and synchronize those cleared flags with the backend if needed

#### Scenario: Forward movement resumes after timed turn
- **WHEN** backend wheel speeds return to `60/60` after a timed turn
- **THEN** the frontend SHALL move the robot forward again, continue coverage painting, and only reassert collision signals if a real blocker remains
