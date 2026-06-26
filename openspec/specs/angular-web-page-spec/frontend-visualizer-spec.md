## Purpose

Define the functional requirements for the Angular frontend that visualizes the vacuum robot's state in a 2D space, consuming the firmware's HTTP/JSON API (`GET /status`, `POST /commands/*`).

## Notes / Open Items (not yet resolved by the firmware spec)

- The firmware spec defines the `/commands/*` and `/status` contract, but does **not** define the `RobotStatus` JSON shape. The requirements below assume it includes position, battery, `RobotState`, and active-error info; this must be confirmed against the actual struct/schema.
- The firmware spec does **not** expose any endpoint for static map/walls or detected obstacles. Until such a source is defined, the frontend can only render the robot's position and state — not a floor plan or obstacles.
- Units/scale of the position fields (if present) are undefined and must be confirmed (pixels, cm, meters; origin point).

## Requirements

### Requirement: Frontend shall poll the firmware status endpoint periodically
The frontend SHALL poll `GET /status` at a configurable interval (default suggestion: every 2 seconds) and update the displayed state on each successful response.

#### Scenario: Initial load starts polling
- **WHEN** the visualizer view is loaded
- **THEN** the frontend SHALL begin polling `GET /status` at the configured interval

#### Scenario: Successful poll updates the view
- **WHEN** a poll to `GET /status` returns `200` with a valid `RobotStatus` body
- **THEN** the frontend SHALL update the rendered position, battery, and state to match the response

#### Scenario: Failed poll does not crash the view
- **WHEN** a poll to `GET /status` fails (network error, timeout, or non-2xx response)
- **THEN** the frontend SHALL retain the last known good state, display a connection-error indicator, and continue retrying on the next interval

#### Scenario: Polling stops on view teardown
- **WHEN** the visualizer view is destroyed or navigated away from
- **THEN** the frontend SHALL stop polling and release any associated timers/subscriptions

### Requirement: Frontend shall render the robot's position on a 2D canvas
The frontend SHALL render the robot as a sprite on a PixiJS canvas, positioned according to the coordinates returned in `RobotStatus`.

#### Scenario: Position is rendered
- **WHEN** `RobotStatus` includes valid position coordinates
- **THEN** the frontend SHALL render the robot sprite at the corresponding canvas position, transformed by a configurable origin offset and scale factor

#### Scenario: Heading is rendered when available
- **WHEN** `RobotStatus` includes a heading/orientation value
- **THEN** the frontend SHALL rotate the robot sprite to match that heading

#### Scenario: Missing or malformed position is handled
- **WHEN** `RobotStatus` is missing position data or contains malformed values
- **THEN** the frontend SHALL NOT crash, and SHALL show a fallback indicator instead of rendering a sprite at an invalid location

### Requirement: Frontend shall visually represent robot state and battery
The frontend SHALL display the current `RobotState` (`Standby`, `Cleaning`, `Paused`, `ReturningToDock`, `Error`) and battery level alongside the 2D view.

#### Scenario: State badge reflects RobotState
- **WHEN** `RobotStatus.state` changes
- **THEN** the frontend SHALL update a visible status badge/color to match the new state

#### Scenario: Error state surfaces error details
- **WHEN** `RobotStatus.state` is `Error`
- **THEN** the frontend SHALL surface whatever error information is included in `RobotStatus`, if any

#### Scenario: Battery level is displayed
- **WHEN** `RobotStatus` includes a battery value
- **THEN** the frontend SHALL display it numerically and/or via a battery-level indicator

### Requirement: Map layout and obstacles are out of scope until a data source exists
The frontend SHALL NOT attempt to render a floor plan or obstacles, since the current firmware API contract provides no endpoint or field for this data.

#### Scenario: Canvas renders without map data
- **WHEN** no map/obstacle data source is available
- **THEN** the frontend SHALL render the robot position/state on a blank or placeholder canvas, with no error or broken UI state caused by the missing data

#### Scenario: Map/obstacle support is added later
- **WHEN** a map and/or obstacle data source is defined in the future (new endpoint, static asset, or extended `RobotStatus`)
- **THEN** this requirement SHALL be revisited and the rendering layer extended accordingly

### Requirement: (Optional / future scope) Command controls may reuse existing command endpoints
If the project scope is extended beyond read-only visualization, the frontend MAY expose UI controls mapped to the firmware's command endpoints, reusing their existing validation and error semantics.

#### Scenario: Command succeeds
- **WHEN** a user triggers a command (`start`, `stop`, `pause`, `return-to-dock`, `manual-move`, `mode`, `clear-error`) and the firmware returns `200`
- **THEN** the frontend SHALL update the view with the returned `RobotStatus` without requiring a manual refresh

#### Scenario: Command is rejected with a structured error
- **WHEN** a command request returns HTTP `400` or `409` with a `code`/`message` body
- **THEN** the frontend SHALL surface that `code`/`message` to the user without changing the displayed robot state

#### Scenario: Manual-move respects validation constraints
- **WHEN** a user attempts a manual move
- **THEN** the frontend SHALL enforce the same client-side constraints as the API (`speed` in `0..=100`; `speed > 0` for `FORWARD`/`BACKWARD`/`LEFT`/`RIGHT`) before sending the request, to avoid unnecessary `400` responses
