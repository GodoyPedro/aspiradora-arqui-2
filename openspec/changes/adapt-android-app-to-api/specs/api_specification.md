# Robot Vacuum Firmware API Specification

This document provides a detailed technical specification of the HTTP interface exposed by the robot vacuum firmware.

## 1. General Principles

- **Protocol**: HTTP/1.1
- **Content-Type**: `application/json`
- **Error Handling**: The API uses standard HTTP status codes. Errors include a structured JSON body with a `code` and a `message`.
- **State Consistency**: Most command endpoints return the updated `RobotStatusResponse` upon success (HTTP 200).

---

## 2. Endpoints

### 2.1 Get Status
**`GET /status`**

Returns the complete snapshot of the robot's current state, sensors, and configuration.

- **Response `200 OK`**: [RobotStatusResponse](#31-robotstatusresponse)

---

### 2.2 Control Commands

All commands follow a similar pattern: they are requested via `POST` and return the updated status.

#### `POST /commands/start`
Starts the cleaning process.
- **Preconditions**: Robot must be in `STANDBY` or `PAUSED`.
- **Errors**:
  - `409 COMMAND_NOT_ALLOWED`: If called from an invalid state (e.g., `ERROR`).
  - `409 BATTERY_TOO_LOW`: If battery is below the minimum threshold to start.

#### `POST /commands/stop`
Stops the current activity and returns the robot to `STANDBY`.
- **Preconditions**: Any state except `OFF`.

#### `POST /commands/pause`
Pauses active cleaning or manual movement.
- **Preconditions**: Robot must be in `CLEANING` or `MANUAL_CONTROL`.

#### `POST /commands/return-to-dock`
Commands the robot to find and return to its charging station.
- **Acceptable from**: `STANDBY`, `CLEANING`, `PAUSED`, `MANUAL_CONTROL`.

#### `POST /commands/clear-error`
Attempts to clear a safety or hardware error.
- **Errors**:
  - `409`: Returns if the physical error condition (e.g., bumper still pressed) persists.

---

### 2.3 Parameterized Commands

#### `POST /commands/manual-move`
Executes a precise movement.
- **Request Body**: [ManualMoveRequest](#33-manualmoverequest)
- **Rules**:
  - Validates `speed` (0-255) and `duration_ms` (> 0).
  - Transitions the robot to `MANUAL_CONTROL`.

#### `POST /commands/mode`
Sets the cleaning strategy.
- **Request Body**: [ModeRequest](#34-moderequest)
- **Supported Modes**: `AUTO`, `ZIG_ZAG` (alias `ZIGZAG`), `WALL_FOLLOWING`, `SPOT`.

---

## 3. Data Models (Schemas)

### 3.1 RobotStatusResponse
| Property | Type | Description |
| :--- | :--- | :--- |
| `state` | String | Current state (e.g., `STANDBY`, `CLEANING`, `ERROR`). |
| `cleaning_mode` | String | Active mode (e.g., `AUTO`, `SPOT`). |
| `battery_percent` | Integer | 0 to 100. |
| `is_charging` | Boolean | True if connected to dock power. |
| `suction_enabled` | Boolean | Actuator status. |
| `brushes_enabled` | Boolean | Actuator status. |
| `left_wheel_speed` | Integer | Real-time motor speed. |
| `right_wheel_speed` | Integer | Real-time motor speed. |
| `current_error` | String? | Error code or `null`. |
| `sensors` | Object | [SensorSnapshot](#32-sensorsnapshot). |

### 3.2 SensorSnapshot
Boolean flags for: `obstacle_detected`, `drop_off_detected`, `bumper_pressed`, `dust_container_full`, `wheel_stuck`, `brush_stuck`, `top_cover_open`.

### 3.3 ManualMoveRequest
```json
{
  "direction": "FORWARD | BACKWARD | LEFT | RIGHT | STOP",
  "speed": 0..255,
  "duration_ms": "long"
}
```

### 3.4 ModeRequest
```json
{
  "mode": "AUTO | ZIG_ZAG | ZIGZAG | WALL_FOLLOWING | SPOT"
}
```

---

## 4. Error Responses
**Status Codes**: `400 Bad Request`, `409 Conflict`.
**Body**:
```json
{
  "code": "STRING_CONSTANT",
  "message": "Human readable explanation",
  "current_state": "Optional current robot state"
}
```
