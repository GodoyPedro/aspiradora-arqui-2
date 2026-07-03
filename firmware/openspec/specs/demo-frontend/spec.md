# demo-frontend Specification

## Purpose
TBD - created by archiving change add-local-demo-frontend. Update Purpose after archive.
## Requirements
### Requirement: Firmware shall serve a local browser demo frontend
The firmware SHALL expose a local browser-accessible demo UI at `GET /demo` and SHALL serve the required static assets for that demo under `/static/*` without requiring external frontend tooling. Those assets SHALL be embedded at compile time with `include_str!` or an equivalent mechanism so that `cargo run` from the firmware crate is sufficient to serve the demo.

#### Scenario: Demo page loads from the running backend
- **WHEN** the backend is running locally and a user opens `http://127.0.0.1:3000/demo`
- **THEN** the firmware SHALL return an HTML page that can load its CSS and JavaScript assets from the same backend

#### Scenario: Demo assets do not depend on runtime filesystem layout
- **WHEN** the backend is launched with `cargo run` from the firmware crate
- **THEN** `/demo`, `/static/demo.css`, and `/static/demo.js` SHALL be served without requiring a specific runtime working-directory asset lookup

### Requirement: Demo frontend shall visualize the robot in a simple 2D room
The demo frontend SHALL render a 2D room containing a robot vacuum, static obstacles, a docking station, a heading indicator, and visible sensor cues that help explain what the simulated robot is detecting.

#### Scenario: Demo renders visual simulation elements
- **WHEN** the demo UI initializes
- **THEN** it SHALL display the robot, room boundaries, obstacles, docking station, and a side panel with sensor and status information

### Requirement: Demo frontend shall drive the existing firmware command API
The demo frontend SHALL use the existing firmware command endpoints for robot control, including start, stop, pause, return-to-dock, manual movement, clear-error, and status polling. Manual movement buttons in the demo SHALL send a single `MANUAL_MOVE` request with `duration_ms = 2000` by default per click.

#### Scenario: User triggers a control command from the demo
- **WHEN** a user clicks a control button in the demo UI
- **THEN** the frontend SHALL call the corresponding existing `/commands/*` endpoint and refresh the displayed `RobotStatus`

#### Scenario: Demo manual buttons use the default duration
- **WHEN** a user clicks Manual Forward, Backward, Left, Right, or Stop in the demo UI
- **THEN** the frontend SHALL send exactly one `MANUAL_MOVE` request with `duration_ms = 2000`

### Requirement: Demo frontend shall maintain only local visual movement state
The demo frontend SHALL maintain the robot's visual position and heading locally in the browser, while the backend SHALL remain the source of truth for robot state, wheel speeds, battery, charging, cleaning mode, sensors, and errors. The frontend SHALL expose a clear visual movement scale constant so the vacuum moves noticeably faster in the 2D room while remaining controllable.

#### Scenario: Visual reset does not alter firmware state
- **WHEN** a user clicks the demo's local position reset control
- **THEN** the frontend SHALL reset only the local visual position and SHALL NOT mutate the backend robot state unless it explicitly calls a documented backend endpoint

#### Scenario: Demo uses explicit visual speed scaling
- **WHEN** the frontend computes local movement from wheel speeds
- **THEN** it SHALL apply a named visual speed constant such as `VISUAL_SPEED_SCALE` or `PIXELS_PER_SPEED_UNIT`

### Requirement: Demo frontend shall feed computed local sensor signals back into the simulation
On each animation interval, the demo frontend SHALL poll `GET /status`, update local visual movement from wheel speeds, compute obstacle proximity or collision in the local room, send the resulting sensor flags to `/simulation/sensors`, invoke `/simulation/tick`, and then render the updated status. The frontend SHALL avoid concurrent simulation cycles by waiting for the previous cycle to finish or by using an `isTicking` guard. The demo SHALL recompute obstacle proximity on every cycle and SHALL clear `obstacle_detected` and `bumper_pressed` when the robot is no longer blocked.

#### Scenario: Demo sends simulated time progression
- **WHEN** the demo loop runs a new simulation cycle
- **THEN** the frontend SHALL send a meaningful `delta_ms` value to `POST /simulation/tick` derived from elapsed frame time and the selected speed factor

#### Scenario: Tick delta is clamped for stability
- **WHEN** the browser frame delay or speed factor would produce an excessively large simulated delta
- **THEN** the frontend SHALL clamp `delta_ms` to the documented safe maximum before calling `/simulation/tick`

### Requirement: Documentation shall explain the local demo model
Project documentation SHALL explain how to run the backend, open the demo UI, understand the distinction between local browser movement and backend firmware state, and clarify that `/simulation/*` exists only for the local simulator/demo and is not part of the Android or external client command contract.

#### Scenario: User reads the demo setup instructions
- **WHEN** a developer opens the project README
- **THEN** the documentation SHALL explain how to run `cargo run`, open `/demo`, understand that the browser owns only visual position while the backend owns firmware state, and understand that `/simulation/*` is local-only and simulator-only

### Requirement: Demo frontend shall prevent visual penetration into obstacles
The demo frontend SHALL prevent the robot from remaining visually inside an obstacle or wall by clamping movement or rolling back to the last valid position when a collision is detected.

#### Scenario: Demo rolls back or clamps after collision
- **WHEN** a movement step would place the visual robot overlapping an obstacle or wall
- **THEN** the frontend SHALL clamp or restore the robot to the last valid non-overlapping position

### Requirement: Demo frontend shall visualize sensor activity clearly
The demo frontend SHALL show `obstacle_detected` and `bumper_pressed` clearly in the sensor or status panel when they are active, and SHALL update that visualization when the flags are cleared.

#### Scenario: Sensor panel reflects active obstacle flags
- **WHEN** `obstacle_detected` or `bumper_pressed` is true in the current status
- **THEN** the demo UI SHALL highlight those flags clearly in the sensor or status panel

### Requirement: Demo frontend shall visualize cleaned floor coverage
The demo frontend SHALL maintain a simple local coverage model, such as a fixed grid or covered-cell list, and SHALL continuously paint or shade the areas of the room that have been cleaned by the robot's circular footprint.

#### Scenario: Coverage appears while the robot moves
- **WHEN** the robot advances through the room in the local demo
- **THEN** the frontend SHALL mark the corresponding floor cells as covered and visually distinguish cleaned from uncleaned area

#### Scenario: Coverage can be exported
- **WHEN** the user generates a log dump
- **THEN** the frontend SHALL include covered cells or an equivalent compact coverage representation plus a coverage percentage or enough data to compute it

### Requirement: Demo frontend shall use contact-based obstacle feedback
The demo frontend SHALL keep the robot's visual position and heading locally, SHALL compute motion from backend wheel speeds, SHALL treat `obstacle_detected` as a short-range frontal signal only, and SHALL treat `bumper_pressed` as actual contact with a wall or obstacle. The frontend SHALL allow the robot to approach obstacles closely before asserting `obstacle_detected`, and `bumper_pressed` SHALL be the primary trigger for normal collision reaction in the demo.

#### Scenario: Frontal detection is short range only
- **WHEN** an obstacle is near the robot but outside the short frontal detection range
- **THEN** the frontend SHALL keep `obstacle_detected = false`

#### Scenario: Frontal proximity does not stop the robot too early
- **WHEN** `obstacle_detected = true` but the robot has not made contact and the obstacle is not yet at the immediate turn threshold
- **THEN** the demo SHALL NOT leave the robot frozen far away from the obstacle

#### Scenario: Bumper means real contact
- **WHEN** the robot's circular footprint collides with an obstacle or room wall
- **THEN** the frontend SHALL send `bumper_pressed = true`

#### Scenario: Collision clears after turning away
- **WHEN** the robot has rotated enough that the footprint no longer collides
- **THEN** the frontend SHALL send `bumper_pressed = false`

### Requirement: Demo frontend shall prevent visual penetration at all simulation speeds
The demo frontend SHALL clamp or roll back movement to the last valid non-overlapping position and SHALL keep collision detection stable even when the user increases the visual simulation speed substantially.

#### Scenario: High-speed motion still respects collisions
- **WHEN** the user runs the demo at 5x, 10x, or another high speed
- **THEN** the frontend SHALL use substeps or an equivalent method so the robot does not visually tunnel through walls or obstacles

### Requirement: Demo frontend shall provide map, reset, speed, and log controls
The demo frontend SHALL provide controls for Start, Stop, Pause, Return to dock, Manual Forward, Manual Backward, Manual Left, Manual Right, Manual Stop, Clear Error, Reset cleaning and robot position, Generate new map, Generate Log Dump, and simulation speed adjustment.

#### Scenario: Reset clears local demo state
- **WHEN** the user clicks Reset cleaning and robot position
- **THEN** the frontend SHALL reset the local robot position and heading, clear coverage data, clear local movement history, clear local sensor flags, coordinate with the documented backend reset endpoint if needed, and SHALL keep the current map and session id

#### Scenario: New map creates a new session
- **WHEN** the user clicks Generate new map
- **THEN** the frontend SHALL create a new obstacle layout, clear coverage and movement history, assign a new session id, and start a fresh log session for that map

#### Scenario: Speed control changes visual pacing only
- **WHEN** the user changes the simulation speed selector or slider
- **THEN** the frontend SHALL change visual movement and animation timing without changing the backend wheel speed semantics

#### Scenario: Log dump result is visible in the UI
- **WHEN** the user clicks Generate Log Dump and the backend accepts the payload
- **THEN** the demo SHALL show the returned session id and saved path or download location

#### Scenario: Current session id is always visible
- **WHEN** the user is viewing the demo
- **THEN** the UI SHALL show the current active `session_id` for the loaded map/log session

### Requirement: Demo frontend shall build a complete local log payload per map session
The demo frontend SHALL accumulate a per-session log payload that includes session metadata, map reconstruction data, frontend configuration, movement timeline, sensor flags, event types, coverage data, and an end-of-run summary before posting that payload to the backend. The payload SHALL include `session_id`, `map_id` if separate, `created_at`, room dimensions, robot radius, initial robot position and heading, docking station position, full obstacle list, simulation speed setting, sensor thresholds, and coverage grid configuration so the map can be reconstructed later.

#### Scenario: Log timeline captures movement and events
- **WHEN** the demo runs through a cleaning session
- **THEN** the frontend SHALL record frame or timestamp entries containing robot position, heading, wheel speeds, state, cleaning mode, battery, charging state, sensor flags, coverage percentage at that moment, and notable events such as START, BUMPER_CONTACT, TURN_START, TURN_END, ANTI_LOOP_ESCAPE, RESET, NEW_MAP, or LOG_DUMP

#### Scenario: Log summary captures aggregate stats
- **WHEN** the user generates a dump
- **THEN** the frontend SHALL include totals for frames, simulated time, collisions, obstacle detections, turns, and coverage percentage

#### Scenario: One log target path exists per session
- **WHEN** the frontend is running a given map session
- **THEN** that session SHALL have exactly one associated backend dump target path, such as `demo-logs/<session_id>/log.json`, and Generate Log Dump SHALL persist to that session path

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

