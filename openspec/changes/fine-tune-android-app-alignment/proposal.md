## Why

After a detailed audit of the `firmware/src/api/` source code, several discrepancies and missing telemetry fields were identified in the Android application. To ensure the app is a high-fidelity client for the firmware, it must support all available sensor flags and navigation phase telemetry.

## What Changes

- **Telemetry Expansion**: Update `RobotStatusDto` and `SensorSnapshotDto` in Android to include `auto_navigation_phase`, `proximity_contact`, `contact_type`, `wall_side`, and `forward_clearance_blocked`.
- **Data Type Alignment**: Ensure wheel speeds are correctly treated as signed integers to reflect backward movement.
- **UI Enhancement**: Update the `StatusPanel` in the Android app to display these new telemetry points, providing the user with deeper insight into the robot's behavior and environment perception.
- **DTO Robustness**: Refine `CleaningModeRequest` to handle `ZIGZAG` alias more naturally.

## Capabilities

### New Capabilities
- `extended-telemetry-display`: Visualization of advanced sensor data and navigation phases.
- `bi-directional-motion-telemetry`: Accurate display of negative wheel speeds.

### Modified Capabilities
- `api-client-fidelity`: Synchronized DTOs with the current `axum` implementation in the firmware.
