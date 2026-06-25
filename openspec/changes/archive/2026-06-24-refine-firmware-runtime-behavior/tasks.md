## 1. Status Read Refinement

- [x] 1.1 Update the API status path so `GET /status` reads `RobotStatus` safely without going through command rejection logic
- [x] 1.2 Add regression coverage proving `GET /status` succeeds even when the robot is already in `Error` or a critical condition is active

## 2. AUTO Runtime Reactions

- [x] 2.1 Update `tick()` so `obstacle_detected` triggers a simple avoidance maneuver while preserving `RobotState = Cleaning`
- [x] 2.2 Update `tick()` so `bumper_pressed` triggers the same simple avoidance maneuver while preserving `RobotState = Cleaning`
- [x] 2.3 Add unit tests proving obstacle and bumper reactions change wheel speeds without transitioning out of `Cleaning`

## 3. Dust Container Full Error Handling

- [x] 3.1 Add the `DustContainerFull` robot error variant and ensure status reporting can surface `DUST_CONTAINER_FULL`
- [x] 3.2 Update runtime behavior so `dust_container_full` during `Cleaning` stops wheels, suction, and brushes and transitions the robot to `Error`
- [x] 3.3 Add tests proving `dust_container_full` sets `current_error = DUST_CONTAINER_FULL` and stops all actuators

## 4. MANUAL_MOVE Validation Tightening

- [x] 4.1 Enforce `MANUAL_MOVE` speed validation for the range `0..=100` and direction-aware zero-speed rules while keeping `duration_ms > 0`
- [x] 4.2 Ensure invalid manual move speed combinations return structured bad-request errors that map to HTTP `400`
- [x] 4.3 Add unit and HTTP tests covering invalid manual speed inputs and unchanged robot state after rejection

## 5. Regression Review

- [x] 5.1 Review the refined behavior against the existing architecture constraints and confirm no new simulation endpoints or navigation systems were introduced
- [x] 5.2 Update README or inline documentation only if needed to reflect the refined runtime behavior without broadening project scope
