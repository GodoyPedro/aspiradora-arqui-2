## 1. Simulated Time Consistency

- [x] 1.1 Make batch stop conditions compare `activeRun.lastSimulatedTimeMs` against `config.max_simulated_time_ms`
- [x] 1.2 Maintain one authoritative per-run simulated elapsed time counter that advances with the same `delta_ms` used for `/simulation/tick`
- [x] 1.3 Ensure visual movement, timeline `timestamp_ms`, stuck windows, and `total_simulated_time_ms` all use that same simulated delta basis
- [x] 1.4 Keep timeline `timestamp_ms` non-decreasing milliseconds from the start of each run
- [x] 1.5 If large simulated frame deltas need subdivision, keep backend tick progression, movement, and sensor updates coherent under those substeps

## 2. Return To Dock Refinement

- [x] 2.1 Refine docking stuck detection so heading improvement toward the dock counts as valid progress even if `x/y` barely changes
- [x] 2.2 Track docking progress using distance-to-dock, heading-error improvement, blocked-forward evidence, recovery counts, and lack of both angular and positional progress
- [x] 2.3 Replace aggressive fixed docking turns with capped proportional steering and slower near-dock approach behavior
- [x] 2.4 Keep docking guidance authoritative whenever backend state is `RETURNING_TO_DOCK`, including bounded `DOCK_RECOVERY_TURN` and forward retry behavior after `DOCK_BLOCKED`
- [x] 2.5 On transition to `CHARGING`, stop visual movement, ensure wheel speeds settle at `0/0`, clear transient obstacle/bumper signals, and log `DOCK_DETECTED` plus `CHARGING_STARTED`

## 3. Metric And Event Semantics

- [x] 3.1 Define one canonical `turns` metric based on cleaning turn episodes rather than raw `TURNING` frames
- [x] 3.2 Restrict `ANTI_LOOP_ESCAPE` to actual anti-loop interventions and keep normal left/right turns under `TURN_START` / `TURNING` / `TURN_END`
- [x] 3.3 Distinguish cleaning turns, docking alignment turns, docking recovery turns, and escape turns through event names or event detail
- [x] 3.4 Keep `log.summary.turns`, run summary `turns`, and batch summary `turns` consistent for the same run
- [x] 3.5 Extend README metric definitions so turn, escape, and docking-related counters are unambiguous
- [x] 3.6 Update canonical turn counting so it includes lane-turn episodes, obstacle-bypass turns, and pattern-recovery turns, but excludes docking and manual turns

## 4. Frontend Demo Navigation Ownership

- [x] 4.1 Clarify in code/docs that backend firmware owns only high-level state, actuator/status reporting, safety/error transitions, and dock-detected charging transition
- [x] 4.2 Move or soften any backend-facing lawn-mower requirements so lane geometry, lane shifts, obstacle bypass, and rejoin logic remain frontend demo-only
- [x] 4.3 Implement a frontend-owned `demo_navigation_phase` controller that runs only while backend state is `Cleaning` and mode is `AUTO`
- [x] 4.4 Ensure the frontend controller does not run during `ReturningToDock`, `Charging`, `ManualControl`, `Paused`, or `Error`
- [x] 4.5 Distinguish backend-reported wheel speeds from demo visual wheel speeds in logs and summaries when both are emitted

## 5. Lawn-Mower AUTO Pattern State Machine

- [x] 5.1 Implement a deterministic lane-phase state machine with `LANE_DRIVE`, `LANE_TURN_TO_SHIFT`, `LANE_SHIFT`, `LANE_TURN_TO_REVERSE`, `OBSTACLE_BYPASS_TURN`, `OBSTACLE_BYPASS_OFFSET`, `OBSTACLE_BYPASS_REJOIN`, and `PATTERN_RECOVERY`
- [x] 5.2 Drive long straight lanes, treat wall contact as lane end, shift by a bounded lane spacing, reverse travel direction, and continue the next lane
- [x] 5.3 Keep obstacle avoidance contact-driven with only very short-range frontal detection and no long anticipatory obstacle turn from `obstacle_detected` alone
- [x] 5.4 On obstacle contact, perform a predictable local bypass or lane recovery instead of prolonged random spinning
- [x] 5.5 Add bounded fallback behavior when lane shift or obstacle bypass fails, including `PATTERN_RECOVERY`, and prevent indefinite spinning
- [x] 5.6 Define and document concrete lane constants for spacing, initial heading, shift side, angular tolerance, and shift completion tolerance

## 6. Pattern Logging And Batch Metrics

- [x] 6.1 Add required pattern events `LANE_START`, `LANE_END`, `LANE_SHIFT_START`, `LANE_SHIFT_END`, `LANE_BLOCKED`, `OBSTACLE_BYPASS_START`, `OBSTACLE_BYPASS_END`, `OBSTACLE_BYPASS_FAILED`, and `PATTERN_RECOVERY`
- [x] 6.2 Include event detail such as `demo_navigation_phase`, `lane_index`, `lane_direction`, `lane_heading`, `lane_shift_side`, `lane_spacing_px`, `target_heading`, `blocked_target`, `contact_type`, and `obstacle_id` when available
- [x] 6.3 Make run summary pattern metrics mandatory: `lane_starts`, `lane_ends`, `lane_shifts`, `lane_blocked_events`, `obstacle_bypass_attempts`, `obstacle_bypass_failures`, and `pattern_recoveries`
- [x] 6.4 Make batch aggregate pattern metrics mandatory: `total_lane_starts`, `total_lane_ends`, `total_lane_shifts`, `total_lane_blocked_events`, `total_obstacle_bypass_attempts`, `total_obstacle_bypass_failures`, and `total_pattern_recoveries`
- [x] 6.5 Ensure batch stuck detection treats valid lane turns and lane shifts as progress when heading or phase is changing
- [x] 6.6 Keep batch coverage and contact summaries usable for comparing coverage regularity and contact-per-coverage efficiency across runs

## 7. Backend And Integration Coverage

- [x] 7.1 Keep existing command API behavior unchanged while adjusting only demo/simulator behavior
- [x] 7.2 If backend docking or charging cleanup changes, add or update tests for `RETURNING_TO_DOCK -> CHARGING`, `0/0` wheel speeds after charging starts, and transient sensor cleanup ownership
- [x] 7.3 Re-run existing backend tests to confirm simulator refinements do not regress current command and safety behavior

## 8. Manual And Batch Validation

- [x] 8.1 Manually run the demo at `10x` for at least 60 simulated seconds and confirm lane-like, non-frantic `AUTO` movement
- [x] 8.2 Confirm obstacle contact causes controlled bypass or lane recovery and wall contact causes lane transition behavior
- [x] 8.3 Confirm Return to Dock reaches `CHARGING` more reliably and does not fail solely because heading changes happen without large `x/y` movement
- [x] 8.4 Confirm timeline timestamps are non-decreasing and `ANTI_LOOP_ESCAPE` appears only for real anti-loop intervention
- [x] 8.5 Confirm `log.summary.turns` matches run summary `turns` for the same run
- [x] 8.6 Run a batch of at least 3 maps at `10x` with 2 obstacles, `max_simulated_time_ms = 30000`, target coverage below `100%`, and Return to Dock enabled
- [x] 8.7 Confirm time-limited runs stop based on simulated time rather than real wall-clock duration and that `total_simulated_time_ms` stays close to the configured simulated limit when time-limited
- [x] 8.8 Confirm docking failures are not triggered by valid alignment rotation and that lane/pattern events appear in the saved logs
- [x] 8.9 Validate less-frantic behavior by confirming no repeated random left/right turns without phase progress, no long unexplained `TURNING` streaks, visible straight lane segments, and fewer bumper/wall contacts per simulated minute than the prior random-bounce baseline on comparable maps

## 9. Documentation

- [x] 9.1 Update README with simulated-time semantics, refined docking stuck rules, proportional docking guidance, canonical turn definitions, corrected `ANTI_LOOP_ESCAPE` meaning, and the frontend-owned lawn-mower `AUTO` pattern
- [x] 9.2 Document contact-driven obstacle avoidance, explicit lane constants, phase labels, backend-versus-demo wheel-speed semantics, and how batch logs compare coverage efficiency, docking success, stuck diagnostics, and lane/blockage behavior
