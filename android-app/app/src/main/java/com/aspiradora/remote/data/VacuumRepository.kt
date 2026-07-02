package com.aspiradora.remote.data

import javax.inject.Inject
import javax.inject.Singleton

interface VacuumRepository {
    suspend fun status(baseUrl: String): ApiResult<RobotStatusDto>
    suspend fun start(baseUrl: String): ApiResult<RobotStatusDto>
    suspend fun stop(baseUrl: String): ApiResult<RobotStatusDto>
    suspend fun pause(baseUrl: String): ApiResult<RobotStatusDto>
    suspend fun returnToDock(baseUrl: String): ApiResult<RobotStatusDto>
    suspend fun clearError(baseUrl: String): ApiResult<RobotStatusDto>
    suspend fun setAutoMode(baseUrl: String): ApiResult<RobotStatusDto>
    suspend fun manualMove(
        baseUrl: String,
        direction: ManualDirection,
        speed: Int,
        durationMs: Long,
    ): ApiResult<RobotStatusDto>
}

@Singleton
class HttpVacuumRepository @Inject constructor(
    private val apiClient: VacuumApiClient,
) : VacuumRepository {
    override suspend fun status(baseUrl: String) = apiClient.getStatus(baseUrl)
    override suspend fun start(baseUrl: String) = apiClient.start(baseUrl)
    override suspend fun stop(baseUrl: String) = apiClient.stop(baseUrl)
    override suspend fun pause(baseUrl: String) = apiClient.pause(baseUrl)
    override suspend fun returnToDock(baseUrl: String) = apiClient.returnToDock(baseUrl)
    override suspend fun clearError(baseUrl: String) = apiClient.clearError(baseUrl)
    override suspend fun setAutoMode(baseUrl: String) = apiClient.setAutoMode(baseUrl)
    override suspend fun manualMove(
        baseUrl: String,
        direction: ManualDirection,
        speed: Int,
        durationMs: Long,
    ) = apiClient.manualMove(baseUrl, direction, speed, durationMs)
}
