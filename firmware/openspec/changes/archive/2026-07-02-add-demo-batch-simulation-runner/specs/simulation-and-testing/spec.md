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

#### Scenario: Batch persistence integration coverage
- **WHEN** the HTTP integration suite sends valid and invalid requests to `POST /simulation/batch-summary` and batch-aware `POST /simulation/log-dump`
- **THEN** it SHALL verify successful writes stay under the demo log folder, malformed payloads return HTTP `400`, traversal attempts are rejected, and existing non-batch log dump behavior still works

#### Scenario: Cargo test execution target
- **WHEN** the full Rust test suite is run with `cargo test`
- **THEN** the HTTP integration tests SHALL run deterministically without requiring a real network port

## ADDED Requirements

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
