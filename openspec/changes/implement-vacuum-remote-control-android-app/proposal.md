## Why

The firmware exposes a documented HTTP/JSON API, and the project now has a `vacuum-remote-control` spec describing an Android app that can operate the vacuum over the local network. Implementing the app will make the simulator usable from a phone-like client and provide a concrete external consumer for the existing command protocol.

The current repository does not include an Android project. Adding one alongside the Rust firmware keeps the firmware boundary intact while allowing the remote-control experience to be developed, tested, and demonstrated independently.

## What Changes

- Add an Android remote-control app that connects to the firmware API over WiFi using a configurable base URL.
- Display connection state, current robot state, cleaning mode, battery level, charging status, actuator state, current error, and relevant sensor flags from `GET /status`.
- Provide user controls for start, stop, pause, return-to-dock, manual directional movement, and retry/reconnect using the existing firmware endpoints.
- Map the existing high-level remote-control "power" behavior to supported firmware commands: start/resume cleaning and stop to standby. True OFF/ON power management remains out of scope because the firmware API does not currently expose reachable OFF transitions.
- Handle HTTP validation and conflict errors with clear UI feedback without mutating local UI state optimistically beyond confirmed responses.
- Add Android-side tests for API models, client behavior, state rendering, command validation, and error handling.

## Capabilities

### New Capabilities
- Android app module for remote vacuum control over the local network.

### Modified Capabilities
- `vacuum-remote-control`: Refine Android app behavior against the currently implemented firmware HTTP API, including connection setup, telemetry display, command controls, manual movement, and error handling.

## Impact

- Adds Android build files and app source code to the repository.
- Does not require changes to the firmware API contract for the first app implementation.
- May require README updates documenting how to run the firmware and configure the Android app base URL for emulator and physical-device use.
- Keeps Bluetooth, cloud control, authentication, room mapping, and real embedded power management out of scope.
