## Data Model Updates

### `RobotStatusDto` Updates
```kotlin
@Serializable
data class RobotStatusDto(
    // ... existing fields ...
    @SerialName("auto_navigation_phase") val autoNavigationPhase: String?,
    val sensors: SensorSnapshotDto,
)
```

### `SensorSnapshotDto` Updates
```kotlin
@Serializable
data class SensorSnapshotDto(
    // ... existing fields ...
    @SerialName("proximity_contact") val proximityContact: Boolean,
    @SerialName("contact_type") val contactType: String?,
    @SerialName("wall_side") val wallSide: String?,
    @SerialName("forward_clearance_blocked") val forwardClearanceBlocked: Boolean,
)
```

## UI Logic Updates

- **StatusPanel**: Add rows for "Nav Phase", "Proximity", "Contact Type", "Wall Side", and "Forward Clearance".
- **Formatting**: Ensure "Nav Phase" and "Contact Type" are humanized (e.g., lowercase with first letter capital).
