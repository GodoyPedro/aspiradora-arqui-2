## Context

The project already has the production-side pieces needed for HTTP integration testing: an `axum` router, JSON handlers, a `RobotController`, in-memory simulated drivers, and a growing set of domain and HTTP tests. What is missing is a broad, structured integration suite that validates the service the way an external API client sees it.

This change is intentionally test-focused. It should not redesign the architecture or introduce new production-side capabilities beyond small clarifications that are strictly necessary to make the API contract deterministic and testable. Where the current implementation already behaves one way but the spec is vague, the change should document that existing behavior rather than re-architect it.

## Goals / Non-Goals

**Goals:**
- Add a broad HTTP integration test suite that exercises the existing router in memory and covers all documented endpoints.
- Validate externally visible behavior: status codes, JSON bodies, state transitions, actuator state, and structured error responses.
- Add reusable test helpers for app setup, JSON requests, response parsing, and deterministic state injection through test-only access.
- Clarify existing endpoint behavior where the suite must pin down an already implemented path, such as low-battery start behavior or invalid-state responses.
- Keep each HTTP test isolated with fresh application state and deterministic simulator inputs.

**Non-Goals:**
- Rewriting the router, controller, state machine, HAL, or simulator architecture.
- Requiring real network ports when the router can be tested in memory.
- Adding Android, Bluetooth, GPIO, board-specific code, SLAM, mapping, lidar, cameras, or real hardware drivers.
- Exposing public HTTP endpoints for simulation mutation.

## Decisions

### 1. Test the in-memory axum router directly

The suite will exercise the existing `axum` router in memory rather than starting a real port-bound server. Each test will create a fresh app with isolated simulated state and use request helpers to drive the router directly.

Rationale: this keeps the tests fast, deterministic, and aligned with `cargo test` usage while still validating the full API layer.

Alternative considered: spawning a real network server per test. Rejected because it adds port management, runtime overhead, and more sources of nondeterminism without improving contract coverage meaningfully.

### 2. Introduce dedicated HTTP integration test helpers

The change will add reusable helpers for:
- building a fresh test router and shared state,
- sending JSON and empty-body requests,
- parsing `RobotStatus` and error responses from JSON,
- mutating simulated sensor, battery, and docking state through test-only access,
- triggering `tick()` from tests when the scenario depends on asynchronous-looking runtime progress.

Rationale: the requested suite is too broad to keep readable if every test hand-rolls app setup and response parsing.

Alternative considered: embedding setup code inline in every test file. Rejected because it would make the suite repetitive and harder to maintain.

### 3. Prefer documenting existing behavior over changing production logic

Where current edge-case behavior already exists, the integration suite should lock it down and the spec should clarify it instead of changing the production implementation just for the sake of a new expected result. This applies especially to cases like:
- start cleaning under low battery,
- stop from `Standby`,
- return-to-dock from `Standby`,
- clear-error from `Standby`.

Rationale: the change is about testing and contract clarity, not about silently redefining runtime semantics.

Alternative considered: forcing new endpoint behaviors to make the suite more symmetrical. Rejected because it would expand scope from verification into behavioral redesign.

### 4. Assert structured error semantics at the HTTP boundary

The suite will verify that HTTP `400` and `409` responses include structured error information with the repository-standard machine-readable field name `code` and the human-readable field `message`. The current repo already uses `code`, so this change will keep that explicit field name instead of introducing `error_code` in a test-only refinement.

Rationale: external clients need stable error payload semantics, and the current implementation already exposes a consistent `code` field.

Alternative considered: asserting only status codes and ignoring error payload shape. Rejected because that leaves client-facing regressions undetected.

### 5. Lock down current edge-case behavior explicitly

The integration suite and spec will use the currently intended implementation behavior for ambiguous endpoint cases:
- `POST /commands/start` while already `Cleaning` returns HTTP `409` and does not mutate state.
- `POST /commands/start` with battery `<= 15` returns HTTP `409` and does not transition to `Cleaning`.
- `POST /commands/stop` from `Standby` returns HTTP `409`.
- `POST /commands/return-to-dock` from `Standby` returns HTTP `200` and transitions to `ReturningToDock` when docking is available.
- `POST /commands/clear-error` from non-error states returns HTTP `409`.

Rationale: these behaviors already reflect the project’s current controller contract closely enough to be locked down by HTTP integration tests without broadening scope.

Alternative considered: changing some of these paths to more permissive no-op success responses. Rejected because this change is about verification and explicit contract documentation, not runtime redesign.

## Risks / Trade-offs

- A very broad HTTP suite can become slow or noisy if helpers are weak. -> Centralize setup and parsing helpers to keep tests compact and deterministic.
- Existing implementation behavior may not match intuitive expectations for every edge case. -> Clarify those cases in the spec instead of silently rewriting behavior.
- More integration assertions increase maintenance cost when response shapes evolve. -> Keep helpers focused on stable fields and require explicit spec updates when the contract changes.
- Test-only state injection could accidentally leak into production code paths. -> Keep helpers internal to tests and avoid adding public simulation endpoints.

## Migration Plan

1. Add shared HTTP test helpers and isolated app builders.
2. Add endpoint success-path integration tests.
3. Add validation and invalid-state conflict tests.
4. Add safety, battery, and docking tests that observe behavior through `/status`.
5. Add structured error body assertions and ensure `cargo test` passes.

Rollback strategy: revert the new test suite and any minimal contract clarifications if they prove too coupled to implementation details. No runtime migration is involved.
