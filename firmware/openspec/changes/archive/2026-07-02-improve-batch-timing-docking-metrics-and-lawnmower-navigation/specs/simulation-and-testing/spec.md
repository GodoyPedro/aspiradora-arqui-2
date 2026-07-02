## MODIFIED Requirements

### Requirement: Simulated defaults and helpers shall support deterministic tests
Simulated battery initialization SHALL be configurable with a default suitable for tests. Simulation mutation helpers MAY be provided for tests or demos, but they SHALL remain separate from the main command protocol.

#### Scenario: Test configures startup battery
- **WHEN** a test initializes the simulated battery driver with an explicit percentage
- **THEN** the first reported `RobotStatus` SHALL reflect that configured battery percentage

#### Scenario: HTTP test mutates simulated state without public endpoints
- **WHEN** an HTTP integration test needs to inject sensor, battery, or docking conditions
- **THEN** the test SHALL do so through test-only access to the in-memory simulation state rather than through public HTTP endpoints

#### Scenario: Batch run reset is deterministic
- **WHEN** the local batch runner starts a new run
- **THEN** it SHALL reset frontend-owned pose, heading, coverage, timeline, local sensors, collision state, docking state, turn episode state, and stuck counters before starting the next simulation

### Requirement: Firmware shall include the minimum required HTTP integration coverage
The test suite SHALL include HTTP integration tests that exercise the in-memory `axum` router directly, use isolated application state per test, and validate status codes, JSON bodies, state transitions, actuator state, and structured error responses for the documented API.

#### Scenario: Status endpoint integration coverage
- **WHEN** the HTTP integration suite runs
- **THEN** it SHALL verify that `GET /status` returns HTTP `200`, exposes the initial `Standby` state and required telemetry fields, remains readable in `Error`, and does not mutate robot state

#### Scenario: Command success-path integration coverage
- **WHEN** the HTTP integration suite runs
- **THEN** it SHALL verify documented success-path behavior for `start`, `stop`, `pause`, `return-to-dock`, `manual-move`, `mode`, and `clear-error`

#### Scenario: Command validation integration coverage
- **WHEN** the HTTP integration suite runs
- **THEN** it SHALL verify malformed JSON, missing fields, invalid manual speed inputs, invalid direction values, and unsupported mode requests return the documented HTTP `400` behavior

#### Scenario: Invalid-state integration coverage
- **WHEN** the HTTP integration suite runs
- **THEN** it SHALL verify disallowed commands such as invalid pause, manual movement while `Charging` or `Error`, and other documented state conflicts return HTTP `409` with structured error payloads

#### Scenario: Safety and runtime visibility integration coverage
- **WHEN** the HTTP integration suite injects critical sensor conditions and triggers `tick()`
- **THEN** it SHALL verify `/status` reflects `Error`, the correct `current_error`, and stopped actuators for the documented safety cases

#### Scenario: Battery and docking visibility integration coverage
- **WHEN** the HTTP integration suite injects low battery or docking conditions and triggers `tick()`
- **THEN** it SHALL verify `/status` reflects the documented `Cleaning`, `ReturningToDock`, `Charging`, and `Standby` transitions visible to an external API client

#### Scenario: Cargo test execution target
- **WHEN** the full Rust test suite is run with `cargo test`
- **THEN** the HTTP integration tests SHALL run deterministically without requiring a real network port

## ADDED Requirements

### Requirement: Demo batch timing shall use one authoritative simulated clock
The local browser demo SHALL maintain one simulated elapsed-time counter per run. Every frontend frame or simulation step SHALL advance that counter by the same `delta_ms` used for `/simulation/tick`, visual motion integration, timeline `timestamp_ms`, batch stop-condition checks, stuck-detection windows, and `total_simulated_time_ms`.

#### Scenario: Batch time limit uses simulated elapsed time
- **WHEN** a batch run compares elapsed time against `max_simulated_time_ms`
- **THEN** it SHALL compare `activeRun.lastSimulatedTimeMs` to the configured limit and SHALL NOT use real elapsed wall-clock time for that stop condition

#### Scenario: Timeline timestamps follow simulated delta
- **WHEN** the demo loop records timeline frames during a run
- **THEN** `timestamp_ms` SHALL remain non-decreasing milliseconds from the start of that run and SHALL advance from the same simulated `delta_ms` basis used by the backend tick

#### Scenario: Large frame deltas stay coherent
- **WHEN** one visual frame represents a large simulated delta
- **THEN** the demo MAY split that interval into bounded substeps, but movement, sensors, tick progression, and timeline updates SHALL remain coherent with the same total simulated elapsed time

### Requirement: Return-to-dock diagnostics shall distinguish valid alignment from real stuck conditions
During `RETURNING_TO_DOCK`, in-place rotation that improves heading toward the dock SHALL count as valid progress. Docking stuck detection SHALL consider distance-to-dock progress, heading-error improvement, blocked forward approach, repeated recovery events, and lack of both positional and angular progress before classifying a failure.

#### Scenario: Heading improvement is not treated as docking stuck
- **WHEN** the robot rotates in place during `RETURNING_TO_DOCK` and heading error to the dock is decreasing
- **THEN** the batch runner SHALL NOT classify that interval as no-movement stuck solely because `x/y` changed little or not at all

#### Scenario: Docking failure requires lack of meaningful progress
- **WHEN** the robot neither reduces distance to dock, nor improves heading toward the dock, nor escapes repeated blocked-forward recovery for the configured simulated duration
- **THEN** the run SHALL finish as `DOCKING_FAILED_OR_TIMEOUT`

#### Scenario: Charging transition clears transient docking sensors
- **WHEN** docking is detected and the backend transitions the robot to `CHARGING`
- **THEN** the demo SHALL stop frontend movement, reflect `0/0` wheel speeds, clear transient `obstacle_detected` and `bumper_pressed`, and log both `DOCK_DETECTED` and `CHARGING_STARTED`

### Requirement: Docking guidance shall be proportional and bounded
The demo SHALL guide `RETURNING_TO_DOCK` with simple proportional steering rather than aggressive fixed over-corrections. It SHALL compute target heading and heading error, use capped proportional rotation for large errors, differential steering for moderate errors, forward approach for small errors, and slower speeds near the dock.

#### Scenario: Docking guidance avoids oscillation at high speed
- **WHEN** the demo runs at high `simulation_speed`, such as `10x`, while returning to dock
- **THEN** the guidance SHALL reduce overshoot and repeated left/right heading oscillation compared with fixed aggressive turn logic

#### Scenario: Docking recovery remains bounded and explicit
- **WHEN** the robot is physically blocked while approaching the dock
- **THEN** the demo SHALL log `DOCK_BLOCKED`, perform a bounded `DOCK_RECOVERY_TURN` and optional short retry movement, recompute target heading, and continue docking guidance without falling back to normal random cleaning behavior

### Requirement: Cleaning and docking metrics shall use canonical event semantics
Run logs, run summaries, and batch summaries SHALL use consistent definitions for turn-related metrics and event names. `turns` SHALL count canonical cleaning turn episodes, including `LANE_TURN_TO_SHIFT`, `LANE_TURN_TO_REVERSE`, obstacle-bypass turns, and pattern-recovery turns. `TURNING` frames SHALL NOT count as separate turns, docking alignment/recovery events SHALL remain distinguishable from cleaning turns, and `ANTI_LOOP_ESCAPE` SHALL be reserved for actual anti-loop intervention.

#### Scenario: Anti-loop escape is not reused for normal left turns
- **WHEN** a normal cleaning turn starts with `turn_direction = left`
- **THEN** the log SHALL use normal turn events such as `TURN_START`, `TURNING`, and `TURN_END`, and SHALL NOT emit `ANTI_LOOP_ESCAPE` unless a real anti-loop intervention happened

#### Scenario: Turn counts remain consistent across summaries
- **WHEN** one run is persisted and later summarized in batch metrics
- **THEN** the turn-counting rule used by the timeline summary, run summary, and batch summary SHALL match for that same run

### Requirement: Demo AUTO visual navigation shall be controlled by a frontend-owned lane-pattern controller
The browser demo SHALL use a frontend-owned lane-pattern controller for visual `AUTO` navigation. It SHALL run only while backend state is `Cleaning` and the active mode is `AUTO`. It SHALL NOT run during `ReturningToDock`, `Charging`, `ManualControl`, `Paused`, or `Error`. The controller SHALL produce `demo_left_wheel_speed`, `demo_right_wheel_speed`, and `demo_navigation_phase` as the visual movement command used by the frontend simulation.

#### Scenario: Frontend controller applies only during AUTO cleaning
- **WHEN** backend state is `Cleaning` and active mode is `AUTO`
- **THEN** the frontend lane-pattern controller SHALL own the visual movement command for simulated `x/y/heading`

#### Scenario: Frontend controller stays off outside AUTO cleaning
- **WHEN** the demo enters `ReturningToDock`, `Charging`, `ManualControl`, `Paused`, or `Error`
- **THEN** the frontend lane-pattern controller SHALL stop issuing lane-pattern movement commands until `Cleaning/AUTO` resumes

#### Scenario: Logs can distinguish backend and visual wheel commands
- **WHEN** the demo persists a run log while frontend visual navigation is active
- **THEN** the log SHOULD distinguish backend-reported wheel speeds, frontend `demo_left_wheel_speed` / `demo_right_wheel_speed`, and `demo_navigation_phase`

### Requirement: Demo AUTO cleaning shall use an explicit lane-phase state machine
The frontend lane-pattern controller SHALL implement explicit phases `LANE_DRIVE`, `LANE_TURN_TO_SHIFT`, `LANE_SHIFT`, `LANE_TURN_TO_REVERSE`, `OBSTACLE_BYPASS_TURN`, `OBSTACLE_BYPASS_OFFSET`, `OBSTACLE_BYPASS_REJOIN`, and `PATTERN_RECOVERY`.

#### Scenario: Lane drive keeps straight progress
- **WHEN** `demo_navigation_phase` is `LANE_DRIVE`
- **THEN** the robot SHALL drive straight along `lane_heading`, SHALL NOT start a long anticipatory turn from normal frontal detection alone, and SHALL continue the lane until contact or blocked motion occurs

#### Scenario: Lane end turns into shift
- **WHEN** wall contact or blocked forward motion marks the end of a lane
- **THEN** the frontend controller SHALL enter `LANE_TURN_TO_SHIFT` and rotate approximately 90 degrees toward `lane_shift_side`

#### Scenario: Lane shift completion or blockage is explicit
- **WHEN** the controller is in `LANE_SHIFT`
- **THEN** it SHALL move approximately one `lane_spacing_px`, SHALL log `LANE_BLOCKED` and enter `PATTERN_RECOVERY` if the shift is blocked, and SHALL otherwise proceed to `LANE_TURN_TO_REVERSE`

#### Scenario: Lane reverse increments lane index
- **WHEN** `LANE_TURN_TO_REVERSE` completes
- **THEN** the controller SHALL face the opposite lane direction, increment `lane_index`, and return to `LANE_DRIVE`

#### Scenario: Obstacle bypass is deterministic
- **WHEN** obstacle contact interrupts a lane
- **THEN** the controller SHALL enter `OBSTACLE_BYPASS_TURN`, turn away from the obstacle, optionally use bounded `OBSTACLE_BYPASS_OFFSET`, and then attempt `OBSTACLE_BYPASS_REJOIN` parallel to the original lane direction instead of spinning randomly

#### Scenario: Pattern recovery is bounded
- **WHEN** lane shift or obstacle bypass fails
- **THEN** the controller SHALL enter `PATTERN_RECOVERY`, log `PATTERN_RECOVERY`, avoid indefinite spinning, and either resume `LANE_DRIVE` or choose a bounded lane transition

### Requirement: Demo AUTO lane parameters shall use documented constants
The frontend lane-pattern controller SHALL use documented constants for `lane_spacing_px`, initial `lane_heading`, alternating `lane_direction`, `lane_shift_side`, angular tolerance, and shift-completion distance tolerance. Default demo values SHALL be `lane_spacing_px = robot_radius * 1.6`, horizontal starting `lane_heading` unless a map-specific heading is already defined, approximately 10 degrees angular tolerance, and approximately 5 px shift-completion tolerance. Exact values MAY be adjusted in implementation, but they SHALL remain named constants documented in code comments and README.

#### Scenario: Lane constants start from stable defaults
- **WHEN** a run starts without a map-specific override
- **THEN** `lane_index` SHALL start at `0`, `lane_direction` SHALL alternate forward/reverse by lane, and `lane_shift_side` SHALL stay consistent unless repeated blockage requires recovery

### Requirement: Demo AUTO obstacle avoidance shall be contact-driven
The frontend lane-pattern controller SHALL treat `bumper_pressed` or actual collision/contact as the primary trigger for wall and obstacle avoidance. `obstacle_detected` SHALL remain very short range, and `obstacle_detected` alone SHALL NOT trigger a full lane transition or long avoidance maneuver unless the obstacle is effectively at contact distance. Drop-off or other critical safety sensors SHALL remain immediate safety exceptions.

#### Scenario: Wall contact triggers lane transition
- **WHEN** the robot reaches a wall by actual contact or blocked forward motion during `Cleaning/AUTO`
- **THEN** the demo SHALL treat that event as lane end and perform a bounded lane-turn / lane-shift / reverse-direction sequence

#### Scenario: Obstacle contact triggers controlled bypass
- **WHEN** the robot contacts an obstacle inside a lane
- **THEN** the demo SHALL log `BUMPER_CONTACT`, perform a predictable local bypass or lane recovery maneuver, and resume the lane pattern if possible instead of spinning randomly for many frames

### Requirement: Pattern event logging shall be required for lane and bypass phases
The demo SHALL emit required pattern events whenever the corresponding lane-pattern phase transition happens. Event details SHALL include `demo_navigation_phase`, `lane_index`, `lane_direction`, `lane_heading`, `lane_shift_side`, `lane_spacing_px`, `target_heading`, `blocked_target`, `contact_type`, and `obstacle_id` when available.

#### Scenario: Pattern events map to phase transitions
- **WHEN** the frontend lane-pattern controller enters or completes a documented phase transition
- **THEN** entering a lane SHALL emit `LANE_START`, ending a lane due to wall/contact SHALL emit `LANE_END`, entering `LANE_SHIFT` SHALL emit `LANE_SHIFT_START`, completing the shift SHALL emit `LANE_SHIFT_END`, blocked lane shift SHALL emit `LANE_BLOCKED`, obstacle bypass start SHALL emit `OBSTACLE_BYPASS_START`, obstacle bypass success SHALL emit `OBSTACLE_BYPASS_END`, obstacle bypass failure SHALL emit `OBSTACLE_BYPASS_FAILED`, and fallback/recovery SHALL emit `PATTERN_RECOVERY`

### Requirement: Pattern metrics shall be mandatory in run summaries and batch summaries
Run summaries SHALL include `lane_starts`, `lane_ends`, `lane_shifts`, `lane_blocked_events`, `obstacle_bypass_attempts`, `obstacle_bypass_failures`, and `pattern_recoveries`. Batch aggregate metrics SHALL include `total_lane_starts`, `total_lane_ends`, `total_lane_shifts`, `total_lane_blocked_events`, `total_obstacle_bypass_attempts`, `total_obstacle_bypass_failures`, and `total_pattern_recoveries`. Their definitions SHALL be documented consistently across logs, run summaries, and batch summaries.

#### Scenario: Pattern metrics remain structurally stable
- **WHEN** a run summary and batch summary are generated
- **THEN** the required pattern metric fields SHALL be present even when their values are `0`

### Requirement: Batch validation shall confirm calmer pattern behavior
Manual and batch validation SHALL verify that the lane-pattern controller produces calmer behavior than the prior random-bounce baseline without requiring perfect coverage.

#### Scenario: Batch validation confirms simulated-time cutoff and calmer pattern
- **WHEN** a batch of at least 3 maps runs at `10x` with 2 obstacles, `max_simulated_time_ms = 30000`, target coverage below `100%`, and Return to Dock enabled
- **THEN** the resulting logs and summaries SHALL show that time-limited runs stop according to simulated elapsed time, docking failures are not caused by valid alignment rotation, there are visible straight lane segments, coverage growth is more regular across the room, there are no repeated random left/right turns without phase progress, there is no long unexplained `TURNING` streak, and bumper/wall contacts per simulated minute are lower than the prior random-bounce baseline on comparable maps
