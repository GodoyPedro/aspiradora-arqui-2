## Why

The project needs a firmware architecture that behaves like production robot vacuum software without depending on physical hardware, so the team can validate behavior, boundaries, and API contracts early. Defining this as an OpenSpec change now keeps the implementation scoped, testable, and aligned with a short academic timeline.

## What Changes

- Define a Rust firmware architecture centered on an application controller, explicit robot state machine, and domain-specific controllers for motion, cleaning, battery, docking, and safety.
- Standardize the HTTP/JSON command interface on `axum`, mapping external requests into internal `RobotCommand` values and exposing `RobotStatus` responses.
- Specify deterministic transport semantics: malformed JSON or invalid payloads return HTTP `400`, valid commands disallowed by the current state return HTTP `409 Conflict` with a structured error body, and unsupported cleaning modes return HTTP `400`.
- Specify a hardware abstraction layer based on Rust traits so firmware logic depends on interfaces rather than board-specific code.
- Add simulated in-memory driver implementations for actuators, sensors, battery, docking, and time to support development without hardware, while keeping simulation mutation helpers outside the main command protocol.
- Define the periodic `tick()` firmware loop, required safety rules, initial runtime state, state transitions, and minimum telemetry surface.
- Establish test expectations, phased delivery tasks, and future-extension boundaries for non-implemented navigation modes.

## Capabilities

### New Capabilities
- `embedded-firmware`: Overall firmware architecture, module boundaries, controller responsibilities, deterministic startup behavior, and implementation scope for the robot vacuum.
- `command-protocol`: HTTP endpoints, JSON payload validation, transport error semantics, command translation, and status response behavior.
- `robot-state-machine`: Allowed robot states, transitions, initial runtime state, unreachable `Off` behavior, and lifecycle behavior across cleaning, docking, charging, and error handling.
- `hardware-abstraction`: Trait-based interfaces for motors, suction, brushes, sensors, battery, docking, and clock dependencies with non-duplicated docking ownership.
- `sensor-actuator-control`: Domain controller behavior for motion, cleaning, battery, docking, safety, supported modes, and periodic control-loop decisions.
- `simulation-and-testing`: Simulated drivers, startup defaults, mutation helpers, and required unit coverage for the firmware logic.

### Modified Capabilities

## Impact

- Affects the planned Rust crate structure, internal domain model, controller orchestration, and API contract for external clients.
- Introduces a local `axum` HTTP/JSON interface intended for consumers such as an Android app, without implementing the client itself.
- Establishes trait contracts and simulator behavior that future real hardware drivers can implement without rewriting controller logic.
- Constrains the simulation surface so test/demo mutation helpers stay outside the primary `/commands/*` and `/status` protocol.
