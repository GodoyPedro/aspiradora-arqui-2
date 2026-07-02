## ADDED Requirements

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
