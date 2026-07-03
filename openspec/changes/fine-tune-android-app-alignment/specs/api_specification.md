# Robot Vacuum Firmware API Specification (Detailed Alignment)

This specification reflects the exact implementation found in `firmware/src/api/`.

## Data Models

### RobotStatusResponse
| Property | Type | Description |
| :--- | :--- | :--- |
| `state` | String | Current robot state. |
| `cleaning_mode` | String | Active cleaning mode. |
| `battery_percent` | u8 | 0-100. |
| `is_charging` | bool | Power connection status. |
| `suction_enabled` | bool | Suction status. |
| `brushes_enabled` | bool | Brushes status. |
| `left_wheel_speed` | i16 | Signed speed (allows negative). |
| `right_wheel_speed` | i16 | Signed speed (allows negative). |
| `current_error` | String? | Error code if state is ERROR. |
| `auto_navigation_phase` | String? | Internal phase for AUTO mode. |
| `sensors` | Object | [SensorSnapshot](#sensorsnapshot). |

### SensorSnapshot
| Property | Type | Description |
| :--- | :--- | :--- |
| `obstacle_detected` | bool | Generic proximity detection. |
| `drop_off_detected` | bool | Floor sensors. |
| `bumper_pressed` | bool | Physical collision. |
| `proximity_contact` | bool | Near-field detection without impact. |
| `contact_type` | String? | `wall`, `obstacle`, `unknown`. |
| `wall_side` | String? | `left`, `right`, `top`, `bottom`. |
| `forward_clearance_blocked` | bool | Safety stop condition. |
| `dust_container_full` | bool | Maintenance flag. |
| `wheel_stuck` | bool | Motor stall flag. |
| `brush_stuck` | bool | Motor stall flag. |
| `top_cover_open` | bool | Safety flag. |
