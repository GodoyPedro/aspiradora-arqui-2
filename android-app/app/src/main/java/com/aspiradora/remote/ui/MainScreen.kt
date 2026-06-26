package com.aspiradora.remote.ui

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.AssistChip
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ElevatedButton
import androidx.compose.material3.LinearProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import com.aspiradora.remote.data.ManualDirection
import com.aspiradora.remote.data.RobotStatusDto

@Composable
fun MainRoute(viewModel: MainViewModel = hiltViewModel()) {
    val state by viewModel.uiState.collectAsState()
    val lifecycleOwner = LocalLifecycleOwner.current

    DisposableEffect(lifecycleOwner, viewModel) {
        val observer = LifecycleEventObserver { _, event ->
            when (event) {
                Lifecycle.Event.ON_START -> viewModel.startPolling()
                Lifecycle.Event.ON_STOP -> viewModel.stopPolling()
                else -> Unit
            }
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose {
            lifecycleOwner.lifecycle.removeObserver(observer)
            viewModel.stopPolling()
        }
    }

    MainScreen(
        state = state,
        onBaseUrlChange = viewModel::updateBaseUrl,
        onRetry = viewModel::retry,
        onRefresh = viewModel::manualRefresh,
        onStart = viewModel::startCleaning,
        onStop = viewModel::stopCleaning,
        onPause = viewModel::pauseCleaning,
        onReturnToDock = viewModel::returnToDock,
        onClearError = viewModel::clearError,
        onAutoMode = viewModel::setAutoMode,
        onSpeedChange = viewModel::updateSpeed,
        onDurationChange = viewModel::updateDuration,
        onManualMove = viewModel::manualMove,
    )
}

@Composable
fun MainScreen(
    state: MainUiState,
    onBaseUrlChange: (String) -> Unit,
    onRetry: () -> Unit,
    onRefresh: () -> Unit,
    onStart: () -> Unit,
    onStop: () -> Unit,
    onPause: () -> Unit,
    onReturnToDock: () -> Unit,
    onClearError: () -> Unit,
    onAutoMode: () -> Unit,
    onSpeedChange: (String) -> Unit,
    onDurationChange: (String) -> Unit,
    onManualMove: (ManualDirection) -> Unit,
) {
    val busy = state.isLoading || state.isCommandPending

    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(16.dp),
        verticalArrangement = Arrangement.spacedBy(12.dp),
    ) {
        Text("Vacuum Remote", style = MaterialTheme.typography.headlineMedium)
        ConnectionPanel(state, onBaseUrlChange, onRetry, onRefresh)
        if (busy) LinearProgressIndicator(modifier = Modifier.fillMaxWidth())
        state.lastError?.let { ErrorText(it) }
        state.validationError?.let { ErrorText(it) }
        StatusPanel(status = state.status)
        CommandPanel(state.availability, busy, onStart, onStop, onPause, onReturnToDock, onClearError, onAutoMode)
        DPadPanel(
            state = state,
            busy = busy,
            onSpeedChange = onSpeedChange,
            onDurationChange = onDurationChange,
            onManualMove = onManualMove,
        )
    }
}

@Composable
private fun ConnectionPanel(
    state: MainUiState,
    onBaseUrlChange: (String) -> Unit,
    onRetry: () -> Unit,
    onRefresh: () -> Unit,
) {
    Card(colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surfaceVariant)) {
        Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                AssistChip(onClick = {}, label = { Text(state.connectionState.name) })
                Text("Firmware API", style = MaterialTheme.typography.titleMedium)
            }
            OutlinedTextField(
                value = state.baseUrl,
                onValueChange = onBaseUrlChange,
                modifier = Modifier.fillMaxWidth(),
                singleLine = true,
                label = { Text("Base URL") },
            )
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Button(onClick = onRetry) { Text("Retry") }
                TextButton(onClick = onRefresh) { Text("Refresh") }
            }
        }
    }
}

@Composable
private fun StatusPanel(status: RobotStatusDto?) {
    Card {
        Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text("Status", style = MaterialTheme.typography.titleLarge)
            if (status == null) {
                Text("No confirmed status yet.")
            } else {
                Metric("State", status.state)
                Metric("Mode", status.cleaningMode)
                Metric("Battery", "${status.batteryPercent}%")
                Metric("Charging", status.isCharging.yesNo())
                Metric("Suction", status.suctionEnabled.onOff())
                Metric("Brushes", status.brushesEnabled.onOff())
                Metric("Wheels", "L ${status.leftWheelSpeed} / R ${status.rightWheelSpeed}")
                Metric("Current error", status.currentError ?: "None")
                Spacer(Modifier.height(4.dp))
                Text("Sensors", style = MaterialTheme.typography.titleMedium)
                Metric("Obstacle", status.sensors.obstacleDetected.yesNo())
                Metric("Drop off", status.sensors.dropOffDetected.yesNo())
                Metric("Bumper", status.sensors.bumperPressed.yesNo())
                Metric("Dust full", status.sensors.dustContainerFull.yesNo())
                Metric("Wheel stuck", status.sensors.wheelStuck.yesNo())
                Metric("Brush stuck", status.sensors.brushStuck.yesNo())
                Metric("Top cover open", status.sensors.topCoverOpen.yesNo())
            }
        }
    }
}

@Composable
private fun CommandPanel(
    availability: CommandAvailability,
    busy: Boolean,
    onStart: () -> Unit,
    onStop: () -> Unit,
    onPause: () -> Unit,
    onReturnToDock: () -> Unit,
    onClearError: () -> Unit,
    onAutoMode: () -> Unit,
) {
    Card {
        Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text("Commands", style = MaterialTheme.typography.titleLarge)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Button(enabled = availability.canStart && !busy, onClick = onStart) { Text("Start") }
                Button(enabled = availability.canStop && !busy, onClick = onStop) { Text("Stop") }
                Button(enabled = availability.canPause && !busy, onClick = onPause) { Text("Pause") }
            }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Button(enabled = availability.canReturnToDock && !busy, onClick = onReturnToDock) { Text("Dock") }
                Button(enabled = availability.canClearError && !busy, onClick = onClearError) { Text("Clear error") }
                Button(enabled = availability.canSetAutoMode && !busy, onClick = onAutoMode) { Text("AUTO") }
            }
        }
    }
}

@Composable
private fun DPadPanel(
    state: MainUiState,
    busy: Boolean,
    onSpeedChange: (String) -> Unit,
    onDurationChange: (String) -> Unit,
    onManualMove: (ManualDirection) -> Unit,
) {
    Card {
        Column(
            modifier = Modifier.padding(12.dp),
            horizontalAlignment = Alignment.CenterHorizontally,
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Text("Manual control", style = MaterialTheme.typography.titleLarge)
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                OutlinedTextField(
                    value = state.dpadSpeedText,
                    onValueChange = onSpeedChange,
                    modifier = Modifier.weight(1f),
                    label = { Text("Speed") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                )
                OutlinedTextField(
                    value = state.dpadDurationText,
                    onValueChange = onDurationChange,
                    modifier = Modifier.weight(1f),
                    label = { Text("Duration ms") },
                    singleLine = true,
                    keyboardOptions = KeyboardOptions(keyboardType = KeyboardType.Number),
                )
            }
            DirectionButton("▲", enabled = state.availability.canManualMove && !busy) {
                onManualMove(ManualDirection.FORWARD)
            }
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp), verticalAlignment = Alignment.CenterVertically) {
                DirectionButton("◀", enabled = state.availability.canManualMove && !busy) {
                    onManualMove(ManualDirection.LEFT)
                }
                ElevatedButton(enabled = state.availability.canManualMove && !busy, onClick = {
                    onManualMove(ManualDirection.STOP)
                }) {
                    Text("Stop")
                }
                DirectionButton("▶", enabled = state.availability.canManualMove && !busy) {
                    onManualMove(ManualDirection.RIGHT)
                }
            }
            DirectionButton("▼", enabled = state.availability.canManualMove && !busy) {
                onManualMove(ManualDirection.BACKWARD)
            }
        }
    }
}

@Composable
private fun DirectionButton(label: String, enabled: Boolean, onClick: () -> Unit) {
    Button(
        modifier = Modifier
            .width(72.dp)
            .height(48.dp),
        enabled = enabled,
        onClick = onClick,
    ) {
        Text(label)
    }
}

@Composable
private fun Metric(label: String, value: String) {
    Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
        Text(label, style = MaterialTheme.typography.bodyMedium)
        Text(value, style = MaterialTheme.typography.bodyMedium)
    }
}

@Composable
private fun ErrorText(value: String) {
    Text(value, color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodyMedium)
}

private fun Boolean.yesNo(): String = if (this) "Yes" else "No"
private fun Boolean.onOff(): String = if (this) "On" else "Off"
