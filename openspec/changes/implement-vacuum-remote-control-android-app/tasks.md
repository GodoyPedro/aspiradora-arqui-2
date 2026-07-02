## 1. Android Project Foundation
- [x] 1.1 Add a minimal Android application project under `android-app/` with
      source, Gradle configuration, app manifest, and ignored build outputs.
      Stack: Kotlin, Jetpack Compose, Ktor Client (CIO), Hilt, kotlinx.serialization.
- [x] 1.2 Configure required Android permissions for local HTTP networking and
      support cleartext traffic for development firmware URLs.
- [x] 1.3 Add app constants for the default firmware base URL
      (`http://10.0.2.2:3000` for emulator). Default D-pad speed and duration
      are user-configurable (see 4.4); do not hard-code them.

## 2. Firmware API Client
- [x] 2.1 Add Android DTOs for `RobotStatus`, sensor flags, structured API
      errors, manual movement requests, and mode requests using
      `kotlinx.serialization`.
- [x] 2.2 Implement `GET /status` with connection success, unreachable-host,
      malformed-response, and non-2xx handling via Ktor Client.
- [x] 2.3 Implement command calls for start, stop, pause, return-to-dock,
      manual-move, mode, and clear-error.
- [x] 2.4 Decode HTTP `400` and `409` structured errors into user-presentable
      app state without replacing the last confirmed robot status.

## 3. App State And Validation
- [x] 3.1 Add a `MainViewModel` (Hilt-injected) that owns base URL, connection
      status, last robot status, pending command state, last error, and
      user-configured D-pad speed and duration.
- [x] 3.2 Poll `GET /status` automatically every 2 seconds while the app is
      in the foreground. Cancel polling when backgrounded; resume on return.
- [x] 3.3 Validate manual movement speed (0..=100) and duration (> 0) in the
      ViewModel before sending requests. Surface validation errors inline.
- [x] 3.4 Derive command enabled/disabled states from the current `RobotStatus`
      while still tolerating server-side conflicts.
- [x] 3.5 Refresh status on initial load, retry, and after successful commands
      when a fresh status is needed.

## 4. Remote-Control UI (Jetpack Compose, English)
- [x] 4.1 Build the main screen with connection status banner, editable host
      field, retry/reconnect button, and manual refresh button.
- [x] 4.2 Display robot state, cleaning mode, battery percent, charging status,
      actuator state, wheel speeds, current error, and key sensor flags.
- [x] 4.3 Add controls for start/stop, pause, return-to-dock, clear-error,
      and AUTO mode.
- [x] 4.4 Add a D-pad gamepad layout for forward, backward, left,
      right, and stop/manual halt. Include two user-editable fields on the
      main screen for D-pad speed (0-100) and duration (ms); persist them
      in ViewModel state. Each D-pad press uses the current values.
- [x] 4.5 Show loading, disabled, offline, validation-error, and
      command-conflict states clearly without hiding the last known status.
      All visible text and labels in English.

## 5. Documentation
- [x] 5.1 Update README with instructions for running the Rust firmware and
      Android app together.
- [x] 5.2 Document emulator (`10.0.2.2`) versus physical-device base URL setup.
- [x] 5.3 Document that the app controls supported firmware commands but does
      not implement true hardware power off/on.

## 6. Verification
- [x] 6.1 Add tests for DTO serialization/deserialization and structured API
      error decoding.
- [x] 6.2 Add tests for manual movement validation (speed and duration) and
      command availability rules derived from `RobotStatus`.
- [x] 6.3 Add ViewModel tests for connection failure, retry, successful status
      display, 2-second polling cycle, successful command update, and command
      conflict feedback.
- [ ] 6.4 Run the Android test suite.
- [ ] 6.5 Run `cargo test` to ensure the firmware remains unaffected.
