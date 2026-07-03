## ADDED Requirements

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
The demo frontend SHALL use the existing firmware command endpoints for robot control, including start, stop, pause, return-to-dock, manual movement, clear-error, and status polling.

#### Scenario: User triggers a control command from the demo
- **WHEN** a user clicks a control button in the demo UI
- **THEN** the frontend SHALL call the corresponding existing `/commands/*` endpoint and refresh the displayed `RobotStatus`

### Requirement: Demo frontend shall maintain only local visual movement state
The demo frontend SHALL maintain the robot's visual position and heading locally in the browser, while the backend SHALL remain the source of truth for robot state, wheel speeds, battery, charging, cleaning mode, sensors, and errors.

#### Scenario: Visual reset does not alter firmware state
- **WHEN** a user clicks the demo's local position reset control
- **THEN** the frontend SHALL reset only the local visual position and SHALL NOT mutate the backend robot state unless it explicitly calls a documented backend endpoint

### Requirement: Demo frontend shall feed computed local sensor signals back into the simulation
On each animation interval, the demo frontend SHALL poll `GET /status`, update local visual movement from wheel speeds, compute obstacle proximity or collision in the local room, send the resulting sensor flags to `/simulation/sensors`, invoke `/simulation/tick`, and then render the updated status. The frontend SHALL avoid concurrent simulation cycles by waiting for the previous cycle to finish or by using an `isTicking` guard.

#### Scenario: Demo detects local collision and updates simulation
- **WHEN** the visual robot approaches or collides with an obstacle in the local room
- **THEN** the frontend SHALL send sensor updates reflecting obstacle proximity or bumper activation before invoking `/simulation/tick`

#### Scenario: Demo loop survives HTTP failures
- **WHEN** a demo HTTP request fails during polling, sensor update, or tick
- **THEN** the frontend SHALL surface the failure in the UI and SHALL continue running the animation loop without crashing the page

### Requirement: Documentation shall explain the local demo model
Project documentation SHALL explain how to run the backend, open the demo UI, understand the distinction between local browser movement and backend firmware state, and clarify that `/simulation/*` exists only for the local simulator/demo and is not part of the Android or external client command contract.

#### Scenario: User reads the demo setup instructions
- **WHEN** a developer opens the project README
- **THEN** the documentation SHALL explain how to run `cargo run`, open `/demo`, understand that the browser owns only visual position while the backend owns firmware state, and understand that `/simulation/*` is local-only and simulator-only
