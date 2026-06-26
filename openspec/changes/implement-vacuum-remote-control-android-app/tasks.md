## 1. Android Project Foundation

- [ ] 1.1 Add a minimal Android application project under `android-app/` with source, Gradle configuration, app manifest, and ignored build outputs
- [ ] 1.2 Configure required Android permissions for local HTTP networking and support cleartext traffic for development firmware URLs
- [ ] 1.3 Add app constants or settings for the default firmware base URL, including emulator-friendly `http://10.0.2.2:3000`

## 2. Firmware API Client

- [ ] 2.1 Add Android DTOs for `RobotStatus`, sensor flags, structured API errors, manual movement requests, and mode requests
- [ ] 2.2 Implement `GET /status` with connection success, unreachable-host, malformed-response, and non-2xx handling
- [ ] 2.3 Implement command calls for start, stop, pause, return-to-dock, manual-move, mode, and clear-error
- [ ] 2.4 Decode HTTP `400` and `409` structured errors into user-presentable app state without replacing the last confirmed robot status

## 3. App State And Validation

- [ ] 3.1 Add a repository or view-model layer that owns base URL, connection status, last robot status, pending command state, and last error
- [ ] 3.2 Validate manual movement speed and duration in the app before sending requests
- [ ] 3.3 Derive command enabled/disabled states from the current `RobotStatus` while still tolerating server-side conflicts
- [ ] 3.4 Refresh status on initial load, retry, and after successful commands when a fresh status is needed

## 4. Remote-Control UI

- [ ] 4.1 Build the main Android screen with connection status, editable host/base URL, retry/reconnect control, and refresh control
- [ ] 4.2 Display robot state, cleaning mode, battery percent, charging status, actuator state, wheel speeds, current error, and key sensor flags
- [ ] 4.3 Add controls for start/stop, pause, return-to-dock, clear-error, and AUTO mode
- [ ] 4.4 Add a directional control pad for forward, backward, left, right, and stop/manual halt behavior using duration-based manual commands
- [ ] 4.5 Show loading, disabled, offline, validation-error, and command-conflict states clearly without hiding the last known status

## 5. Documentation

- [ ] 5.1 Update README instructions for running the Rust firmware and Android app together
- [ ] 5.2 Document emulator versus physical-device base URL setup
- [ ] 5.3 Document the current limitation that the app controls supported firmware commands but does not implement true hardware power off/on

## 6. Verification

- [ ] 6.1 Add tests for DTO serialization/deserialization and structured API error decoding
- [ ] 6.2 Add tests for manual movement validation and command availability rules
- [ ] 6.3 Add UI or view-model tests for connection failure, retry, successful status display, successful command update, and command conflict feedback
- [ ] 6.4 Run the Android test suite
- [ ] 6.5 Run `cargo test` to ensure the firmware remains unaffected
