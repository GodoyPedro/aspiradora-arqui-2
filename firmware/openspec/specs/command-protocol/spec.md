## Purpose

Define the public HTTP/JSON command API, request validation rules, and transport error semantics for the firmware.
## Requirements
### Requirement: Firmware shall expose the required HTTP command endpoints
The firmware SHALL expose `POST /commands/start`, `POST /commands/stop`, `POST /commands/pause`, `POST /commands/return-to-dock`, `POST /commands/manual-move`, `POST /commands/mode`, `POST /commands/clear-error`, and `GET /status`.

#### Scenario: Client requests robot status
- **WHEN** a client sends `GET /status`
- **THEN** the firmware SHALL return the current `RobotStatus` serialized as JSON

#### Scenario: Status read during active critical condition
- **WHEN** a client sends `GET /status` while a critical condition is already active or the robot is already in `Error`
- **THEN** the firmware SHALL still return the current `RobotStatus` and SHALL NOT fail with a safety-condition conflict

### Requirement: Command endpoints shall translate requests into RobotCommand values
The command interface SHALL parse JSON payloads, validate request structure, convert supported requests into internal `RobotCommand` values, and invoke the application controller instead of implementing firmware logic in the route handlers.

#### Scenario: Manual move request is translated successfully
- **WHEN** a client sends `POST /commands/manual-move` with a valid direction, speed, and `duration_ms`
- **THEN** the command interface SHALL produce a `MANUAL_MOVE` command with equivalent internal fields and pass it to the application controller

#### Scenario: Invalid payload is rejected
- **WHEN** a client sends a command payload that is missing required fields or uses unsupported values
- **THEN** the command interface SHALL return HTTP `400 Bad Request` and SHALL NOT mutate robot state

#### Scenario: Malformed JSON is rejected
- **WHEN** a client sends malformed JSON to a command endpoint
- **THEN** the command interface SHALL return HTTP `400 Bad Request` and SHALL NOT mutate robot state

### Requirement: Command behavior shall match the defined control contract
`POST /commands/start` SHALL start `AUTO` cleaning only from `Standby` or `Paused` when battery conditions are sufficient. If the robot is already `Cleaning`, the endpoint SHALL return HTTP `409 Conflict` and SHALL NOT mutate state. If the battery is already `15` or lower, `POST /commands/start` SHALL return HTTP `409 Conflict` and SHALL NOT transition to `Cleaning`. `POST /commands/stop` SHALL stop wheels, suction, and brushes before returning to `Standby` when stopping is allowed; when called from `Standby`, it SHALL return HTTP `409 Conflict`. `POST /commands/pause` SHALL stop cleaning actuators and move the robot to `Paused`. `POST /commands/return-to-dock` SHALL stop cleaning actuators and move the robot to `ReturningToDock` when allowed; when called from `Standby` with docking available, it SHALL return HTTP `200` and transition to `ReturningToDock`. `POST /commands/manual-move` SHALL require `direction`, `speed`, and `duration_ms`. `POST /commands/manual-move` SHALL accept `speed` only in the range `0..=100`. For `FORWARD`, `BACKWARD`, `LEFT`, and `RIGHT`, `speed` SHALL be greater than zero. For `STOP`, `speed` MAY be zero. `POST /commands/mode` SHALL accept only `AUTO` in this implementation. `POST /commands/clear-error` SHALL clear the active error only when the robot is safe to resume; calls from non-error states SHALL return HTTP `409 Conflict`.

#### Scenario: Start endpoint from standby
- **WHEN** a client sends `POST /commands/start` while the robot is in `Standby`
- **THEN** the firmware SHALL transition to `Cleaning` in `AUTO` mode and return the updated `RobotStatus`

#### Scenario: Start endpoint while already cleaning
- **WHEN** a client sends `POST /commands/start` while the robot is already in `Cleaning`
- **THEN** the command interface SHALL return HTTP `409 Conflict` and SHALL NOT mutate state

#### Scenario: Start endpoint with low battery
- **WHEN** a client sends `POST /commands/start` while battery is `15` or lower
- **THEN** the command interface SHALL return HTTP `409 Conflict` and SHALL NOT transition the robot to `Cleaning`

#### Scenario: Clear error while unsafe
- **WHEN** a client sends `POST /commands/clear-error` while the critical sensor condition is still active
- **THEN** the firmware SHALL keep the robot in `Error` and report that the error cannot be cleared yet

#### Scenario: Valid command conflicts with current state
- **WHEN** a client sends a syntactically valid command that is not allowed in the current `RobotState`
- **THEN** the command interface SHALL return HTTP `409 Conflict` with a structured error body and SHALL NOT change robot state

#### Scenario: Stop endpoint from standby
- **WHEN** a client sends `POST /commands/stop` while the robot is in `Standby`
- **THEN** the command interface SHALL return HTTP `409 Conflict`

#### Scenario: Return-to-dock endpoint from standby
- **WHEN** a client sends `POST /commands/return-to-dock` while the robot is in `Standby` and docking is available
- **THEN** the command interface SHALL return HTTP `200` and the response state SHALL become `RETURNING_TO_DOCK`

#### Scenario: Clear-error endpoint from standby
- **WHEN** a client sends `POST /commands/clear-error` while the robot is not in `Error`
- **THEN** the command interface SHALL return HTTP `409 Conflict`

#### Scenario: Unsupported cleaning mode is rejected
- **WHEN** a client sends `POST /commands/mode` with `ZigZag`, `WallFollowing`, or `Spot`
- **THEN** the command interface SHALL return HTTP `400 Bad Request` and SHALL NOT change robot state

#### Scenario: Invalid movement speed is rejected
- **WHEN** a client sends `POST /commands/manual-move` with a speed outside `0..=100`, or with zero speed for `FORWARD`, `BACKWARD`, `LEFT`, or `RIGHT`
- **THEN** the command interface SHALL return HTTP `400 Bad Request` with a structured bad-request error body and SHALL NOT change robot state

### Requirement: Error responses shall expose structured client-facing fields
HTTP `400` and HTTP `409` responses SHALL include structured error information with the fields `code` and `message`.

#### Scenario: Bad request response is structured
- **WHEN** a client sends a malformed or invalid command request
- **THEN** the HTTP `400` response SHALL include `code` and `message`

#### Scenario: Conflict response is structured
- **WHEN** a client sends a valid command that is disallowed in the current state
- **THEN** the HTTP `409` response SHALL include `code` and `message`

### Requirement: Simulation helpers shall remain outside the main command protocol
Simulation mutation helpers MAY exist for tests or demos, but they SHALL NOT be part of the primary `/commands/*` and `/status` HTTP contract. The firmware MAY expose demo/testing-only helper endpoints under `/simulation/*`, but those endpoints SHALL remain explicitly documented as local simulator helpers and SHALL NOT replace the documented external command API.

#### Scenario: Production command surface remains limited
- **WHEN** a client consumes the main firmware HTTP API
- **THEN** only the documented `/commands/*` and `/status` endpoints SHALL be required for the supported command protocol

#### Scenario: Demo-only simulation helpers stay separate
- **WHEN** a local demo or test client uses `/simulation/*`
- **THEN** those endpoints SHALL be documented as local simulator helpers and SHALL NOT change the contract or semantics of the primary command endpoints

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

### Requirement: Demo-only simulation persistence endpoints shall support batch artifacts safely
The firmware SHALL expose `POST /simulation/batch-summary` for local simulator batch persistence. `POST /simulation/log-dump` SHALL support batch-mode logs under `demo-logs/<batch_id>/<session_id>/log.json` while remaining backward-compatible with existing non-batch manual log dumps. These endpoints SHALL validate request payloads, sanitize `batch_id` and `session_id`, reject path traversal, create required directories safely, write JSON only inside the demo log area, and return the saved file path.

#### Scenario: Batch summary is persisted under a batch folder
- **WHEN** a local demo client sends `POST /simulation/batch-summary` with a valid batch summary payload and a safe `batch_id`
- **THEN** the firmware SHALL write pretty JSON under `demo-logs/<batch_id>/batch-summary.json` and return a success response containing that saved path

#### Scenario: Batch summary request and response use the exact JSON shape
- **WHEN** a local demo client sends `POST /simulation/batch-summary`
- **THEN** the request body SHALL use the shape `{ "batch_id": "batch-20260628-001", "summary": { "batch_id": "batch-20260628-001", "created_at": "ISO-8601 string", "finished_at": "ISO-8601 string", "requested_runs": 10, "completed_runs": 10, "cancelled": false, "config": {}, "runs": [], "aggregate_metrics": {}, "worst_runs": {} } }`
- **THEN** top-level `batch_id` SHALL be required, `summary` SHALL be required, and `summary.batch_id` SHALL match top-level `batch_id`
- **THEN** the backend SHALL persist exactly the `summary` object as pretty JSON under `demo-logs/<batch_id>/batch-summary.json`
- **THEN** the response body SHALL use the shape `{ "saved_path": "demo-logs/<batch_id>/batch-summary.json" }`

#### Scenario: Invalid batch identifier is rejected
- **WHEN** a local demo client sends a batch summary or log dump request whose `batch_id` or `session_id` attempts path traversal or uses rejected unsafe path characters
- **THEN** the firmware SHALL return HTTP `400 Bad Request` and SHALL NOT write any file outside the allowed demo log directory

#### Scenario: Mismatched batch identifiers are rejected
- **WHEN** a local demo client sends `POST /simulation/batch-summary` with a missing `batch_id`, missing `summary`, or a `summary.batch_id` that does not match top-level `batch_id`
- **THEN** the firmware SHALL return HTTP `400 Bad Request` and SHALL NOT persist any file

#### Scenario: Batch log dump uses the batch folder layout
- **WHEN** a local demo client sends `POST /simulation/log-dump` with valid batch metadata for `batch_id` and `session_id`
- **THEN** the firmware SHALL persist the log under `demo-logs/<batch_id>/<session_id>/log.json`

#### Scenario: Batch log dump request and response use the exact JSON shape
- **WHEN** a local demo client sends a batch-mode `POST /simulation/log-dump`
- **THEN** the request body SHALL use the shape `{ "batch_id": "batch-20260628-001", "session_id": "demo-map-20260628-001-run-001", "log": {} }`
- **THEN** `batch_id`, `session_id`, and `log` SHALL be required for batch-mode log dumps
- **THEN** `log.batch_id`, `log.session_id`, `log.run_index`, `log.batch_total_runs`, and `log.batch_mode = true` SHALL be present in the embedded log object
- **THEN** if `log.batch_id` exists it SHALL match top-level `batch_id`, and if `log.session_id` exists it SHALL match top-level `session_id`
- **THEN** the backend SHALL persist exactly the `log` object under `demo-logs/<batch_id>/<session_id>/log.json`
- **THEN** the response body SHALL use the shape `{ "saved_path": "demo-logs/<batch_id>/<session_id>/log.json" }`

#### Scenario: Mismatched batch log identifiers are rejected
- **WHEN** a local demo client sends a batch-mode `POST /simulation/log-dump` with missing `batch_id`, missing `session_id`, missing `log`, or mismatched top-level and embedded `batch_id` or `session_id`
- **THEN** the firmware SHALL return HTTP `400 Bad Request` and SHALL NOT persist any file

#### Scenario: Manual log dump remains backward-compatible
- **WHEN** a local demo client sends `POST /simulation/log-dump` without batch metadata using the existing manual log format
- **THEN** the firmware SHALL continue to persist the log successfully using the documented non-batch session layout

