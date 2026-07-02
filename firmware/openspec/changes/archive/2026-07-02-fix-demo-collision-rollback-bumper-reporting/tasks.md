## 1. Frontend Collision Reporting

- [x] 1.1 Detect blocked forward movement when backend wheel speeds indicate advance but the proposed visual movement is rejected by obstacle or wall collision
- [x] 1.2 Implement the explicit blocked-movement API sequence: keep last valid pose, set local `bumper_pressed = true`, `POST /simulation/sensors`, immediately `POST /simulation/tick`, then consume the returned backend status
- [x] 1.3 Keep `obstacle_detected` independent from rollback contact unless the frontal proximity sensor also sees an obstacle
- [x] 1.4 Record a clear timeline event such as `BUMPER_CONTACT` or `COLLISION_ROLLBACK` when visual movement is blocked
- [x] 1.5 Avoid duplicate bumper event spam while already waiting for backend reaction, and emit a diagnostic event such as `BUMPER_REPORTED_WAITING_FOR_TURN` if the backend still returns forward speeds after bumper was reported

## 2. High-Speed Visual Movement Robustness

- [x] 2.1 Split visual movement into simple internal substeps before collision checks, especially at high simulation speed
- [x] 2.2 Define a bounded maximum substep distance and a maximum substep count per frame, dropping any safe remainder if the frame cap is reached
- [x] 2.3 Stop at the last valid pose when a substep collides with a wall or obstacle instead of tunneling through geometry
- [x] 2.4 Ensure the same blocked substep path reports bumper/contact so the backend can start a turn on the next tick
- [x] 2.5 Clear `bumper_pressed` only after the robot is no longer physically blocked and a future forward movement from the current heading would be valid

## 3. Logging And Documentation

- [x] 3.1 Ensure the log timeline shows blocked movement, bumper/contact reporting, turn start/end, and resumed forward motion with changing `x/y`
- [x] 3.2 Include simple JSON-compatible collision target details in blocked-movement events, such as wall vs obstacle and obstacle id when available
- [x] 3.3 Prevent long log sequences where `x/y` stay unchanged while wheel speeds remain `60/60` and both `obstacle_detected` and `bumper_pressed` stay false
- [x] 3.4 Update README demo documentation to describe collision rollback reporting, explicit `sensors -> tick -> status` sequencing, bounded high-speed substeps, and conservative bumper clearing behavior

## 4. Validation

- [x] 4.1 Add or update automated coverage only if needed to protect the existing bumper-triggered turn behavior
- [x] 4.2 Manually verify at `10x` speed that obstacle or wall contact reports bumper, triggers a turn, and avoids a prolonged visual freeze with `60/60`
- [x] 4.3 Run the demo for at least 30 seconds and confirm `x/y` continue changing except during intentional in-place turns
- [x] 4.4 Validate the generated log against the freeze invariant so it does not contain more than the allowed small number of consecutive silent forward-freeze frames
