## 1. AUTO Navigation Refinement

- [x] 1.1 Update AUTO cleaning so forward movement is the default steady-state behavior when no obstacle or bumper condition is active
- [x] 1.2 Make `bumper_pressed` the primary normal obstacle reaction trigger and keep `obstacle_detected` limited to very short-range frontal proximity that does not stop the robot far from obstacles
- [x] 1.3 Change blocked AUTO handling so bumper contact or very close frontal obstacle triggers an in-place turn without backing up, keeps `state = CLEANING`, and resumes forward movement automatically after the turn window
- [x] 1.4 Add a simple deterministic anti-loop heuristic with a consistent default turn direction and a longer-turn or temporary opposite-turn escape only after repeated events
- [x] 1.5 Preserve existing critical safety-stop behavior for drop-off, wheel stuck, brush stuck, top cover open, and dust container full

## 2. Demo Frontend Movement And Coverage

- [x] 2.1 Rework the local sensor model so `obstacle_detected` is short-range frontal detection only and `bumper_pressed` is actual collision/contact plus the primary collision reaction trigger
- [x] 2.2 Clamp or roll back visual movement to the last valid position so the robot never remains visually inside walls or obstacles
- [x] 2.3 Maintain a frontend coverage grid or covered-cell set and paint cleaned floor continuously as the robot moves
- [x] 2.4 Track movement history, sensor flags, events, coverage data, initial robot pose, and full map reconstruction metadata in a frontend log session object tied to the current map
- [x] 2.5 Add controls for reset cleaning/position, generate new map, generate log dump, and simulation speed selection
- [x] 2.6 Show the current `session_id` in the UI at all times so the active map/log session is always visible
- [x] 2.7 Ensure the speed control changes only frontend visual simulation timing and remains stable at higher multipliers through substeps or equivalent

## 3. Demo Reset, Maps, And Log Dump API

- [x] 3.1 Add a demo-only `POST /simulation/reset` endpoint that restores the simulator/controller to a safe initial state and returns `RobotStatus`
- [x] 3.2 Add a demo-only `POST /simulation/log-dump` endpoint that validates a complete log payload, sanitizes `session_id`, and writes JSON only inside the demo log folder
- [x] 3.3 Optionally add a demo-only log retrieval endpoint if needed for download UX, while keeping `/commands/*` unchanged
- [x] 3.4 Guarantee that each generated map immediately creates a new session id and exactly one log target path for that session, such as `demo-logs/<session_id>/log.json`
- [x] 3.5 Ensure reset keeps the current map/session while Generate New Map creates new obstacles, a new session id, cleared coverage/history, and a new log target path
- [x] 3.6 Add or update git ignore rules and README notes for the chosen backend log folder

## 4. Testing And Verification

- [x] 4.1 Add or update unit tests proving AUTO collision and bumper behavior trigger a turn without entering `Error`
- [x] 4.2 Add or update unit tests proving AUTO resumes forward motion after the turn window, repeated bumper events do not cause permanent spinning, and normal obstacle/bumper events never leave the robot stopped indefinitely
- [x] 4.3 Add or update HTTP tests for `/simulation/reset`, valid `/simulation/log-dump`, and invalid log dump payload handling
- [x] 4.4 Add or update frontend/manual verification notes proving reset keeps the same map/session while Generate New Map creates a new session
- [x] 4.5 Re-run `cargo test` to confirm existing command API coverage still passes
- [x] 4.6 Manually verify the demo behavior for speed changes, coverage painting, reset, new map, session rotation, and dump generation
