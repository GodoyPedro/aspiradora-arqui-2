## 1. AUTO Movement Recovery

- [x] 1.1 Update `tick()` so AUTO cleaning restores forward wheel speeds only after a prior avoidance maneuver or an equivalent avoidance-pattern condition
- [x] 1.2 Keep the existing simple avoidance maneuver for active `obstacle_detected` and `bumper_pressed`
- [x] 1.3 Add or update backend tests proving obstacle avoidance and forward AUTO recovery without changing unrelated command behavior

## 2. Demo Frontend Refinement

- [x] 2.1 Update the browser demo loop to recompute obstacle proximity and collision on every cycle using the robot heading or a forward sensor cone/ray
- [x] 2.2 Ensure the demo sends `obstacle_detected = false` when the obstacle is no longer in front of the robot, even if the robot is still nearby
- [x] 2.3 Ensure the demo sends `bumper_pressed = true` only for actual collision/overlap and clears it once collision ends
- [x] 2.4 Prevent visual penetration into obstacles or walls by rolling back or clamping to the last valid position
- [x] 2.5 Add an optional simple frontend-only avoidance cooldown if needed to make turn-away behavior stable
- [x] 2.6 Change the demo manual controls to send a single `MANUAL_MOVE` request with `duration_ms = 2000` per click
- [x] 2.7 Increase the local visual movement speed and expose it through a named frontend constant
- [x] 2.8 Make active and cleared obstacle-related sensor flags update clearly in the demo status panel

## 3. HTTP Coverage

- [x] 3.1 Add or update HTTP tests proving obstacle flags can be set through `/simulation/sensors`
- [x] 3.2 Add or update HTTP tests proving that clearing obstacle flags and calling `/simulation/tick` restores forward AUTO wheel speeds
- [x] 3.3 Re-run the existing HTTP API and simulation endpoint coverage to confirm no regressions

## 4. Documentation And Verification

- [x] 4.1 Update the README demo section to document the 2-second manual default and that manual clicks send one request per click
- [x] 4.2 Update the README demo section to document that visual speed is a frontend simulation parameter
- [x] 4.3 Update the README demo section to explain that `obstacle_detected` means an object in front or in the forward cone, while `bumper_pressed` means actual collision
- [x] 4.4 Update the README demo section to explain that the browser demo prevents visual overlap by rolling back or clamping position
- [x] 4.5 Run `cargo test` and confirm all existing and updated tests pass
