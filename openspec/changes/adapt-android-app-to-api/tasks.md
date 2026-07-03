## 1. DTO Updates
- [ ] 1.1 Update `ManualMoveRequest` to match the required structure (direction, speed, duration_ms).
- [ ] 1.2 Add `ModeRequest` DTO.
- [ ] 1.3 Update `ApiErrorResponse` if necessary to capture structured error codes.

## 2. API Client Refinement
- [ ] 2.1 Implement `POST /commands/mode` in `VacuumApiClient`.
- [ ] 2.2 Implement `POST /commands/clear-error` in `VacuumApiClient`.
- [ ] 2.3 Ensure `ZIGZAG` alias logic is handled (either client-side or confirmed server-side).
- [ ] 2.4 Update existing command methods to return `RobotStatusResponse` correctly.

## 3. ViewModel & UI
- [ ] 3.1 Update `MainViewModel` to handle cleaning mode selection.
- [ ] 3.2 Add `clearError` action to `MainViewModel`.
- [ ] 3.3 Add UI controls for all cleaning modes.
- [ ] 3.4 Add "Clear Error" button to the UI, visible during error states.
- [ ] 3.5 Verify that D-pad uses the updated `ManualMoveRequest` structure.

## 4. Verification
- [ ] 4.1 Unit test DTO serialization.
- [ ] 4.2 Unit test API client error mapping (400, 409).
- [ ] 4.3 Manual verification with firmware simulator if available.
