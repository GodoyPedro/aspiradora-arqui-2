## Purpose

Define the in-memory simulation model, deterministic helper behavior, and minimum test coverage expectations for the firmware.
## Requirements
### Requirement: Firmware shall provide in-memory simulated drivers
The firmware SHALL implement `SimulatedWheelMotorDriver`, `SimulatedSuctionDriver`, `SimulatedBrushDriver`, `SimulatedSensorReader`, `SimulatedBatteryDriver`, and `SimulatedDockingDriver` using in-memory state that can be inspected and mutated during tests.

#### Scenario: Simulator stores actuator state
- **WHEN** the controller commands wheel, suction, or brush changes
- **THEN** the corresponding simulated driver SHALL retain the new state in memory for later status reporting and test assertions

### Requirement: Simulated drivers shall expose the required state surface
The simulated drivers SHALL model at least `left_speed`, `right_speed`, `running` for suction, `running` for brushes, sensor flags for obstacle, drop-off, bumper, dust container full, wheel stuck, brush stuck, and top cover open, battery `percentage`, charging state, and docking availability or detection state through `SimulatedDockingDriver`.

#### Scenario: Test injects sensor fault
- **WHEN** a unit test sets `brush_stuck` to true in the simulated sensor reader
- **THEN** the next firmware evaluation SHALL observe that flag through the normal `SensorReader` interface

### Requirement: Status reporting shall expose required telemetry
`RobotStatus` SHALL include at least `state`, `cleaning_mode`, `battery_percent`, `is_charging`, `suction_enabled`, `brushes_enabled`, `left_wheel_speed`, `right_wheel_speed`, `current_error`, and the relevant sensor flags needed to understand the robot's current condition.

#### Scenario: Status reflects simulated state
- **WHEN** the robot is queried after command execution or a periodic tick
- **THEN** the returned `RobotStatus` SHALL reflect the current driver state, battery state, robot state, and active error consistently

#### Scenario: Status remains readable in error conditions
- **WHEN** the robot is already in `Error` or a critical condition is active
- **THEN** reading status SHALL still return the current `RobotStatus` instead of failing with a conflict

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

### Requirement: Firmware shall include the minimum required unit coverage
The test suite SHALL cover starting cleaning from `Standby`, stopping cleaning from `Cleaning`, pausing cleaning from `Cleaning`, manual movement changing wheel speeds, low battery triggering `ReturningToDock`, dock detection transitioning `ReturningToDock` to `Charging`, each required critical sensor fault transitioning to `Error` and stopping all actuators, clearing an error when safe, rejecting unsupported cleaning modes, and rejecting or safely ignoring invalid command/state combinations.

#### Scenario: AUTO turn expiry regression test
- **WHEN** a unit test starts an `AUTO` turn via obstacle or bumper, advances simulated time beyond the turn window, and clears the blocker
- **THEN** the test SHALL verify that the robot resumes forward movement instead of spinning indefinitely

#### Scenario: Manual deadline expiry regression test
- **WHEN** a unit test starts a manual move and then advances simulated time beyond the manual deadline
- **THEN** the test SHALL verify transition back to `Standby` with stopped wheels

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

### Requirement: Demo verification shall cover local simulator controls
Manual verification of the plain JavaScript demo SHALL confirm coverage painting, speed control, reset behavior, new map generation, session rotation, and that generated log dumps include map, movement timeline, and coverage data.

#### Scenario: Long turn guard breaks repeated in-place spinning
- **WHEN** the robot remains in `CLEANING`, `x/y` stay effectively unchanged, and wheel speeds indicate repeated turning for longer than the documented demo threshold
- **THEN** the demo SHALL emit a guard or escape event such as `LONG_TURN_GUARD`, `ESCAPE_TURN`, or `ESCAPE_FORWARD_ATTEMPT` and SHALL NOT continue producing only repeated `TURNING` frames indefinitely

#### Scenario: Long turn guard causes a real follow-up behavior
- **WHEN** `LONG_TURN_GUARD` intervenes
- **THEN** the next few frames SHALL show either changed `x/y` or a clear blocked-movement event with collision target detail, and the demo SHALL NOT simply fall back into another indefinite `TURNING` chain without evaluating a forward attempt

#### Scenario: Proximity does not retrigger the same turn forever
- **WHEN** the frontend recomputes `obstacle_detected` while a timed turn is already in progress
- **THEN** that proximity signal MAY remain visible for UI/logging, but it SHALL NOT restart a fresh turn episode until the previous turn ended and a forward attempt or fresh blocker justified a new turn

#### Scenario: Turn episodes remain stable during one turn
- **WHEN** a turn episode is already open
- **THEN** `TURNING` frames SHALL belong to the current episode, `TURN_END` SHALL close it, and `obstacle_detected` changes during that open episode SHALL NOT create a new turn episode

#### Scenario: High-speed rotation uses bounded angular substeps
- **WHEN** the demo runs at a high simulation speed and the robot performs a large in-place rotation
- **THEN** the frontend SHALL split that rotation into bounded angular substeps so heading changes remain stable and sensor toggling does not alias the same turn into repeated retriggers

#### Scenario: Return to Dock button reflects backend response
- **WHEN** the user presses the Return to Dock button
- **THEN** the frontend SHALL call `POST /commands/return-to-dock`, consume the returned status, log `RETURN_TO_DOCK` when accepted, update the UI immediately to backend state `RETURNING_TO_DOCK`, and surface the backend error if the command is refused

#### Scenario: Rejected Return to Dock does not start homing
- **WHEN** `POST /commands/return-to-dock` fails
- **THEN** the frontend SHALL NOT start local docking guidance, SHALL show the backend error, and SHALL log `RETURN_TO_DOCK_REJECTED` with error detail

#### Scenario: Return-to-dock steers toward the docking station
- **WHEN** backend state is `RETURNING_TO_DOCK` and the robot is far from the docking station
- **THEN** the demo SHALL use the current map docking position to orient the visual robot toward the dock, advance when sufficiently aligned, and reduce distance to dock over time rather than continuing forever on its old heading

#### Scenario: Docking guidance overrides normal AUTO movement
- **WHEN** backend state is `RETURNING_TO_DOCK`
- **THEN** the frontend SHALL use demo-only docking guidance derived from robot pose, dock position, heading error, distance to dock, and blocked movement state, and SHALL NOT blindly interpret backend `30/30` wheel speeds as straight-line movement

#### Scenario: Docking log distinguishes backend speeds from demo guidance
- **WHEN** the demo logs return-to-dock movement
- **THEN** event details SHALL be able to distinguish backend wheel speeds from frontend docking guidance by including fields such as `backend_left_wheel_speed`, `backend_right_wheel_speed`, `demo_docking_phase`, `demo_target_heading`, `heading_error`, and `distance_to_dock`

#### Scenario: Dock detection completes charging transition
- **WHEN** the robot reaches or overlaps the docking station closely enough during `RETURNING_TO_DOCK`
- **THEN** the frontend SHALL send `dock_detected = true`, the backend SHALL transition to `CHARGING` on the next tick, and visual movement SHALL stop

#### Scenario: Backend return-to-dock state transitions stay explicit
- **WHEN** tests exercise `POST /commands/return-to-dock` and docking completion
- **THEN** they SHALL verify allowed or refused command states explicitly, verify `RETURNING_TO_DOCK -> CHARGING` through `/simulation/docking`, and verify that charging begins with wheel speeds `0/0`

#### Scenario: Docking blockage triggers recovery instead of permanent 30/30 stall
- **WHEN** return-to-dock movement is blocked by a wall or obstacle
- **THEN** the demo SHALL emit a docking blockage or recovery event such as `DOCK_BLOCKED`, `DOCK_RECOVERY_TURN`, or `DOCK_STUCK_DIAGNOSTIC`, perform a simple local recovery maneuver, and continue trying to reach the dock

#### Scenario: Docking progress invariant prevents silent stalls
- **WHEN** a generated log is reviewed for `RETURNING_TO_DOCK`
- **THEN** it SHALL NOT contain a long period where the robot stays in `RETURNING_TO_DOCK`, distance to dock is not decreasing, `x/y` are unchanged or moving away from the dock, and no `DOCK_BLOCKED`, `DOCK_RECOVERY_TURN`, or `DOCK_STUCK_DIAGNOSTIC` event appears

#### Scenario: Timeline timestamps remain analyzable
- **WHEN** the demo generates timeline frames
- **THEN** `timestamp_ms` values SHALL be non-decreasing across the log so downstream analysis can rely on frame order and duration calculations

#### Scenario: Frontend manual verification checklist covers turning and docking
- **WHEN** the implementation is reviewed before merge
- **THEN** the verification notes SHALL confirm that AUTO at `10x` no longer shows long silent endless turning, the guard or escape events appear when needed, the Return to Dock button changes backend state when accepted, return-to-dock visibly reorients toward the dock, distance to dock generally decreases, and the log contains the expected docking and charging events

### Requirement: Demo verification shall cover finite turning and clean event logs
Manual verification of the plain JavaScript demo SHALL confirm that turning is finite, forward movement resumes, stale sensors clear on stop/reset, and the resulting log dump no longer shows fake anti-loop events every frame.

#### Scenario: Frontend manual verification checklist
- **WHEN** the implementation is reviewed before merge
- **THEN** the verification notes SHALL confirm that the robot turns for finite time, resumes movement with changing `x/y`, Stop clears stale obstacle/bumper flags, and the log shows bounded `TURN_START` plus ordinary turning frames rather than endless repeated fake escape events

### Requirement: Demo batch simulation runner shall orchestrate repeatable multi-run evidence collection
The local browser demo SHALL provide a batch runner that executes multiple generated-map simulations without manual intervention, keeps the existing static frontend model, and preserves the primary command API unchanged. The batch runner SHALL allow configuring run count, max simulated time, target coverage percentage, batch simulation speed, whether return-to-dock is requested after the main stop condition, and which stop conditions are enabled.

#### Scenario: Batch UI exposes configuration and current progress
- **WHEN** a user opens `/demo`
- **THEN** the page SHALL expose a `Batch Simulation` section with batch configuration fields, `Start Batch`, `Stop Batch`, `Clear Batch Results`, and visible progress fields for current batch id, run index, current session id, elapsed simulated time, coverage percentage, run state, finish condition, completed count, failed/stuck count, last saved log path, and final batch summary path

#### Scenario: Manual controls are isolated during a batch
- **WHEN** a batch run is active
- **THEN** manual demo controls SHALL be disabled or clearly marked unavailable so they cannot corrupt the active run, and they SHALL become usable again after batch completion or cancellation

#### Scenario: Each batch run follows the required lifecycle
- **WHEN** the batch runner starts a run
- **THEN** it SHALL generate a new map, create a new session id, associate the run with the current batch id, reset local state, reuse the backend simulation reset flow, start cleaning through `POST /commands/start`, run the simulation loop automatically, stop on the configured condition, optionally request `POST /commands/return-to-dock`, append a final run-end event, persist the run log, and add the run metrics to the in-memory batch summary before continuing

### Requirement: Demo batch runs shall stop deterministically and classify completion consistently
Each batch run SHALL end with exactly one final `finish_reason`, chosen from `TIME_LIMIT_REACHED`, `COVERAGE_REACHED`, `DOCKED`, `DOCKING_FAILED_OR_TIMEOUT`, `STUCK_DIAGNOSTIC`, `USER_CANCELLED`, or `COMMAND_ERROR`. Batch mode SHALL include demo-only stuck detection so runs cannot continue forever.

#### Scenario: Time or coverage threshold ends a non-docking run
- **WHEN** a batch run reaches its configured time limit or target coverage while return-to-dock is disabled
- **THEN** the runner SHALL stop the run directly and assign the matching `finish_reason`

#### Scenario: Return-to-dock extends the run until a docking outcome exists
- **WHEN** a batch run reaches time or coverage and return-to-dock is enabled
- **THEN** the runner SHALL request `POST /commands/return-to-dock` and continue until the robot reaches `CHARGING`, a docking timeout/stuck condition occurs, or the user cancels the batch

#### Scenario: Batch stuck detection prevents endless runs
- **WHEN** the robot shows no meaningful `x/y` progress with non-zero wheels for a configured duration, repeatedly hits long-turn or docking stuck diagnostics too often, or stops reducing distance to dock while returning
- **THEN** the runner SHALL end the run with `STUCK_DIAGNOSTIC` or `DOCKING_FAILED_OR_TIMEOUT` according to the active phase

#### Scenario: Command errors stop the active run and are recorded
- **WHEN** a command request fails during batch mode
- **THEN** the current run SHALL stop with `finish_reason = COMMAND_ERROR`, the timeline SHALL include `BATCH_COMMAND_ERROR` with error detail, the runner SHALL attempt to save the run log, and the runner MAY continue to the next run unless the error prevents the whole batch from continuing

#### Scenario: User-triggered batch stop preserves partial results
- **WHEN** the user clicks `Stop Batch`
- **THEN** the active run SHALL finish with `finish_reason = USER_CANCELLED`, the timeline SHALL include `BATCH_STOP_USER_CANCELLED`, the runner SHALL attempt to save the active run log, the runner SHALL save a partial batch summary, and no new runs SHALL be started

### Requirement: Demo batch artifacts shall capture run-level and batch-level analysis data
Each batch run log SHALL stay compatible with the existing dump format while adding batch metadata, batch event labels, and enough information to correlate the run with the batch summary. The batch summary SHALL be self-contained and SHALL include the config used, all run summaries, aggregate metrics, and worst-run lists without requiring the UI.

#### Scenario: Batch run log records batch metadata and event labels
- **WHEN** a batch run is persisted
- **THEN** the log SHALL include `batch_id`, `run_index`, `batch_total_runs`, `batch_mode = true`, and `run_config`
- **THEN** `run_config` SHALL include `max_simulated_time`, `target_coverage`, `simulation_speed`, `return_to_dock_after_run`, enabled stop conditions, and map complexity or obstacle count if configured
- **THEN** `max_simulated_time` inside `run_config` SHALL be milliseconds
- **THEN** the timeline SHALL include relevant event labels such as `BATCH_RUN_START`, `BATCH_RUN_END`, `BATCH_STOP_TIME_LIMIT`, `BATCH_STOP_COVERAGE_REACHED`, `BATCH_RETURN_TO_DOCK_REQUESTED`, `BATCH_STOP_DOCKED`, `BATCH_STOP_DOCKING_FAILED`, `BATCH_STOP_STUCK`, `BATCH_STOP_USER_CANCELLED`, and `BATCH_COMMAND_ERROR`
- **THEN** timeline `timestamp_ms` SHALL be non-decreasing milliseconds from the start of the run or session

#### Scenario: Run summary captures the required metrics
- **WHEN** a batch run finishes
- **THEN** the runner SHALL compute a run summary using snake_case field names containing `batch_id`, `run_index`, `session_id`, `map_id`, `log_path`, `created_at`, `finished_at`, `finish_reason`, `final_state`, `final_coverage_percentage`, `total_frames`, `total_simulated_time_ms`, `total_real_time_ms` if available, and `log_save_error`
- **THEN** the run summary SHALL also contain `obstacle_count`, `obstacle_detections`, `bumper_contacts`, `wall_contacts`, `obstacle_contacts`, `turns`, `long_turn_guards`, `escape_turns`, `dock_attempted`, `docking_succeeded`, `reached_charging`, `dock_blocked_events`, `dock_stuck_diagnostics`, `stuck_diagnostics`, `max_consecutive_no_movement_frames_with_nonzero_wheels`, `max_consecutive_turning_frames_without_xy_change`, `final_x`, `final_y`, and `final_heading`
- **THEN** `total_simulated_time_ms` SHALL be milliseconds, `total_real_time_ms` SHALL be milliseconds if available, `log_save_error` SHALL be `null` when log saving succeeds, and if log saving fails `log_path` MAY be `null` and `log_save_error` SHALL contain a short error message

#### Scenario: Batch summary captures aggregate metrics and worst runs
- **WHEN** the batch completes or is cancelled
- **THEN** the runner SHALL produce a batch summary JSON containing `batch_id`, `created_at`, `finished_at`, `requested_runs`, `completed_runs`, `cancelled`, `config`, and `runs`
- **THEN** the batch summary SHALL group aggregate fields under `aggregate_metrics`
- **THEN** `aggregate_metrics` SHALL contain `average_coverage`, `min_coverage`, `max_coverage`, `average_simulated_time`, `docking_success_rate`, `most_common_finish_reason`, `total_bumper_contacts`, `total_wall_contacts`, `total_obstacle_contacts`, `total_long_turn_guards`, `total_escape_turns`, `total_stuck_diagnostics`, `total_dock_blocked_events`, and `total_dock_stuck_diagnostics`
- **THEN** the batch summary SHALL group worst-run lists under `worst_runs`
- **THEN** `worst_runs` SHALL contain `lowest_coverage`, `highest_wall_contacts`, `highest_obstacle_contacts`, `most_long_turn_guards`, `most_stuck_diagnostics`, `failed_docking`, `longest_no_movement_streak`, and `longest_turning_without_movement_streak`
- **THEN** the batch summary JSON shape SHALL be stable as `{ "batch_id": "...", "created_at": "...", "finished_at": "...", "requested_runs": 10, "completed_runs": 10, "cancelled": false, "config": {}, "runs": [], "aggregate_metrics": { "average_coverage": 0, "min_coverage": 0, "max_coverage": 0, "average_simulated_time": 0, "docking_success_rate": null, "most_common_finish_reason": null, "total_bumper_contacts": 0, "total_wall_contacts": 0, "total_obstacle_contacts": 0, "total_long_turn_guards": 0, "total_escape_turns": 0, "total_stuck_diagnostics": 0, "total_dock_blocked_events": 0, "total_dock_stuck_diagnostics": 0 }, "worst_runs": { "lowest_coverage": [], "highest_wall_contacts": [], "highest_obstacle_contacts": [], "most_long_turn_guards": [], "most_stuck_diagnostics": [], "failed_docking": [], "longest_no_movement_streak": [], "longest_turning_without_movement_streak": [] } }`

#### Scenario: Log-save failures remain visible without crashing the batch UI
- **WHEN** saving a run log fails during batch mode
- **THEN** the UI SHALL show the error, the run summary SHALL remain in memory, the run summary SHALL include log-save failure information, and the demo loop SHALL not crash

### Requirement: Demo batch validation shall prove repeatable artifact generation
Manual validation for the batch runner SHALL confirm that short batches generate unique sessions and maps, persist per-run logs and a batch summary, preserve completed artifacts after cancellation, report `finish_reason` for every run, respect coverage and docking stop modes, maintain non-decreasing timestamps, and prevent endless runs with stuck detection.

#### Scenario: Batch manual validation checklist
- **WHEN** the implementation is reviewed before merge
- **THEN** verification notes SHALL confirm a short multi-run batch completes with unique session ids and maps, every run has a saved log and a final `finish_reason`, the batch summary references those logs, stop/cancel behavior is clean, manual controls work again after completion, and batch stuck detection prevents infinite execution

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

