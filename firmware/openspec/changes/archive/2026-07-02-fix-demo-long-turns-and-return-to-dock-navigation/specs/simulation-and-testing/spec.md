## MODIFIED Requirements

### Requirement: Demo verification shall cover local simulator controls
Manual verification of the plain JavaScript demo SHALL confirm coverage painting, speed control, reset behavior, new map generation, session rotation, and that generated log dumps include map, movement timeline, and coverage data.

#### Scenario: Long turn guard breaks repeated in-place spinning
- **WHEN** the robot remains in `CLEANING`, `x/y` stay effectively unchanged, and wheel speeds indicate repeated turning for longer than the documented demo threshold
- **THEN** the demo SHALL emit a guard or escape event such as `LONG_TURN_GUARD`, `ESCAPE_TURN`, or `ESCAPE_FORWARD_ATTEMPT` and SHALL NOT continue producing only repeated `TURNING` frames indefinitely

#### Scenario: Long turn guard causes a real follow-up behavior
- **WHEN** `LONG_TURN_GUARD` intervenes
- **THEN** the next few frames SHALL show either changed `x/y` or a clear blocked-movement event with collision target detail, and the demo SHALL NOT simply fall back into another indefinite `TURNING` chain without evaluating a forward attempt

#### Scenario: Proximity does not retrigger the same turn forever
- **WHEN** the frontend recomputes `obstacle_detected` while a timed turn is already in progress
- **THEN** that proximity signal MAY remain visible for UI/logging, but it SHALL NOT restart a fresh turn episode until the previous turn ended and a forward attempt or fresh blocker justified a new turn

#### Scenario: Turn episodes remain stable during one turn
- **WHEN** a turn episode is already open
- **THEN** `TURNING` frames SHALL belong to the current episode, `TURN_END` SHALL close it, and `obstacle_detected` changes during that open episode SHALL NOT create a new turn episode

#### Scenario: High-speed rotation uses bounded angular substeps
- **WHEN** the demo runs at a high simulation speed and the robot performs a large in-place rotation
- **THEN** the frontend SHALL split that rotation into bounded angular substeps so heading changes remain stable and sensor toggling does not alias the same turn into repeated retriggers

#### Scenario: Return to Dock button reflects backend response
- **WHEN** the user presses the Return to Dock button
- **THEN** the frontend SHALL call `POST /commands/return-to-dock`, consume the returned status, log `RETURN_TO_DOCK` when accepted, update the UI immediately to backend state `RETURNING_TO_DOCK`, and surface the backend error if the command is refused

#### Scenario: Rejected Return to Dock does not start homing
- **WHEN** `POST /commands/return-to-dock` fails
- **THEN** the frontend SHALL NOT start local docking guidance, SHALL show the backend error, and SHALL log `RETURN_TO_DOCK_REJECTED` with error detail

#### Scenario: Return-to-dock steers toward the docking station
- **WHEN** backend state is `RETURNING_TO_DOCK` and the robot is far from the docking station
- **THEN** the demo SHALL use the current map docking position to orient the visual robot toward the dock, advance when sufficiently aligned, and reduce distance to dock over time rather than continuing forever on its old heading

#### Scenario: Docking guidance overrides normal AUTO movement
- **WHEN** backend state is `RETURNING_TO_DOCK`
- **THEN** the frontend SHALL use demo-only docking guidance derived from robot pose, dock position, heading error, distance to dock, and blocked movement state, and SHALL NOT blindly interpret backend `30/30` wheel speeds as straight-line movement

#### Scenario: Docking log distinguishes backend speeds from demo guidance
- **WHEN** the demo logs return-to-dock movement
- **THEN** event details SHALL be able to distinguish backend wheel speeds from frontend docking guidance by including fields such as `backend_left_wheel_speed`, `backend_right_wheel_speed`, `demo_docking_phase`, `demo_target_heading`, `heading_error`, and `distance_to_dock`

#### Scenario: Dock detection completes charging transition
- **WHEN** the robot reaches or overlaps the docking station closely enough during `RETURNING_TO_DOCK`
- **THEN** the frontend SHALL send `dock_detected = true`, the backend SHALL transition to `CHARGING` on the next tick, and visual movement SHALL stop

#### Scenario: Backend return-to-dock state transitions stay explicit
- **WHEN** tests exercise `POST /commands/return-to-dock` and docking completion
- **THEN** they SHALL verify allowed or refused command states explicitly, verify `RETURNING_TO_DOCK -> CHARGING` through `/simulation/docking`, and verify that charging begins with wheel speeds `0/0`

#### Scenario: Docking blockage triggers recovery instead of permanent 30/30 stall
- **WHEN** return-to-dock movement is blocked by a wall or obstacle
- **THEN** the demo SHALL emit a docking blockage or recovery event such as `DOCK_BLOCKED`, `DOCK_RECOVERY_TURN`, or `DOCK_STUCK_DIAGNOSTIC`, perform a simple local recovery maneuver, and continue trying to reach the dock

#### Scenario: Docking progress invariant prevents silent stalls
- **WHEN** a generated log is reviewed for `RETURNING_TO_DOCK`
- **THEN** it SHALL NOT contain a long period where the robot stays in `RETURNING_TO_DOCK`, distance to dock is not decreasing, `x/y` are unchanged or moving away from the dock, and no `DOCK_BLOCKED`, `DOCK_RECOVERY_TURN`, or `DOCK_STUCK_DIAGNOSTIC` event appears

#### Scenario: Timeline timestamps remain analyzable
- **WHEN** the demo generates timeline frames
- **THEN** `timestamp_ms` values SHALL be non-decreasing across the log so downstream analysis can rely on frame order and duration calculations

#### Scenario: Frontend manual verification checklist covers turning and docking
- **WHEN** the implementation is reviewed before merge
- **THEN** the verification notes SHALL confirm that AUTO at `10x` no longer shows long silent endless turning, the guard or escape events appear when needed, the Return to Dock button changes backend state when accepted, return-to-dock visibly reorients toward the dock, distance to dock generally decreases, and the log contains the expected docking and charging events
