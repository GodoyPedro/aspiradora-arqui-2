package com.aspiradora.remote.data

import io.ktor.client.HttpClient
import io.ktor.client.engine.mock.MockEngine
import io.ktor.client.engine.mock.respond
import io.ktor.client.plugins.contentnegotiation.ContentNegotiation
import io.ktor.http.ContentType
import io.ktor.http.HttpHeaders
import io.ktor.http.HttpStatusCode
import io.ktor.http.headersOf
import io.ktor.serialization.kotlinx.json.json
import kotlinx.coroutines.test.runTest
import kotlinx.serialization.json.Json
import org.junit.Assert.assertEquals
import org.junit.Assert.assertTrue
import org.junit.Test

class VacuumApiClientTest {
    private val json = Json { ignoreUnknownKeys = true }

    @Test
    fun getStatusReturnsSuccess() = runTest {
        val client = clientWithResponse(statusJson())
        val result = client.getStatus("http://host:3000")

        assertTrue(result is ApiResult.Success)
        assertEquals("STANDBY", (result as ApiResult.Success).value.state)
    }

    @Test
    fun decodesStructuredConflictError() = runTest {
        val httpClient = HttpClient(
            MockEngine {
                respond(
                    content = """{"code":"COMMAND_NOT_ALLOWED","message":"Stop is unavailable","current_state":"STANDBY"}""",
                    status = HttpStatusCode.Conflict,
                    headers = headersOf(HttpHeaders.ContentType, ContentType.Application.Json.toString()),
                )
            },
        ) {
            install(ContentNegotiation) { json(json) }
            expectSuccess = false
        }
        val result = VacuumApiClient(httpClient, json).stop("http://host:3000")

        assertTrue(result is ApiResult.Failure)
        assertEquals("COMMAND_NOT_ALLOWED", (result as ApiResult.Failure).error.code)
    }

    @Test
    fun malformedStatusReturnsFailure() = runTest {
        val client = clientWithResponse("{")
        val result = client.getStatus("http://host:3000")

        assertTrue(result is ApiResult.Failure)
        assertEquals("MALFORMED_RESPONSE", (result as ApiResult.Failure).error.code)
    }

    private fun clientWithResponse(body: String): VacuumApiClient {
        val httpClient = HttpClient(
            MockEngine {
                respond(
                    content = body,
                    status = HttpStatusCode.OK,
                    headers = headersOf(HttpHeaders.ContentType, ContentType.Application.Json.toString()),
                )
            },
        ) {
            install(ContentNegotiation) { json(json) }
            expectSuccess = false
        }
        return VacuumApiClient(httpClient, json)
    }
}

fun statusJson(state: String = "STANDBY"): String =
    """
    {
      "state": "$state",
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
    """.trimIndent()
