package com.aspiradora.remote.data

import kotlinx.serialization.encodeToString
import kotlinx.serialization.json.Json
import org.junit.Assert.assertEquals
import org.junit.Test

class ApiModelsTest {
    private val json = Json { ignoreUnknownKeys = true }

    @Test
    fun decodesRobotStatus() {
        val status = json.decodeFromString<RobotStatusDto>(
            """
            {
              "state": "STANDBY",
              "cleaning_mode": "AUTO",
              "battery_percent": 80,
              "is_charging": false,
              "suction_enabled": false,
              "brushes_enabled": false,
              "left_wheel_speed": 0,
              "right_wheel_speed": 0,
              "current_error": null,
              "sensors": {
                "obstacle_detected": false,
                "drop_off_detected": false,
                "bumper_pressed": false,
                "dust_container_full": false,
                "wheel_stuck": false,
                "brush_stuck": false,
                "top_cover_open": false
              }
            }
            """.trimIndent(),
        )

        assertEquals("STANDBY", status.state)
        assertEquals("AUTO", status.cleaningMode)
        assertEquals(80, status.batteryPercent)
    }

    @Test
    fun encodesManualMoveRequest() {
        val body = json.encodeToString(
            ManualMoveRequestDto(
                direction = ManualDirection.FORWARD,
                speed = 45,
                durationMs = 500,
            ),
        )

        assertEquals("""{"direction":"FORWARD","speed":45,"duration_ms":500}""", body)
    }

    @Test
    fun decodesStructuredApiError() {
        val error = json.decodeFromString<ApiErrorDto>(
            """{"code":"COMMAND_NOT_ALLOWED","message":"Nope","current_state":"STANDBY"}""",
        )

        assertEquals("COMMAND_NOT_ALLOWED", error.code)
        assertEquals("Nope", error.message)
        assertEquals("STANDBY", error.currentState)
    }

    @Test
    fun encodesSetModeRequest() {
        val bodyAuto = json.encodeToString(SetModeRequestDto(CleaningModeRequest.AUTO))
        assertEquals("""{"mode":"AUTO"}""", bodyAuto)

        val bodyZigZag = json.encodeToString(SetModeRequestDto(CleaningModeRequest.ZIG_ZAG))
        assertEquals("""{"mode":"ZIG_ZAG"}""", bodyZigZag)

        val bodyWall = json.encodeToString(SetModeRequestDto(CleaningModeRequest.WALL_FOLLOWING))
        assertEquals("""{"mode":"WALL_FOLLOWING"}""", bodyWall)
    }

    @Test
    fun decodesZigZagAlias() {
        val status = json.decodeFromString<RobotStatusDto>(
            """{"state":"STANDBY","cleaning_mode":"ZIGZAG","battery_percent":100,"is_charging":true,"suction_enabled":false,"brushes_enabled":false,"left_wheel_speed":0,"right_wheel_speed":0,"current_error":null,"sensors":{"obstacle_detected":false,"drop_off_detected":false,"bumper_pressed":false,"dust_container_full":false,"wheel_stuck":false,"brush_stuck":false,"top_cover_open":false}}"""
        )
        // Ensure it decodes correctly even with the alias from server
        assertEquals("ZIGZAG", status.cleaningMode)
    }
}
