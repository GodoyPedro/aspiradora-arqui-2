## 1. Batch UI Controls

- [x] 1.1 Add a `Batch Simulation` section to `/demo` with fields for run count, max simulated time, target coverage, batch simulation speed, return-to-dock toggle, and enabled stop conditions
- [x] 1.2 Surface current batch id, run index, current session id, elapsed simulated time, coverage, run state, finish condition, completed count, failed/stuck count, last saved log path, and final batch summary path
- [x] 1.3 Add `Start Batch`, `Stop Batch`, and `Clear Batch Results` controls
- [x] 1.4 Disable or clearly isolate manual controls while a batch run is active, then restore them after batch completion or cancellation

## 2. Batch Run Lifecycle

- [x] 2.1 Generate a fresh map and session id for every run and associate both with one batch id
- [x] 2.2 Reset frontend-owned simulation state between runs, including pose, heading, coverage, timeline, sensors, collision, docking, turn episode state, and stuck counters
- [x] 2.3 Reuse the existing backend simulation reset flow before starting each run
- [x] 2.4 Start cleaning through `POST /commands/start`, run the existing simulation loop automatically, and stop launching new runs when the user cancels the batch
- [x] 2.5 If `return_to_dock_after_run` is enabled, request `POST /commands/return-to-dock` after time/coverage completion and continue until charging, timeout, stuck, or cancel

## 3. Batch Stop Conditions And Stuck Detection

- [x] 3.1 Implement configured stop conditions for time limit, coverage threshold, docking success, stuck diagnostic, user cancellation, and command error
- [x] 3.2 Guarantee that each run closes with exactly one final `finish_reason`
- [x] 3.3 Add batch-only stuck detection for no movement with non-zero wheels, repeated long-turn guard patterns, repeated docking stuck/block events, and no progress toward dock while returning
- [x] 3.4 Map docking-phase failures to `DOCKING_FAILED_OR_TIMEOUT` and non-docking stalls to `STUCK_DIAGNOSTIC`

## 4. Batch Log Metadata And Events

- [x] 4.1 Keep run logs compatible with the existing dump format while adding exact batch metadata `batch_id`, `run_index`, `batch_total_runs`, `batch_mode = true`, and `run_config`
- [x] 4.2 Ensure `run_config` captures `max_simulated_time`, `target_coverage`, `simulation_speed`, `return_to_dock_after_run`, enabled stop conditions, and map complexity or obstacle count when configured
- [x] 4.3 Verify `max_simulated_time` in `run_config` is expressed in milliseconds and timeline `timestamp_ms` is non-decreasing milliseconds from the start of the run or session
- [x] 4.4 Emit batch-specific timeline labels such as `BATCH_RUN_START`, `BATCH_RUN_END`, `BATCH_STOP_TIME_LIMIT`, `BATCH_STOP_COVERAGE_REACHED`, `BATCH_RETURN_TO_DOCK_REQUESTED`, `BATCH_STOP_DOCKED`, `BATCH_STOP_DOCKING_FAILED`, `BATCH_STOP_STUCK`, `BATCH_STOP_USER_CANCELLED`, and `BATCH_COMMAND_ERROR`
- [x] 4.5 Preserve existing useful demo events and maintain non-decreasing timestamps across the full run timeline
- [x] 4.6 Add a final run-end event before persisting each log

## 5. Run Summary And Batch Summary Generation

- [x] 5.1 Compute one run summary per simulation with the exact snake_case fields `batch_id`, `run_index`, `session_id`, `map_id`, `log_path`, `created_at`, `finished_at`, `finish_reason`, `final_state`, `final_coverage_percentage`, `total_frames`, `total_simulated_time_ms`, optional `total_real_time_ms`, `obstacle_count`, `obstacle_detections`, `bumper_contacts`, `wall_contacts`, `obstacle_contacts`, `turns`, `long_turn_guards`, `escape_turns`, `dock_attempted`, `docking_succeeded`, `reached_charging`, `dock_blocked_events`, `dock_stuck_diagnostics`, `stuck_diagnostics`, `max_consecutive_no_movement_frames_with_nonzero_wheels`, `max_consecutive_turning_frames_without_xy_change`, `final_x`, `final_y`, `final_heading`, and `log_save_error`
- [x] 5.2 Ensure `total_simulated_time_ms` is milliseconds, `total_real_time_ms` is milliseconds if available, `log_save_error` is `null` on save success, and failed saves may leave `log_path = null` with a short `log_save_error` message
- [x] 5.3 Maintain an in-memory batch summary containing exact top-level fields `batch_id`, `created_at`, `finished_at`, `requested_runs`, `completed_runs`, `cancelled`, `config`, and `runs`
- [x] 5.4 Group aggregate fields under `aggregate_metrics`
- [x] 5.5 Compute `aggregate_metrics.average_coverage`, `aggregate_metrics.min_coverage`, `aggregate_metrics.max_coverage`, `aggregate_metrics.average_simulated_time`, conditional `aggregate_metrics.docking_success_rate`, `aggregate_metrics.most_common_finish_reason`, `aggregate_metrics.total_bumper_contacts`, `aggregate_metrics.total_wall_contacts`, `aggregate_metrics.total_obstacle_contacts`, `aggregate_metrics.total_long_turn_guards`, `aggregate_metrics.total_escape_turns`, `aggregate_metrics.total_stuck_diagnostics`, `aggregate_metrics.total_dock_blocked_events`, and `aggregate_metrics.total_dock_stuck_diagnostics`
- [x] 5.6 Group worst-run lists under `worst_runs`
- [x] 5.7 Produce `worst_runs.lowest_coverage`, `worst_runs.highest_wall_contacts`, `worst_runs.highest_obstacle_contacts`, `worst_runs.most_long_turn_guards`, `worst_runs.most_stuck_diagnostics`, `worst_runs.failed_docking`, `worst_runs.longest_no_movement_streak`, and `worst_runs.longest_turning_without_movement_streak`
- [x] 5.8 Persist a partial batch summary when the user cancels and a final summary when the batch completes

## 6. Backend Persistence Endpoint And Path Support

- [x] 6.1 Implement the exact `POST /simulation/batch-summary` request shape with required top-level `batch_id`, required `summary`, and matching `summary.batch_id`
- [x] 6.2 Implement the exact `POST /simulation/batch-summary` response shape `{ "saved_path": "demo-logs/<batch_id>/batch-summary.json" }`
- [x] 6.3 Persist only the `summary` object as pretty JSON under `demo-logs/<batch_id>/batch-summary.json`
- [x] 6.4 Validate malformed or mismatched top-level and embedded `batch_id` values return HTTP `400`
- [x] 6.5 Implement the exact batch-mode `POST /simulation/log-dump` request shape with required `batch_id`, `session_id`, and `log`
- [x] 6.6 Implement the exact batch-mode `POST /simulation/log-dump` response shape `{ "saved_path": "demo-logs/<batch_id>/<session_id>/log.json" }`
- [x] 6.7 Validate matching top-level and embedded `batch_id` and `session_id`, require `log.run_index`, `log.batch_total_runs`, and `log.batch_mode = true`, and reject malformed or mismatched payloads with HTTP `400`
- [x] 6.8 Persist only the `log` object under `demo-logs/<batch_id>/<session_id>/log.json` while preserving manual non-batch log payload and folder behavior
- [x] 6.9 Add backend tests proving mandatory `POST /simulation/batch-summary` behavior, malformed payload rejection, traversal rejection, batch log path writes, and unchanged manual log behavior
- [x] 6.10 Verify batch-mode `POST /simulation/log-dump` path layout writes exactly under `demo-logs/<batch_id>/<session_id>/log.json`

## 7. Tests And Manual Validation

- [x] 7.1 Add or update frontend/manual verification steps for short 3-run batches, unique map/session generation, per-run log creation, batch summary references, finish reasons, clean cancellation, coverage-triggered stops, return-to-dock batch outcomes, timestamp monotonicity, and anti-infinite-run stuck detection
- [x] 7.2 Implement and verify command error handling so failed commands mark the run as `COMMAND_ERROR`, emit `BATCH_COMMAND_ERROR` with detail, attempt log persistence, and continue only when the batch is still viable
- [x] 7.3 Implement and verify log-save failure handling so the UI shows the error, the run summary stays in memory with log-save failure information, and the demo loop does not crash
- [x] 7.4 Verify `Stop Batch` marks the active run as `USER_CANCELLED`, emits `BATCH_STOP_USER_CANCELLED`, attempts active log persistence, saves a partial batch summary, and starts no new runs
- [x] 7.5 Re-run the relevant backend test suite to confirm `/simulation/*` persistence changes do not regress existing endpoints

## 8. README Documentation

- [x] 8.1 Document how to use `Batch Simulation`, what each configuration field means, how stop conditions and return-to-dock behave, where logs and batch summaries are saved, and how to bundle artifacts for analysis
- [x] 8.2 Document which metrics are useful for diagnosing weak robot behavior and clarify that batch mode is demo/simulator-only
