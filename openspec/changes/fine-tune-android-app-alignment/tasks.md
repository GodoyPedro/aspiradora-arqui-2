## 1. DTO Synchronization
- [ ] 1.1 Add `auto_navigation_phase` to `RobotStatusDto`.
- [ ] 1.2 Add `proximity_contact`, `contact_type`, `wall_side`, and `forward_clearance_blocked` to `SensorSnapshotDto`.
- [ ] 1.3 Verify wheel speed types (`Int` in Kotlin is fine for `i16`).

## 2. UI Telemetry Display
- [ ] 2.1 Update `StatusPanel` in `MainScreen.kt` to include new sensor fields.
- [ ] 2.2 Add navigation phase display to `StatusPanel`.
- [ ] 2.3 Normalize display strings for new enums/strings.

## 3. Verification
- [ ] 3.1 Update `ApiModelsTest.kt` to include the new fields in JSON decoding tests.
- [ ] 3.2 Perform static analysis to ensure no regressions.
