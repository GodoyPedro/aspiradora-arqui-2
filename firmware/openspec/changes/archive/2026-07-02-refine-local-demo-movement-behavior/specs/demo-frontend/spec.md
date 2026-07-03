## MODIFIED Requirements

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

#### Scenario: Demo detects obstacle only when it is in front of the robot
- **WHEN** the robot is near an obstacle but the obstacle is no longer in front of its heading or forward sensor cone
- **THEN** the frontend SHALL clear `obstacle_detected` even if the robot is still nearby

#### Scenario: Demo detects local collision and updates simulation
- **WHEN** the visual robot overlaps an obstacle or wall
- **THEN** the frontend SHALL set `bumper_pressed = true` and treat that as collision feedback distinct from forward-only obstacle sensing

#### Scenario: Demo clears stale obstacle flags
- **WHEN** the robot has rotated away from the obstacle or is no longer colliding
- **THEN** the frontend SHALL send `obstacle_detected = false` and `bumper_pressed = false`

#### Scenario: Demo loop survives HTTP failures
- **WHEN** a demo HTTP request fails during polling, sensor update, or tick
- **THEN** the frontend SHALL surface the failure in the UI and SHALL continue running the animation loop without crashing the page

## ADDED Requirements

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
