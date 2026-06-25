## Why

The firmware already exposes a usable HTTP/JSON API, but the current HTTP-level test coverage is too narrow to serve as a strong client-facing regression suite. Expanding the integration tests now will make the documented API behavior more reliable and will surface edge-case drift at the boundary that external consumers actually use.

## What Changes

- Add a broad HTTP integration test suite that exercises the existing `axum` router in memory and validates status codes, JSON bodies, state transitions, actuator state, and error responses.
- Add reusable test helpers for building isolated app state, sending JSON requests, parsing `RobotStatus`, and injecting simulated sensor, battery, and docking conditions without exposing public simulation endpoints.
- Expand HTTP tests across all documented endpoints: `/status`, `/commands/start`, `/commands/stop`, `/commands/pause`, `/commands/return-to-dock`, `/commands/manual-move`, `/commands/mode`, and `/commands/clear-error`.
- Clarify existing documented API behavior only where the test suite must lock down an already implemented path, such as low-battery start behavior, standby stop behavior, standby return-to-dock behavior, and standby clear-error behavior.
- Preserve the current architecture, module boundaries, HAL traits, in-memory simulated drivers, and production firmware behavior unless a minimal change is strictly necessary to make the tests deterministic and aligned with the current spec.

## Capabilities

### New Capabilities

### Modified Capabilities
- `command-protocol`: Clarify externally visible HTTP behavior that the expanded integration suite must assert, including deterministic responses for existing edge cases and structured error body expectations.
- `simulation-and-testing`: Add broad HTTP integration coverage, reusable HTTP test helpers, deterministic state injection helpers for tests, and a dedicated phase for API-level integration testing.

## Impact

- Affects the HTTP integration test layout, test helper modules, and API-facing regression coverage.
- May require small spec clarifications where behavior already exists in the current implementation but is not explicit enough for stable HTTP assertions.
- Does not redesign the firmware architecture or introduce non-test-facing platform dependencies.
