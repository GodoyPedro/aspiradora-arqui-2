package com.aspiradora.remote.ui

import com.aspiradora.remote.data.ApiResult
import com.aspiradora.remote.data.AppError
import com.aspiradora.remote.data.ManualDirection
import com.aspiradora.remote.data.RobotStatusDto
import com.aspiradora.remote.data.SensorSnapshotDto
import com.aspiradora.remote.data.VacuumRepository
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.test.StandardTestDispatcher
import kotlinx.coroutines.test.advanceTimeBy
import kotlinx.coroutines.test.runCurrent
import kotlinx.coroutines.test.runTest
import kotlinx.coroutines.test.setMain
import kotlinx.coroutines.test.resetMain
import org.junit.After
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertTrue
import org.junit.Before
import org.junit.Test

@OptIn(ExperimentalCoroutinesApi::class)
class MainViewModelTest {
    private val dispatcher = StandardTestDispatcher()

    @Before
    fun setUp() {
        Dispatchers.setMain(dispatcher)
    }

    @After
    fun tearDown() {
        Dispatchers.resetMain()
    }

    @Test
    fun validatesManualMovement() {
        val viewModel = MainViewModel(FakeRepository())

        assertEquals("Speed must be between 0 and 100.", viewModel.validateManualMove(ManualDirection.FORWARD, 101, 500))
        assertEquals("Duration must be greater than 0 ms.", viewModel.validateManualMove(ManualDirection.FORWARD, 45, 0))
        assertEquals("Movement speed must be greater than 0.", viewModel.validateManualMove(ManualDirection.FORWARD, 0, 500))
        assertEquals(null, viewModel.validateManualMove(ManualDirection.STOP, 0, 500))
    }

    @Test
    fun derivesCommandAvailabilityFromStatus() {
        val standby = status("STANDBY").toAvailability()
        assertTrue(standby.canStart)
        assertFalse(standby.canStop)

        val error = status("ERROR").toAvailability()
        assertTrue(error.canClearError)
        assertFalse(error.canManualMove)
    }

    @Test
    fun retryDisplaysSuccessfulStatus() = runTest {
        val repository = FakeRepository(statusResult = ApiResult.Success(status("STANDBY")))
        val viewModel = MainViewModel(repository)

        viewModel.retry()
        runCurrent()

        assertEquals(ConnectionState.CONNECTED, viewModel.uiState.value.connectionState)
        assertEquals("STANDBY", viewModel.uiState.value.status?.state)
    }

    @Test
    fun connectionFailurePreservesLastStatus() = runTest {
        val repository = FakeRepository(statusResult = ApiResult.Success(status("STANDBY")))
        val viewModel = MainViewModel(repository)
        viewModel.retry()
        runCurrent()

        repository.statusResult = ApiResult.Failure(AppError("CONNECTION_ERROR", "Offline"))
        viewModel.retry()
        runCurrent()

        assertEquals(ConnectionState.OFFLINE, viewModel.uiState.value.connectionState)
        assertEquals("STANDBY", viewModel.uiState.value.status?.state)
        assertNotNull(viewModel.uiState.value.lastError)
    }

    @Test
    fun successfulCommandUpdatesStatus() = runTest {
        val repository = FakeRepository(commandResult = ApiResult.Success(status("CLEANING")))
        val viewModel = MainViewModel(repository)

        viewModel.startCleaning()
        runCurrent()

        assertEquals("CLEANING", viewModel.uiState.value.status?.state)
        assertEquals(ConnectionState.CONNECTED, viewModel.uiState.value.connectionState)
    }

    @Test
    fun commandConflictKeepsLastStatus() = runTest {
        val repository = FakeRepository(statusResult = ApiResult.Success(status("STANDBY")))
        val viewModel = MainViewModel(repository)
        viewModel.retry()
        runCurrent()

        repository.commandResult = ApiResult.Failure(AppError("COMMAND_NOT_ALLOWED", "Already stopped"))
        viewModel.stopCleaning()
        runCurrent()

        assertEquals("STANDBY", viewModel.uiState.value.status?.state)
        assertEquals("COMMAND_NOT_ALLOWED: Already stopped", viewModel.uiState.value.lastError)
    }

    @Test
    fun pollingRefreshesEveryTwoSeconds() = runTest {
        val repository = FakeRepository(statusResult = ApiResult.Success(status("STANDBY")))
        val viewModel = MainViewModel(repository)

        viewModel.startPolling()
        runCurrent()
        advanceTimeBy(2_000)
        runCurrent()
        viewModel.stopPolling()

        assertEquals(2, repository.statusCalls)
    }

    private fun status(state: String) = RobotStatusDto(
        state = state,
        cleaningMode = "AUTO",
        batteryPercent = 80,
        isCharging = false,
        suctionEnabled = false,
        brushesEnabled = false,
        leftWheelSpeed = 0,
        rightWheelSpeed = 0,
        currentError = null,
        sensors = SensorSnapshotDto(
            obstacleDetected = false,
            dropOffDetected = false,
            bumperPressed = false,
            dustContainerFull = false,
            wheelStuck = false,
            brushStuck = false,
            topCoverOpen = false,
        ),
    )
}

private class FakeRepository(
    var statusResult: ApiResult<RobotStatusDto> = ApiResult.Failure(AppError("CONNECTION_ERROR", "Offline")),
    var commandResult: ApiResult<RobotStatusDto> = statusResult,
) : VacuumRepository {
    var statusCalls = 0

    override suspend fun status(baseUrl: String): ApiResult<RobotStatusDto> {
        statusCalls += 1
        return statusResult
    }

    override suspend fun start(baseUrl: String) = commandResult
    override suspend fun stop(baseUrl: String) = commandResult
    override suspend fun pause(baseUrl: String) = commandResult
    override suspend fun returnToDock(baseUrl: String) = commandResult
    override suspend fun clearError(baseUrl: String) = commandResult
    override suspend fun setAutoMode(baseUrl: String) = commandResult
    override suspend fun manualMove(
        baseUrl: String,
        direction: ManualDirection,
        speed: Int,
        durationMs: Long,
    ) = commandResult
}
