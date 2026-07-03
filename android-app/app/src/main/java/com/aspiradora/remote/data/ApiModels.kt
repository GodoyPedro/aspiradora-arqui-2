package com.aspiradora.remote.data

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable

const val DEFAULT_BASE_URL = "http://10.0.2.2:3000"
const val DEFAULT_DPAD_SPEED = 45
const val DEFAULT_DPAD_DURATION_MS = 500L

@Serializable
data class RobotStatusDto(
    val state: String,
    @SerialName("cleaning_mode") val cleaningMode: String,
    @SerialName("battery_percent") val batteryPercent: Int,
    @SerialName("is_charging") val isCharging: Boolean,
    @SerialName("suction_enabled") val suctionEnabled: Boolean,
    @SerialName("brushes_enabled") val brushesEnabled: Boolean,
    @SerialName("left_wheel_speed") val leftWheelSpeed: Int,
    @SerialName("right_wheel_speed") val rightWheelSpeed: Int,
    @SerialName("current_error") val currentError: String?,
    @SerialName("auto_navigation_phase") val autoNavigationPhase: String?,
    val sensors: SensorSnapshotDto,
)

@Serializable
data class SensorSnapshotDto(
    @SerialName("obstacle_detected") val obstacleDetected: Boolean,
    @SerialName("drop_off_detected") val dropOffDetected: Boolean,
    @SerialName("bumper_pressed") val bumperPressed: Boolean,
    @SerialName("proximity_contact") val proximityContact: Boolean,
    @SerialName("contact_type") val contactType: String?,
    @SerialName("wall_side") val wallSide: String?,
    @SerialName("forward_clearance_blocked") val forwardClearanceBlocked: Boolean,
    @SerialName("dust_container_full") val dustContainerFull: Boolean,
    @SerialName("wheel_stuck") val wheelStuck: Boolean,
    @SerialName("brush_stuck") val brushStuck: Boolean,
    @SerialName("top_cover_open") val topCoverOpen: Boolean,
)

@Serializable
data class ApiErrorDto(
    val code: String,
    val message: String,
    @SerialName("current_state") val currentState: String? = null,
)

@Serializable
data class ManualMoveRequestDto(
    val direction: ManualDirection,
    val speed: Int,
    @SerialName("duration_ms") val durationMs: Long,
)

@Serializable
data class SetModeRequestDto(
    val mode: CleaningModeRequest,
)

@Serializable
enum class ManualDirection {
    @SerialName("FORWARD")
    FORWARD,

    @SerialName("BACKWARD")
    BACKWARD,

    @SerialName("LEFT")
    LEFT,

    @SerialName("RIGHT")
    RIGHT,

    @SerialName("STOP")
    STOP,
}

@Serializable
enum class CleaningModeRequest {
    @SerialName("AUTO")
    AUTO,

    @SerialName("ZIG_ZAG")
    ZIG_ZAG,

    @SerialName("ZIGZAG")
    ZIGZAG,

    @SerialName("WALL_FOLLOWING")
    WALL_FOLLOWING,

    @SerialName("SPOT")
    SPOT,
}
