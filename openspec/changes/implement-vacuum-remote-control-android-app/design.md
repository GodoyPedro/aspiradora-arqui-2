## Context

The firmware is a Rust `axum` application that exposes `GET /status` and command endpoints under `/commands/*`. It starts in `STANDBY`, supports `START`, `STOP`, `PAUSE`, `RETURN_TO_DOCK`, `MANUAL_MOVE`, `AUTO` mode selection, and `CLEAR_ERROR`, and returns structured HTTP errors for invalid or conflicting commands.

The existing `vacuum-remote-control` spec describes an Android app on the same WiFi network. The firmware README explicitly states that a real Android app and true OFF-state power control are not currently implemented. This change introduces the Android client while respecting the already documented firmware limitations.

## Goals / Non-Goals

**Goals:**
- Provide a native Android app that can control the local firmware simulator over HTTP.
- Keep API mapping thin and explicit so the app behaves like a real external client.
- Make connection state and robot telemetry visible without requiring developer tools.
- Support direct manual movement with validated direction, speed, and duration.
- Handle unreachable firmware, malformed responses, validation errors, and command conflicts gracefully.
- Add focused automated tests for the app's API layer and UI state logic.

**Non-Goals:**
- Firmware OFF/ON power-management endpoints or embedded sleep/wake behavior.
- Bluetooth, cloud relay, account login, authentication, TLS provisioning, or device discovery.
- Advanced navigation modes beyond the firmware's supported `AUTO` and `MANUAL` behavior.
- Publishing to the Play Store or adding production mobile analytics.
- Replacing the Rust firmware API or moving firmware logic into the app.

## Decisions

### 1. Add the Android project as a sibling app module

Create an Android application directory such as `android-app/` at the repository root. Keep the Rust firmware crate unchanged at the root so `cargo test` and `cargo run` continue working as they do today.

Rationale: the repository currently models one product with a firmware service and OpenSpec artifacts. A sibling Android app preserves that layout without forcing a Cargo workspace redesign.

Alternative considered: generate a separate repository for the Android app. Rejected because the user asked for this project, and shared OpenSpec artifacts are already in this repository.

### 2. Use a small layered Android architecture

Structure the app around:
- API DTOs matching the firmware JSON contract.
- A `VacuumApiClient` responsible for HTTP requests and response/error decoding.
- A repository or use-case layer that exposes connection, status refresh, and command operations.
- UI state models that render robot telemetry and command availability.
- A single main screen for control and status.

Rationale: this mirrors the firmware's clean boundary between transport and domain behavior, while staying small enough for the current project.

Alternative considered: call HTTP endpoints directly from UI event handlers. Rejected because it makes error handling and tests brittle.

### 3. Make the base URL configurable

The app will support a configurable firmware base URL with sensible defaults:
- Android emulator default: `http://10.0.2.2:3000`
- Physical device/local network: user-entered `http://<host>:3000`

Rationale: `127.0.0.1` means the Android device itself, not the development machine. Making the host configurable avoids hard-coding a value that only works in one runtime.

Alternative considered: automatic network discovery. Rejected as out of scope because the firmware does not advertise itself through mDNS or another discovery protocol.

### 4. Treat firmware responses as the source of truth

After each successful command, the app will render the returned `RobotStatus` when available or refresh `GET /status`. Failed commands will leave the last confirmed status visible and show the returned error message/code.

Rationale: the firmware owns state transitions and conflict rules. The app should not infer state changes that the controller may reject.

Alternative considered: optimistic UI updates. Rejected because conflicts such as low battery, invalid state, or active safety errors are common and already modeled by the firmware.

### 5. Map "power" to supported control semantics

Because the firmware's `OFF` state is not reachable through HTTP, the Android app will not claim to turn hardware power fully on or off. The primary control may be presented as start/stop cleaning, where:
- start sends `POST /commands/start`;
- stop sends `POST /commands/stop` when the current state allows it;
- unavailable states disable or relabel the control based on current status.

Rationale: this preserves the intent of simple remote control without inventing unsupported firmware behavior.

Alternative considered: add `/commands/power-on` and `/commands/power-off` to the firmware. Rejected for this change because it would expand embedded power-management semantics beyond the implemented simulator contract.

### 6. Manual movement uses explicit short-duration commands

Directional controls will send `POST /commands/manual-move` with direction, speed, and duration. The app will validate that speed is `0..=100`, movement directions use speed greater than zero, and duration is greater than zero before sending.

Rationale: this matches the firmware command contract and prevents avoidable `400` responses.

Alternative considered: continuous press-and-hold streaming. Rejected for the first implementation because the firmware API accepts discrete duration-based movement commands, not a streaming control channel.

### 7. Keep the first UI operational, not marketing-oriented

The first screen will be the remote control itself: connection banner, status/telemetry summary, command controls, directional pad, and error feedback. It will not use a landing page or explanatory marketing view.

Rationale: the app is a control surface for repeated operation, and the user should reach the robot controls immediately.

## Risks / Trade-offs

- Android build tooling may add substantial generated files. Mitigation: keep the app minimal and avoid committing local build output.
- A physical phone cannot reach the host machine through `10.0.2.2`. Mitigation: document emulator versus physical-device base URL setup.
- Firmware command conflicts may surprise users if controls are always enabled. Mitigation: derive button enabled states from `RobotStatus` and still handle conflicts returned by the API.
- Without discovery, users must know the firmware host IP. Mitigation: provide an editable host field and retry control.
- If the firmware JSON contract changes, app DTOs can drift. Mitigation: add focused tests for status/error decoding and command request payloads.

## Migration Plan

1. Add the Android project scaffold and commit only source/config files, not build outputs.
2. Implement API DTOs and HTTP client mapping for current firmware endpoints.
3. Implement repository/state handling for connection, refresh, commands, and errors.
4. Build the main remote-control screen and command controls.
5. Add tests for API mapping, UI state, validation, and conflict/error paths.
6. Update README with Android run instructions and base URL examples.

Rollback strategy: remove the `android-app/` directory and README additions. The firmware crate remains independently runnable throughout the change.
