## 1. Long Turn Guard

- [x] 1.1 Detect long continuous in-place turning during `CLEANING` when `x/y` stay unchanged and wheel speeds remain a turn pattern
- [x] 1.2 Add a demo-only max continuous turn guard threshold and emit `LONG_TURN_GUARD` when it intervenes
- [x] 1.3 Ensure stale `obstacle_detected` does not endlessly restart new turns during an already active turn
- [x] 1.4 After the guard triggers, prefer a short forward probe/attempt when valid, otherwise perform a deterministic `ESCAPE_TURN` and then an `ESCAPE_FORWARD_ATTEMPT`
- [x] 1.5 Verify that post-guard frames show either changed `x/y` or a clear blocked-movement event with collision target detail, instead of another indefinite turning chain
- [x] 1.6 Track local `turn_episode_id` so `TURN_START`, `TURNING`, and `TURN_END` belong to explicit turn episodes

## 2. High-Speed Angular Stability

- [x] 2.1 Add bounded angular substepping for large in-place rotations at high simulation speed
- [x] 2.2 Define and document a maximum angular substep size and a bounded number of angular substeps per frame
- [x] 2.3 Recompute sensors for display/logging without allowing mid-turn proximity toggles to restart the same turn episode
- [x] 2.4 Clear stale proximity when the robot is no longer physically blocked and a short forward probe from the current heading is valid

## 3. Return To Dock Command And Homing

- [x] 3.1 Wire the Return to Dock button to `POST /commands/return-to-dock`, consume the returned status, log `RETURN_TO_DOCK`, and surface backend errors in the UI if the command is refused
- [x] 3.2 Add validation that pressing the Return to Dock button actually changes backend state to `RETURNING_TO_DOCK` when accepted
- [x] 3.3 Log `RETURN_TO_DOCK_REJECTED` with backend error detail and do not start local docking homing if `POST /commands/return-to-dock` fails
- [x] 3.4 Implement demo-only spatial guidance toward `docking_station.x/y` while backend state is `RETURNING_TO_DOCK`
- [x] 3.5 Keep docking visual guidance as the single movement authority while backend state is `RETURNING_TO_DOCK`, above normal `AUTO` movement
- [x] 3.6 Add simple dock phases such as `DOCK_TARGETING`, `DOCK_ALIGNING`, and `DOCK_APPROACH`
- [x] 3.7 Slow the robot near the docking station and continue sending `/simulation/docking` updates until backend transitions to `CHARGING`
- [x] 3.8 Handle blocked docking movement with `DOCK_BLOCKED`, `DOCK_RECOVERY_TURN`, a small detour, and a renewed attempt toward the dock
- [x] 3.9 Add a demo-only stuck guard for docking progress and emit `DOCK_STUCK_DIAGNOSTIC` when distance to dock does not improve for several seconds

## 4. Logging And Documentation

- [x] 4.1 Ensure the log shows guard/escape intervention during long turns with `LONG_TURN_GUARD`, `ESCAPE_TURN`, and `ESCAPE_FORWARD_ATTEMPT`
- [x] 4.2 Ensure docking logs include `RETURN_TO_DOCK`, `DOCK_TARGETING`, `DOCK_ALIGNING`, `DOCK_APPROACH`, `DOCK_BLOCKED`, `DOCK_DETECTED`, and `CHARGING_STARTED`
- [x] 4.3 Include simple JSON-compatible docking diagnostics such as `backend_left_wheel_speed`, `backend_right_wheel_speed`, `demo_docking_phase`, `demo_target_heading`, `heading_error`, and `distance_to_dock` in `event_detail` when useful
- [x] 4.4 Include `turn_episode_id` in turn-related event details where useful for log analysis
- [x] 4.5 Keep timeline `timestamp_ms` non-decreasing across the full generated log
- [x] 4.6 Update README to document the long-turn guard, angular substepping, demo-only dock homing, backend/frontend responsibility split, rejected docking commands, and docking diagnostics in the dump
- [x] 4.7 Add a dedicated `demo-frontend` spec delta if that capability is introduced later; today the repo has no base `demo-frontend` spec so frontend requirements remain under `simulation-and-testing`

## 5. Validation

- [x] 5.1 Add or keep backend tests for `POST /commands/return-to-dock`, allowed/refused states, `RETURNING_TO_DOCK -> CHARGING` via `/simulation/docking`, and `0/0` wheel speeds after charging starts
- [x] 5.2 Manually run AUTO at `10x` for at least 60 seconds and confirm no long endless in-place turning without guard/escape events
- [x] 5.3 Manually trigger `Return to Dock` while far from the dock and confirm heading changes toward the docking station and distance generally decreases over time
- [x] 5.4 Confirm the robot reaches the dock, backend state becomes `CHARGING`, and the log contains the expected docking events
- [x] 5.5 Validate that generated logs do not contain long repeated `TURNING` frames with unchanged `x/y` and no guard/escape event, or `RETURNING_TO_DOCK` periods with no dock progress, unchanged or regressing `x/y`, and no `DOCK_BLOCKED`, `DOCK_RECOVERY_TURN`, or `DOCK_STUCK_DIAGNOSTIC`
- [x] 5.6 Add or confirm a backend/state transition spec delta for the existing `robot-state-machine` capability so return-to-dock command/state expectations stay explicit
