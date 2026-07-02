## 1. Simulated Time Progression

- [x] 1.1 Extend `POST /simulation/tick` to accept optional `delta_ms`, validate it, advance `SimulatedClock`, and keep backward compatibility with a safe default when omitted
- [x] 1.2 Ensure timed AUTO turns and manual movement deadlines expire when simulated time advances
- [x] 1.3 Keep the clock advancement strictly demo/testing-only under `/simulation/*`

## 2. AUTO Turn Re-trigger Guard

- [x] 2.1 Prevent stale `obstacle_detected` from restarting a fresh timed turn every frame while a current turn window is still active
- [x] 2.2 Allow a new turn only after the previous window expired, sensors were reevaluated, and the robot is still blocked
- [x] 2.3 Distinguish regular turning from explicit anti-loop escape behavior in controller/frontend event semantics

## 3. Frontend Loop And Sensor Cleanup

- [x] 3.1 Send speed-adjusted, clamped `delta_ms` from the demo loop in every `/simulation/tick` request
- [x] 3.2 Correct event labeling so `TURN_START` only fires on transition into turning and `ANTI_LOOP_ESCAPE` only fires on a real escape maneuver
- [x] 3.3 Clear `obstacle_detected`, `bumper_pressed`, and local collision state on Stop, Reset, and Generate New Map
- [x] 3.4 Ensure the robot resumes forward visual movement once backend wheel speeds return to `60/60`
- [x] 3.5 Ensure the demo log shows finite `TURN_START`, optional `TURNING` frames, and then renewed forward `x/y` movement instead of endless in-place spin

## 4. Testing And Documentation

- [x] 4.1 Add unit tests proving obstacle/bumper start a timed turn and advancing simulated time beyond the window resumes forward movement
- [x] 4.2 Add unit tests proving manual movement expires when simulated time advances
- [x] 4.3 Add HTTP tests for valid `/simulation/tick` with `delta_ms`, invalid `delta_ms`, and continued backward compatibility without payload
- [x] 4.4 Re-run `cargo test` to confirm existing tests still pass
- [x] 4.5 Update README to document simulated time progression, speed-adjusted `delta_ms`, timer semantics, and stale sensor cleanup on Stop/Reset
- [x] 4.6 Manually verify from the demo that the robot turns for finite time, resumes moving, and no longer logs fake anti-loop events every frame
