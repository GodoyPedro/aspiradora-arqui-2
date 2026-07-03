## Why

The current Android application needs to be strictly aligned with the robot vacuum firmware's HTTP API specification. This ensures that the application can correctly command the robot, handle domain-specific errors (like `BATTERY_TOO_LOW` or `COMMAND_NOT_ALLOWED`), and support all cleaning modes and manual movement parameters defined in the firmware.

## What Changes

- Update the Android API client to support the full set of defined endpoints: `GET /status`, `POST /commands/start`, `POST /commands/stop`, `POST /commands/pause`, `POST /commands/return-to-dock`, `POST /commands/manual-move`, `POST /commands/mode`, and `POST /commands/clear-error`.
- Refine manual movement DTOs to include `direction`, `speed`, and `duration_ms` with specific type requirements (`u8` for speed, `u64` for duration).
- Implement cleaning mode selection supporting `AUTO`, `ZIG_ZAG` (with `ZIGZAG` alias), `WALL_FOLLOWING`, and `SPOT`.
- Enhance error handling to decode and surface structured `400 Bad Request` and `409 Conflict` responses, specifically handling `COMMAND_NOT_ALLOWED`, `BATTERY_TOO_LOW`, and persistent error conditions during `clear-error`.
- Ensure the UI correctly enables/disables commands based on the current `RobotStatus`.

## Capabilities

### New Capabilities
- `extended-command-support`: Full implementation of all firmware commands including `clear-error` and specific cleaning modes.
- `structured-error-handling`: UI feedback for specific firmware conflicts and validation errors.

### Modified Capabilities
- `api-client-sync`: Updated DTOs and Ktor calls to match the firmware's transport semantics and payload structures.
- `manual-control-refinement`: D-pad integration with precise speed and duration parameters.
