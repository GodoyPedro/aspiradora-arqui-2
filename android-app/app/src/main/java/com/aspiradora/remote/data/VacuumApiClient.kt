package com.aspiradora.remote.data

import io.ktor.client.HttpClient
import io.ktor.client.call.body
import io.ktor.client.plugins.ClientRequestException
import io.ktor.client.plugins.RedirectResponseException
import io.ktor.client.plugins.ServerResponseException
import io.ktor.client.request.get
import io.ktor.client.request.post
import io.ktor.client.request.setBody
import io.ktor.client.statement.HttpResponse
import io.ktor.client.statement.bodyAsText
import io.ktor.http.ContentType
import io.ktor.http.HttpStatusCode
import io.ktor.http.contentType
import kotlinx.serialization.SerializationException
import kotlinx.serialization.json.Json
import java.io.IOException
import javax.inject.Inject
import javax.inject.Singleton

sealed interface ApiResult<out T> {
    data class Success<T>(val value: T) : ApiResult<T>
    data class Failure(val error: AppError) : ApiResult<Nothing>
}

data class AppError(
    val code: String,
    val message: String,
    val currentState: String? = null,
) {
    val displayMessage: String
        get() = if (code.isBlank()) message else "$code: $message"
}

@Singleton
class VacuumApiClient @Inject constructor(
    private val client: HttpClient,
    private val json: Json,
) {
    suspend fun getStatus(baseUrl: String): ApiResult<RobotStatusDto> =
        requestStatus { client.get(baseUrl.endpoint("/status")) }

    suspend fun start(baseUrl: String): ApiResult<RobotStatusDto> =
        requestStatus { client.post(baseUrl.endpoint("/commands/start")) }

    suspend fun stop(baseUrl: String): ApiResult<RobotStatusDto> =
        requestStatus { client.post(baseUrl.endpoint("/commands/stop")) }

    suspend fun pause(baseUrl: String): ApiResult<RobotStatusDto> =
        requestStatus { client.post(baseUrl.endpoint("/commands/pause")) }

    suspend fun returnToDock(baseUrl: String): ApiResult<RobotStatusDto> =
        requestStatus { client.post(baseUrl.endpoint("/commands/return-to-dock")) }

    suspend fun clearError(baseUrl: String): ApiResult<RobotStatusDto> =
        requestStatus { client.post(baseUrl.endpoint("/commands/clear-error")) }

    suspend fun setMode(baseUrl: String, mode: CleaningModeRequest): ApiResult<RobotStatusDto> =
        requestStatus {
            client.post(baseUrl.endpoint("/commands/mode")) {
                contentType(ContentType.Application.Json)
                setBody(SetModeRequestDto(mode))
            }
        }

    suspend fun manualMove(
        baseUrl: String,
        direction: ManualDirection,
        speed: Int,
        durationMs: Long,
    ): ApiResult<RobotStatusDto> =
        requestStatus {
            client.post(baseUrl.endpoint("/commands/manual-move")) {
                contentType(ContentType.Application.Json)
                setBody(ManualMoveRequestDto(direction, speed, durationMs))
            }
        }

    private suspend fun requestStatus(call: suspend () -> HttpResponse): ApiResult<RobotStatusDto> =
        try {
            val response = call()
            if (response.status == HttpStatusCode.OK) {
                ApiResult.Success(response.body())
            } else {
                ApiResult.Failure(response.toAppError())
            }
        } catch (error: ClientRequestException) {
            ApiResult.Failure(error.response.toAppError())
        } catch (error: ServerResponseException) {
            ApiResult.Failure(error.response.toAppError())
        } catch (error: RedirectResponseException) {
            ApiResult.Failure(error.response.toAppError())
        } catch (error: SerializationException) {
            ApiResult.Failure(AppError("MALFORMED_RESPONSE", "The firmware returned invalid JSON."))
        } catch (error: IOException) {
            ApiResult.Failure(AppError("CONNECTION_ERROR", "Could not reach the vacuum API."))
        } catch (error: IllegalArgumentException) {
            ApiResult.Failure(AppError("INVALID_URL", "Check the firmware URL and try again."))
        }

    private suspend fun HttpResponse.toAppError(): AppError {
        val rawBody = bodyAsText()
        val decoded = runCatching { json.decodeFromString<ApiErrorDto>(rawBody) }.getOrNull()
        return AppError(
            code = decoded?.code ?: "HTTP_${status.value}",
            message = decoded?.message ?: rawBody.ifBlank { status.description },
            currentState = decoded?.currentState,
        )
    }

    private fun String.endpoint(path: String): String = trimEnd('/') + path
}
