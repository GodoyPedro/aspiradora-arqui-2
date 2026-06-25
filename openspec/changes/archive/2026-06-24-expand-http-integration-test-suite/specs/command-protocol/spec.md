## MODIFIED Requirements

### Requirement: Firmware shall expose the required HTTP command endpoints
The firmware SHALL expose `POST /commands/start`, `POST /commands/stop`, `POST /commands/pause`, `POST /commands/return-to-dock`, `POST /commands/manual-move`, `POST /commands/mode`, `POST /commands/clear-error`, and `GET /status`.

#### Scenario: Client requests robot status
- **WHEN** a client sends `GET /status`
- **THEN** the firmware SHALL return the current `RobotStatus` serialized as JSON

#### Scenario: Status read during active critical condition
- **WHEN** a client sends `GET /status` while a critical condition is already active or the robot is already in `Error`
- **THEN** the firmware SHALL still return the current `RobotStatus` and SHALL NOT fail with a safety-condition conflict

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

#### Scenario: Invalid movement speed is rejected
- **WHEN** a client sends `POST /commands/manual-move` with a speed outside `0..=100`, or with zero speed for `FORWARD`, `BACKWARD`, `LEFT`, or `RIGHT`
- **THEN** the command interface SHALL return HTTP `400 Bad Request` with a structured bad-request error body and SHALL NOT change robot state

### Requirement: Simulation helpers shall remain outside the main command protocol
Simulation mutation helpers MAY exist for tests or demos, but they SHALL NOT be part of the primary `/commands/*` and `/status` HTTP contract.

#### Scenario: Production command surface remains limited
- **WHEN** a client consumes the main firmware HTTP API
- **THEN** only the documented `/commands/*` and `/status` endpoints SHALL be required for the supported command protocol

### Requirement: Error responses shall expose structured client-facing fields
HTTP `400` and HTTP `409` responses SHALL include structured error information with the fields `code` and `message`.

#### Scenario: Bad request response is structured
- **WHEN** a client sends a malformed or invalid command request
- **THEN** the HTTP `400` response SHALL include `code` and `message`

#### Scenario: Conflict response is structured
- **WHEN** a client sends a valid command that is disallowed in the current state
- **THEN** the HTTP `409` response SHALL include `code` and `message`
