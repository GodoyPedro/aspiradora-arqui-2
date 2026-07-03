## Data Models (DTOs)

### `ManualMoveRequest`
```kotlin
@Serializable
data class ManualMoveRequest(
    val direction: String, // FORWARD, BACKWARD, LEFT, RIGHT, STOP
    val speed: Int,        // u8 equivalent
    val duration_ms: Long  // u64 equivalent
)
```

### `ModeRequest`
```kotlin
@Serializable
data class ModeRequest(
    val mode: String // AUTO, ZIG_ZAG, ZIGZAG, WALL_FOLLOWING, SPOT
)
```

### `ApiErrorResponse`
```kotlin
@Serializable
data class ApiErrorResponse(
    val code: String,
    val message: String
)
```

## API Client Implementation (Ktor)

- Use `HttpClient.post` for all `/commands/*` endpoints.
- Map `400` and `409` status codes to a custom `FirmwareException` containing the `ApiErrorResponse`.
- Ensure `GET /status` is called after every successful command to refresh the local state.

## UI Logic

- Cleaning modes will be presented in a selection menu or set of buttons.
- `clear-error` button will only be visible/enabled when the robot is in an `ERROR` state.
- `start`, `pause`, `stop` buttons will react to the current `state` field in `RobotStatusResponse`.
