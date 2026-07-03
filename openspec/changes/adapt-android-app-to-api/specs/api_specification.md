# Robot Vacuum Firmware API Specification

This document provides a detailed technical specification of the HTTP interface exposed by the robot vacuum firmware, derived directly from the `axum` implementation in `firmware/src/api/`.

## 1. General Principles

- **Protocol**: HTTP/1.1
- **Content-Type**: `application/json`
- **Serialization**: Enums use `SCREAMING_SNAKE_CASE` for requests.
- **Base URL**: Typically `http://localhost:3000` or `http://10.0.2.2:3000` (for Android emulators).

---

## 2. Core Control Endpoints

These endpoints manage the primary robot operations and are the ones used by the mobile application.

### 2.1 Get Status
**`GET /status`**
Returns the complete snapshot of the robot.
- **Response `200 OK`**: [RobotStatusResponse](#31-robotstatusresponse)

### 2.2 Movement & Mode
#### `POST /commands/start`
Starts cleaning. Transitions to `CLEANING`.
- **Response `200 OK`**: [RobotStatusResponse](#31-robotstatusresponse)

#### `POST /commands/stop`
Stops activity. Transitions to `STANDBY`.
- **Response `200 OK`**: [RobotStatusResponse](#31-robotstatusresponse)

#### `POST /commands/pause`
Pauses active cleaning.
- **Response `200 OK`**: [RobotStatusResponse](#31-robotstatusresponse)

#### `POST /commands/return-to-dock`
Commands the robot to return to the charging station.
- **Response `200 OK`**: [RobotStatusResponse](#31-robotstatusresponse)

#### `POST /commands/manual-move`
Executes precise manual movement.
- **Request Body**: `{"direction": "FORWARD | BACKWARD | LEFT | RIGHT | STOP", "speed": u8, "duration_ms": u64}`
- **Response `200 OK`**: [RobotStatusResponse](#31-robotstatusresponse)

#### `POST /commands/mode`
Sets the cleaning mode.
- **Request Body**: `{"mode": "AUTO | ZIG_ZAG | WALL_FOLLOWING | SPOT"}`
- **Note**: `ZIGZAG` is accepted as an alias for `ZIG_ZAG`.
- **Response `200 OK`**: [RobotStatusResponse](#31-robotstatusresponse)

#### `POST /commands/clear-error`
Clears current error conditions.
- **Response `200 OK`**: [RobotStatusResponse](#31-robotstatusresponse)

---

## 3. Simulation & Testing Endpoints

These endpoints are used for development, simulation control, and integration testing. They bypass real-world constraints.

### 3.1 Environment Manipulation
- **`POST /simulation/sensors`**: Force sensor values (obstacle, drop-off, bumper, etc.).
- **`POST /simulation/battery`**: Set battery percentage (0-100).
- **`POST /simulation/docking`**: Set dock availability and detection flags.
- **`POST /simulation/reset`**: Resets the simulation to the initial config state.

### 3.2 Runtime Control
- **`POST /simulation/tick`**: Advances the internal simulation clock.
  - Request: `{"delta_ms": u64}` (defaults to 100ms, max 1000ms).
- **`POST /simulation/log-dump`**: Persists simulation logs to disk.
- **`POST /simulation/batch-summary`**: Persists batch simulation results.

---

## 4. UI Demo Endpoints
- **`GET /demo`**: Returns the interactive HTML demo page.
- **`GET /static/demo.css`**: Stylesheet for the demo.
- **`GET /static/demo.js`**: Frontend logic for the demo.

---

## 5. Data Models

### 5.1 RobotStatusResponse
| Property | Type | Description |
| :--- | :--- | :--- |
| `state` | String | e.g., `STANDBY`, `CLEANING`, `ERROR`, `PAUSED`, `MANUAL_CONTROL`. |
| `cleaning_mode` | String | `AUTO`, `ZIG_ZAG`, `WALL_FOLLOWING`, `SPOT`. |
| `battery_percent` | u8 | 0..=100. |
| `is_charging` | bool | True if connected to dock. |
| `suction_enabled` | bool | True if suction is running. |
| `brushes_enabled` | bool | True if brushes are spinning. |
| `left_wheel_speed` | i16 | Current wheel velocity. |
| `right_wheel_speed` | i16 | Current wheel velocity. |
| `current_error` | String? | Error description if in `ERROR` state. |
| `sensors` | Object | Full sensor snapshot (bumper, proximity, dust, etc.). |

---

## 6. Error Handling
Errors return a standard `ErrorResponse` with appropriate HTTP codes (400, 409, 500).

```json
{
  "code": "STRING_CODE",
  "message": "Detailed message",
  "current_state": "Optional current state name"
}
```
Typical codes: `COMMAND_NOT_ALLOWED`, `BATTERY_TOO_LOW`, `MALFORMED_JSON`.
