package com.aspiradora.remote.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import com.aspiradora.remote.data.ApiResult
import com.aspiradora.remote.data.DEFAULT_BASE_URL
import com.aspiradora.remote.data.DEFAULT_DPAD_DURATION_MS
import com.aspiradora.remote.data.DEFAULT_DPAD_SPEED
import com.aspiradora.remote.data.ManualDirection
import com.aspiradora.remote.data.RobotStatusDto
import com.aspiradora.remote.data.VacuumRepository
import dagger.hilt.android.lifecycle.HiltViewModel
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import javax.inject.Inject

private const val POLL_INTERVAL_MS = 2_000L

data class CommandAvailability(
    val canStart: Boolean = false,
    val canStop: Boolean = false,
    val canPause: Boolean = false,
    val canReturnToDock: Boolean = false,
    val canClearError: Boolean = false,
    val canManualMove: Boolean = false,
    val canSetMode: Boolean = false,
)

enum class ConnectionState {
    UNKNOWN,
    CONNECTED,
    OFFLINE,
}

data class MainUiState(
    val baseUrl: String = DEFAULT_BASE_URL,
    val connectionState: ConnectionState = ConnectionState.UNKNOWN,
    val status: RobotStatusDto? = null,
    val isLoading: Boolean = false,
    val isCommandPending: Boolean = false,
    val lastError: String? = null,
    val validationError: String? = null,
    val dpadSpeedText: String = DEFAULT_DPAD_SPEED.toString(),
    val dpadDurationText: String = DEFAULT_DPAD_DURATION_MS.toString(),
    val availability: CommandAvailability = CommandAvailability(),
)

@HiltViewModel
class MainViewModel @Inject constructor(
    private val repository: VacuumRepository,
) : ViewModel() {
    private val _uiState = MutableStateFlow(MainUiState())
    val uiState: StateFlow<MainUiState> = _uiState.asStateFlow()

    private var pollingJob: Job? = null

    fun startPolling() {
        if (pollingJob?.isActive == true) return
        pollingJob = viewModelScope.launch {
            refreshStatus()
            while (true) {
                delay(POLL_INTERVAL_MS)
                refreshStatus(silent = true)
            }
        }
    }

    fun stopPolling() {
        pollingJob?.cancel()
        pollingJob = null
    }

    fun updateBaseUrl(value: String) {
        _uiState.update { it.copy(baseUrl = value, lastError = null) }
    }

    fun updateSpeed(value: String) {
        _uiState.update { it.copy(dpadSpeedText = value, validationError = null) }
    }

    fun updateDuration(value: String) {
        _uiState.update { it.copy(dpadDurationText = value, validationError = null) }
    }

    fun retry() {
        viewModelScope.launch { refreshStatus() }
    }

    fun manualRefresh() {
        viewModelScope.launch { refreshStatus() }
    }

    fun startCleaning() = runCommand { repository.start(uiState.value.baseUrl) }
    fun stopCleaning() = runCommand { repository.stop(uiState.value.baseUrl) }
    fun pauseCleaning() = runCommand { repository.pause(uiState.value.baseUrl) }
    fun returnToDock() = runCommand { repository.returnToDock(uiState.value.baseUrl) }
    fun clearError() = runCommand { repository.clearError(uiState.value.baseUrl) }
    fun setMode(mode: com.aspiradora.remote.data.CleaningModeRequest) =
        runCommand { repository.setMode(uiState.value.baseUrl, mode) }

    fun manualMove(direction: ManualDirection) {
        val speed = uiState.value.dpadSpeedText.toIntOrNull()
        val duration = uiState.value.dpadDurationText.toLongOrNull()
        val validationError = validateManualMove(direction, speed, duration)
        if (validationError != null) {
            _uiState.update { it.copy(validationError = validationError) }
            return
        }

        runCommand {
            repository.manualMove(uiState.value.baseUrl, direction, speed ?: 0, duration ?: 0)
        }
    }

    fun validateManualMove(
        direction: ManualDirection,
        speed: Int?,
        durationMs: Long?,
    ): String? {
        if (speed == null || speed !in 0..100) return "Speed must be between 0 and 100."
        if (durationMs == null || durationMs <= 0) return "Duration must be greater than 0 ms."
        if (direction != ManualDirection.STOP && speed == 0) {
            return "Movement speed must be greater than 0."
        }
        return null
    }

    private suspend fun refreshStatus(silent: Boolean = false) {
        if (!silent) {
            _uiState.update { it.copy(isLoading = true, lastError = null) }
        }
        when (val result = repository.status(uiState.value.baseUrl)) {
            is ApiResult.Success -> applyStatus(result.value, isLoading = false)
            is ApiResult.Failure -> _uiState.update {
                it.copy(
                    connectionState = ConnectionState.OFFLINE,
                    isLoading = false,
                    lastError = result.error.displayMessage,
                )
            }
        }
    }

    private fun runCommand(command: suspend () -> ApiResult<RobotStatusDto>) {
        viewModelScope.launch {
            _uiState.update {
                it.copy(isCommandPending = true, lastError = null, validationError = null)
            }
            when (val result = command()) {
                is ApiResult.Success -> applyStatus(result.value, isCommandPending = false)
                is ApiResult.Failure -> _uiState.update {
                    it.copy(
                        connectionState = ConnectionState.OFFLINE.takeIf {
                            result.error.code == "CONNECTION_ERROR"
                        } ?: it.connectionState,
                        isCommandPending = false,
                        lastError = result.error.displayMessage,
                    )
                }
            }
        }
    }

    private fun applyStatus(
        status: RobotStatusDto,
        isLoading: Boolean = uiState.value.isLoading,
        isCommandPending: Boolean = uiState.value.isCommandPending,
    ) {
        _uiState.update {
            it.copy(
                connectionState = ConnectionState.CONNECTED,
                status = status,
                isLoading = isLoading,
                isCommandPending = isCommandPending,
                lastError = null,
                availability = status.toAvailability(),
            )
        }
    }
}

fun RobotStatusDto.toAvailability(): CommandAvailability {
    val normalizedState = state.uppercase()
    return CommandAvailability(
        canStart = normalizedState == "STANDBY" || normalizedState == "PAUSED",
        canStop = normalizedState != "STANDBY" && normalizedState != "OFF",
        canPause = normalizedState == "CLEANING" || normalizedState == "MANUAL_CONTROL",
        canReturnToDock = normalizedState != "OFF" && normalizedState != "CHARGING" && normalizedState != "ERROR",
        canClearError = normalizedState == "ERROR",
        canManualMove = normalizedState != "OFF" && normalizedState != "CHARGING" && normalizedState != "ERROR",
        canSetMode = normalizedState != "OFF" && normalizedState != "ERROR",
    )
}
