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

### Requirement: Firmware shall expose demo-only reset and log dump endpoints
The firmware SHALL expose `POST /simulation/reset` and `POST /simulation/log-dump` as demo/testing-only endpoints for the local simulator.

#### Scenario: Reset endpoint returns a safe initial status
- **WHEN** a client sends `POST /simulation/reset`
- **THEN** the firmware SHALL reset the simulated controller to its safe initial configuration and SHALL return the resulting `RobotStatus`

#### Scenario: Log dump endpoint persists a session
- **WHEN** a client sends `POST /simulation/log-dump` with a valid demo log payload
- **THEN** the firmware SHALL write the payload to the configured demo log folder and SHALL return at least `session_id` and `file_path`

### Requirement: Log dump persistence shall be validated and path-safe
`POST /simulation/log-dump` SHALL accept a complete JSON payload containing a session id, map metadata, frontend configuration, movement timeline, coverage data, and summary. The backend SHALL sanitize the session id, reject path traversal, and write logs only inside the chosen demo log directory. Each generated map session SHALL resolve to exactly one backend log target path, such as `demo-logs/<session_id>/log.json`.

#### Scenario: Invalid log dump payload is rejected
- **WHEN** a client sends malformed JSON or a structurally invalid log dump payload
- **THEN** the firmware SHALL return HTTP `400 Bad Request` and SHALL NOT create a log file

#### Scenario: Unsafe session id is sanitized or rejected
- **WHEN** a client sends a `session_id` containing path separators or traversal markers
- **THEN** the firmware SHALL sanitize or reject that value so persisted files remain inside the demo log directory

#### Scenario: Session persists to its own stable path
- **WHEN** a client persists a valid log dump for the active map session
- **THEN** the backend SHALL write that dump to the stable per-session target path for that `session_id`
