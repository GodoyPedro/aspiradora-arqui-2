## Context

This change defines the firmware layer for a domestic robot vacuum as a Rust application that exposes an HTTP/JSON command API while remaining independent from physical hardware. The current stage has no board, GPIO, Bluetooth, or embedded target available, so all hardware-facing behavior must be modeled through traits and in-memory simulated drivers.

The architecture must support realistic firmware concerns rather than a toy sample: command validation, explicit robot states, periodic control-loop evaluation, safety cutoffs, battery-aware behavior, docking transitions, and externally observable status. At the same time, the scope must stay executable within a short academic timeline, which rules out SLAM, mapping, advanced navigation, and board-specific integration.

The implemented runtime starts in `Standby`, with all actuators stopped, `current_error` set to `None`, `cleaning_mode` set to `AUTO`, and a configurable simulated battery percentage initialized to a test-friendly default. The `Off` state remains part of the domain model for future embedded power-management work, but it is not reachable through the HTTP API in this implementation.

## Goals / Non-Goals

**Goals:**
- Provide a layered Rust architecture with clear boundaries between transport, application logic, domain controllers, state machine, and hardware abstractions.
- Define a stable internal command and status model so the HTTP layer is replaceable without changing firmware behavior.
- Ensure firmware logic is testable without hardware by depending on traits and supplying simulated drivers.
- Encode required safety, battery, and docking rules in a deterministic periodic `tick()` loop.
- Limit the first implementation to `AUTO` cleaning and `MANUAL` movement while documenting future navigation modes.

**Non-Goals:**
- Real hardware drivers, GPIO integration, RTOS concerns, or board bring-up.
- Android client implementation, Bluetooth transport, or cloud connectivity.
- SLAM, room mapping, cameras, lidar, or advanced path-planning.
- Multiple sophisticated cleaning algorithms beyond the single `AUTO` mode and direct `MANUAL` movement behavior.
- Distributed services, persistence, or telemetry ingestion pipelines outside process memory.

## Decisions

### 1. Use a five-layer architecture

The implementation will be split into:
- Command interface layer for HTTP routes, payload decoding, and response serialization.
- Application controller layer centered on `RobotController`.
- State machine and domain model layer for `RobotState`, `RobotCommand`, `RobotError`, `CleaningMode`, and `RobotStatus`.
- Domain controllers for motion, cleaning, battery, docking, and safety.
- Hardware abstraction and simulated driver layer.

Rationale: this keeps web concerns isolated from firmware rules and avoids leaking simulated driver details into business logic.

Alternative considered: a single controller with direct HTTP handlers and shared mutable simulator state. Rejected because it couples transport, domain rules, and hardware behavior too tightly and makes unit testing state transitions harder.

### 2. Model hardware dependencies as focused Rust traits

Each hardware capability will be represented as a trait: `WheelMotorDriver`, `SuctionDriver`, `BrushDriver`, `SensorReader`, `BatteryDriver`, `DockingDriver`, and `Clock`. `RobotController` will own domain controllers that depend on these traits rather than concrete simulators.

Rationale: this keeps the controller logic close to production firmware architecture and allows replacing simulators with real drivers later.

Alternative considered: a single monolithic `HardwareFacade` trait. Rejected because it obscures responsibilities and makes tests more cumbersome by forcing irrelevant mocks into every scenario.

### 3. Centralize state transition authority in RobotController

`RobotStateMachine` defines legal states and transitions, but `RobotController` remains the orchestrator that receives commands, invokes domain controllers, runs `tick()`, and applies transitions only after validating state and safety preconditions.

Rationale: a separate state machine defines policy while keeping orchestration in one place. This avoids duplicating rules across HTTP handlers and domain controllers.

Alternative considered: let each domain controller mutate robot state directly. Rejected because safety, docking, and cleaning behavior would become fragmented and conflicting transitions would be harder to reason about.

### 4. Use synchronous in-memory execution for the first stage

The firmware will run as a normal Rust process with synchronous, in-memory state updates. `MANUAL_MOVE` will require `direction`, `speed`, and `duration_ms`, and duration expiry will be evaluated against the `Clock` abstraction during `tick()` rather than through background real-time scheduling.

Rationale: deterministic synchronous logic is simpler to test and sufficient for the current no-hardware scope.

Alternative considered: async actor-based subsystems. Rejected because the extra concurrency model adds complexity without clear benefit in the first milestone.

### 5. Use axum and explicit transport error semantics

The HTTP command interface will use `axum` as the framework for routing, JSON extraction, and response handling. Malformed JSON and structurally invalid payloads will return HTTP `400 Bad Request`. Valid commands that are not allowed in the current `RobotState` will return HTTP `409 Conflict` with a structured error body. Unsupported cleaning modes will return HTTP `400 Bad Request` without changing robot state.

Rationale: `axum` fits the Rust ecosystem well for a small HTTP service, and explicit error semantics keep client behavior deterministic and testable.

Alternative considered: deferring framework choice or returning generic success/no-op for invalid state combinations. Rejected because the design needs a concrete transport target and clients need unambiguous failures.

### 6. Represent status as a derived snapshot, not scattered fields

`RobotStatus` will be assembled from controller state and driver readings after commands and on each `tick()`. It will include robot state, mode, battery, charging, actuator status, wheel speeds, current error, and relevant sensor flags.

Rationale: a single observable status object simplifies HTTP responses and tests, and prevents drift between internal state and external reporting.

Alternative considered: separate endpoint-specific response models. Rejected because it creates inconsistent client behavior and duplicates mapping logic.

### 7. Encode safety as fail-fast actuator shutdown plus Error

Safety checks in `tick()` will inspect sensor snapshots for `drop_off_detected`, `wheel_stuck`, `brush_stuck`, and `top_cover_open`. On any critical condition, the firmware must stop wheels, suction, and brushes, set the corresponding `RobotError`, and transition to `Error`.

Rationale: this makes safety deterministic and auditable, and mirrors realistic firmware priorities.

Alternative considered: let cleaning continue with warnings. Rejected because the requested behavior explicitly requires immediate stop and error transition.

### 8. Keep navigation intentionally narrow

`AUTO` cleaning will be implemented as the only accepted `/commands/mode` value in this change, and `MANUAL` will be limited to direct wheel commands with validated `direction`, `speed`, and required `duration_ms`. `ZigZag`, `WallFollowing`, and `Spot` will be documented as future extensions only and rejected as unsupported in this implementation.

Rationale: this preserves a realistic architecture while keeping the implementation small enough for the project timeline.

Alternative considered: stubbing all future modes as partially working features. Rejected because incomplete mode semantics would dilute tests and complicate the API contract.

### 9. Keep docking ownership in DockingDriver only

Dock detection will be represented by `DockingDriver` and `SimulatedDockingDriver` only. `SensorSnapshot` and `SimulatedSensorReader` will not expose `dock_detected`. The `tick()` loop will consult `DockingDriver` when evaluating `ReturningToDock` to `Charging`.

Rationale: docking is a separate device concern and duplicating it in the sensor snapshot would create conflicting sources of truth.

Alternative considered: exposing dock detection both through sensors and docking abstractions. Rejected because it increases ambiguity in tests and controller logic.

### 10. Keep simulation mutation helpers outside the main API contract

Simulation mutation helpers may exist for tests or demos so callers can manipulate battery, sensor, or docking conditions, but they are not part of the main command protocol. The primary HTTP interface remains limited to the documented `/commands/*` and `/status` endpoints.

Rationale: this keeps the public firmware contract clean while still allowing controlled simulation support during development.

Alternative considered: exposing simulator mutation routes alongside main commands. Rejected because it would blur the boundary between firmware behavior and test harness concerns.

## Risks / Trade-offs

- Trait boundaries may still need adjustment when real hardware appears. -> Keep traits small and capability-based so replacements are localized.
- Synchronous `tick()` logic may not map one-to-one to embedded scheduling later. -> Route time-dependent behavior through `Clock` and explicit loop semantics.
- Simulated drivers can create false confidence if they are too permissive. -> Keep them stateful and expose enough sensor flags to test failure paths.
- The HTTP API could leak transport concerns into the controller if route logic becomes too smart. -> Restrict handlers to validation, translation, and serialization only.
- Tight HTTP semantics add more response cases for clients to handle. -> Use one structured error body shape across `400` and `409` responses.
- Simple `AUTO` behavior may not reflect future navigation complexity. -> Keep navigation behind dedicated mode/domain types so richer policies can be added without breaking the current controller contract.

## Migration Plan

1. Initialize a Rust crate with modules that match the architecture layers.
2. Implement core domain models and the robot state machine.
3. Add hardware traits and in-memory simulated drivers.
4. Build domain controllers and `RobotController` with `tick()`.
5. Add `axum` HTTP routes that translate requests to `RobotCommand`.
6. Add unit tests for state transitions, safety rules, and status reporting.

Rollback strategy: because this is a new capability set rather than a production migration, rollback consists of reverting the change or disabling the HTTP server entrypoint until the implementation stabilizes.
