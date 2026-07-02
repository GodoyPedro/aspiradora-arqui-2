## MODIFIED Requirements

### Requirement: Connection to vacuum
The Android app SHALL connect to the vacuum via its existing HTTP/JSON API over WiFi on the local network, using a configurable firmware base URL.

#### Scenario: Successful connection
- GIVEN the phone or emulator can reach the firmware API
- WHEN the user opens the app or taps retry
- THEN the app sends `GET /status`
- AND displays the current connection status
- AND displays the latest robot status returned by the firmware

#### Scenario: Connection failure
- GIVEN the vacuum API is unreachable
- WHEN the app attempts to connect
- THEN the app displays an offline or connection-error state
- AND keeps any last confirmed robot status visible if one exists
- AND offers a retry option

#### Scenario: User changes firmware address
- GIVEN the app is displaying the configured firmware base URL
- WHEN the user enters a different local-network URL and retries
- THEN the app uses the new base URL for subsequent status and command requests

### Requirement: Power control
The Android app SHALL provide primary start/stop controls that map to the firmware's supported command API. The app SHALL NOT present true hardware power off/on as available unless the firmware exposes reachable power-management endpoints.

#### Scenario: Start cleaning
- GIVEN the vacuum is connected and the current status allows starting
- WHEN the user taps the start control
- THEN the app sends `POST /commands/start`
- AND updates the UI from the firmware response or a follow-up status refresh

#### Scenario: Stop cleaning
- GIVEN the vacuum is connected and the current status allows stopping
- WHEN the user taps the stop control
- THEN the app sends `POST /commands/stop`
- AND updates the UI from the firmware response or a follow-up status refresh

#### Scenario: Start or stop not allowed
- GIVEN the firmware rejects a start or stop command with HTTP `409`
- WHEN the app receives the structured error response
- THEN the app displays the conflict message
- AND does not replace the last confirmed robot status with an optimistic state

### Requirement: Directional control
The Android app SHALL allow the user to control the movement of the vacuum through the firmware's duration-based manual movement command.

#### Scenario: Movement command
- GIVEN the vacuum is connected and the current status allows manual movement
- WHEN the user presses a directional control
- THEN the app sends `POST /commands/manual-move` with the selected direction, speed, and positive `duration_ms`
- AND updates the UI from the firmware response or a follow-up status refresh

#### Scenario: Movement while command unavailable
- GIVEN the current robot status does not allow manual movement
- WHEN the user views the directional controls
- THEN the app disables manual movement controls or blocks the command before sending
- AND displays feedback indicating the movement command is currently unavailable

#### Scenario: Invalid manual movement input
- GIVEN the user selects an invalid speed or duration
- WHEN the user attempts to send a movement command
- THEN the app does not send the request
- AND displays validation feedback

## ADDED Requirements

### Requirement: Status dashboard
The Android app SHALL display the latest robot telemetry returned by `GET /status` or command responses.

#### Scenario: Status is displayed
- GIVEN the app has received a valid `RobotStatus`
- WHEN the main screen renders
- THEN it displays robot state, cleaning mode, battery percent, charging status, suction state, brush state, wheel speeds, current error, and relevant sensor flags

#### Scenario: Status refresh
- GIVEN the app is connected
- WHEN the user triggers a refresh
- THEN the app sends `GET /status`
- AND replaces the displayed telemetry with the latest confirmed status

### Requirement: Command error handling
The Android app SHALL handle firmware validation errors, command conflicts, and network failures as first-class UI states.

#### Scenario: Firmware validation error
- GIVEN the firmware returns HTTP `400` with a structured error body
- WHEN the app receives the response
- THEN it displays the error code or message
- AND preserves the last confirmed status

#### Scenario: Firmware command conflict
- GIVEN the firmware returns HTTP `409` with a structured error body
- WHEN the app receives the response
- THEN it displays the conflict code or message
- AND preserves the last confirmed status

#### Scenario: Network error during command
- GIVEN the app previously displayed a confirmed robot status
- WHEN a command request fails because the firmware is unreachable
- THEN the app displays a connection error
- AND keeps the previous status visible as stale or last known data

### Requirement: Supported firmware commands
The Android app SHALL expose controls for the firmware commands that are currently implemented and documented.

#### Scenario: Pause command
- GIVEN the vacuum is connected and the current status allows pausing
- WHEN the user taps pause
- THEN the app sends `POST /commands/pause`

#### Scenario: Return-to-dock command
- GIVEN the vacuum is connected and the current status allows returning to dock
- WHEN the user taps return-to-dock
- THEN the app sends `POST /commands/return-to-dock`

#### Scenario: Clear-error command
- GIVEN the vacuum is connected and currently reports an error
- WHEN the user taps clear-error
- THEN the app sends `POST /commands/clear-error`

#### Scenario: Auto mode command
- GIVEN the vacuum is connected
- WHEN the user selects AUTO mode
- THEN the app sends `POST /commands/mode` with mode `AUTO`
