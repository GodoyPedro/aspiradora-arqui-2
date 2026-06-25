## ADDED Requirements

### Requirement: Firmware shall expose the required HTTP command endpoints
The firmware SHALL expose `POST /commands/start`, `POST /commands/stop`, `POST /commands/pause`, `POST /commands/return-to-dock`, `POST /commands/manual-move`, `POST /commands/mode`, `POST /commands/clear-error`, and `GET /status`.

#### Scenario: Client requests robot status
- **WHEN** a client sends `GET /status`
- **THEN** the firmware SHALL return the current `RobotStatus` serialized as JSON

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
`POST /commands/start` SHALL start `AUTO` cleaning only from `Standby` or `Paused` when battery conditions are sufficient. `POST /commands/stop` SHALL stop wheels, suction, and brushes before returning to `Standby`. `POST /commands/pause` SHALL stop cleaning actuators and move the robot to `Paused`. `POST /commands/return-to-dock` SHALL stop cleaning actuators and move the robot to `ReturningToDock`. `POST /commands/manual-move` SHALL require `direction`, `speed`, and `duration_ms`. `POST /commands/mode` SHALL accept only `AUTO` in this implementation. `POST /commands/clear-error` SHALL clear the active error only when the robot is safe to resume.

#### Scenario: Start endpoint from standby
- **WHEN** a client sends `POST /commands/start` while the robot is in `Standby`
- **THEN** the firmware SHALL transition to `Cleaning` in `AUTO` mode and return the updated `RobotStatus`

#### Scenario: Clear error while unsafe
- **WHEN** a client sends `POST /commands/clear-error` while the critical sensor condition is still active
- **THEN** the firmware SHALL keep the robot in `Error` and report that the error cannot be cleared yet

#### Scenario: Valid command conflicts with current state
- **WHEN** a client sends a syntactically valid command that is not allowed in the current `RobotState`
- **THEN** the command interface SHALL return HTTP `409 Conflict` with a structured error body and SHALL NOT change robot state

#### Scenario: Unsupported cleaning mode is rejected
- **WHEN** a client sends `POST /commands/mode` with `ZigZag`, `WallFollowing`, or `Spot`
- **THEN** the command interface SHALL return HTTP `400 Bad Request` and SHALL NOT change robot state

### Requirement: Simulation helpers shall remain outside the main command protocol
Simulation mutation helpers MAY exist for tests or demos, but they SHALL NOT be part of the primary `/commands/*` and `/status` HTTP contract.

#### Scenario: Production command surface remains limited
- **WHEN** a client consumes the main firmware HTTP API
- **THEN** only the documented `/commands/*` and `/status` endpoints SHALL be required for the supported command protocol
