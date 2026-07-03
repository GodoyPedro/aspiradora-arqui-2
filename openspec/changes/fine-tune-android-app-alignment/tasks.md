## 1. DTO Synchronization
- [x] 1.1 Add `auto_navigation_phase` to `RobotStatusDto`.
- [x] 1.2 Add `proximity_contact`, `contact_type`, `wall_side`, and `forward_clearance_blocked` to `SensorSnapshotDto`.
- [x] 1.3 Verify wheel speed types (`Int` in Kotlin is fine for `i16`).

## 2. UI Telemetry Display
- [x] 2.1 Update `StatusPanel` in `MainScreen.kt` to include new sensor fields.
- [x] 2.2 Add navigation phase display to `StatusPanel`.
- [x] 2.3 Normalize display strings for new enums/strings.

## 3. Verification
- [x] 3.1 Update `ApiModelsTest.kt` to include the new fields in JSON decoding tests.
- [x] 3.2 Perform static analysis to ensure no regressions.
