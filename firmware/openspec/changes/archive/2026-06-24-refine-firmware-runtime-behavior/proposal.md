## Why

The current firmware architecture is in place, but several runtime edge cases remain ambiguous enough to produce avoidable inconsistencies in status reads, AUTO cleaning reactions, and manual command validation. This refinement change closes those gaps now so later implementation work can build on clearer, more predictable firmware behavior without redesigning the project.

## What Changes

- Refine the HTTP command contract so `GET /status` is always safe and never fails due to command-style safety rejection.
- Tighten AUTO runtime behavior by adding a simple obstacle or bumper avoidance maneuver that preserves `Cleaning` state without introducing advanced navigation.
- Make `dust_container_full` a first-class error condition that stops all actuators and transitions the robot to `Error` with a dedicated `DustContainerFull` error value.
- Strengthen `MANUAL_MOVE` validation so invalid speed and direction combinations are rejected consistently with structured HTTP `400` responses.
- Extend unit and HTTP test expectations to cover the refined runtime behavior while preserving the existing architecture, `axum` API, HAL traits, in-memory simulators, and controller design.

## Capabilities

### New Capabilities

### Modified Capabilities
- `command-protocol`: Refine `GET /status` behavior and strengthen `MANUAL_MOVE` request validation and HTTP `400` behavior.
- `robot-state-machine`: Clarify error-state transitions triggered by `dust_container_full` while preserving the existing lifecycle model.
- `sensor-actuator-control`: Add basic AUTO obstacle reaction and consistent actuator shutdown plus `Error` transition for a full dust container.
- `simulation-and-testing`: Expand required test coverage for safe status reads, AUTO obstacle reactions, dust-container-full errors, and invalid manual speed handling.

## Impact

- Affects runtime behavior in `RobotController`, the `tick()` loop, motion and cleaning coordination, and API status-read handling.
- Affects HTTP error handling for manual move validation and status reads.
- Requires updates to the simulated driver behavior and test suite, but does not change the established architecture, HTTP framework, or hardware abstraction strategy.
