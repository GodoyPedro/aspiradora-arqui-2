## MODIFIED Requirements

### Requirement: Simulation helpers shall remain outside the main command protocol
Simulation mutation helpers MAY exist for tests or demos, but they SHALL NOT be part of the primary `/commands/*` and `/status` HTTP contract. The firmware MAY expose demo/testing-only helper endpoints under `/simulation/*`, but those endpoints SHALL remain explicitly documented as local simulator helpers and SHALL NOT replace the documented external command API.

#### Scenario: Production command surface remains limited
- **WHEN** a client consumes the main firmware HTTP API
- **THEN** only the documented `/commands/*` and `/status` endpoints SHALL be required for the supported command protocol

#### Scenario: Demo-only simulation helpers stay separate
- **WHEN** a local demo or test client uses `/simulation/*`
- **THEN** those endpoints SHALL be documented as local simulator helpers and SHALL NOT change the contract or semantics of the primary command endpoints

## ADDED Requirements

### Requirement: Simulation tick shall support demo-only clock advancement
`POST /simulation/tick` SHALL accept an optional demo/testing-only JSON payload containing `delta_ms`. When provided, the backend SHALL validate that `delta_ms` is greater than zero and within a safe upper bound before advancing the simulated clock and executing the controller tick. When omitted, the endpoint SHALL remain backward-compatible by advancing time with a safe default tick interval.

#### Scenario: Tick advances simulated time with explicit delta
- **WHEN** a client sends `POST /simulation/tick` with `{ "delta_ms": 100 }`
- **THEN** the backend SHALL advance `SimulatedClock` by that amount before returning the resulting `RobotStatus`

#### Scenario: Tick remains backward-compatible without payload
- **WHEN** a client sends `POST /simulation/tick` without a JSON payload
- **THEN** the backend SHALL still execute a safe default simulated time advancement and return `RobotStatus`

#### Scenario: Invalid tick delta is rejected
- **WHEN** a client sends `POST /simulation/tick` with `delta_ms <= 0` or above the documented safe maximum
- **THEN** the backend SHALL return HTTP `400 Bad Request` and SHALL NOT advance simulated time
