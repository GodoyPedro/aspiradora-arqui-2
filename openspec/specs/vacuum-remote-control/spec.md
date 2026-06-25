# vacuum-remote-control Specification

## Purpose
Allow the user to remotely control a WiFi-connected vacuum cleaner
from an Android application on the same local network.

## Requirements

### Requirement: Connection to vacuum
The system SHALL connect to the vacuum via its existing API over WiFi
on the local network.

#### Scenario: Successful connection
- GIVEN the phone and vacuum are on the same WiFi network
- WHEN the user opens the app
- THEN the app connects automatically to the vacuum API
- AND displays the current connection status

#### Scenario: Connection failure
- GIVEN the vacuum is unreachable
- WHEN the app attempts to connect
- THEN the app displays an error message
- AND offers a retry option

---

### Requirement: Power control
The system SHALL allow the user to turn the vacuum on and off.

#### Scenario: Turn on
- GIVEN the vacuum is connected and off
- WHEN the user taps the power button
- THEN the app sends a power-on command to the API
- AND the UI reflects the new state

#### Scenario: Turn off
- GIVEN the vacuum is connected and on
- WHEN the user taps the power button
- THEN the app sends a power-off command to the API
- AND the UI reflects the new state

---

### Requirement: Directional control
The system SHALL allow the user to control the movement of the vacuum
(forward, backward, left turn, right turn).

#### Scenario: Movement command
- GIVEN the vacuum is connected and on
- WHEN the user presses a directional control
- THEN the app sends the corresponding movement command to the API
- AND the vacuum responds accordingly

#### Scenario: Movement while off
- GIVEN the vacuum is off
- WHEN the user presses a directional control
- THEN the app does NOT send a command
- AND displays a warning indicating the vacuum must be on first