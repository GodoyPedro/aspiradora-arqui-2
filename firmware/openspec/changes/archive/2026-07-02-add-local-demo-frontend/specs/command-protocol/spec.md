## MODIFIED Requirements

### Requirement: Simulation helpers shall remain outside the main command protocol
Simulation mutation helpers MAY exist for tests or demos, but they SHALL NOT be part of the primary `/commands/*` and `/status` HTTP contract. The firmware MAY expose demo/testing-only helper endpoints under `/simulation/*`, but those endpoints SHALL remain explicitly documented as non-production helpers, SHALL be acceptable only because this project is a simulator/demo, and SHALL NOT replace the documented external command API used by Android or other clients.

#### Scenario: Production command surface remains limited
- **WHEN** a client consumes the main firmware HTTP API
- **THEN** only the documented `/commands/*` and `/status` endpoints SHALL be required for the supported command protocol

#### Scenario: Demo helper endpoints remain separate from the command contract
- **WHEN** a local browser demo or test client uses `/simulation/*`
- **THEN** those endpoints SHALL be documented as demo/testing-only helpers and SHALL NOT change the meaning of the primary command endpoints or the external client contract

## ADDED Requirements

### Requirement: Firmware shall expose demo and simulation helper endpoints for local visualization
The firmware SHALL expose `GET /demo` for the local browser demo and SHALL expose demo/testing-only helper endpoints `POST /simulation/sensors`, `POST /simulation/battery`, `POST /simulation/docking`, and `POST /simulation/tick`.

#### Scenario: Demo page is served by the backend
- **WHEN** a browser requests `GET /demo`
- **THEN** the firmware SHALL return the local demo page required to visualize and control the simulator

#### Scenario: Simulation tick is triggered over HTTP
- **WHEN** a client sends `POST /simulation/tick`
- **THEN** the firmware SHALL execute the controller's periodic `tick()` behavior and return the resulting `RobotStatus` as JSON

### Requirement: Simulation helper endpoints shall return status and use the existing error format
`POST /simulation/sensors`, `POST /simulation/battery`, and `POST /simulation/docking` SHALL return the updated `RobotStatus`. `POST /simulation/tick` SHALL execute `tick()` and return the resulting `RobotStatus`. Invalid JSON or invalid payload shape for any `/simulation/*` endpoint SHALL return HTTP `400 Bad Request` using the existing structured error body format.

#### Scenario: Simulation sensor update returns status
- **WHEN** a client sends a valid `POST /simulation/sensors` request
- **THEN** the firmware SHALL update the simulated sensor state and return the updated `RobotStatus`

#### Scenario: Simulation battery update returns status
- **WHEN** a client sends a valid `POST /simulation/battery` request
- **THEN** the firmware SHALL update the simulated battery state and return the updated `RobotStatus`

#### Scenario: Simulation docking update returns status
- **WHEN** a client sends a valid `POST /simulation/docking` request
- **THEN** the firmware SHALL update the simulated docking state and return the updated `RobotStatus`

#### Scenario: Invalid simulation payload is rejected
- **WHEN** a client sends malformed JSON or an invalid payload shape to a `/simulation/*` endpoint
- **THEN** the firmware SHALL return HTTP `400 Bad Request` with the existing structured error body format

### Requirement: Simulation helper endpoints shall mutate only simulated backend state
The simulation helper endpoints SHALL mutate the shared in-memory simulation state used by the firmware backend and SHALL expose only the minimum payloads needed for demo/testing control of sensors, battery, and docking. Sensor and docking payloads SHALL support partial updates through optional boolean fields, preserving current values for unspecified fields. `battery_percent` SHALL be validated and any value outside `0..=100` SHALL be rejected with HTTP `400 Bad Request`.

#### Scenario: Simulation sensors are updated over HTTP
- **WHEN** a client sends `POST /simulation/sensors` with sensor flag values
- **THEN** the firmware SHALL update the simulated sensor state and expose those values through subsequent status reads or tick results

#### Scenario: Simulation battery is updated over HTTP
- **WHEN** a client sends `POST /simulation/battery` with a battery percentage
- **THEN** the firmware SHALL update the simulated battery state used by `GET /status` and `tick()`

#### Scenario: Simulation docking state is updated over HTTP
- **WHEN** a client sends `POST /simulation/docking` with docking availability or dock detection values
- **THEN** the firmware SHALL update the simulated docking state used by return-to-dock and charging transitions

#### Scenario: Simulation sensors preserve unspecified fields
- **WHEN** a client sends `POST /simulation/sensors` with only a subset of supported sensor fields
- **THEN** the firmware SHALL update only the provided fields and SHALL preserve the current values of unspecified sensor fields

#### Scenario: Simulation docking preserves unspecified fields
- **WHEN** a client sends `POST /simulation/docking` with only `dock_available` or only `dock_detected`
- **THEN** the firmware SHALL update only the provided field and SHALL preserve the current value of the unspecified docking field

#### Scenario: Simulation battery rejects out-of-range values
- **WHEN** a client sends `POST /simulation/battery` with `battery_percent` outside `0..=100`
- **THEN** the firmware SHALL return HTTP `400 Bad Request` and SHALL NOT mutate the stored battery percentage
