## MODIFIED Requirements

### Requirement: Demo verification shall cover local simulator controls
Manual verification of the plain JavaScript demo SHALL confirm coverage painting, speed control, reset behavior, new map generation, session rotation, and that generated log dumps include map, movement timeline, and coverage data.

#### Scenario: Forward rollback reports contact instead of silent freeze
- **WHEN** the backend reports forward wheel speeds, the frontend proposes a visual movement step, and that step is rejected because it would collide with a wall or obstacle
- **THEN** the demo SHALL keep the robot at the last valid pose, set local `bumper_pressed = true`, `POST /simulation/sensors` first, immediately `POST /simulation/tick` with the current `delta_ms`, and use the returned `RobotStatus` so the backend can enter the normal `AUTO` turn behavior instead of remaining visually frozen with `60/60`

#### Scenario: High-speed movement uses bounded substeps to avoid tunneling
- **WHEN** the demo runs at a high simulation speed such as `10x`
- **THEN** the frontend SHALL split visual movement into smaller internal substeps, constrain each substep to a small maximum distance such as `4px` to `8px`, cap the total substeps per frame, stop at the last valid pose on collision, and report blocked contact if a substep cannot advance

#### Scenario: Collision log includes source detail
- **WHEN** the demo records a blocked-movement event such as `COLLISION_ROLLBACK` or `BUMPER_CONTACT`
- **THEN** the timeline event SHALL include simple JSON-compatible detail that distinguishes wall contact from obstacle contact and includes obstacle identity when available

#### Scenario: Duplicate bumper events are suppressed while waiting for turn
- **WHEN** a blocked movement was already reported as bumper/contact and the frontend is waiting for the backend to react
- **THEN** the demo SHALL avoid spamming repeated identical `BUMPER_CONTACT` events every frame, MAY keep `bumper_pressed = true`, and SHALL emit a diagnostic waiting event if the backend still returns forward wheel speeds after the bumper report

#### Scenario: Bumper clears after the robot is no longer blocked
- **WHEN** the robot turns away from the obstacle or wall, is no longer physically blocked, and a future forward substep becomes valid again
- **THEN** the frontend SHALL clear `bumper_pressed` so the demo does not remain stuck in a permanent avoidance loop

#### Scenario: Freeze invariant prevents silent forward stalls
- **WHEN** a generated demo log is reviewed for this failure mode
- **THEN** it SHALL NOT contain more than a small bounded number of consecutive frames, such as `3` to `5`, where the robot remains in `CLEANING`, wheel speeds indicate forward motion, `x/y` do not change, `obstacle_detected = false`, `bumper_pressed = false`, and no collision/rollback diagnostic event is present

#### Scenario: Frontend manual verification checklist covers rollback contact
- **WHEN** the implementation is reviewed before merge
- **THEN** the verification notes SHALL confirm that at `10x` speed the robot does not freeze in place with `60/60`, blocked movement is logged as contact or rollback with source detail, the backend turns immediately after bumper is reported, and `x/y` resume changing after the turn
