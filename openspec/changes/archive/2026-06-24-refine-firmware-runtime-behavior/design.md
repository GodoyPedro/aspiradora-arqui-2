## Context

The Rust robot vacuum firmware already has the intended layered architecture: `axum` routes, a central `RobotController`, a state machine, trait-based hardware abstractions, in-memory simulated drivers, and tests covering the baseline command and safety flows. This change is intentionally narrow and focuses on runtime behavior refinements rather than architectural restructuring.

The current implementation still has a few ambiguous or inconsistent edges: status reads can be treated like active commands, AUTO cleaning does not react to obstacles in even a minimal way, `dust_container_full` is handled differently from other stop-worthy conditions, and `MANUAL_MOVE` validation is not fully explicit at the contract level. These are all small changes individually, but they affect controller behavior, API semantics, and tests across multiple modules.

## Goals / Non-Goals

**Goals:**
- Make `GET /status` always safe and readable, even when the robot is already in `Error` or a critical condition is active.
- Add a minimal deterministic AUTO obstacle response for `obstacle_detected` and `bumper_pressed` without changing the navigation model.
- Treat `dust_container_full` as a consistent runtime error condition that stops actuators and transitions to `Error`.
- Tighten `MANUAL_MOVE` validation so speed and direction rules are explicit, testable, and reflected in HTTP `400` behavior.
- Preserve the existing architecture, module layout, HTTP framework, HAL traits, and simulator-based approach.

**Non-Goals:**
- Rewriting the architecture, splitting the controller, or replacing the state machine.
- Adding SLAM, room mapping, path planning, cameras, lidar, or any advanced navigation.
- Exposing simulator mutation helpers as public HTTP endpoints.
- Adding Android, Bluetooth, GPIO, or board-specific embedded code.

## Decisions

### 1. Separate status reads from command rejection paths

`GET /status` will use a safe read path that returns the current `RobotStatus` directly instead of going through a command flow that can reject operations due to active safety conditions. The API may call `current_status()` directly or use an equivalent read-only controller path, but it must not treat status reads as state-mutating commands.

Rationale: status reads are observational, not operational. Returning conflicts for a status read makes recovery and debugging harder precisely when the robot is already in a bad state.

Alternative considered: preserving the current command-style path and adding a special-case bypass inside general command handling. Rejected because it keeps read-only behavior coupled to mutation-oriented logic.

### 2. Implement obstacle reaction as a minimal AUTO maneuver

When the robot is in `Cleaning` and `tick()` sees `obstacle_detected` or `bumper_pressed`, the controller will perform a simple avoidance maneuver by setting wheel speeds to turn in place while keeping `RobotState = Cleaning`. The maneuver remains intentionally simple and deterministic, such as `left = 30`, `right = -30`.

Rationale: this satisfies the need for a visible AUTO reaction without expanding the scope into real navigation.

Alternative considered: introducing a richer navigation sub-state or temporary pause/backup sequence. Rejected because it adds complexity without being required by this refinement.

### 3. Treat dust-container-full as an error-class stop condition

`dust_container_full` will stop wheels, suction, and brushes, set `current_error = DustContainerFull`, and transition the robot to `Error` when observed during `Cleaning`. This aligns it with the project goal of tightening runtime behavior and removes the ambiguity of stopping some actuators without surfacing a durable robot error.

Rationale: the user-facing contract becomes more consistent when stop-worthy operational blockers produce explicit error state and error reporting.

Alternative considered: leaving `dust_container_full` as a non-error cleaning stop. Rejected because it makes runtime behavior less consistent than the other blocking conditions.

### 4. Enforce direction-aware MANUAL_MOVE validation

`MANUAL_MOVE` will require `speed` in `0..=100` and `duration_ms > 0`. For `FORWARD`, `BACKWARD`, `LEFT`, and `RIGHT`, `speed` must be greater than zero. For `STOP`, `speed` may be zero. Violations will produce structured bad-request errors that map to HTTP `400`.

Rationale: this eliminates ambiguous cases like turning or driving with zero speed while preserving a valid explicit stop request.

Alternative considered: allowing zero speed for all directions as a harmless no-op. Rejected because it weakens the API contract and makes tests less meaningful.

## Risks / Trade-offs

- `GET /status` may now surface a robot state that was not first normalized through command handling. -> Keep `current_status()` derived from authoritative controller and driver state only.
- The simple avoidance maneuver could override some prior wheel command in `Cleaning`. -> Limit it to `Cleaning` and document it as an intentionally minimal AUTO reaction.
- Elevating `dust_container_full` to `Error` may require updating more tests than the apparent behavior change suggests. -> Update both controller tests and HTTP-facing expectations where `current_error` is observed.
- Direction-aware speed validation introduces more request edge cases. -> Use one structured bad-request format and cover it in both unit and HTTP tests.

## Migration Plan

1. Update command/status handling so `GET /status` uses a safe read-only path.
2. Refine `tick()` to handle AUTO obstacle and bumper reactions while preserving `Cleaning`.
3. Add `DustContainerFull` to the error model and route `dust_container_full` to actuator stop plus `Error`.
4. Tighten `MANUAL_MOVE` validation in controller and API-facing paths.
5. Update unit and HTTP tests to cover the refined runtime behavior.

Rollback strategy: revert the refinement change if the tighter runtime rules introduce regressions. No data migration or external dependency rollback is required.
