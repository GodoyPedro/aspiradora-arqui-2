## 1. Backend Demo Routing

- [x] 1.1 Add backend routes for `GET /demo` and `/static/demo.css` and `/static/demo.js`
- [x] 1.2 Add demo/testing-only routes for `POST /simulation/sensors`, `POST /simulation/battery`, `POST /simulation/docking`, and `POST /simulation/tick`
- [x] 1.3 Add request and response models for the simulation endpoints, including partial-update payloads for sensors and docking
- [x] 1.4 Keep the new `/simulation/*` routes isolated from the main command API behavior and error handling
- [x] 1.5 Embed demo HTML, CSS, and JavaScript assets at compile time so `cargo run` serves them without runtime filesystem dependencies

## 2. Static Demo Frontend

- [x] 2.1 Add the demo HTML page with a room canvas or scene area, controls, status panel, and sensor panel
- [x] 2.2 Add demo CSS for a clear local visualization of the room, robot, obstacles, dock, and robot heading
- [x] 2.3 Add demo JavaScript to call the existing command endpoints, poll `GET /status`, and render `RobotStatus`

## 3. Local Visual Simulation Loop

- [x] 3.1 Implement local browser-only robot position and heading state with a reset control
- [x] 3.2 Update local visual movement from `left_wheel_speed` and `right_wheel_speed` on a fixed interval
- [x] 3.3 Compute obstacle proximity and collision locally and send `obstacle_detected` and `bumper_pressed` through `/simulation/sensors`
- [x] 3.4 Trigger `POST /simulation/tick` from the demo loop and re-render the returned status
- [x] 3.5 Add a simple guard to prevent concurrent simulation cycles and render HTTP failures in the UI without crashing the loop

## 4. Simulation Endpoint Coverage

- [x] 4.1 Add HTTP integration tests for `POST /simulation/sensors` and verify visible sensor flags in returned or subsequent status
- [x] 4.2 Add HTTP integration tests for `POST /simulation/battery` and verify `battery_percent` changes
- [x] 4.3 Add HTTP integration tests for `POST /simulation/docking` and verify docking-related state changes after `tick()`
- [x] 4.4 Add HTTP integration tests for `POST /simulation/tick` and verify it returns `RobotStatus`
- [x] 4.5 Add validation and error-handling tests for malformed JSON, invalid simulation payload shapes, and invalid battery values on `/simulation/*`
- [x] 4.6 Add static asset serving tests for `GET /demo`, `GET /static/demo.css`, and `GET /static/demo.js`
- [x] 4.7 Re-run existing HTTP API coverage to confirm the main command endpoints still behave as specified

## 5. Documentation And Verification

- [x] 5.1 Update the README with demo usage instructions and a clear explanation of `/simulation/*` as demo/testing-only endpoints
- [x] 5.2 Document that robot movement in the demo is local browser simulation while firmware state remains backend-driven
- [x] 5.3 Add a README note that the demo is local-only and uses simulator-only endpoints outside the Android/external command contract
- [x] 5.4 Run `cargo test` and confirm the existing suite plus new simulation endpoint coverage passes
