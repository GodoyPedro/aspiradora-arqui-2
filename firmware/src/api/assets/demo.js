const room = document.getElementById("room");
const robotEl = document.getElementById("robot");
const headingEl = document.getElementById("heading");
const sensorConeEl = document.getElementById("sensor-cone");
const bumperRingEl = document.getElementById("bumper-ring");
const dockEl = document.getElementById("dock");
const obstacleLayerEl = document.getElementById("obstacle-layer");
const coverageCanvas = document.getElementById("coverage-layer");
const coverageCtx = coverageCanvas.getContext("2d");
const errorBannerEl = document.getElementById("error-banner");
const dumpBannerEl = document.getElementById("dump-banner");
const loopIndicatorEl = document.getElementById("loop-indicator");
const statusJsonEl = document.getElementById("status-json");
const sensorListEl = document.getElementById("sensor-flags");
const speedSelectEl = document.getElementById("speed-select");
const normalObstacleCountEl = document.getElementById("normal-obstacle-count");
const sessionIdEl = document.getElementById("session-id");
const mapIdEl = document.getElementById("map-id");
const logPathEl = document.getElementById("log-path");
const batchModeIndicatorEl = document.getElementById("batch-mode-indicator");
const batchInputs = {
  runCount: document.getElementById("batch-run-count"),
  maxTimeMs: document.getElementById("batch-max-time-ms"),
  targetCoverage: document.getElementById("batch-target-coverage"),
  speed: document.getElementById("batch-speed-select"),
  obstacleCount: document.getElementById("batch-obstacle-count"),
  returnToDock: document.getElementById("batch-return-to-dock"),
  stopTime: document.getElementById("batch-stop-time"),
  stopCoverage: document.getElementById("batch-stop-coverage"),
  stopDocked: document.getElementById("batch-stop-docked"),
  stopStuck: document.getElementById("batch-stop-stuck"),
};
const batchFields = {
  batchId: document.getElementById("batch-id"),
  runProgress: document.getElementById("batch-run-progress"),
  sessionId: document.getElementById("batch-session-id"),
  elapsedMs: document.getElementById("batch-elapsed-ms"),
  coverage: document.getElementById("batch-coverage"),
  runState: document.getElementById("batch-run-state"),
  finishCondition: document.getElementById("batch-finish-condition"),
  completedCount: document.getElementById("batch-completed-count"),
  failedCount: document.getElementById("batch-failed-count"),
  lastLogPath: document.getElementById("batch-last-log-path"),
  summaryPath: document.getElementById("batch-summary-path"),
};
const manualControlEls = Array.from(document.querySelectorAll(".manual-control"));

const fields = {
  state: document.getElementById("state-value"),
  mode: document.getElementById("mode-value"),
  battery: document.getElementById("battery-value"),
  charging: document.getElementById("charging-value"),
  suction: document.getElementById("suction-value"),
  brushes: document.getElementById("brushes-value"),
  leftWheel: document.getElementById("left-wheel-value"),
  rightWheel: document.getElementById("right-wheel-value"),
  error: document.getElementById("error-value"),
  coverage: document.getElementById("coverage-value"),
};

const ROOM = { width: 860, height: 560 };
const ROBOT_RADIUS = 29;
const BASE_VISUAL_SPEED_SCALE = 0.9;
const TURN_SCALE = 0.012;
const PROXIMITY_DETECTION_RANGE_PX = 28;
const PROXIMITY_CONTACT_THRESHOLD_PX = 5;
const BODY_CLEARANCE_LOOKAHEAD_PX = Math.round(ROBOT_RADIUS * 1.25);
const BODY_CLEARANCE_MARGIN_PX = 2;
const BODY_CLEARANCE_SAMPLE_COUNT = 5;
const SENSOR_HALF_ANGLE = Math.PI / 12;
const SENSOR_RAY_COUNT = 3;
const SENSOR_STEP = 7;
const LOOP_INTERVAL_MS = 180;
const MANUAL_DURATION_MS = 2000;
const MANUAL_SPEED = 45;
const COVERAGE_GRID = { cols: 40, rows: 28 };
const COVERAGE_TRAIL_RADIUS_SCALE = 0.34;
const COVERAGE_TRAIL_ALPHA = 0.12;
const COVERAGE_TRAIL_CORE_ALPHA = 0.18;
const COVERAGE_TRAIL_BLUR_SCALE = 0.62;
const MAX_LINEAR_SUBSTEP_PX = 4;
const MAX_VISUAL_SUBSTEPS_PER_FRAME = 96;
const MAX_ANGULAR_SUBSTEP_RAD = 0.06;
const FORWARD_FREEZE_DIAGNOSTIC_THRESHOLD = 3;
const LONG_TURN_GUARD_MS = 2600;
const ESCAPE_TURN_MS = 540;
const ESCAPE_FORWARD_ATTEMPT_MS = 650;
const DOCK_ALIGN_ERROR_RAD = 0.24;
const DOCK_TARGET_ERROR_RAD = 0.75;
const DOCK_STOP_DISTANCE = 18;
const DOCK_NEAR_DISTANCE = 90;
const DOCK_ROUTE_X_OFFSET = 120;
const DOCK_ROUTE_BOTTOM_OFFSET = 32;
const DOCK_ROUTE_WALL_CLEARANCE = 80;
const DOCK_ROUTE_SAMPLE_STEP = 8;
const DOCK_ROUTE_ALIGNMENT_TOLERANCE = 36;
const DOCK_FINAL_APPROACH_DISTANCE = 170;
const DOCK_RECOVERY_REVERSE_MS = 520;
const DOCK_RECOVERY_ARC_MS = 720;
const DOCK_STUCK_DIAGNOSTIC_MS = 3000;
const DOCK_PROGRESS_DISTANCE_EPSILON = 4;
const DOCK_PROGRESS_HEADING_EPSILON_RAD = 0.08;
const BATCH_NO_MOVEMENT_LIMIT_MS = 4200;
const BATCH_MAX_LONG_TURN_GUARDS = 4;
const BATCH_MAX_DOCK_BLOCKED_EVENTS = 8;
const BATCH_MAX_DOCK_STUCK_EVENTS = 3;
const BATCH_MAX_WORST_RUNS = 3;
const ROOM_CROSSING_SPEED = 36;
const WALL_FOLLOW_SPEED = 30;
const OBSTACLE_ESCAPE_FORWARD_SPEED = 34;
const ANTI_LOOP_FORWARD_SPEED = 38;
const TURNING_SPEED = 20;
const HEADING_TOLERANCE_RAD = 0.14;
const MIN_TURN_SPEED = 12;
const MAX_TURN_SPEED = 24;
const BACKUP_DISTANCE_PX = 28;
const BACKUP_SPEED = -24;
const MAX_BACKUP_MS = 1000;
const ROOM_CROSSING_MIN_MS = 1600;
const ROOM_CROSSING_COMMIT_MS = 6000;
const ROOM_CROSSING_COMMIT_DISTANCE_PX = 160;
const RECENT_CROSSING_HEADING_LIMIT = 5;
const HEADING_REPEAT_THRESHOLD_RAD = (Math.PI * 25) / 180;
const NORMAL_DIVERSE_HEADING_OFFSET_RAD = (Math.PI * 70) / 180;
const GOLDEN_ANGLE_RAD = (Math.PI * 137.5) / 180;
const WALL_RELEASE_MIN_ANGLE_RAD = Math.PI / 6;
const WALL_RELEASE_MAX_ANGLE_RAD = Math.PI / 3;
const OBSTACLE_TURN_MIN_ANGLE_RAD = Math.PI / 4;
const OBSTACLE_TURN_MAX_ANGLE_RAD = (Math.PI * 100) / 180;
const OBSTACLE_ESCAPE_DISTANCE_PX = 78;
const OBSTACLE_ESCAPE_MAX_MS = 1200;
const ANTI_LOOP_CONTACT_WINDOW_MS = 5000;
const ANTI_LOOP_CONTACT_THRESHOLD = 6;
const ANTI_LOOP_STALL_MS = 4200;
const ANTI_LOOP_TURN_ANGLE_RAD = (Math.PI * 140) / 180;
const ANTI_LOOP_FORWARD_DISTANCE_PX = 112;
const ANTI_LOOP_BACKUP_DISTANCE_PX = 40;
const ANTI_LOOP_IGNORE_PROXIMITY_MS = 650;
const BATCH_RECOVERY_GRACE_MS = 900;

const pose = {
  x: 120,
  y: 120,
  heading: 0,
  lastTimestamp: null,
};

let latestStatus = null;
let isTicking = false;
let latestCoveragePercentage = 0;
let frameIndex = 0;
let mapCounter = 0;
let sessionCounter = 0;
let currentMap = null;
let currentSession = null;
let coveredCellKeys = new Set();
let lastMotionKind = "idle";
let lastTurnDirection = null;
let localProximityContact = false;
let localProximityContactDetail = null;
let lastCollisionSignature = null;
let waitingForTurnAfterBumper = false;
let waitingDiagnosticLogged = false;
let silentForwardFreezeFrames = 0;
let lastRecordedTimestampMs = 0;
let turnEpisodeCounter = 0;
let activeTurnEpisodeId = null;
let turnEpisodeStartedAtMs = null;
let queuedTimelineEvents = [];
let escapePlan = null;
let dockRecoveryPlan = null;
let dockRoutePlan = null;
let dockRecoveryDirectionBias = 1;
let lastBackendState = null;
let lastDockingPhase = null;
let lastDockProgressDistance = null;
let lastDockProgressTimestamp = null;
let lastDockHeadingError = null;
let batchState = null;
let laneController = null;
let lastVisualCommandTelemetry = null;
let lastLoopRealTimestamp = null;
let sessionStartedRealMs = null;
let lastBackendAutoNavigationPhase = null;

function setError(message) {
  errorBannerEl.textContent = message || "";
}

function setDumpMessage(message) {
  dumpBannerEl.textContent = message || "";
}

function setBatchModeIndicator(text) {
  batchModeIndicatorEl.textContent = text;
}

function setLoopIndicator(text, busy = false) {
  loopIndicatorEl.textContent = text;
  loopIndicatorEl.style.color = busy ? "#b05f1a" : "#2d7d46";
}

async function requestJson(url, options = {}) {
  const response = await fetch(url, {
    headers: { "Content-Type": "application/json" },
    ...options,
  });

  let body = null;
  const contentType = response.headers.get("content-type") || "";
  if (contentType.includes("application/json")) {
    body = await response.json();
  } else {
    body = await response.text();
  }

  if (!response.ok) {
    const message = typeof body === "object" && body !== null ? body.message : `HTTP ${response.status}`;
    throw new Error(message || `HTTP ${response.status}`);
  }

  return body;
}

function normalizeAngle(angle) {
  while (angle > Math.PI) angle -= Math.PI * 2;
  while (angle < -Math.PI) angle += Math.PI * 2;
  return angle;
}

function angleError(target, current) {
  return normalizeAngle(target - current);
}

function crossedTargetHeading(errorBefore, errorAfter) {
  return (
    Math.sign(errorBefore) !== Math.sign(errorAfter) &&
    Math.abs(errorBefore) > HEADING_TOLERANCE_RAD &&
    Math.abs(errorAfter) > HEADING_TOLERANCE_RAD
  );
}

function clamp(value, min, max) {
  return Math.max(min, Math.min(max, value));
}

function lastTimelineTimestamp() {
  if (!currentSession?.timeline?.length) {
    return 0;
  }

  const lastFrame = currentSession.timeline[currentSession.timeline.length - 1];
  const timestamp = Number(lastFrame?.timestamp_ms ?? 0);
  return Number.isFinite(timestamp) ? timestamp : 0;
}

function nextTimelineTimestamp(stepMs = 1) {
  const roundedStep = Math.max(1, Math.round(stepMs));
  const baseTimestamp = Math.max(lastRecordedTimestampMs || 0, lastTimelineTimestamp());
  lastRecordedTimestampMs = baseTimestamp + roundedStep;
  return lastRecordedTimestampMs;
}

function queueTimelineEvent(label, detail = null) {
  queuedTimelineEvents.push({ label, detail });
}

function consumeQueuedTimelineEvent() {
  return queuedTimelineEvents.shift() ?? null;
}

function speedFactor() {
  if (batchState?.activeRun) {
    return Number(batchState.config.simulation_speed);
  }
  return Number.parseFloat(speedSelectEl.value || "1");
}

function normalObstacleCount() {
  const value = Number.parseInt(normalObstacleCountEl?.value ?? "4", 10);
  return clamp(Number.isFinite(value) ? value : 4, 1, 12);
}

function setManualControlsDisabled(disabled) {
  manualControlEls.forEach((element) => {
    element.disabled = disabled;
  });
}

function cellSize() {
  return {
    width: ROOM.width / COVERAGE_GRID.cols,
    height: ROOM.height / COVERAGE_GRID.rows,
  };
}

function createSessionId(batchContext = null) {
  sessionCounter += 1;
  if (batchContext) {
    return `${batchContext.batchId}-run-${String(batchContext.runIndex).padStart(3, "0")}`;
  }
  const stamp = new Date().toISOString().replace(/[-:.TZ]/g, "").slice(0, 14);
  return `demo-map-${stamp}-${String(sessionCounter).padStart(2, "0")}`;
}

function createBatchId() {
  const stamp = new Date().toISOString().replace(/[-:.TZ]/g, "").slice(0, 14);
  return `batch-${stamp}-${String(sessionCounter + 1).padStart(3, "0")}`;
}

function createMapId() {
  mapCounter += 1;
  return `map-${String(mapCounter).padStart(3, "0")}`;
}

function lcg(seed) {
  let value = seed >>> 0;
  return () => {
    value = (value * 1664525 + 1013904223) >>> 0;
    return value / 0xffffffff;
  };
}

function circleIntersectsRect(circle, rect, radius = ROBOT_RADIUS) {
  const nearestX = Math.max(rect.x, Math.min(circle.x, rect.x + rect.width));
  const nearestY = Math.max(rect.y, Math.min(circle.y, rect.y + rect.height));
  const dx = circle.x - nearestX;
  const dy = circle.y - nearestY;
  return dx * dx + dy * dy <= radius * radius;
}

function wallSideForCircle(circle, radius = ROBOT_RADIUS) {
  const overlaps = [
    ["left", circle.x - radius],
    ["right", ROOM.width - circle.x - radius],
    ["top", circle.y - radius],
    ["bottom", ROOM.height - circle.y - radius],
  ].filter(([, distance]) => distance < 0);

  if (overlaps.length === 0) {
    return null;
  }

  return overlaps.sort((left, right) => left[1] - right[1])[0][0];
}

function wallCollision(circle, radius = ROBOT_RADIUS) {
  const wallSide = wallSideForCircle(circle, radius);
  if (!wallSide) {
    return null;
  }

  return {
    collision_target: "wall",
    wall_side: wallSide,
  };
}

function detectCollisionAtPose(candidatePose, radius = ROBOT_RADIUS) {
  const circle = { x: candidatePose.x, y: candidatePose.y };
  const wall = wallCollision(circle, radius);
  if (wall) {
    return wall;
  }

  const obstacle = currentMap.obstacles.find((rect) => circleIntersectsRect(circle, rect, radius));
  if (obstacle) {
    return {
      collision_target: "obstacle",
      obstacle_id: obstacle.id,
    };
  }

  return null;
}

function detectForwardProximity(sourcePose = pose, maxDistance = PROXIMITY_DETECTION_RANGE_PX) {
  let closestDetection = null;

  for (let rayIndex = 0; rayIndex < SENSOR_RAY_COUNT; rayIndex += 1) {
    const t = SENSOR_RAY_COUNT === 1 ? 0 : rayIndex / (SENSOR_RAY_COUNT - 1);
    const angleOffset = -SENSOR_HALF_ANGLE + t * SENSOR_HALF_ANGLE * 2;

    for (let distance = 0; distance <= maxDistance; distance += SENSOR_STEP) {
      const probe = projectForward(ROBOT_RADIUS + distance, angleOffset, sourcePose);
      let detection = null;
      if (probe.x <= 2 || probe.y <= 2 || probe.x >= ROOM.width - 2 || probe.y >= ROOM.height - 2) {
        detection = {
          collision_target: "wall",
          obstacle_id: null,
          wall_side: probe.x <= 2 ? "left" : probe.x >= ROOM.width - 2 ? "right" : probe.y <= 2 ? "top" : "bottom",
          distance_px: distance,
        };
      } else {
        const obstacle = currentMap.obstacles.find((rect) => pointInsideRect(probe, rect));
        if (obstacle) {
          detection = {
            collision_target: "obstacle",
            obstacle_id: obstacle.id,
            distance_px: distance,
          };
        }
      }

      if (detection && (!closestDetection || detection.distance_px < closestDetection.distance_px)) {
        closestDetection = detection;
        break;
      }
    }
  }

  return closestDetection;
}

function pointInsideRect(point, rect) {
  return point.x >= rect.x && point.x <= rect.x + rect.width && point.y >= rect.y && point.y <= rect.y + rect.height;
}

function projectForward(distance, angleOffset = 0, sourcePose = pose) {
  const heading = sourcePose.heading + angleOffset;
  return {
    x: sourcePose.x + Math.cos(heading) * distance,
    y: sourcePose.y + Math.sin(heading) * distance,
  };
}

function sampleForwardObstacle(sourcePose = pose) {
  return Boolean(detectForwardProximity(sourcePose));
}

function forwardClearanceBlockedByBody(sourcePose = pose) {
  for (let sampleIndex = 1; sampleIndex <= BODY_CLEARANCE_SAMPLE_COUNT; sampleIndex += 1) {
    const distance = (BODY_CLEARANCE_LOOKAHEAD_PX * sampleIndex) / BODY_CLEARANCE_SAMPLE_COUNT;
    const candidatePose = {
      x: sourcePose.x + Math.cos(sourcePose.heading) * distance,
      y: sourcePose.y + Math.sin(sourcePose.heading) * distance,
      heading: sourcePose.heading,
    };

    const collision = detectCollisionAtPose(candidatePose, ROBOT_RADIUS + BODY_CLEARANCE_MARGIN_PX);
    if (collision) {
      return true;
    }
  }

  return false;
}

function generateObstacleMap(seed, requestedCount = 4) {
  const random = lcg(seed);
  const startCircle = { x: 120, y: 120 };
  const dockRect = { x: 24, y: ROOM.height - 50, width: 120, height: 26 };
  const obstacles = [];
  const obstacleCount = clamp(Number(requestedCount) || 4, 1, 12);

  for (let index = 0; index < obstacleCount; index += 1) {
    let obstacle = null;
    for (let attempt = 0; attempt < 40; attempt += 1) {
      const width = 80 + Math.round(random() * 70);
      const height = 70 + Math.round(random() * 90);
      const x = 110 + Math.round(random() * (ROOM.width - width - 150));
      const y = 40 + Math.round(random() * (ROOM.height - height - 80));
      const candidate = { id: `obstacle-${index + 1}`, x, y, width, height };

      const overlapsStart = circleIntersectsRect(startCircle, candidate, ROBOT_RADIUS + 28);
      const overlapsDock = !(
        candidate.x + candidate.width < dockRect.x - 24 ||
        candidate.x > dockRect.x + dockRect.width + 24 ||
        candidate.y + candidate.height < dockRect.y - 24 ||
        candidate.y > dockRect.y + dockRect.height + 24
      );
      const overlapsObstacle = obstacles.some((existing) => {
        return !(
          candidate.x + candidate.width + 18 < existing.x ||
          candidate.x > existing.x + existing.width + 18 ||
          candidate.y + candidate.height + 18 < existing.y ||
          candidate.y > existing.y + existing.height + 18
        );
      });

      if (!overlapsStart && !overlapsDock && !overlapsObstacle) {
        obstacle = candidate;
        break;
      }
    }

    if (obstacle) {
      obstacles.push(obstacle);
    }
  }

  return obstacles;
}

function createMap(seed = Date.now(), options = {}) {
  const mapId = createMapId();
  const obstacleCount = Number.isFinite(options.obstacleCount)
    ? Number(options.obstacleCount)
    : normalObstacleCount();
  return {
    mapId,
    roomWidth: ROOM.width,
    roomHeight: ROOM.height,
    robotRadius: ROBOT_RADIUS,
    dockingStation: { x: 84, y: ROOM.height - 37 },
    initialPose: { x: 120, y: 120, heading: 0 },
    obstacles: generateObstacleMap(seed, obstacleCount),
  };
}

function createSessionForMap(map, batchContext = null) {
  const sessionId = createSessionId(batchContext);
  const targetPath = batchContext
    ? `demo-logs/${batchContext.batchId}/${sessionId}/log.json`
    : `demo-logs/${sessionId}/log.json`;
  return {
    sessionId,
    mapId: map.mapId,
    createdAt: new Date().toISOString(),
    targetPath,
    batchId: batchContext?.batchId ?? null,
    runIndex: batchContext?.runIndex ?? null,
    totalRuns: batchContext?.totalRuns ?? null,
    runConfig: batchContext?.runConfig ?? null,
    timeline: [],
  };
}

function createLaneController() {
  return {
    phase: "ROOM_CROSSING",
    phaseStartedAtMs: 0,
    phaseStartPose: { x: pose.x, y: pose.y, heading: pose.heading },
    phaseTargetHeading: null,
    contactType: null,
    obstacleId: null,
    wallSide: null,
    escapeSide: "right",
    recoveryAttempts: 0,
    backupBlocked: false,
    wallFollowUntilMs: 0,
    ignoreProximityUntilMs: 0,
    recentContactTimes: [],
    repeatedObstacleId: null,
    repeatedObstacleHits: 0,
    lastProgressMs: 0,
    lastProgressCoverage: 0,
    coverageStalledSinceMs: 0,
    noMovementSinceMs: 0,
    roomCrossingSegments: 0,
    antiLoopEscapes: 0,
    recentCrossingHeadings: [],
    committedUntilMs: 0,
    committedMinDistancePx: 0,
  };
}

function resetLaneController() {
  laneController = createLaneController();
}

function isCleaningAuto(status) {
  return status?.state === "CLEANING" && status?.cleaning_mode === "AUTO";
}

function currentRealElapsedMs() {
  if (batchState?.activeRun?.startedRealMs != null) {
    return Math.round(performance.now() - batchState.activeRun.startedRealMs);
  }
  if (sessionStartedRealMs != null) {
    return Math.round(performance.now() - sessionStartedRealMs);
  }
  return null;
}

function phaseDetail(extra = {}) {
  if (!laneController) {
    return Object.keys(extra).length > 0 ? extra : null;
  }
  return {
    demo_navigation_phase: laneController.phase,
    heading: Number(normalizeAngle(pose.heading).toFixed(6)),
    target_heading:
      laneController.phaseTargetHeading == null ? null : Number(normalizeAngle(laneController.phaseTargetHeading).toFixed(6)),
    contact_type: laneController.contactType,
    obstacle_id: laneController.obstacleId,
    wall_side: laneController.wallSide,
    escape_side: laneController.escapeSide,
    recovery_attempts: laneController.recoveryAttempts ?? 0,
    backup_distance_px: Number(distanceFromPhaseStart().toFixed(3)),
    heading_before:
      laneController.phaseStartPose?.heading == null ? null : Number(normalizeAngle(laneController.phaseStartPose.heading).toFixed(6)),
    heading_after: Number(normalizeAngle(pose.heading).toFixed(6)),
    backup_blocked: laneController.backupBlocked ?? false,
    real_time_ms: currentRealElapsedMs(),
    simulated_time_ms: currentBatchRunTimestampMs(),
    ...extra,
  };
}

function setLanePhase(nextPhase, timestampMs, extra = {}) {
  if (!laneController) {
    resetLaneController();
  }

  const has = (key) => Object.prototype.hasOwnProperty.call(extra, key);

  laneController.phase = nextPhase;
  laneController.phaseStartedAtMs = timestampMs;
  laneController.phaseStartPose = { x: pose.x, y: pose.y, heading: pose.heading };

  laneController.phaseTargetHeading = has("phaseTargetHeading")
    ? extra.phaseTargetHeading == null
      ? null
      : normalizeAngle(extra.phaseTargetHeading)
    : null;

  laneController.contactType = has("contactType")
    ? extra.contactType
    : laneController.contactType ?? null;

  laneController.obstacleId = has("obstacleId")
    ? extra.obstacleId
    : laneController.obstacleId ?? null;

  laneController.wallSide = has("wallSide")
    ? extra.wallSide
    : laneController.wallSide ?? null;

  if (has("escapeSide")) {
    laneController.escapeSide = extra.escapeSide;
  }

  if (has("recoveryAttempts")) {
    laneController.recoveryAttempts = extra.recoveryAttempts;
  }

  if (has("backupBlocked")) {
    laneController.backupBlocked = extra.backupBlocked;
  }

  if (has("wallFollowUntilMs")) {
    laneController.wallFollowUntilMs = extra.wallFollowUntilMs;
  }

  if (has("ignoreProximityUntilMs")) {
    laneController.ignoreProximityUntilMs = extra.ignoreProximityUntilMs;
  }

  if (has("committedUntilMs")) {
    laneController.committedUntilMs = extra.committedUntilMs;
  }

  if (has("committedMinDistancePx")) {
    laneController.committedMinDistancePx = extra.committedMinDistancePx;
  }

  laneController.lastProgressMs = timestampMs;
  laneController.lastProgressCoverage = latestCoveragePercentage;
  laneController.coverageStalledSinceMs = timestampMs;
  laneController.noMovementSinceMs = timestampMs;
}

function queueLaneEvent(label, extra = {}) {
  queueTimelineEvent(label, phaseDetail(extra));
}

function shouldUseLaneController(status) {
  return false;
}

function recoverySignature(detail = null) {
  if (!detail) return null;
  return JSON.stringify({
    phase: detail.demo_navigation_phase ?? null,
    recovery_attempts: detail.recovery_attempts ?? null,
    escape_side: detail.escape_side ?? null,
    wall_side: detail.wall_side ?? null,
  });
}

function backupTimedOut(timestampMs) {
  return timestampMs - (laneController?.phaseStartedAtMs ?? 0) >= MAX_BACKUP_MS;
}

function backupComplete(timestampMs) {
  return distanceFromPhaseStart() >= BACKUP_DISTANCE_PX || backupTimedOut(timestampMs);
}

function roomCenter() {
  return { x: ROOM.width / 2, y: ROOM.height / 2 };
}

function obstacleById(obstacleId) {
  return currentMap.obstacles.find((obstacle) => obstacle.id === obstacleId) ?? null;
}

function chooseAlternateEscapeSide(side = laneController?.escapeSide ?? "right") {
  return side === "left" ? "right" : "left";
}

function nearestWallSide() {
  const distances = {
    left: pose.x - ROBOT_RADIUS,
    right: ROOM.width - pose.x - ROBOT_RADIUS,
    top: pose.y - ROBOT_RADIUS,
    bottom: ROOM.height - pose.y - ROBOT_RADIUS,
  };
  return Object.entries(distances).sort((left, right) => left[1] - right[1])[0]?.[0] ?? "wall";
}

function normalizeContactType(collisionTarget) {
  return collisionTarget === "wall" ? "wall" : collisionTarget === "obstacle" ? "obstacle" : "unknown";
}

function classifyNavigationContact(moveResult) {
  const detail = moveResult?.collisionDetail ?? moveResult?.proximityContactDetail ?? null;
  const contactType = normalizeContactType(detail?.collision_target);
  return {
    detail,
    contactType: contactType === "unknown" ? "obstacle" : contactType,
    obstacleId: detail?.obstacle_id ?? null,
    wallSide: detail?.collision_target === "wall" ? nearestWallSide() : null,
    distancePx: detail?.distance_px ?? null,
  };
}

function chooseWallFollowHeading(wallSide, escapeSide = laneController?.escapeSide ?? "right") {
  if (wallSide === "top" || wallSide === "bottom") {
    return escapeSide === "left" ? Math.PI : 0;
  }
  return escapeSide === "left" ? -Math.PI / 2 : Math.PI / 2;
}

function chooseWallReleaseHeading(baseHeading) {
  const centerHeading = Math.atan2(roomCenter().y - pose.y, roomCenter().x - pose.x);
  const delta = angleError(centerHeading, baseHeading);
  const magnitude = clamp(Math.abs(delta), WALL_RELEASE_MIN_ANGLE_RAD, WALL_RELEASE_MAX_ANGLE_RAD);
  return normalizeAngle(baseHeading + Math.sign(delta || 1) * magnitude);
}

function angularDistance(a, b) {
  return Math.abs(angleError(a, b));
}

function rememberCrossingHeading(heading) {
  if (!laneController) {
    return;
  }

  laneController.recentCrossingHeadings.push(normalizeAngle(heading));
  if (laneController.recentCrossingHeadings.length > RECENT_CROSSING_HEADING_LIMIT) {
    laneController.recentCrossingHeadings = laneController.recentCrossingHeadings.slice(
      laneController.recentCrossingHeadings.length - RECENT_CROSSING_HEADING_LIMIT,
    );
  }
}

function chooseDiverseCrossingHeading(candidateHeading, options = {}) {
  const originalHeading = normalizeAngle(candidateHeading);
  const recentHeadings = laneController?.recentCrossingHeadings ?? [];
  const isRepeated = recentHeadings.some(
    (recentHeading) => angularDistance(originalHeading, recentHeading) < HEADING_REPEAT_THRESHOLD_RAD,
  );

  if (!isRepeated && !options.forceGoldenAngle) {
    return {
      heading: originalHeading,
      originalHeading,
      diverseHeadingApplied: false,
      recentHeadings: [...recentHeadings],
    };
  }

  const direction = laneController?.escapeSide === "left" ? -1 : 1;
  const offset = options.forceGoldenAngle ? GOLDEN_ANGLE_RAD : NORMAL_DIVERSE_HEADING_OFFSET_RAD * direction;
  let adjustedHeading = normalizeAngle(originalHeading + offset);

  const stillRepeated = recentHeadings.some(
    (recentHeading) => angularDistance(adjustedHeading, recentHeading) < HEADING_REPEAT_THRESHOLD_RAD,
  );

  if (stillRepeated) {
    adjustedHeading = normalizeAngle(originalHeading + GOLDEN_ANGLE_RAD);
  }

  return {
    heading: adjustedHeading,
    originalHeading,
    diverseHeadingApplied: true,
    recentHeadings: [...recentHeadings],
  };
}

function chooseObstacleTurnHeading(obstacleId, escapeSide = laneController?.escapeSide ?? "right") {
  const obstacle = obstacleById(obstacleId);
  if (obstacle) {
    const centerX = obstacle.x + obstacle.width / 2;
    const centerY = obstacle.y + obstacle.height / 2;
    const awayHeading = Math.atan2(pose.y - centerY, pose.x - centerX);
    const delta = angleError(awayHeading, pose.heading);
    const sign = Math.sign(delta || (escapeSide === "left" ? -1 : 1));
    const magnitude = clamp(Math.abs(delta), OBSTACLE_TURN_MIN_ANGLE_RAD, OBSTACLE_TURN_MAX_ANGLE_RAD);
    return normalizeAngle(pose.heading + sign * magnitude);
  }

  return normalizeAngle(
    pose.heading + (escapeSide === "left" ? -OBSTACLE_TURN_MIN_ANGLE_RAD * 1.3 : OBSTACLE_TURN_MIN_ANGLE_RAD * 1.3),
  );
}

function pruneRecentContacts(timestampMs) {
  laneController.recentContactTimes = laneController.recentContactTimes.filter(
    (value) => timestampMs - value <= ANTI_LOOP_CONTACT_WINDOW_MS,
  );
}

function registerContact(timestampMs, contact) {
  pruneRecentContacts(timestampMs);
  laneController.recentContactTimes.push(timestampMs);
  if (contact.obstacleId && contact.obstacleId === laneController.repeatedObstacleId) {
    laneController.repeatedObstacleHits += 1;
  } else {
    laneController.repeatedObstacleId = contact.obstacleId ?? null;
    laneController.repeatedObstacleHits = contact.obstacleId ? 1 : 0;
  }
}

function beginRoomCrossing(timestampMs, options = {}) {
  const rawHeading = normalizeAngle(options.targetHeading ?? pose.heading);
  const shouldDiversify = options.diversify !== false;
  const headingChoice = shouldDiversify
    ? chooseDiverseCrossingHeading(rawHeading, { forceGoldenAngle: options.forceGoldenAngle === true })
    : {
        heading: rawHeading,
        originalHeading: rawHeading,
        diverseHeadingApplied: false,
        recentHeadings: [...(laneController?.recentCrossingHeadings ?? [])],
      };

  const phase = options.committed ? "ROOM_CROSSING_COMMITTED" : "ROOM_CROSSING";

  setLanePhase(phase, timestampMs, {
    phaseTargetHeading: headingChoice.heading,
    contactType: null,
    obstacleId: null,
    wallSide: null,
    recoveryAttempts: options.recoveryAttempts ?? 0,
    backupBlocked: false,
    ignoreProximityUntilMs: 0,
    committedUntilMs: options.committed ? timestampMs + ROOM_CROSSING_COMMIT_MS : 0,
    committedMinDistancePx: options.committed ? ROOM_CROSSING_COMMIT_DISTANCE_PX : 0,
  });

  rememberCrossingHeading(headingChoice.heading);
  laneController.roomCrossingSegments += 1;

  queueLaneEvent(options.committed ? "ROOM_CROSSING_COMMITTED_START" : "ROOM_CROSSING_START", {
    selected_heading: Number(normalizeAngle(headingChoice.heading).toFixed(6)),
    original_candidate_heading: Number(normalizeAngle(headingChoice.originalHeading).toFixed(6)),
    diverse_heading_applied: headingChoice.diverseHeadingApplied,
    recent_crossing_headings: headingChoice.recentHeadings.map((value) => Number(normalizeAngle(value).toFixed(6))),
    committed_until_ms: options.committed ? timestampMs + ROOM_CROSSING_COMMIT_MS : null,
    committed_min_distance_px: options.committed ? ROOM_CROSSING_COMMIT_DISTANCE_PX : null,
  });
}

function queueProximityContactEvent(contact, extra = {}) {
  queueLaneEvent("PROXIMITY_CONTACT", {
    collision_target: contact.contactType,
    contact_type: contact.contactType,
    obstacle_id: contact.obstacleId ?? null,
    wall_side: contact.wallSide ?? null,
    distance_px: contact.distancePx ?? null,
    contact_sensor_type: "proximity",
    physical_bumper: false,
    proximity_contact: true,
    compatibility_event: "BUMPER_CONTACT",
    ...extra,
  });
}

function beginWallContact(timestampMs, contact) {
  registerContact(timestampMs, contact);
  const escapeSide = chooseAlternateEscapeSide();
  setLanePhase("CONTACT_BACKUP", timestampMs, {
    contactType: "wall",
    obstacleId: null,
    wallSide: contact.wallSide ?? nearestWallSide(),
    escapeSide,
    recoveryAttempts: 0,
    backupBlocked: false,
  });
  queueProximityContactEvent(contact);
  queueLaneEvent("WALL_CONTACT", {
    contact_type: "wall",
    collision_target: "wall",
    distance_px: contact.distancePx,
  });
}

function beginObstacleContact(timestampMs, contact, attempts = 0) {
  registerContact(timestampMs, contact);
  const escapeSide =
    contact.obstacleId && contact.obstacleId === laneController.repeatedObstacleId && laneController.repeatedObstacleHits > 1
      ? chooseAlternateEscapeSide()
      : laneController.escapeSide ?? "right";
  setLanePhase("OBSTACLE_BACKUP", timestampMs, {
    contactType: "obstacle",
    obstacleId: contact.obstacleId,
    wallSide: null,
    escapeSide,
    recoveryAttempts: attempts,
    backupBlocked: false,
  });
  queueProximityContactEvent(contact);
  queueLaneEvent("OBSTACLE_CONTACT", {
    contact_type: "obstacle",
    collision_target: "obstacle",
    obstacle_id: contact.obstacleId,
    distance_px: contact.distancePx,
  });
  queueLaneEvent("OBSTACLE_BACKUP");
}

function beginAntiLoopEscape(timestampMs, reason, extra = {}) {
  laneController.antiLoopEscapes += 1;
  setLanePhase("ANTI_LOOP_ESCAPE", timestampMs, {
    contactType: extra.contactType ?? laneController.contactType ?? "obstacle",
    obstacleId: extra.obstacleId ?? laneController.obstacleId ?? null,
    wallSide: extra.wallSide ?? laneController.wallSide ?? null,
    escapeSide: chooseAlternateEscapeSide(),
    recoveryAttempts: (laneController.recoveryAttempts ?? 0) + 1,
    backupBlocked: false,
  });
  queueLaneEvent("ANTI_LOOP_ESCAPE", {
    reason,
    contact_type: laneController.contactType ?? extra.contactType ?? null,
    obstacle_id: laneController.obstacleId ?? extra.obstacleId ?? null,
    wall_side: laneController.wallSide ?? extra.wallSide ?? null,
  });
  setLanePhase("ANTI_LOOP_BACKUP", timestampMs, {
    contactType: extra.contactType ?? laneController.contactType ?? "obstacle",
    obstacleId: extra.obstacleId ?? laneController.obstacleId ?? null,
    wallSide: extra.wallSide ?? laneController.wallSide ?? null,
    escapeSide: laneController.escapeSide,
    recoveryAttempts: laneController.recoveryAttempts,
    backupBlocked: false,
  });
  queueLaneEvent("ANTI_LOOP_BACKUP");
}

function renderMap() {
  obstacleLayerEl.innerHTML = "";
  currentMap.obstacles.forEach((obstacle) => {
    const el = document.createElement("div");
    el.className = "obstacle";
    el.style.left = `${obstacle.x}px`;
    el.style.top = `${obstacle.y}px`;
    el.style.width = `${obstacle.width}px`;
    el.style.height = `${obstacle.height}px`;
    obstacleLayerEl.appendChild(el);
  });

  const dockWidth = 120;
  const dockHeight = 26;
  dockEl.style.left = `${currentMap.dockingStation.x - dockWidth / 2}px`;
  dockEl.style.top = `${currentMap.dockingStation.y - dockHeight / 2}px`;
}

function renderSessionMeta() {
  sessionIdEl.textContent = currentSession.sessionId;
  mapIdEl.textContent = currentSession.mapId;
  logPathEl.textContent = currentSession.targetPath;
  if (batchState?.activeRun) {
    batchFields.sessionId.textContent = currentSession.sessionId;
  }
}

function updateBatchUi() {
  if (!batchState) {
    setBatchModeIndicator("Idle");
    batchFields.batchId.textContent = "-";
    batchFields.runProgress.textContent = "0 / 0";
    batchFields.sessionId.textContent = "-";
    batchFields.elapsedMs.textContent = "0 ms";
    batchFields.coverage.textContent = "0.0%";
    batchFields.runState.textContent = "Idle";
    batchFields.finishCondition.textContent = "-";
    batchFields.completedCount.textContent = "0";
    batchFields.failedCount.textContent = "0";
    batchFields.lastLogPath.textContent = "-";
    batchFields.summaryPath.textContent = "-";
    return;
  }

  const activeRun = batchState.activeRun;
  setBatchModeIndicator(activeRun ? "Running" : batchState.completed ? "Complete" : batchState.cancelled ? "Cancelled" : "Ready");
  batchFields.batchId.textContent = batchState.batchId;
  batchFields.runProgress.textContent = `${activeRun?.runIndex ?? batchState.completedRuns} / ${batchState.requestedRuns}`;
  batchFields.sessionId.textContent = activeRun?.sessionId ?? "-";
  batchFields.elapsedMs.textContent = `${activeRun?.lastElapsedRealMs ?? 0} ms`;
  batchFields.coverage.textContent = `${(activeRun?.lastCoveragePercentage ?? 0).toFixed(1)}%`;
  batchFields.runState.textContent = activeRun?.runState ?? (batchState.completed ? "Complete" : "Idle");
  batchFields.finishCondition.textContent = activeRun?.finishReason ?? batchState.lastFinishReason ?? "-";
  batchFields.completedCount.textContent = String(batchState.completedRuns);
  batchFields.failedCount.textContent = String(batchState.failedOrStuckRuns);
  batchFields.lastLogPath.textContent = batchState.lastSavedLogPath ?? "-";
  batchFields.summaryPath.textContent = batchState.summarySavedPath ?? "-";
}

function readBatchConfig() {
  const runCount = Number.parseInt(batchInputs.runCount.value, 10);
  const maxTimeMs = Number.parseInt(batchInputs.maxTimeMs.value, 10);
  const targetCoverage = Number.parseFloat(batchInputs.targetCoverage.value);
  const simulationSpeed = Number.parseFloat(batchInputs.speed.value);
  const obstacleCountRaw = batchInputs.obstacleCount.value.trim();
  const obstacleCount = obstacleCountRaw ? Number.parseInt(obstacleCountRaw, 10) : null;
  const config = {
    run_count: runCount,
    max_simulated_time_ms: maxTimeMs,
    target_coverage: targetCoverage,
    simulation_speed: simulationSpeed,
    return_to_dock_after_run: batchInputs.returnToDock.checked,
    stop_conditions: {
      time_limit: batchInputs.stopTime.checked,
      coverage_threshold: batchInputs.stopCoverage.checked,
      successful_docking: batchInputs.stopDocked.checked,
      stuck_diagnostic: batchInputs.stopStuck.checked,
    },
    obstacle_count: Number.isFinite(obstacleCount) ? obstacleCount : null,
  };

  if (!Number.isFinite(runCount) || runCount < 1) {
    throw new Error("Batch runs must be at least 1");
  }
  if (!Number.isFinite(maxTimeMs) || maxTimeMs < 1000) {
    throw new Error("Max run time must be at least 1000 ms");
  }
  if (!Number.isFinite(targetCoverage) || targetCoverage < 0 || targetCoverage > 100) {
    throw new Error("Target coverage must be in the range 0..100");
  }
  if (!Number.isFinite(simulationSpeed) || simulationSpeed <= 0) {
    throw new Error("Batch speed must be greater than zero");
  }

  return config;
}

function buildBatchRunConfig(config) {
  return {
    max_simulated_time: config.max_simulated_time_ms,
    target_coverage: Number(config.target_coverage.toFixed(3)),
    simulation_speed: config.simulation_speed,
    return_to_dock_after_run: config.return_to_dock_after_run,
    enabled_stop_conditions: {
      time_limit: config.stop_conditions.time_limit,
      coverage_threshold: config.stop_conditions.coverage_threshold,
      successful_docking: config.stop_conditions.successful_docking,
      stuck_diagnostic: config.stop_conditions.stuck_diagnostic,
    },
    ...(Number.isFinite(config.obstacle_count) ? { obstacle_count: config.obstacle_count } : {}),
  };
}

function resetCoverage() {
  coveredCellKeys = new Set();
  latestCoveragePercentage = 0;
  coverageCtx.clearRect(0, 0, coverageCanvas.width, coverageCanvas.height);
  fields.coverage.textContent = "0.0%";
}

function paintCoverageCell(col, row, sizes) {
  const centerX = (col + 0.5) * sizes.width;
  const centerY = (row + 0.5) * sizes.height;
  const radius = Math.max(3, Math.min(sizes.width, sizes.height) * COVERAGE_TRAIL_RADIUS_SCALE);
  const blurRadius = radius * COVERAGE_TRAIL_BLUR_SCALE;
  const gradient = coverageCtx.createRadialGradient(centerX, centerY, 0, centerX, centerY, radius + blurRadius);
  gradient.addColorStop(0, `rgba(72, 166, 108, ${COVERAGE_TRAIL_CORE_ALPHA})`);
  gradient.addColorStop(0.55, `rgba(72, 166, 108, ${COVERAGE_TRAIL_ALPHA})`);
  gradient.addColorStop(1, "rgba(72, 166, 108, 0)");
  coverageCtx.fillStyle = gradient;
  coverageCtx.beginPath();
  coverageCtx.arc(centerX, centerY, radius + blurRadius, 0, Math.PI * 2);
  coverageCtx.fill();
}

function markCoverage() {
  const sizes = cellSize();
  const minCol = clamp(Math.floor((pose.x - ROBOT_RADIUS) / sizes.width), 0, COVERAGE_GRID.cols - 1);
  const maxCol = clamp(Math.floor((pose.x + ROBOT_RADIUS) / sizes.width), 0, COVERAGE_GRID.cols - 1);
  const minRow = clamp(Math.floor((pose.y - ROBOT_RADIUS) / sizes.height), 0, COVERAGE_GRID.rows - 1);
  const maxRow = clamp(Math.floor((pose.y + ROBOT_RADIUS) / sizes.height), 0, COVERAGE_GRID.rows - 1);
  let changed = false;

  for (let col = minCol; col <= maxCol; col += 1) {
    for (let row = minRow; row <= maxRow; row += 1) {
      const centerX = (col + 0.5) * sizes.width;
      const centerY = (row + 0.5) * sizes.height;
      const dx = centerX - pose.x;
      const dy = centerY - pose.y;
      if (dx * dx + dy * dy > ROBOT_RADIUS * ROBOT_RADIUS) {
        continue;
      }

      const key = `${col}:${row}`;
      if (!coveredCellKeys.has(key)) {
        coveredCellKeys.add(key);
        paintCoverageCell(col, row, sizes);
        changed = true;
      }
    }
  }

  if (changed || coveredCellKeys.size === 0) {
    latestCoveragePercentage = (coveredCellKeys.size / (COVERAGE_GRID.cols * COVERAGE_GRID.rows)) * 100;
    fields.coverage.textContent = `${latestCoveragePercentage.toFixed(1)}%`;
  }
}

function coverageCells() {
  return Array.from(coveredCellKeys, (value) => {
    const [col, row] = value.split(":").map(Number);
    return { col, row };
  });
}

function clearMotionEventState() {
  lastMotionKind = "idle";
  lastTurnDirection = null;
  localProximityContact = false;
  localProximityContactDetail = null;
  lastCollisionSignature = null;
  waitingForTurnAfterBumper = false;
  waitingDiagnosticLogged = false;
  silentForwardFreezeFrames = 0;
  lastRecordedTimestampMs = 0;
  turnEpisodeCounter = 0;
  activeTurnEpisodeId = null;
  turnEpisodeStartedAtMs = null;
  queuedTimelineEvents = [];
  escapePlan = null;
  dockRecoveryPlan = null;
  dockRoutePlan = null;
  dockRecoveryDirectionBias = 1;
  lastBackendState = null;
  lastDockingPhase = null;
  lastDockProgressDistance = null;
  lastDockProgressTimestamp = null;
  lastDockHeadingError = null;
  lastVisualCommandTelemetry = null;
  lastLoopRealTimestamp = null;
  lastBackendAutoNavigationPhase = null;
  resetLaneController();
}

function resetPose() {
  pose.x = currentMap.initialPose.x;
  pose.y = currentMap.initialPose.y;
  pose.heading = currentMap.initialPose.heading;
  pose.lastTimestamp = null;
  lastLoopRealTimestamp = null;
}

function currentDockRect() {
  return {
    x: currentMap.dockingStation.x - 60,
    y: currentMap.dockingStation.y - 13,
    width: 120,
    height: 26,
  };
}

function isForwardLike(left, right) {
  return left > 0 && right > 0;
}

function isForwardCommand(status) {
  return status.left_wheel_speed > 0 && status.left_wheel_speed === status.right_wheel_speed;
}

function forwardProbeCollision(sourcePose = pose, distance = MAX_LINEAR_SUBSTEP_PX) {
  const probe = {
    x: sourcePose.x + Math.cos(sourcePose.heading) * distance,
    y: sourcePose.y + Math.sin(sourcePose.heading) * distance,
    heading: sourcePose.heading,
  };
  return detectCollisionAtPose(probe);
}

function computeProximityContactDetail(sourcePose = pose) {
  const detection = detectForwardProximity(sourcePose, PROXIMITY_CONTACT_THRESHOLD_PX);
  if (!detection) {
    return null;
  }

  return {
    collision_target: detection.collision_target,
    obstacle_id: detection.obstacle_id ?? null,
    distance_px: detection.distance_px,
    contact_sensor_type: "proximity",
    physical_bumper: false,
    proximity_contact: true,
    proximity_contact_threshold_px: PROXIMITY_CONTACT_THRESHOLD_PX,
  };
}

function distanceToDock(sourcePose = pose) {
  return Math.hypot(currentMap.dockingStation.x - sourcePose.x, currentMap.dockingStation.y - sourcePose.y);
}

function distanceToPoint(target, sourcePose = pose) {
  return Math.hypot(target.x - sourcePose.x, target.y - sourcePose.y);
}

function segmentDistance(fromPoint, toPoint) {
  return Math.hypot(toPoint.x - fromPoint.x, toPoint.y - fromPoint.y);
}

function pathSegmentCollisionFree(fromPoint, toPoint) {
  const distance = segmentDistance(fromPoint, toPoint);
  if (distance <= 0) {
    return true;
  }

  const heading = Math.atan2(toPoint.y - fromPoint.y, toPoint.x - fromPoint.x);
  const steps = Math.max(1, Math.ceil(distance / DOCK_ROUTE_SAMPLE_STEP));
  for (let step = 1; step <= steps; step += 1) {
    const t = step / steps;
    const samplePose = {
      x: fromPoint.x + (toPoint.x - fromPoint.x) * t,
      y: fromPoint.y + (toPoint.y - fromPoint.y) * t,
      heading,
    };
    if (detectCollisionAtPose(samplePose)) {
      return false;
    }
  }

  return true;
}

function normalizeDockCandidate(points, startPoint) {
  const normalized = [];
  let currentPoint = startPoint;
  for (const point of points) {
    if (segmentDistance(currentPoint, point) <= DOCK_ROUTE_ALIGNMENT_TOLERANCE) {
      currentPoint = point;
      continue;
    }
    normalized.push(point);
    currentPoint = point;
  }
  return normalized;
}

function candidateRouteDistance(points, startPoint) {
  let total = 0;
  let currentPoint = startPoint;
  for (const point of points) {
    total += segmentDistance(currentPoint, point);
    currentPoint = point;
  }
  return total;
}

function buildDockRoutePlan() {
  const dockingStation = currentMap.dockingStation;
  const routeX = clamp(dockingStation.x + DOCK_ROUTE_X_OFFSET, ROBOT_RADIUS, ROOM.width - ROBOT_RADIUS);
  const routeY = clamp(ROOM.height - ROBOT_RADIUS - DOCK_ROUTE_BOTTOM_OFFSET, ROBOT_RADIUS, ROOM.height - ROBOT_RADIUS);
  const rightCorridorX = ROOM.width - ROBOT_RADIUS - DOCK_ROUTE_WALL_CLEARANCE;
  const startPoint = { x: pose.x, y: pose.y };
  const directDockPoint = {
    x: dockingStation.x,
    y: dockingStation.y,
    stage: "FINAL_APPROACH",
  };

  const routeCandidates = [
    [directDockPoint],
    [
      { x: routeX, y: startPoint.y, stage: "ROUTE_TO_ENTRY_COLUMN" },
      { x: routeX, y: routeY, stage: "ROUTE_TO_BOTTOM_CORRIDOR" },
      directDockPoint,
    ],
    [
      { x: rightCorridorX, y: startPoint.y, stage: "ROUTE_TO_SAFE_COLUMN" },
      { x: rightCorridorX, y: routeY, stage: "ROUTE_TO_BOTTOM_CORRIDOR" },
      { x: routeX, y: routeY, stage: "ROUTE_TO_DOCK_ENTRY" },
      directDockPoint,
    ],
  ];

  let bestCandidate = null;
  for (const candidate of routeCandidates) {
    const normalizedCandidate = normalizeDockCandidate(candidate, startPoint);
    let feasible = true;
    let currentPoint = startPoint;
    for (const point of normalizedCandidate) {
      if (!pathSegmentCollisionFree(currentPoint, point)) {
        feasible = false;
        break;
      }
      currentPoint = point;
    }
    if (!feasible || normalizedCandidate.length === 0) {
      continue;
    }
    const totalDistance = candidateRouteDistance(normalizedCandidate, startPoint);
    if (!bestCandidate || totalDistance < bestCandidate.totalDistance) {
      bestCandidate = {
        points: normalizedCandidate,
        totalDistance,
      };
    }
  }

  if (bestCandidate) {
    return bestCandidate.points;
  }

  return [directDockPoint];
}

function dockWaypointReached(targetPoint) {
  return distanceToPoint(targetPoint) <= DOCK_ROUTE_ALIGNMENT_TOLERANCE;
}

function computeDockTargetPoint() {
  const directDockPoint = {
    x: currentMap.dockingStation.x,
    y: currentMap.dockingStation.y,
    stage: "FINAL_APPROACH",
  };
  if (distanceToDock() <= DOCK_FINAL_APPROACH_DISTANCE) {
    dockRoutePlan = [directDockPoint];
    return directDockPoint;
  }

  if (!dockRoutePlan || dockRoutePlan.length === 0) {
    dockRoutePlan = buildDockRoutePlan();
  }

  while (dockRoutePlan.length > 1 && dockWaypointReached(dockRoutePlan[0])) {
    dockRoutePlan.shift();
  }

  return dockRoutePlan[0] ?? directDockPoint;
}

function dockingGuidanceDetail(status, phase, targetHeading = null, targetPoint = null) {
  const dockTarget = targetPoint ?? computeDockTargetPoint();
  const distance = distanceToDock();
  const routeDistance = distanceToPoint(dockTarget);
  const effectiveTargetHeading =
    targetHeading ?? Math.atan2(dockTarget.y - pose.y, dockTarget.x - pose.x);
  return {
    backend_left_wheel_speed: status.left_wheel_speed,
    backend_right_wheel_speed: status.right_wheel_speed,
    demo_docking_phase: phase,
    dock_route_stage: dockTarget.stage,
    demo_target_heading: Number(normalizeAngle(effectiveTargetHeading).toFixed(6)),
    heading_error: Number(normalizeAngle(effectiveTargetHeading - pose.heading).toFixed(6)),
    distance_to_dock: Number(distance.toFixed(3)),
    distance_to_route_target: Number(routeDistance.toFixed(3)),
    dock_route_target_x: Number(dockTarget.x.toFixed(3)),
    dock_route_target_y: Number(dockTarget.y.toFixed(3)),
  };
}

function turnEpisodeDetail(extra = {}) {
  if (activeTurnEpisodeId == null) {
    return Object.keys(extra).length > 0 ? extra : null;
  }

  return {
    turn_episode_id: activeTurnEpisodeId,
    ...extra,
  };
}

function applyLocalBumperState(moveResult) {
  const liveProximityContact = computeProximityContactDetail(pose);
  const detail =
    moveResult?.collisionDetail ??
    moveResult?.proximityContactDetail ??
    liveProximityContact ??
    null;

  if (detail?.collision_target === "wall" && !detail.wall_side) {
    detail.wall_side = nearestWallSide();
  }

  if (moveResult?.blockedForward || detail) {
    localProximityContact = true;
    localProximityContactDetail = detail;
    return;
  }

  const currentCollision = detectCollisionAtPose(pose);
  const futureForwardCollision = forwardProbeCollision(pose);
  if (!currentCollision && !futureForwardCollision && !computeProximityContactDetail(pose)) {
    localProximityContact = false;
    localProximityContactDetail = null;
  }
}

function buildEscapeCommand(timestampMs) {
  if (!escapePlan) {
    return null;
  }

  if (escapePlan.mode === "escape_turn" && timestampMs >= escapePlan.untilMs) {
    escapePlan = {
      mode: "forward_attempt",
      untilMs: timestampMs + ESCAPE_FORWARD_ATTEMPT_MS,
      turnDirection: escapePlan.turnDirection,
      episodeId: escapePlan.episodeId,
    };
    queueTimelineEvent("ESCAPE_FORWARD_ATTEMPT", turnEpisodeDetail({ phase: "forward_attempt" }));
  }

  if (escapePlan.mode === "forward_attempt") {
    return {
      left: 38,
      right: 38,
      source: "escape_forward_attempt",
      detail: turnEpisodeDetail({ phase: "forward_attempt" }),
    };
  }

  const turnDirection = escapePlan.turnDirection === "left" ? "left" : "right";
  return {
    left: turnDirection === "left" ? -34 : 34,
    right: turnDirection === "left" ? 34 : -34,
    source: "escape_turn",
    detail: turnEpisodeDetail({ phase: "escape_turn", turn_direction: turnDirection }),
  };
}

function maybeStartDockRecovery(status, timestampMs, collisionDetail, reason = "blocked") {
  if (dockRecoveryPlan) {
    return;
  }

  const dockTarget = computeDockTargetPoint();
  const preferredTurnDirection = normalizeAngle(Math.atan2(dockTarget.y - pose.y, dockTarget.x - pose.x) - pose.heading) >= 0 ? "left" : "right";
  const turnDirection =
    dockRecoveryDirectionBias > 0
      ? preferredTurnDirection
      : preferredTurnDirection === "left"
        ? "right"
        : "left";
  dockRecoveryDirectionBias *= -1;
  dockRecoveryPlan = {
    reverseUntilMs: timestampMs + DOCK_RECOVERY_REVERSE_MS,
    arcUntilMs: timestampMs + DOCK_RECOVERY_REVERSE_MS + DOCK_RECOVERY_ARC_MS,
    turnDirection,
  };
  lastDockingPhase = null;
  dockRoutePlan = null;
  lastDockProgressDistance = null;
  lastDockProgressTimestamp = timestampMs;
  lastDockHeadingError = null;
  queueTimelineEvent("DOCK_RECOVERY_MANEUVER", {
    ...dockingGuidanceDetail(status, "DOCK_RECOVERY_MANEUVER", null, dockTarget),
    collision_target: collisionDetail?.collision_target ?? null,
    obstacle_id: collisionDetail?.obstacle_id ?? null,
    recovery_reason: reason,
    turn_direction: turnDirection,
  });
}

function buildDockingCommand(status, timestampMs) {
  const dockTarget = computeDockTargetPoint();
  const targetHeading = Math.atan2(dockTarget.y - pose.y, dockTarget.x - pose.x);
  const headingError = normalizeAngle(targetHeading - pose.heading);
  const routeDistance = distanceToPoint(dockTarget);
  const headingImproved =
    lastDockHeadingError == null || Math.abs(headingError) + DOCK_PROGRESS_HEADING_EPSILON_RAD < Math.abs(lastDockHeadingError);

  if (lastDockProgressDistance == null || routeDistance + DOCK_PROGRESS_DISTANCE_EPSILON < lastDockProgressDistance || headingImproved) {
    lastDockProgressDistance = routeDistance;
    lastDockProgressTimestamp = timestampMs;
    lastDockHeadingError = headingError;
  } else if (
    lastDockProgressTimestamp != null &&
    timestampMs - lastDockProgressTimestamp >= DOCK_STUCK_DIAGNOSTIC_MS &&
    !dockRecoveryPlan
  ) {
    queueTimelineEvent("DOCK_STUCK_DIAGNOSTIC", dockingGuidanceDetail(status, "DOCK_STUCK_DIAGNOSTIC", targetHeading, dockTarget));
    lastDockProgressTimestamp = timestampMs;
    maybeStartDockRecovery(status, timestampMs, null, "no_progress");
  }

  if (dockRecoveryPlan) {
    if (timestampMs >= dockRecoveryPlan.arcUntilMs) {
      dockRecoveryPlan = null;
      lastDockingPhase = null;
      lastDockProgressDistance = null;
      lastDockProgressTimestamp = timestampMs;
      lastDockHeadingError = null;
    } else if (timestampMs < dockRecoveryPlan.reverseUntilMs) {
      return {
        left: -20,
        right: -20,
        source: "dock_recovery_reverse",
        phase: "DOCK_RECOVERY_REVERSE",
        targetHeading,
        completeOnHeadingTarget: false,
      };
    } else {
      return {
        left: dockRecoveryPlan.turnDirection === "left" ? -10 : -28,
        right: dockRecoveryPlan.turnDirection === "left" ? -28 : -10,
        source: "dock_recovery_arc",
        phase: "DOCK_RECOVERY_ARC",
        targetHeading,
        completeOnHeadingTarget: false,
      };
    }
  }

  let phase = "DOCK_APPROACH";
  let left = 22;
  let right = 22;

  if (Math.abs(headingError) > DOCK_ALIGN_ERROR_RAD) {
    phase = Math.abs(headingError) > DOCK_TARGET_ERROR_RAD ? "DOCK_TARGETING" : "DOCK_ALIGNING";
    const turnSpeed = Math.round(clamp(Math.abs(headingError) * 20, 12, 24));
    left = headingError > 0 ? -turnSpeed : turnSpeed;
    right = headingError > 0 ? turnSpeed : -turnSpeed;
  } else {
    const distanceToDockTarget = dockTarget.stage === "FINAL_APPROACH" ? distanceToDock() : routeDistance;
    const baseSpeed = distanceToDockTarget <= DOCK_NEAR_DISTANCE ? 14 : 24;
    const correction = clamp(headingError * 12, -5, 5);
    left = Math.max(8, Math.round(baseSpeed - correction));
    right = Math.max(8, Math.round(baseSpeed + correction));
  }

  if (phase !== lastDockingPhase) {
    queueTimelineEvent(phase, dockingGuidanceDetail(status, phase, targetHeading, dockTarget));
    lastDockingPhase = phase;
  }

  return {
    left,
    right,
    source: "dock_guidance",
    phase,
    targetHeading,
    completeOnHeadingTarget: phase === "DOCK_TARGETING" || phase === "DOCK_ALIGNING",
  };
}

function buildHeadingCommand(targetHeading, forwardSpeed, turnSpeed = TURNING_SPEED, options = {}) {
  const completeOnHeadingTarget = Boolean(options.completeOnHeadingTarget);
  const error = angleError(targetHeading, pose.heading);
  if (Math.abs(error) <= HEADING_TOLERANCE_RAD) {
    return {
      left: forwardSpeed,
      right: forwardSpeed,
      targetHeading,
      completeOnHeadingTarget,
    };
  }

  const proportionalTurnSpeed = Math.round(clamp(Math.abs(error) * 18, MIN_TURN_SPEED, Math.max(MIN_TURN_SPEED, turnSpeed, MAX_TURN_SPEED)));
  return {
    left: error > 0 ? -proportionalTurnSpeed : proportionalTurnSpeed,
    right: error > 0 ? proportionalTurnSpeed : -proportionalTurnSpeed,
    targetHeading,
    completeOnHeadingTarget,
  };
}

function buildReactiveForwardCommand(targetHeading, speed, phase, turnSpeed = TURNING_SPEED) {
  return {
    ...buildHeadingCommand(targetHeading, speed, turnSpeed),
    source: "reactive_auto",
    phase,
  };
}

function buildReactiveTurnCommand(targetHeading, phase, turnSpeed = TURNING_SPEED) {
  return {
    ...buildHeadingCommand(targetHeading, 0, turnSpeed, { completeOnHeadingTarget: true }),
    source: "reactive_auto",
    phase,
  };
}

function buildLaneControllerCommand(status, timestampMs) {
  if (!laneController) {
    resetLaneController();
  }

  if (laneController.phase === "ROOM_CROSSING" || laneController.phase === "ROOM_CROSSING_COMMITTED") {
    if (laneController.phaseStartedAtMs === 0) {
      beginRoomCrossing(timestampMs, { targetHeading: pose.heading });
    }
    laneController.phaseTargetHeading = normalizeAngle(laneController.phaseTargetHeading ?? pose.heading);
    return buildReactiveForwardCommand(laneController.phaseTargetHeading, ROOM_CROSSING_SPEED, laneController.phase);
  }

  if (laneController.phase === "CONTACT_BACKUP" || laneController.phase === "OBSTACLE_BACKUP" || laneController.phase === "ANTI_LOOP_BACKUP") {
    return {
      left: BACKUP_SPEED,
      right: BACKUP_SPEED,
      source: "reactive_auto",
      phase: laneController.phase,
      targetHeading: null,
      completeOnHeadingTarget: false,
    };
  }

  if (laneController.phase === "WALL_ALIGN") {
    laneController.phaseTargetHeading =
      laneController.phaseTargetHeading ?? chooseWallFollowHeading(laneController.wallSide, laneController.escapeSide);
    return buildReactiveTurnCommand(laneController.phaseTargetHeading, laneController.phase);
  }

  if (laneController.phase === "WALL_FOLLOW") {
    laneController.phaseTargetHeading = chooseWallFollowHeading(laneController.wallSide, laneController.escapeSide);
    return buildReactiveForwardCommand(laneController.phaseTargetHeading, WALL_FOLLOW_SPEED, laneController.phase);
  }

  if (laneController.phase === "WALL_RELEASE") {
    laneController.phaseTargetHeading =
      laneController.phaseTargetHeading ??
      chooseWallReleaseHeading(chooseWallFollowHeading(laneController.wallSide, laneController.escapeSide));
    return buildReactiveTurnCommand(laneController.phaseTargetHeading, laneController.phase);
  }

  if (laneController.phase === "OBSTACLE_TURN_AWAY") {
    laneController.phaseTargetHeading =
      laneController.phaseTargetHeading ?? chooseObstacleTurnHeading(laneController.obstacleId, laneController.escapeSide);
    return buildReactiveTurnCommand(laneController.phaseTargetHeading, laneController.phase);
  }

  if (laneController.phase === "OBSTACLE_ESCAPE_FORWARD") {
    laneController.phaseTargetHeading = laneController.phaseTargetHeading ?? pose.heading;
    return buildReactiveForwardCommand(
      laneController.phaseTargetHeading,
      OBSTACLE_ESCAPE_FORWARD_SPEED,
      laneController.phase,
    );
  }

  if (laneController.phase === "ANTI_LOOP_TURN") {
    laneController.phaseTargetHeading = normalizeAngle(
      laneController.phaseStartPose.heading +
        (laneController.escapeSide === "left" ? -ANTI_LOOP_TURN_ANGLE_RAD : ANTI_LOOP_TURN_ANGLE_RAD),
    );
    return buildReactiveTurnCommand(laneController.phaseTargetHeading, laneController.phase, MAX_TURN_SPEED);
  }

  if (laneController.phase === "ANTI_LOOP_FORWARD") {
    laneController.phaseTargetHeading = laneController.phaseTargetHeading ?? pose.heading;
    return buildReactiveForwardCommand(laneController.phaseTargetHeading, ANTI_LOOP_FORWARD_SPEED, laneController.phase);
  }

  return buildReactiveForwardCommand(pose.heading, ROOM_CROSSING_SPEED, "ROOM_CROSSING");
}

function buildVisualCommand(status, timestampMs) {
  return null;
}

function distanceFromPhaseStart() {
  if (!laneController?.phaseStartPose) return 0;
  return Math.hypot(pose.x - laneController.phaseStartPose.x, pose.y - laneController.phaseStartPose.y);
}

function lanePhaseProgressed() {
  if (!laneController) return false;
  if (["ROOM_CROSSING", "ROOM_CROSSING_COMMITTED", "WALL_FOLLOW", "OBSTACLE_ESCAPE_FORWARD", "ANTI_LOOP_FORWARD"].includes(laneController.phase)) {
    return distanceFromPhaseStart() > 1;
  }
  return Math.abs(normalizeAngle(pose.heading - laneController.phaseStartPose.heading)) > 0.05;
}

function advanceLaneController(status, timestampMs, moveResult) {
  if (!shouldUseLaneController(status) || !laneController) {
    return;
  }

  const phaseDurationMs = timestampMs - laneController.phaseStartedAtMs;
  const headingTargetReached =
    Boolean(moveResult?.headingTargetReached) ||
    Boolean(moveResult?.headingTargetCrossed);
  const headingCloseToTarget =
    headingTargetReached ||
    laneController.phaseTargetHeading == null ||
    Math.abs(normalizeAngle(laneController.phaseTargetHeading - pose.heading)) <= HEADING_TOLERANCE_RAD;
  const contact = classifyNavigationContact(moveResult);
  const blockedByContact = moveResult.blockedForward || Boolean(moveResult.proximityContactDetail);
  const proximityIgnored = timestampMs < (laneController.ignoreProximityUntilMs ?? 0);
  if (moveResult.movementChanged) {
    laneController.lastProgressMs = timestampMs;
  }
  laneController.lastProgressCoverage = latestCoveragePercentage;

  pruneRecentContacts(timestampMs);
  const antiLoopMinPhaseMs = 2500;
  const antiLoopEligible =
    ["ROOM_CROSSING", "ROOM_CROSSING_COMMITTED", "WALL_FOLLOW", "OBSTACLE_ESCAPE_FORWARD"].includes(laneController.phase) &&
    phaseDurationMs >= antiLoopMinPhaseMs;

  if (antiLoopEligible) {
    const stalled = !moveResult.movementChanged && phaseDurationMs >= ANTI_LOOP_STALL_MS;
    const tooManyContacts = laneController.recentContactTimes.length >= ANTI_LOOP_CONTACT_THRESHOLD;
    const repeatedObstacleLoop = laneController.repeatedObstacleHits >= 4 && laneController.recoveryAttempts >= 1;
    const repeatedEscapeFailures =
      laneController.phase === "OBSTACLE_ESCAPE_FORWARD" && laneController.recoveryAttempts >= 2;
    if (tooManyContacts || repeatedObstacleLoop || stalled || repeatedEscapeFailures) {
      beginAntiLoopEscape(
        timestampMs,
        tooManyContacts
          ? "dense_contacts"
          : repeatedObstacleLoop
            ? "same_obstacle_repeat"
            : repeatedEscapeFailures
              ? "failed_escape_repeat"
              : "no_motion",
        {
          contactType: contact.contactType,
          obstacleId: contact.obstacleId,
          wallSide: contact.wallSide,
        },
      );
      return;
    }
  }

  if ((laneController.phase === "ROOM_CROSSING" || laneController.phase === "ROOM_CROSSING_COMMITTED") && blockedByContact && !proximityIgnored) {
    if (contact.contactType === "wall") {
      beginWallContact(timestampMs, contact);
    } else {
      beginObstacleContact(timestampMs, contact);
    }
    return;
  }

  if (laneController.phase === "ROOM_CROSSING_COMMITTED") {
    const committedTimeDone = timestampMs >= (laneController.committedUntilMs ?? 0);
    const committedDistanceDone = distanceFromPhaseStart() >= (laneController.committedMinDistancePx ?? ROOM_CROSSING_COMMIT_DISTANCE_PX);

    if (committedTimeDone || committedDistanceDone) {
      beginRoomCrossing(timestampMs, {
        targetHeading: laneController.phaseTargetHeading ?? pose.heading,
        diversify: false,
        recoveryAttempts: 0,
      });
      return;
    }
  }

  if (["CONTACT_BACKUP", "OBSTACLE_BACKUP", "ANTI_LOOP_BACKUP"].includes(laneController.phase)) {
    const targetDistance =
      laneController.phase === "ANTI_LOOP_BACKUP" ? ANTI_LOOP_BACKUP_DISTANCE_PX : BACKUP_DISTANCE_PX;
    if (moveResult.blockedMovement) {
      laneController.escapeSide = chooseAlternateEscapeSide(laneController.escapeSide);
      queueLaneEvent("BACKUP_BLOCKED", {
        contact_type: laneController.contactType,
        obstacle_id: laneController.obstacleId,
        wall_side: laneController.wallSide,
      });
      const nextPhase =
        laneController.phase === "CONTACT_BACKUP"
          ? "WALL_ALIGN"
          : laneController.phase === "OBSTACLE_BACKUP"
            ? "OBSTACLE_TURN_AWAY"
            : "ANTI_LOOP_TURN";
      let nextTargetHeading = null;

      if (nextPhase === "WALL_ALIGN") {
        nextTargetHeading = chooseWallFollowHeading(laneController.wallSide, laneController.escapeSide);
      } else if (nextPhase === "OBSTACLE_TURN_AWAY") {
        nextTargetHeading = chooseObstacleTurnHeading(laneController.obstacleId, laneController.escapeSide);
      } else if (nextPhase === "ANTI_LOOP_TURN") {
        nextTargetHeading = normalizeAngle(
          laneController.phaseStartPose.heading +
            (laneController.escapeSide === "left" ? -ANTI_LOOP_TURN_ANGLE_RAD : ANTI_LOOP_TURN_ANGLE_RAD),
        );
      }

      setLanePhase(nextPhase, timestampMs, {
        phaseTargetHeading: nextTargetHeading,
        backupBlocked: true,
      });
      queueLaneEvent(nextPhase === "WALL_ALIGN" ? "WALL_ALIGN" : nextPhase === "OBSTACLE_TURN_AWAY" ? "OBSTACLE_TURN_AWAY" : "ANTI_LOOP_TURN");
      return;
    }
    if (distanceFromPhaseStart() >= targetDistance || backupTimedOut(timestampMs)) {
      if (laneController.phase === "CONTACT_BACKUP") {
        setLanePhase("WALL_ALIGN", timestampMs, {
          phaseTargetHeading: chooseWallFollowHeading(laneController.wallSide, laneController.escapeSide),
          backupBlocked: false,
        });
        queueLaneEvent("WALL_ALIGN");
      } else if (laneController.phase === "OBSTACLE_BACKUP") {
        setLanePhase("OBSTACLE_TURN_AWAY", timestampMs, {
          phaseTargetHeading: chooseObstacleTurnHeading(laneController.obstacleId, laneController.escapeSide),
          backupBlocked: false,
        });
        queueLaneEvent("OBSTACLE_TURN_AWAY");
      } else {
        setLanePhase("ANTI_LOOP_TURN", timestampMs, {
          phaseTargetHeading: normalizeAngle(
            laneController.phaseStartPose.heading +
              (laneController.escapeSide === "left" ? -ANTI_LOOP_TURN_ANGLE_RAD : ANTI_LOOP_TURN_ANGLE_RAD),
          ),
          backupBlocked: false,
        });
        queueLaneEvent("ANTI_LOOP_TURN");
      }
      return;
    }
  }

  if (laneController.phase === "WALL_ALIGN" && headingCloseToTarget) {
    setLanePhase("WALL_FOLLOW", timestampMs, {
      phaseTargetHeading: laneController.phaseTargetHeading,
      backupBlocked: false,
    });
    queueLaneEvent("WALL_FOLLOW_START");
    return;
  }

  if (laneController.phase === "WALL_FOLLOW") {
    if (blockedByContact && !proximityIgnored) {
      if (contact.contactType === "obstacle") {
        beginObstacleContact(timestampMs, contact, laneController.recoveryAttempts + 1);
      } else {
        const releaseChoice = chooseDiverseCrossingHeading(
          chooseWallReleaseHeading(laneController.phaseTargetHeading ?? pose.heading),
        );

        setLanePhase("WALL_RELEASE", timestampMs, {
          phaseTargetHeading: releaseChoice.heading,
        });
        queueLaneEvent("WALL_FOLLOW_END");
        queueLaneEvent("WALL_RELEASE", {
          selected_heading: Number(normalizeAngle(releaseChoice.heading).toFixed(6)),
          original_candidate_heading: Number(normalizeAngle(releaseChoice.originalHeading).toFixed(6)),
          diverse_heading_applied: releaseChoice.diverseHeadingApplied,
          recent_crossing_headings: releaseChoice.recentHeadings.map((value) => Number(normalizeAngle(value).toFixed(6))),
        });
      }
      return;
    }
  }

  if (laneController.phase === "WALL_RELEASE" && headingCloseToTarget) {
    beginRoomCrossing(timestampMs, {
      targetHeading: laneController.phaseTargetHeading,
      committed: true,
      diversify: false,
    });
    return;
  }

  if (laneController.phase === "OBSTACLE_TURN_AWAY" && headingCloseToTarget) {
    setLanePhase("OBSTACLE_ESCAPE_FORWARD", timestampMs, {
      phaseTargetHeading: laneController.phaseTargetHeading,
      ignoreProximityUntilMs: 0,
      backupBlocked: false,
    });
    queueLaneEvent("OBSTACLE_ESCAPE_FORWARD");
    return;
  }

  if (laneController.phase === "OBSTACLE_ESCAPE_FORWARD") {
    if ((blockedByContact && !proximityIgnored) || moveResult.blockedMovement) {
      const nextAttempts = laneController.recoveryAttempts + 1;

      queueLaneEvent("OBSTACLE_ESCAPE_FAILED", {
        contact_type: contact.contactType,
        obstacle_id: contact.obstacleId,
        wall_side: contact.wallSide,
        recovery_attempts: nextAttempts,
      });

      if (nextAttempts >= 4) {
        beginAntiLoopEscape(timestampMs, "failed_escape_repeat", {
          contactType: contact.contactType,
          obstacleId: contact.obstacleId,
          wallSide: contact.wallSide,
        });
        return;
      }

      if (contact.contactType === "wall") {
        beginWallContact(timestampMs, contact);
      } else {
        beginObstacleContact(timestampMs, contact, nextAttempts);
      }
      return;
    }
    if (distanceFromPhaseStart() >= OBSTACLE_ESCAPE_DISTANCE_PX || phaseDurationMs >= OBSTACLE_ESCAPE_MAX_MS) {
      const escapeChoice = chooseDiverseCrossingHeading(laneController.phaseTargetHeading ?? pose.heading);

      queueLaneEvent("OBSTACLE_ESCAPE_SUCCESS", {
        selected_heading: Number(normalizeAngle(escapeChoice.heading).toFixed(6)),
        original_candidate_heading: Number(normalizeAngle(escapeChoice.originalHeading).toFixed(6)),
        diverse_heading_applied: escapeChoice.diverseHeadingApplied,
        recent_crossing_headings: escapeChoice.recentHeadings.map((value) => Number(normalizeAngle(value).toFixed(6))),
      });

      beginRoomCrossing(timestampMs, {
        targetHeading: escapeChoice.heading,
        committed: true,
        diversify: false,
        recoveryAttempts: 0,
      });
      return;
    }
  }

  if (laneController.phase === "ANTI_LOOP_TURN" && headingCloseToTarget) {
    setLanePhase("ANTI_LOOP_FORWARD", timestampMs, {
      phaseTargetHeading: laneController.phaseTargetHeading,
      ignoreProximityUntilMs: timestampMs + ANTI_LOOP_IGNORE_PROXIMITY_MS,
      backupBlocked: false,
    });
    queueLaneEvent("ANTI_LOOP_FORWARD");
    return;
  }

  if (laneController.phase === "ANTI_LOOP_FORWARD") {
    if ((blockedByContact && !proximityIgnored) || moveResult.blockedMovement) {
      if (contact.contactType === "wall") {
        beginWallContact(timestampMs, contact);
      } else {
        beginObstacleContact(timestampMs, contact, laneController.recoveryAttempts + 1);
      }
      return;
    }
    if (distanceFromPhaseStart() >= ANTI_LOOP_FORWARD_DISTANCE_PX) {
      const antiLoopChoice = chooseDiverseCrossingHeading(normalizeAngle(pose.heading + GOLDEN_ANGLE_RAD), {
        forceGoldenAngle: true,
      });

      queueLaneEvent("ANTI_LOOP_END", {
        selected_heading: Number(normalizeAngle(antiLoopChoice.heading).toFixed(6)),
        original_candidate_heading: Number(normalizeAngle(antiLoopChoice.originalHeading).toFixed(6)),
        diverse_heading_applied: antiLoopChoice.diverseHeadingApplied,
        recent_crossing_headings: antiLoopChoice.recentHeadings.map((value) => Number(normalizeAngle(value).toFixed(6))),
      });

      beginRoomCrossing(timestampMs, {
        targetHeading: antiLoopChoice.heading,
        committed: true,
        diversify: false,
        recoveryAttempts: 0,
      });
    }
  }
}

function updatePoseFromStatus(status, timestamp, visualCommand = null) {
  if (pose.lastTimestamp == null) {
    pose.lastTimestamp = timestamp;
    markCoverage();
    return {
      requestedForward: false,
      movementChanged: false,
      blockedForward: false,
      blockedMovement: false,
      collisionDetail: null,
      maxSubstepsReached: false,
      maxAngularSubstepsReached: false,
      visualCommand,
    };
  }

  const deltaSeconds = Math.max(0, (timestamp - pose.lastTimestamp) / 1000);
  pose.lastTimestamp = timestamp;

  const left = visualCommand?.left ?? status.left_wheel_speed;
  const right = visualCommand?.right ?? status.right_wheel_speed;
  const speedScale = BASE_VISUAL_SPEED_SCALE;
  const linearTotal = ((left + right) / 2) * speedScale * deltaSeconds;
  const angularTotal = (right - left) * TURN_SCALE * deltaSeconds;
  const requestedForward = isForwardLike(left, right);
  const absoluteDistance = Math.abs(linearTotal);
  const absoluteAngular = Math.abs(angularTotal);
  const linearSubsteps = absoluteDistance > 0 ? Math.max(1, Math.ceil(absoluteDistance / MAX_LINEAR_SUBSTEP_PX)) : 1;
  const angularSubsteps = absoluteAngular > 0 ? Math.max(1, Math.ceil(absoluteAngular / MAX_ANGULAR_SUBSTEP_RAD)) : 1;
  const substeps = Math.max(linearSubsteps, angularSubsteps, 1);
  const maxSubstepsReached = substeps > MAX_VISUAL_SUBSTEPS_PER_FRAME;
  const appliedSubsteps = Math.min(substeps, MAX_VISUAL_SUBSTEPS_PER_FRAME);
  const linearStep = appliedSubsteps > 0 ? linearTotal / appliedSubsteps : 0;
  const angularStep = appliedSubsteps > 0 ? angularTotal / appliedSubsteps : 0;
  const startX = pose.x;
  const startY = pose.y;
  const startHeading = pose.heading;
  const targetHeading = visualCommand?.targetHeading ?? null;
  const preMoveProximityContactDetail = computeProximityContactDetail(pose);
  const completeOnHeadingTarget = Boolean(visualCommand?.completeOnHeadingTarget);
  const headingErrorBefore =
    targetHeading == null ? null : angleError(targetHeading, startHeading);
  let collisionDetail = null;
  let collidedAtStep = null;
  let headingTargetReached = false;
  let headingTargetCrossed = false;
  let headingErrorAfter = headingErrorBefore;

  for (let step = 0; step < appliedSubsteps; step += 1) {
    const nextHeading = normalizeAngle(pose.heading + angularStep);
    const candidate = {
      x: pose.x + Math.cos(nextHeading) * linearStep,
      y: pose.y + Math.sin(nextHeading) * linearStep,
      heading: nextHeading,
    };

    collisionDetail = detectCollisionAtPose(candidate);
    if (collisionDetail) {
      collidedAtStep = step + 1;
      break;
    }

    pose.x = clamp(candidate.x, ROBOT_RADIUS, ROOM.width - ROBOT_RADIUS);
    pose.y = clamp(candidate.y, ROBOT_RADIUS, ROOM.height - ROBOT_RADIUS);
    pose.heading = candidate.heading;
    if (completeOnHeadingTarget && targetHeading != null) {
      const beforeError = angleError(targetHeading, normalizeAngle(pose.heading - angularStep));
      const afterError = angleError(targetHeading, pose.heading);
      headingErrorAfter = afterError;
      if (Math.abs(afterError) <= HEADING_TOLERANCE_RAD || crossedTargetHeading(beforeError, afterError)) {
        headingTargetReached = true;
        headingTargetCrossed = crossedTargetHeading(beforeError, afterError);
        pose.heading = normalizeAngle(targetHeading);
        headingErrorAfter = 0;
        markCoverage();
        break;
      }
    }
    markCoverage();
  }

  if (headingTargetReached && completeOnHeadingTarget && targetHeading != null) {
    queueTimelineEvent("HEADING_TARGET_REACHED", {
      ...phaseDetail(),
      target_heading: Number(normalizeAngle(targetHeading).toFixed(6)),
      heading_before: Number(normalizeAngle(startHeading).toFixed(6)),
      heading_after: Number(normalizeAngle(pose.heading).toFixed(6)),
      heading_error_before: headingErrorBefore == null ? null : Number(headingErrorBefore.toFixed(6)),
      heading_error_after: headingErrorAfter == null ? null : Number(headingErrorAfter.toFixed(6)),
      crossed_target_heading: headingTargetCrossed,
      heading_target_reached: true,
      angular_substeps: angularSubsteps,
    });
  }

  if (collisionDetail) {
    queueTimelineEvent("MICROSTEP_COLLISION", {
      ...phaseDetail(),
      ...buildCollisionEventDetail(collisionDetail, {
        maxSubstepsReached,
        maxAngularSubstepsReached: angularSubsteps > MAX_VISUAL_SUBSTEPS_PER_FRAME,
        linearSubsteps,
        angularSubsteps,
        substeps: appliedSubsteps,
        headingBefore: startHeading,
        headingAfter: pose.heading,
        collidedAtStep,
        targetHeading,
        headingErrorBefore,
        headingErrorAfter,
        headingTargetReached,
        headingTargetCrossed,
      }),
    });
  }

  const postMoveProximityContactDetail = computeProximityContactDetail(pose);
  const proximityContactDetail = postMoveProximityContactDetail;
  const proximityContact = Boolean(postMoveProximityContactDetail);

  return {
    requestedForward,
    movementChanged: pose.x !== startX || pose.y !== startY,
    blockedForward: requestedForward && Boolean(collisionDetail),
    blockedMovement: Boolean(collisionDetail),
    collisionDetail,
    proximityContact,
    proximityContactDetail,
    preMoveProximityContactDetail,
    maxSubstepsReached,
    maxAngularSubstepsReached: angularSubsteps > MAX_VISUAL_SUBSTEPS_PER_FRAME,
    linearSubsteps,
    angularSubsteps,
    appliedSubsteps,
    headingBefore: startHeading,
    headingAfter: pose.heading,
    targetHeading,
    headingErrorBefore,
    headingErrorAfter,
    headingTargetReached,
    headingTargetCrossed,
    visualCommand,
  };
}

function computeSimulationSignals() {
  const liveProximityContactDetail = computeProximityContactDetail(pose);
  const effectiveContactDetail = liveProximityContactDetail ?? localProximityContactDetail ?? null;
  const proximityContact = Boolean(effectiveContactDetail) || localProximityContact;
  const obstacleDetected = sampleForwardObstacle();
  const forwardClearanceBlocked = forwardClearanceBlockedByBody();
  const dockDetected = circleIntersectsRect({ x: pose.x, y: pose.y }, currentDockRect(), ROBOT_RADIUS + 10);
  let contactType = null;
  let wallSide = null;

  if (proximityContact) {
    if (effectiveContactDetail?.collision_target === "wall") {
      contactType = "wall";
      wallSide = effectiveContactDetail.wall_side ?? nearestWallSide();
    } else if (effectiveContactDetail?.collision_target === "obstacle") {
      contactType = "obstacle";
      wallSide = null;
    } else {
      contactType = "unknown";
      wallSide = null;
    }
  }

  return {
    sensors: {
      obstacle_detected: obstacleDetected,
      bumper_pressed: proximityContact,
      proximity_contact: proximityContact,
      contact_type: contactType,
      wall_side: wallSide,
      forward_clearance_blocked: forwardClearanceBlocked,
    },
    docking: {
      dock_detected: dockDetected,
    },
    proximityContactDetail: effectiveContactDetail,
    obstacleDetected,
    forwardClearanceBlocked,
  };
}

function renderRobot(status) {
  robotEl.style.left = `${pose.x}px`;
  robotEl.style.top = `${pose.y}px`;
  robotEl.style.transform = `translate(-29px, -29px) rotate(${pose.heading}rad)`;
  // Canvas heading visual offset used to align robot direction marker with sensor cone.
  headingEl.style.transform = `rotate(${Math.PI / 2}rad)`;
  sensorConeEl.style.transform = "translate(0, -21px) rotate(0rad)";
  bumperRingEl.classList.toggle("active", Boolean(status?.sensors?.bumper_pressed));
}

function renderStatus(status) {
  latestStatus = status;
  fields.state.textContent = status.state;
  fields.mode.textContent = status.cleaning_mode;
  fields.battery.textContent = `${status.battery_percent}%`;
  fields.charging.textContent = String(status.is_charging);
  fields.suction.textContent = String(status.suction_enabled);
  fields.brushes.textContent = String(status.brushes_enabled);
  fields.leftWheel.textContent = String(status.left_wheel_speed);
  fields.rightWheel.textContent = String(status.right_wheel_speed);
  fields.error.textContent = status.current_error ?? "None";
  fields.coverage.textContent = `${latestCoveragePercentage.toFixed(1)}%`;

  sensorListEl.innerHTML = "";
  Object.entries(status.sensors).forEach(([name, value]) => {
    const item = document.createElement("li");
    if (value && (name === "obstacle_detected" || name === "bumper_pressed")) {
      item.classList.add("attention");
    }
    const label = document.createElement("span");
    label.textContent = name === "bumper_pressed" ? "proximity_contact (compat)" : name;
    const state = document.createElement("span");
    state.textContent = value ? "ON" : "OFF";
    state.className = value ? "sensor-on" : "sensor-off";
    item.append(label, state);
    sensorListEl.appendChild(item);
  });

  statusJsonEl.textContent = JSON.stringify(status, null, 2);
  renderRobot(status);
  if (batchState?.activeRun) {
    batchState.activeRun.runState = status.state;
    batchState.activeRun.lastCoveragePercentage = latestCoveragePercentage;
    updateBatchUi();
  }
}

function maybeQueueBackendAutoPhaseEvent(status) {
  const phase = status?.auto_navigation_phase ?? null;
  const autoCleaning = status?.state === "CLEANING" && status?.cleaning_mode === "AUTO";

  if (!autoCleaning) {
    lastBackendAutoNavigationPhase = null;
    return;
  }

  if (phase && phase !== lastBackendAutoNavigationPhase) {
    queueTimelineEvent(phase, {
      source: "backend_auto_navigation",
      previous_phase: lastBackendAutoNavigationPhase,
      new_phase: phase,
    });
    lastBackendAutoNavigationPhase = phase;
  }
}

function snapshotFrame(status, eventLabel = null, timestampMs = null, eventDetail = null) {
  const telemetry = lastVisualCommandTelemetry;
  return {
    frame_index: frameIndex++,
    timestamp_ms: timestampMs,
    x: Number(pose.x.toFixed(3)),
    y: Number(pose.y.toFixed(3)),
    heading: Number(pose.heading.toFixed(6)),
    left_wheel_speed: status.left_wheel_speed,
    right_wheel_speed: status.right_wheel_speed,
    backend_left_wheel_speed: telemetry?.backend_left_wheel_speed ?? null,
    backend_right_wheel_speed: telemetry?.backend_right_wheel_speed ?? null,
    demo_left_wheel_speed: telemetry?.demo_left_wheel_speed ?? null,
    demo_right_wheel_speed: telemetry?.demo_right_wheel_speed ?? null,
    demo_navigation_phase: status.auto_navigation_phase ?? telemetry?.demo_navigation_phase ?? null,
    state: status.state,
    cleaning_mode: status.cleaning_mode,
    current_error: status.current_error,
    battery_percent: status.battery_percent,
    is_charging: status.is_charging,
    sensors: { ...status.sensors },
    coverage_percentage: Number(latestCoveragePercentage.toFixed(3)),
    event: eventLabel,
    event_detail: eventDetail,
  };
}

function recordFrame(status, eventLabel = null, timestampMs = null, eventDetail = null) {
  if (!currentSession) return;

  const previousTimestamp = lastTimelineTimestamp();
  let safeTimestamp = timestampMs == null ? nextTimelineTimestamp() : Number(timestampMs);

  if (!Number.isFinite(safeTimestamp)) {
    safeTimestamp = nextTimelineTimestamp();
  }

  if (safeTimestamp < previousTimestamp) {
    safeTimestamp = previousTimestamp + 1;
  }

  lastRecordedTimestampMs = Math.max(lastRecordedTimestampMs || 0, safeTimestamp);
  currentSession.timeline.push(snapshotFrame(status, eventLabel, safeTimestamp, eventDetail));
}

function simulatedTimeMsFromTimeline(timeline = currentSession?.timeline ?? []) {
  return timeline[timeline.length - 1]?.timestamp_ms ?? timeline.length * LOOP_INTERVAL_MS;
}

function summarizePatternMetrics(timeline = currentSession?.timeline ?? []) {
  const counters = {
    room_crossing_segments: 0,
    wall_follow_segments: 0,
    proximity_contacts: 0,
    obstacle_escape_attempts: 0,
    obstacle_escape_successes: 0,
    obstacle_escape_failures: 0,
    anti_loop_escapes: 0,
    backup_blocked_events: 0,
    turns: 0,
    obstacle_detections: 0,
    bumper_contacts: 0,
    wall_contacts: 0,
    obstacle_contacts: 0,
    long_turn_guards: 0,
    escape_turns: 0,
    dock_blocked_events: 0,
    dock_stuck_diagnostics: 0,
    stuck_diagnostics: 0,
    max_no_movement_with_wheels: 0,
    max_turning_without_movement: 0,
    max_no_movement_ms: 0,
    coverage_per_real_second: 0,
    coverage_per_simulated_second: 0,
  };
  let noMovementWithWheels = 0;
  let turningWithoutMovement = 0;
  let noMovementMs = 0;

  timeline.forEach((frame, index) => {
    const previous = index > 0 ? timeline[index - 1] : null;
    const frameDeltaMs = previous ? Math.max(1, frame.timestamp_ms - previous.timestamp_ms) : LOOP_INTERVAL_MS;
    if (frame.sensors.obstacle_detected) counters.obstacle_detections += 1;
    if (frame.event === "PROXIMITY_CONTACT" || frame.event === "BUMPER_CONTACT") {
      counters.bumper_contacts += 1;
      counters.proximity_contacts += 1;
    }
    if (frame.event === "ROOM_CROSSING_START") counters.room_crossing_segments += 1;
    if (frame.event === "WALL_FOLLOW_START") counters.wall_follow_segments += 1;
    if (frame.event === "OBSTACLE_BACKUP") counters.obstacle_escape_attempts += 1;
    if (frame.event === "OBSTACLE_ESCAPE_SUCCESS") counters.obstacle_escape_successes += 1;
    if (frame.event === "OBSTACLE_ESCAPE_FAILED") counters.obstacle_escape_failures += 1;
    if (frame.event === "ANTI_LOOP_ESCAPE") counters.anti_loop_escapes += 1;
    if (frame.event === "BACKUP_BLOCKED") counters.backup_blocked_events += 1;
    if (frame.event === "LONG_TURN_GUARD") counters.long_turn_guards += 1;
    if (frame.event === "ESCAPE_TURN") counters.escape_turns += 1;
    if (frame.event === "DOCK_BLOCKED") counters.dock_blocked_events += 1;
    if (frame.event === "DOCK_STUCK_DIAGNOSTIC") counters.dock_stuck_diagnostics += 1;
    if (frame.event === "FORWARD_FREEZE_DIAGNOSTIC" || frame.event === "BATCH_STOP_STUCK") counters.stuck_diagnostics += 1;
    if (["WALL_ALIGN", "WALL_RELEASE", "OBSTACLE_TURN_AWAY", "ANTI_LOOP_TURN", "TURN_START"].includes(frame.event)) counters.turns += 1;
    if (frame.event === "WALL_CONTACT") counters.wall_contacts += 1;
    if (frame.event === "OBSTACLE_CONTACT") counters.obstacle_contacts += 1;

    const demoLeft = frame.demo_left_wheel_speed ?? frame.left_wheel_speed;
    const demoRight = frame.demo_right_wheel_speed ?? frame.right_wheel_speed;
    const nonZeroWheels = demoLeft !== 0 || demoRight !== 0;
    const unchangedPosition = previous ? previous.x === frame.x && previous.y === frame.y : false;
    const phaseProgress =
      previous && frame.demo_navigation_phase != null && frame.demo_navigation_phase !== previous.demo_navigation_phase;
    const headingProgress =
      previous && Math.abs(normalizeAngle(frame.heading - previous.heading)) > 0.02;
    if (nonZeroWheels && unchangedPosition && !phaseProgress && !headingProgress) {
      noMovementWithWheels += 1;
      noMovementMs += frameDeltaMs;
    } else {
      noMovementWithWheels = 0;
      noMovementMs = 0;
    }
    if (demoLeft !== demoRight && unchangedPosition && !phaseProgress) {
      turningWithoutMovement += 1;
    } else {
      turningWithoutMovement = 0;
    }
    counters.max_no_movement_with_wheels = Math.max(counters.max_no_movement_with_wheels, noMovementWithWheels);
    counters.max_turning_without_movement = Math.max(counters.max_turning_without_movement, turningWithoutMovement);
    counters.max_no_movement_ms = Math.max(counters.max_no_movement_ms, noMovementMs);
  });

  const simulatedSeconds = Math.max(0.001, simulatedTimeMsFromTimeline(timeline) / 1000);
  const realSeconds = Math.max(0.001, (currentRealElapsedMs() ?? simulatedTimeMsFromTimeline(timeline)) / 1000);
  counters.coverage_per_real_second = Number((latestCoveragePercentage / realSeconds).toFixed(4));
  counters.coverage_per_simulated_second = Number((latestCoveragePercentage / simulatedSeconds).toFixed(4));

  return counters;
}

function summarizeTimeline() {
  const counters = summarizePatternMetrics(currentSession.timeline);
  const collisions = counters.bumper_contacts;

  return {
    total_frames: currentSession.timeline.length,
    total_simulated_time_ms: simulatedTimeMsFromTimeline(),
    collisions,
    obstacle_detections: counters.obstacle_detections,
    turns: counters.turns,
    coverage_percentage: Number(latestCoveragePercentage.toFixed(3)),
  };
}

function buildBaseLogObject() {
  const payload = {
    session_id: currentSession.sessionId,
    map_id: currentSession.mapId,
    created_at: currentSession.createdAt,
    map: {
      room_width: currentMap.roomWidth,
      room_height: currentMap.roomHeight,
      robot_radius: currentMap.robotRadius,
      initial_robot_pose: {
        x: currentMap.initialPose.x,
        y: currentMap.initialPose.y,
        heading: currentMap.initialPose.heading,
      },
      docking_station: {
        x: currentMap.dockingStation.x,
        y: currentMap.dockingStation.y,
      },
      obstacles: currentMap.obstacles.map((obstacle) => ({ ...obstacle })),
    },
    frontend_config: {
      simulation_speed: speedFactor(),
      sensor_thresholds: {
        obstacle_distance: PROXIMITY_DETECTION_RANGE_PX,
        obstacle_half_angle_rad: SENSOR_HALF_ANGLE,
        proximity_detection_range_px: PROXIMITY_DETECTION_RANGE_PX,
        proximity_contact_threshold_px: PROXIMITY_CONTACT_THRESHOLD_PX,
        bumper_contact_distance: PROXIMITY_CONTACT_THRESHOLD_PX,
        body_clearance_lookahead_px: BODY_CLEARANCE_LOOKAHEAD_PX,
        body_clearance_margin_px: BODY_CLEARANCE_MARGIN_PX,
        body_clearance_sample_count: BODY_CLEARANCE_SAMPLE_COUNT,
      },
      movement_substeps: {
        max_linear_substep_px: MAX_LINEAR_SUBSTEP_PX,
        max_angular_substep_rad: MAX_ANGULAR_SUBSTEP_RAD,
        max_substeps_per_frame: MAX_VISUAL_SUBSTEPS_PER_FRAME,
      },
      coverage_grid: {
        cols: COVERAGE_GRID.cols,
        rows: COVERAGE_GRID.rows,
        cell_width: ROOM.width / COVERAGE_GRID.cols,
        cell_height: ROOM.height / COVERAGE_GRID.rows,
      },
    },
    timeline: currentSession.timeline,
    coverage: {
      covered_cells: coverageCells(),
      coverage_percentage: Number(latestCoveragePercentage.toFixed(3)),
    },
    summary: summarizeTimeline(),
  };

  if (currentSession.batchId) {
    payload.batch_id = currentSession.batchId;
    payload.run_index = currentSession.runIndex;
    payload.batch_total_runs = currentSession.totalRuns;
    payload.batch_mode = true;
    payload.run_config = currentSession.runConfig;
  }

  return payload;
}

function normalizeTimelineForDump(payload) {
  if (!payload || !Array.isArray(payload.timeline)) {
    return payload;
  }

  let lastTimestamp = 0;

  payload.timeline = payload.timeline.map((frame, index) => {
    const nextFrame = { ...frame };
    let timestamp = Number(nextFrame.timestamp_ms);

    if (!Number.isFinite(timestamp)) {
      timestamp = lastTimestamp + 1;
    }

    if (index > 0 && timestamp < lastTimestamp) {
      timestamp = lastTimestamp + 1;
    }

    nextFrame.timestamp_ms = timestamp;
    nextFrame.frame_index = index;

    lastTimestamp = timestamp;
    return nextFrame;
  });

  lastRecordedTimestampMs = Math.max(lastRecordedTimestampMs || 0, lastTimestamp);
  return payload;
}

function logPayload() {
  return buildBaseLogObject();
}

function collisionSignature(detail) {
  if (!detail) return null;
  return `${detail.collision_target}:${detail.obstacle_id ?? ""}`;
}

function buildCollisionEventDetail(detail, moveResult) {
  if (!detail && !moveResult?.maxSubstepsReached && !moveResult?.maxAngularSubstepsReached) {
    return null;
  }

  const proximityDistance =
    detail?.distance_px ??
    moveResult?.proximityContactDetail?.distance_px ??
    null;
  return {
    collision_target: detail?.collision_target ?? null,
    obstacle_id: detail?.obstacle_id ?? null,
    distance_px: proximityDistance,
    max_linear_substep_px: MAX_LINEAR_SUBSTEP_PX,
    max_substeps_per_frame: MAX_VISUAL_SUBSTEPS_PER_FRAME,
    max_substeps_reached: Boolean(moveResult?.maxSubstepsReached),
    max_angular_substep_rad: MAX_ANGULAR_SUBSTEP_RAD,
    max_angular_substeps_per_frame: MAX_VISUAL_SUBSTEPS_PER_FRAME,
    max_angular_substeps_reached: Boolean(moveResult?.maxAngularSubstepsReached),
    linear_substeps: moveResult?.linearSubsteps ?? null,
    angular_substeps: moveResult?.angularSubsteps ?? null,
    applied_substeps: moveResult?.substeps ?? moveResult?.appliedSubsteps ?? null,
    heading_before:
      moveResult?.headingBefore == null ? null : Number(normalizeAngle(moveResult.headingBefore).toFixed(6)),
    heading_after:
      moveResult?.headingAfter == null ? null : Number(normalizeAngle(moveResult.headingAfter).toFixed(6)),
    target_heading:
      moveResult?.targetHeading == null ? null : Number(normalizeAngle(moveResult.targetHeading).toFixed(6)),
    heading_error_before:
      moveResult?.headingErrorBefore == null ? null : Number(moveResult.headingErrorBefore.toFixed(6)),
    heading_error_after:
      moveResult?.headingErrorAfter == null ? null : Number(moveResult.headingErrorAfter.toFixed(6)),
    crossed_target_heading: Boolean(moveResult?.headingTargetCrossed),
    heading_target_reached: Boolean(moveResult?.headingTargetReached),
    blocked_target: detail?.collision_target ?? null,
    contact_type: detail?.collision_target ?? null,
    contact_sensor_type: detail?.contact_sensor_type ?? "proximity",
    physical_bumper: false,
    proximity_contact: Boolean(detail?.proximity_contact ?? moveResult?.proximityContact),
    proximity_contact_threshold_px: PROXIMITY_CONTACT_THRESHOLD_PX,
  };
}

function detectLoopEvent(status, timestampMs, visualCommand = null) {
  const autoCleaning = status?.state === "CLEANING" && status?.cleaning_mode === "AUTO";

  if (autoCleaning || shouldUseLaneController(status)) {
    activeTurnEpisodeId = null;
    turnEpisodeStartedAtMs = null;
    lastMotionKind = autoCleaning ? "backend_auto" : "reactive_auto";
    lastTurnDirection = null;
    return {
      label: null,
      detail: null,
      motion: { kind: autoCleaning ? "backend_auto" : "reactive_auto", turnDirection: null },
    };
  }

  const motion = classifyMotion(status, visualCommand);
  let eventLabel = null;
  let eventDetail = null;

  if (motion.kind === "turning") {
    if (activeTurnEpisodeId == null) {
      turnEpisodeCounter += 1;
      activeTurnEpisodeId = turnEpisodeCounter;
      turnEpisodeStartedAtMs = timestampMs;
      eventLabel = "TURN_START";
      eventDetail = turnEpisodeDetail({ turn_direction: motion.turnDirection });
    } else {
      eventLabel = "TURNING";
      eventDetail = turnEpisodeDetail({ turn_direction: motion.turnDirection });
    }
  } else if (activeTurnEpisodeId != null) {
    eventLabel = "TURN_END";
    eventDetail = turnEpisodeDetail({ turn_direction: lastTurnDirection });
    activeTurnEpisodeId = null;
    turnEpisodeStartedAtMs = null;
  }

  lastMotionKind = motion.kind;
  lastTurnDirection = motion.turnDirection;
  return { label: eventLabel, detail: eventDetail, motion };
}

function maybeTriggerLongTurnGuard(status, motion, frameTimestampMs, moveResult) {
  const autoCleaning = status?.state === "CLEANING" && status?.cleaning_mode === "AUTO";
  if (autoCleaning) {
    escapePlan = null;
    return;
  }

  if (
    status.state !== "CLEANING" ||
    motion.kind !== "turning" ||
    activeTurnEpisodeId == null ||
    turnEpisodeStartedAtMs == null ||
    escapePlan ||
    moveResult.movementChanged ||
    frameTimestampMs - turnEpisodeStartedAtMs < LONG_TURN_GUARD_MS
  ) {
    return;
  }

  const forwardCollision = forwardProbeCollision(pose);
  queueTimelineEvent("LONG_TURN_GUARD", turnEpisodeDetail({
    guard_threshold_ms: LONG_TURN_GUARD_MS,
    collision_target: forwardCollision?.collision_target ?? null,
    obstacle_id: forwardCollision?.obstacle_id ?? null,
  }));

  if (!forwardCollision) {
    escapePlan = {
      mode: "forward_attempt",
      untilMs: frameTimestampMs + ESCAPE_FORWARD_ATTEMPT_MS,
      turnDirection: lastTurnDirection ?? "right",
      episodeId: activeTurnEpisodeId,
    };
    queueTimelineEvent("ESCAPE_FORWARD_ATTEMPT", turnEpisodeDetail({ phase: "forward_attempt" }));
    return;
  }

  escapePlan = {
    mode: "escape_turn",
    untilMs: frameTimestampMs + ESCAPE_TURN_MS,
    turnDirection: lastTurnDirection === "right" ? "left" : "right",
    episodeId: activeTurnEpisodeId,
  };
  queueTimelineEvent("ESCAPE_TURN", turnEpisodeDetail({
    phase: "escape_turn",
    collision_target: forwardCollision.collision_target,
    obstacle_id: forwardCollision.obstacle_id ?? null,
  }));
}

function resolveLoopEvent(moveResult, updatedStatus, motionEvent, visualCommand) {
  const queuedEvent = consumeQueuedTimelineEvent();
  if (queuedEvent) {
    return queuedEvent;
  }

  if (lastBackendState === "RETURNING_TO_DOCK" && updatedStatus.state === "CHARGING") {
    return {
      label: "CHARGING_STARTED",
      detail: dockingGuidanceDetail(updatedStatus, "CHARGING_STARTED"),
    };
  }

  if (moveResult.blockedForward || moveResult.proximityContact) {
    const contactDetail = moveResult.collisionDetail ?? moveResult.proximityContactDetail;
    if ((visualCommand?.source ?? "").startsWith("dock")) {
      maybeStartDockRecovery(updatedStatus, lastRecordedTimestampMs, contactDetail);
      return {
        label: "DOCK_BLOCKED",
        detail: {
          ...dockingGuidanceDetail(updatedStatus, "DOCK_BLOCKED", visualCommand?.targetHeading ?? null),
          ...buildCollisionEventDetail(contactDetail, moveResult),
        },
      };
    }

    const signature = collisionSignature(contactDetail) ?? `proximity:${contactDetail?.distance_px ?? "near"}`;
    const isNewBlockedContact = signature !== lastCollisionSignature || !waitingForTurnAfterBumper;
    lastCollisionSignature = signature;
    waitingForTurnAfterBumper = true;

    if (isNewBlockedContact) {
      waitingDiagnosticLogged = false;
      return {
        label: "PROXIMITY_CONTACT",
        detail: {
          ...phaseDetail(),
          ...buildCollisionEventDetail(contactDetail, moveResult),
          compatibility_event: "BUMPER_CONTACT",
          ...turnEpisodeDetail(),
        },
      };
    }
  }

  if (motionEvent.label === "TURN_END") {
    waitingForTurnAfterBumper = false;
    waitingDiagnosticLogged = false;
  }

  if (motionEvent.label === "TURN_START" || motionEvent.label === "TURNING") {
    waitingForTurnAfterBumper = false;
    waitingDiagnosticLogged = false;
    return {
      label: motionEvent.label,
      detail: motionEvent.detail,
    };
  }

  if (
    waitingForTurnAfterBumper &&
    motionEvent.motion.kind === "forward" &&
    !waitingDiagnosticLogged
  ) {
    waitingDiagnosticLogged = true;
    return {
      label: "BUMPER_REPORTED_WAITING_FOR_TURN",
      detail: {
        ...phaseDetail(),
        ...buildCollisionEventDetail(moveResult.collisionDetail ?? moveResult.proximityContactDetail, moveResult),
        ...turnEpisodeDetail(),
      },
    };
  }

  const silentForwardFreeze =
    updatedStatus.state === "CLEANING" &&
    motionEvent.motion.kind === "forward" &&
    !moveResult.movementChanged &&
    !updatedStatus.sensors.obstacle_detected &&
    !updatedStatus.sensors.bumper_pressed;

  if (silentForwardFreeze) {
    silentForwardFreezeFrames += 1;
  } else {
    silentForwardFreezeFrames = 0;
  }

  if (silentForwardFreezeFrames >= FORWARD_FREEZE_DIAGNOSTIC_THRESHOLD) {
    return {
      label: "FORWARD_FREEZE_DIAGNOSTIC",
      detail: {
        silent_forward_freeze_frames: silentForwardFreezeFrames,
      },
    };
  }

  return {
    label: motionEvent.label,
    detail: motionEvent.detail,
  };
}

async function sendCommand(endpoint, payload, eventLabel = null) {
  const body = payload ? JSON.stringify(payload) : undefined;
  const status = await requestJson(endpoint, {
    method: "POST",
    body,
  });
  renderStatus(status);
  if (eventLabel) {
    recordFrame(status, eventLabel, nextTimelineTimestamp());
  }
  setError("");
  return status;
}

async function clearTransientSensors() {
  await requestJson("/simulation/sensors", {
    method: "POST",
    body: JSON.stringify({
      obstacle_detected: false,
      bumper_pressed: false,
      proximity_contact: false,
      contact_type: null,
      wall_side: null,
      forward_clearance_blocked: false,
    }),
  });
  clearMotionEventState();
}

async function resetSimulationSession() {
  await clearTransientSensors();
  const status = await requestJson("/simulation/reset", { method: "POST" });
  sessionStartedRealMs = performance.now();
  resetPose();
  resetCoverage();
  currentSession.timeline = [];
  frameIndex = 0;
  clearMotionEventState();
  renderStatus(status);
  recordFrame(status, "RESET", nextTimelineTimestamp());
}

async function createNewMapSession(options = {}, batchContext = null) {
  currentMap = createMap(Date.now() + mapCounter * 13, options);
  currentSession = createSessionForMap(currentMap, batchContext);
  renderMap();
  renderSessionMeta();
  await clearTransientSensors();
  const status = await requestJson("/simulation/reset", { method: "POST" });
  sessionStartedRealMs = performance.now();
  resetPose();
  resetCoverage();
  currentSession.timeline = [];
  frameIndex = 0;
  clearMotionEventState();
  renderStatus(status);
  recordFrame(status, "NEW_MAP", nextTimelineTimestamp());
  setDumpMessage(`Active session ${currentSession.sessionId}`);
}

function classifyMotion(status) {
  const left = lastVisualCommandTelemetry?.demo_left_wheel_speed ?? status.left_wheel_speed;
  const right = lastVisualCommandTelemetry?.demo_right_wheel_speed ?? status.right_wheel_speed;
  if (left === 0 && right === 0) {
    return { kind: "idle", turnDirection: null };
  }
  if (left === right) {
    return { kind: "forward", turnDirection: null };
  }
  return {
    kind: "turning",
    turnDirection: left > right ? "right" : "left",
  };
}

function currentBatchRunTimestampMs() {
  return currentSession ? simulatedTimeMsFromTimeline(currentSession.timeline) : 0;
}

function currentBatchRunConfig() {
  return batchState ? buildBatchRunConfig(batchState.config) : null;
}

async function saveManualLogDump() {
  if (!currentSession) {
    throw new Error("No active session to dump");
  }

  if (currentSession.timeline.length === 0 && latestStatus) {
    recordFrame(latestStatus, "LOG_DUMP_REQUEST", nextTimelineTimestamp());
  }

  const payload = normalizeTimelineForDump(buildBaseLogObject());
  delete payload.batch_id;
  delete payload.run_index;
  delete payload.batch_total_runs;
  delete payload.batch_mode;
  delete payload.run_config;

  if (
    !payload.session_id ||
    !payload.created_at ||
    !payload.map ||
    !payload.frontend_config ||
    !Array.isArray(payload.timeline)
  ) {
    throw new Error("Manual log payload is missing required data before sending");
  }

  const response = await requestJson("/simulation/log-dump", {
    method: "POST",
    body: JSON.stringify(payload),
  });
  if (!response?.saved_path) {
    throw new Error("Log dump did not return a saved_path");
  }
  logPathEl.textContent = response.saved_path;
  currentSession.targetPath = response.saved_path;
  setDumpMessage(`Dump saved to ${response.saved_path}`);
  return response.saved_path;
}

async function saveBatchRunLog(activeRun) {
  const response = await requestJson("/simulation/log-dump", {
    method: "POST",
    body: JSON.stringify({
      batch_id: batchState.batchId,
      session_id: currentSession.sessionId,
      log: normalizeTimelineForDump(buildBaseLogObject()),
    }),
  });
  if (!response?.saved_path) {
    throw new Error("Batch log dump did not return a saved_path");
  }
  currentSession.targetPath = response.saved_path;
  logPathEl.textContent = response.saved_path;
  batchState.lastSavedLogPath = response.saved_path;
  activeRun.savedPath = response.saved_path;
  updateBatchUi();
  return response.saved_path;
}

function arraySum(items, selector) {
  return items.reduce((sum, item) => sum + selector(item), 0);
}

function mostCommonString(values) {
  if (values.length === 0) return null;
  const counts = new Map();
  values.forEach((value) => {
    counts.set(value, (counts.get(value) ?? 0) + 1);
  });
  return [...counts.entries()].sort((a, b) => b[1] - a[1])[0]?.[0] ?? null;
}

function topRuns(runs, selector, direction = "desc", limit = BATCH_MAX_WORST_RUNS, filter = null) {
  const filtered = filter ? runs.filter(filter) : [...runs];
  return filtered
    .sort((left, right) => {
      const delta = selector(left) - selector(right);
      return direction === "asc" ? delta : -delta;
    })
    .slice(0, limit);
}

function buildBatchSummary(batch) {
  const runs = [...batch.runs];
  const averageCoverage = runs.length > 0 ? arraySum(runs, (run) => run.final_coverage_percentage) / runs.length : 0;
  const averageSimulatedTime =
    runs.length > 0 ? Math.round(arraySum(runs, (run) => run.total_simulated_time_ms) / runs.length) : 0;
  const dockingRuns = runs.filter((run) => run.dock_attempted);
  const dockingSuccessRate =
    batch.config.return_to_dock_after_run && dockingRuns.length > 0
      ? Number((dockingRuns.filter((run) => run.docking_succeeded).length / dockingRuns.length).toFixed(4))
      : null;

  return {
    batch_id: batch.batchId,
    created_at: batch.createdAt,
    finished_at: new Date().toISOString(),
    requested_runs: batch.requestedRuns,
    completed_runs: batch.completedRuns,
    cancelled: batch.cancelled,
    config: {
      run_count: batch.config.run_count,
      max_simulated_time_ms: batch.config.max_simulated_time_ms,
      target_coverage: batch.config.target_coverage,
      simulation_speed: batch.config.simulation_speed,
      return_to_dock_after_run: batch.config.return_to_dock_after_run,
      stop_conditions: batch.config.stop_conditions,
      obstacle_count: batch.config.obstacle_count,
    },
    runs,
    aggregate_metrics: {
      average_coverage: Number(averageCoverage.toFixed(4)),
      min_coverage: runs.length > 0 ? Math.min(...runs.map((run) => run.final_coverage_percentage)) : 0,
      max_coverage: runs.length > 0 ? Math.max(...runs.map((run) => run.final_coverage_percentage)) : 0,
      average_simulated_time: averageSimulatedTime,
      docking_success_rate: dockingSuccessRate,
      most_common_finish_reason: mostCommonString(runs.map((run) => run.finish_reason)),
      total_bumper_contacts: arraySum(runs, (run) => run.bumper_contacts),
      total_proximity_contacts: arraySum(runs, (run) => run.proximity_contacts ?? run.bumper_contacts),
      total_wall_contacts: arraySum(runs, (run) => run.wall_contacts),
      total_obstacle_contacts: arraySum(runs, (run) => run.obstacle_contacts),
      total_room_crossing_segments: arraySum(runs, (run) => run.room_crossing_segments ?? 0),
      total_wall_follow_segments: arraySum(runs, (run) => run.wall_follow_segments ?? 0),
      total_obstacle_escape_attempts: arraySum(runs, (run) => run.obstacle_escape_attempts ?? 0),
      total_obstacle_escape_successes: arraySum(runs, (run) => run.obstacle_escape_successes ?? 0),
      total_obstacle_escape_failures: arraySum(runs, (run) => run.obstacle_escape_failures ?? 0),
      total_anti_loop_escapes: arraySum(runs, (run) => run.anti_loop_escapes ?? 0),
      total_backup_blocked_events: arraySum(runs, (run) => run.backup_blocked_events ?? 0),
      total_long_turn_guards: arraySum(runs, (run) => run.long_turn_guards),
      total_escape_turns: arraySum(runs, (run) => run.escape_turns),
      total_stuck_diagnostics: arraySum(runs, (run) => run.stuck_diagnostics),
      total_dock_blocked_events: arraySum(runs, (run) => run.dock_blocked_events),
      total_dock_stuck_diagnostics: arraySum(runs, (run) => run.dock_stuck_diagnostics),
    },
    worst_runs: {
      lowest_coverage: topRuns(runs, (run) => run.final_coverage_percentage, "asc"),
      highest_wall_contacts: topRuns(runs, (run) => run.wall_contacts),
      highest_obstacle_contacts: topRuns(runs, (run) => run.obstacle_contacts),
      most_anti_loop_escapes: topRuns(runs, (run) => run.anti_loop_escapes ?? 0),
      most_stuck_diagnostics: topRuns(runs, (run) => run.stuck_diagnostics),
      failed_docking: topRuns(runs, (run) => run.dock_blocked_events + run.dock_stuck_diagnostics, "desc", BATCH_MAX_WORST_RUNS, (run) => run.dock_attempted && !run.docking_succeeded),
      longest_no_movement_streak: topRuns(runs, (run) => run.max_no_movement_ms ?? 0),
      longest_turning_without_movement_streak: topRuns(runs, (run) => run.max_consecutive_turning_frames_without_xy_change),
    },
  };
}

function buildRunSummary(activeRun, finishReason) {
  const timeline = currentSession.timeline;
  const finalFrame = timeline[timeline.length - 1] ?? snapshotFrame(latestStatus ?? {
    left_wheel_speed: 0,
    right_wheel_speed: 0,
    state: activeRun.runState ?? "STANDBY",
    cleaning_mode: "AUTO",
    current_error: null,
    battery_percent: 0,
    is_charging: false,
    sensors: {
      obstacle_detected: false,
      drop_off_detected: false,
      bumper_pressed: false,
      dust_container_full: false,
      wheel_stuck: false,
      brush_stuck: false,
      top_cover_open: false,
    },
  });
  const counters = summarizePatternMetrics(timeline);

  return {
    batch_id: batchState.batchId,
    run_index: activeRun.runIndex,
    session_id: currentSession.sessionId,
    map_id: currentSession.mapId,
    log_path: activeRun.savedPath ?? null,
    created_at: currentSession.createdAt,
    finished_at: new Date().toISOString(),
    finish_reason: finishReason,
    final_state: finalFrame.state,
    final_coverage_percentage: Number(latestCoveragePercentage.toFixed(3)),
    total_frames: timeline.length,
    total_simulated_time_ms: simulatedTimeMsFromTimeline(timeline),
    total_real_time_ms: activeRun.startedRealMs != null ? Math.round(performance.now() - activeRun.startedRealMs) : null,
    obstacle_count: currentMap.obstacles.length,
    obstacle_detections: counters.obstacle_detections,
    bumper_contacts: counters.bumper_contacts,
    proximity_contacts: counters.proximity_contacts,
    wall_contacts: counters.wall_contacts,
    obstacle_contacts: counters.obstacle_contacts,
    turns: counters.turns,
    room_crossing_segments: counters.room_crossing_segments,
    wall_follow_segments: counters.wall_follow_segments,
    obstacle_escape_attempts: counters.obstacle_escape_attempts,
    obstacle_escape_successes: counters.obstacle_escape_successes,
    obstacle_escape_failures: counters.obstacle_escape_failures,
    anti_loop_escapes: counters.anti_loop_escapes,
    backup_blocked_events: counters.backup_blocked_events,
    long_turn_guards: counters.long_turn_guards,
    escape_turns: counters.escape_turns,
    dock_attempted: activeRun.dockRequested,
    docking_succeeded: finishReason === "DOCKED",
    reached_charging: finalFrame.state === "CHARGING",
    dock_blocked_events: counters.dock_blocked_events,
    dock_stuck_diagnostics: counters.dock_stuck_diagnostics,
    stuck_diagnostics: counters.stuck_diagnostics,
    max_consecutive_no_movement_frames_with_nonzero_wheels: counters.max_no_movement_with_wheels,
    max_consecutive_turning_frames_without_xy_change: counters.max_turning_without_movement,
    max_no_movement_ms: counters.max_no_movement_ms,
    coverage_per_real_second: counters.coverage_per_real_second,
    coverage_per_simulated_second: counters.coverage_per_simulated_second,
    final_x: finalFrame.x,
    final_y: finalFrame.y,
    final_heading: finalFrame.heading,
    log_save_error: activeRun.logSaveError ?? null,
  };
}

async function persistBatchSummary() {
  const summary = buildBatchSummary(batchState);
  const response = await requestJson("/simulation/batch-summary", {
    method: "POST",
    body: JSON.stringify({
      batch_id: batchState.batchId,
      summary,
    }),
  });
  if (!response?.saved_path) {
    throw new Error("Batch summary did not return a saved_path");
  }
  batchState.summarySavedPath = response.saved_path;
  updateBatchUi();
  return response.saved_path;
}

async function finalizeBatchRun(finishReason, detail = null) {
  const activeRun = batchState?.activeRun;
  if (!activeRun || activeRun.finishing) {
    return;
  }
  activeRun.finishing = true;
  activeRun.finishReason = finishReason;
  const finishTimestamp = nextTimelineTimestamp();
  const stopEventLabel =
    finishReason === "TIME_LIMIT_REACHED"
      ? "BATCH_STOP_TIME_LIMIT"
      : finishReason === "COVERAGE_REACHED"
        ? "BATCH_STOP_COVERAGE_REACHED"
        : finishReason === "DOCKED"
          ? "BATCH_STOP_DOCKED"
          : finishReason === "DOCKING_FAILED_OR_TIMEOUT"
            ? "BATCH_STOP_DOCKING_FAILED"
            : finishReason === "STUCK_DIAGNOSTIC"
              ? "BATCH_STOP_STUCK"
              : finishReason === "USER_CANCELLED"
                ? "BATCH_STOP_USER_CANCELLED"
                : "BATCH_COMMAND_ERROR";
  if (latestStatus) {
    recordFrame(latestStatus, stopEventLabel, finishTimestamp, detail);
    recordFrame(
      latestStatus,
      "BATCH_RUN_END",
      nextTimelineTimestamp(),
      {
        finish_reason: finishReason,
      },
    );
  }

  try {
    await saveBatchRunLog(activeRun);
  } catch (error) {
    activeRun.logSaveError = error.message;
    batchState.lastSavedLogPath = null;
    setError(error.message);
  }

  const summary = buildRunSummary(activeRun, finishReason);
  if (activeRun.logSaveError) {
    summary.log_path = null;
    summary.log_save_error = activeRun.logSaveError;
  }
  batchState.runs.push(summary);
  batchState.completedRuns += 1;
  batchState.lastFinishReason = finishReason;
  if (["STUCK_DIAGNOSTIC", "DOCKING_FAILED_OR_TIMEOUT", "COMMAND_ERROR"].includes(finishReason)) {
    batchState.failedOrStuckRuns += 1;
  }
  batchState.activeRun = null;
  updateBatchUi();

  if (batchState.stopRequested || batchState.completedRuns >= batchState.requestedRuns) {
    batchState.cancelled = batchState.stopRequested;
    batchState.completed = true;
    await persistBatchSummary();
    setManualControlsDisabled(false);
    setDumpMessage(
      batchState.cancelled
        ? `Batch cancelled. Summary saved to ${batchState.summarySavedPath}`
        : `Batch complete. Summary saved to ${batchState.summarySavedPath}`,
    );
    updateBatchUi();
    return;
  }

  await startNextBatchRun();
}

async function handleBatchCommandError(error, context = {}) {
  if (!batchState?.activeRun) {
    throw error;
  }
  await finalizeBatchRun("COMMAND_ERROR", {
    error: error.message,
    context: context.label ?? "batch_command",
  });
}

function updateBatchRunMetrics(updatedStatus, deltaMs, loopEvent = null) {
  const activeRun = batchState?.activeRun;
  if (!activeRun) return;

  activeRun.lastCoveragePercentage = latestCoveragePercentage;
  activeRun.lastSimulatedTimeMs = currentBatchRunTimestampMs();
  activeRun.lastElapsedRealMs =
    activeRun.startedRealMs != null ? Math.round(performance.now() - activeRun.startedRealMs) : 0;
  activeRun.runState = updatedStatus.state;

  const finalFrame = currentSession.timeline[currentSession.timeline.length - 1];
  const previousFrame = currentSession.timeline[currentSession.timeline.length - 2];
  if (previousFrame && finalFrame) {
    const unchanged = previousFrame.x === finalFrame.x && previousFrame.y === finalFrame.y;
    const demoLeft = finalFrame.demo_left_wheel_speed ?? finalFrame.left_wheel_speed;
    const demoRight = finalFrame.demo_right_wheel_speed ?? finalFrame.right_wheel_speed;
    const nonZeroWheels = demoLeft !== 0 || demoRight !== 0;
    const turning = demoLeft !== demoRight;
    const phaseProgress =
      finalFrame.demo_navigation_phase != null &&
      finalFrame.demo_navigation_phase !== previousFrame.demo_navigation_phase;
    const headingProgress = Math.abs(normalizeAngle(finalFrame.heading - previousFrame.heading)) > 0.03;
    const currentRecoverySignature = recoverySignature({
      demo_navigation_phase: finalFrame.demo_navigation_phase,
      recovery_attempts: laneController?.recoveryAttempts ?? finalFrame.event_detail?.recovery_attempts ?? null,
      escape_side: laneController?.escapeSide ?? finalFrame.event_detail?.escape_side ?? null,
      wall_side: laneController?.wallSide ?? finalFrame.event_detail?.wall_side ?? null,
    });
    const recoveryProgress =
      currentRecoverySignature != null &&
      currentRecoverySignature !== activeRun.lastRecoverySignature;
    activeRun.lastRecoverySignature = currentRecoverySignature;
    const recoveryEvent =
      loopEvent?.label != null &&
      [
        "PROXIMITY_CONTACT",
        "WALL_CONTACT",
        "OBSTACLE_CONTACT",
        "OBSTACLE_BACKUP",
        "ANTI_LOOP_ESCAPE",
        "BACKUP_BLOCKED",
        "MICROSTEP_COLLISION",
      ].includes(loopEvent.label);
    if (recoveryEvent) {
      activeRun.recoveryGraceUntilMs = activeRun.lastSimulatedTimeMs + BATCH_RECOVERY_GRACE_MS;
    }
    const inRecoveryGrace =
      activeRun.recoveryGraceUntilMs != null && activeRun.lastSimulatedTimeMs <= activeRun.recoveryGraceUntilMs;
    if (unchanged && nonZeroWheels && !phaseProgress && !headingProgress && !recoveryProgress && !inRecoveryGrace) {
      activeRun.noMovementMs += deltaMs;
    } else {
      activeRun.noMovementMs = 0;
    }
    if (unchanged && turning && !phaseProgress && !headingProgress && !recoveryProgress && !inRecoveryGrace) {
      activeRun.turningWithoutMovementFrames += 1;
    } else {
      activeRun.turningWithoutMovementFrames = 0;
    }
  }
}

function shouldStopForBatchStuck(loopEvent) {
  const activeRun = batchState?.activeRun;
  if (!activeRun || !batchState.config.stop_conditions.stuck_diagnostic) {
    return null;
  }

  if (activeRun.noMovementMs >= BATCH_NO_MOVEMENT_LIMIT_MS) {
    return {
      reason: activeRun.awaitingDockOutcome ? "DOCKING_FAILED_OR_TIMEOUT" : "STUCK_DIAGNOSTIC",
      detail: { no_movement_ms: activeRun.noMovementMs },
    };
  }

  if (loopEvent?.label === "LONG_TURN_GUARD") {
    activeRun.longTurnGuardCount += 1;
    if (activeRun.longTurnGuardCount >= BATCH_MAX_LONG_TURN_GUARDS) {
      return {
        reason: "STUCK_DIAGNOSTIC",
        detail: { long_turn_guards: activeRun.longTurnGuardCount },
      };
    }
  }

  if (loopEvent?.label === "DOCK_BLOCKED") {
    activeRun.dockBlockedCount += 1;
    if (activeRun.dockBlockedCount >= BATCH_MAX_DOCK_BLOCKED_EVENTS) {
      return {
        reason: "DOCKING_FAILED_OR_TIMEOUT",
        detail: { dock_blocked_events: activeRun.dockBlockedCount },
      };
    }
  }

  if (loopEvent?.label === "DOCK_STUCK_DIAGNOSTIC") {
    activeRun.dockStuckCount += 1;
    if (activeRun.dockStuckCount >= BATCH_MAX_DOCK_STUCK_EVENTS) {
      return {
        reason: "DOCKING_FAILED_OR_TIMEOUT",
        detail: { dock_stuck_diagnostics: activeRun.dockStuckCount },
      };
    }
  }

  return null;
}

async function maybeRequestReturnToDock(updatedStatus) {
  const activeRun = batchState?.activeRun;
  if (!activeRun || activeRun.dockRequested || !batchState.config.return_to_dock_after_run) {
    return false;
  }
  activeRun.dockRequested = true;
  activeRun.awaitingDockOutcome = true;
  try {
    const status = await sendCommand("/commands/return-to-dock", undefined, "RETURN_TO_DOCK");
    recordFrame(status, "BATCH_RETURN_TO_DOCK_REQUESTED", nextTimelineTimestamp(), {
      batch_id: batchState.batchId,
      run_index: activeRun.runIndex,
    });
    activeRun.runState = status.state;
    updateBatchUi();
    return true;
  } catch (error) {
    await handleBatchCommandError(error, { label: "return_to_dock" });
    return true;
  }
}

async function processBatchFrame(updatedStatus, deltaMs, loopEvent) {
  const activeRun = batchState?.activeRun;
  if (!activeRun) return;

  updateBatchRunMetrics(updatedStatus, deltaMs, loopEvent);
  updateBatchUi();

  if (batchState.stopRequested) {
    await finalizeBatchRun("USER_CANCELLED", { stopped_by_user: true });
    return;
  }

  const stuckStop = shouldStopForBatchStuck(loopEvent);
  if (stuckStop) {
    await finalizeBatchRun(stuckStop.reason, stuckStop.detail);
    return;
  }

  if (activeRun.awaitingDockOutcome) {
    if (updatedStatus.state === "CHARGING") {
      await finalizeBatchRun("DOCKED", { reached_charging: true });
    }
    return;
  }

  const stopForTime =
    batchState.config.stop_conditions.time_limit &&
    activeRun.lastElapsedRealMs >= batchState.config.max_simulated_time_ms;
  const stopForCoverage =
    batchState.config.stop_conditions.coverage_threshold &&
    latestCoveragePercentage >= batchState.config.target_coverage;

  if (!stopForTime && !stopForCoverage) {
    return;
  }

  if (batchState.config.return_to_dock_after_run) {
    await maybeRequestReturnToDock(updatedStatus);
    return;
  }

  if (stopForTime) {
      await finalizeBatchRun("TIME_LIMIT_REACHED", { max_real_time_ms: batchState.config.max_simulated_time_ms });
    return;
  }

  if (stopForCoverage) {
    await finalizeBatchRun("COVERAGE_REACHED", { target_coverage: batchState.config.target_coverage });
  }
}

async function startNextBatchRun() {
  const nextRunIndex = batchState.completedRuns + 1;
  const runConfig = currentBatchRunConfig();
  batchState.activeRun = {
    runIndex: nextRunIndex,
    sessionId: null,
    runState: "Preparing",
    finishReason: null,
    lastCoveragePercentage: 0,
    lastSimulatedTimeMs: 0,
    lastElapsedRealMs: 0,
    dockRequested: false,
    awaitingDockOutcome: false,
    noMovementMs: 0,
    turningWithoutMovementFrames: 0,
    longTurnGuardCount: 0,
    dockBlockedCount: 0,
    dockStuckCount: 0,
    recoveryGraceUntilMs: 0,
    lastRecoverySignature: null,
    savedPath: null,
    logSaveError: null,
    startedRealMs: performance.now(),
    finishing: false,
  };
  updateBatchUi();

  try {
    currentMap = createMap(Date.now() + mapCounter * 13, {
      obstacleCount: batchState.config.obstacle_count,
    });
    currentSession = createSessionForMap(currentMap, {
      batchId: batchState.batchId,
      runIndex: nextRunIndex,
      totalRuns: batchState.requestedRuns,
      runConfig,
    });
    batchState.activeRun.sessionId = currentSession.sessionId;
    renderMap();
    renderSessionMeta();
    await clearTransientSensors();
    const resetStatus = await requestJson("/simulation/reset", { method: "POST" });
    sessionStartedRealMs = performance.now();
    resetPose();
    resetCoverage();
    currentSession.timeline = [];
    frameIndex = 0;
    clearMotionEventState();
    renderStatus(resetStatus);
    recordFrame(resetStatus, "BATCH_RUN_START", nextTimelineTimestamp(), {
      batch_id: batchState.batchId,
      run_index: nextRunIndex,
      batch_total_runs: batchState.requestedRuns,
      run_config: runConfig,
    });
    await sendCommand("/commands/start", undefined, "START");
    setDumpMessage(`Batch ${batchState.batchId}: running ${currentSession.sessionId}`);
    updateBatchUi();
  } catch (error) {
    await handleBatchCommandError(error, { label: "start_run" });
  }
}

async function startBatch() {
  if (batchState?.activeRun) {
    throw new Error("A batch is already running");
  }
  const config = readBatchConfig();
  batchState = {
    batchId: createBatchId(),
    createdAt: new Date().toISOString(),
    config,
    requestedRuns: config.run_count,
    completedRuns: 0,
    failedOrStuckRuns: 0,
    runs: [],
    activeRun: null,
    stopRequested: false,
    cancelled: false,
    completed: false,
    lastSavedLogPath: null,
    summarySavedPath: null,
    lastFinishReason: null,
  };
  setManualControlsDisabled(true);
  updateBatchUi();
  await startNextBatchRun();
}

async function stopBatch() {
  if (!batchState) {
    return;
  }
  batchState.stopRequested = true;
  if (!batchState.activeRun) {
    batchState.cancelled = true;
    batchState.completed = true;
    await persistBatchSummary();
    setManualControlsDisabled(false);
    updateBatchUi();
  }
}

function clearBatchResults() {
  if (batchState?.activeRun) {
    throw new Error("Stop the active batch before clearing results");
  }
  batchState = null;
  updateBatchUi();
}

async function handleButtonClick(event) {
  const button = event.target.closest("button");
  if (!button) return;

  try {
    if (button.id === "start-batch") {
      await startBatch();
      return;
    }

    if (button.id === "stop-batch") {
      await stopBatch();
      return;
    }

    if (button.id === "clear-batch-results") {
      clearBatchResults();
      return;
    }

    if (button.id === "reset-cleaning") {
      await resetSimulationSession();
      return;
    }

    if (button.id === "generate-map") {
      await createNewMapSession();
      return;
    }

    if (button.id === "generate-log-dump") {
      if (latestStatus) {
        recordFrame(latestStatus, "LOG_DUMP", nextTimelineTimestamp());
      }
      await saveManualLogDump();
      return;
    }

    const command = button.dataset.command;
    const manual = button.dataset.manual;

    if (command) {
      const eventLabel =
        command === "start"
          ? "START"
          : command === "pause"
            ? "PAUSE"
            : command === "stop"
              ? "STOP"
              : command === "clear-error"
                ? "CLEAR_ERROR"
                : "RETURN_TO_DOCK";
      const endpoint =
        command === "return-to-dock" ? "/commands/return-to-dock" : `/commands/${command}`;
      if (command === "stop") {
        await clearTransientSensors();
      }
      try {
        await sendCommand(endpoint, undefined, eventLabel);
      } catch (error) {
        if (command === "return-to-dock" && latestStatus) {
          recordFrame(latestStatus, "RETURN_TO_DOCK_REJECTED", nextTimelineTimestamp(), {
            error: error.message,
          });
        }
        throw error;
      }
      if (command === "stop") {
        clearMotionEventState();
      }
      return;
    }

    if (manual) {
      const speed = manual === "STOP" ? 0 : MANUAL_SPEED;
      await sendCommand(
        "/commands/manual-move",
        {
          direction: manual,
          speed,
          duration_ms: MANUAL_DURATION_MS,
        },
        `MANUAL_${manual}`,
      );
    }
  } catch (error) {
    setError(error.message);
  }
}

async function runLoop(timestamp) {
  if (isTicking) {
    return;
  }

  isTicking = true;
  setLoopIndicator("Syncing simulation...", true);

  try {
    const elapsedMs = lastLoopRealTimestamp == null ? LOOP_INTERVAL_MS : Math.max(1, timestamp - lastLoopRealTimestamp);
    lastLoopRealTimestamp = timestamp;
    const deltaMs = clamp(Math.round(elapsedMs * speedFactor()), 1, 1000);
    const status = await requestJson("/status");
    const frameTimestampMs = nextTimelineTimestamp(deltaMs);
    const executedVisualCommand = buildVisualCommand(status, frameTimestampMs);
    const autoCleaning = status.state === "CLEANING" && status.cleaning_mode === "AUTO";
    lastVisualCommandTelemetry = {
      backend_left_wheel_speed: status.left_wheel_speed,
      backend_right_wheel_speed: status.right_wheel_speed,
      demo_left_wheel_speed: autoCleaning ? null : executedVisualCommand?.left ?? null,
      demo_right_wheel_speed: autoCleaning ? null : executedVisualCommand?.right ?? null,
      demo_navigation_phase: status.auto_navigation_phase ?? null,
    };
    const moveResult = updatePoseFromStatus(status, frameTimestampMs, executedVisualCommand);
    if (!autoCleaning) {
      advanceLaneController(status, frameTimestampMs, moveResult);
    }
    if (escapePlan?.mode === "forward_attempt" && (moveResult.movementChanged || moveResult.blockedMovement || frameTimestampMs >= escapePlan.untilMs)) {
      escapePlan = null;
    }
    applyLocalBumperState(moveResult);

    const signals = computeSimulationSignals();
    if (status.state === "RETURNING_TO_DOCK" && signals.docking.dock_detected) {
      queueTimelineEvent("DOCK_DETECTED", dockingGuidanceDetail(status, "DOCK_DETECTED", executedVisualCommand?.targetHeading ?? null));
    }
    await requestJson("/simulation/sensors", {
      method: "POST",
      body: JSON.stringify(signals.sensors),
    });
    await requestJson("/simulation/docking", {
      method: "POST",
      body: JSON.stringify(signals.docking),
    });

    const updatedStatus = await requestJson("/simulation/tick", {
      method: "POST",
      body: JSON.stringify({ delta_ms: deltaMs }),
    });
    renderStatus(updatedStatus);
    maybeQueueBackendAutoPhaseEvent(updatedStatus);

    const motionEvent = detectLoopEvent(updatedStatus, frameTimestampMs, executedVisualCommand);
    maybeTriggerLongTurnGuard(updatedStatus, motionEvent.motion, frameTimestampMs, moveResult);
    const loopEvent = resolveLoopEvent(moveResult, updatedStatus, motionEvent, executedVisualCommand);
    const loopDetail =
      loopEvent.detail ??
      ((executedVisualCommand?.source === "dock_guidance" || executedVisualCommand?.source?.startsWith("dock_recovery")) && loopEvent.label
        ? dockingGuidanceDetail(updatedStatus, executedVisualCommand.phase ?? "DOCK_APPROACH", executedVisualCommand.targetHeading ?? null)
        : null);
    recordFrame(updatedStatus, loopEvent.label, frameTimestampMs, loopDetail);
    lastBackendState = updatedStatus.state;
    await processBatchFrame(updatedStatus, deltaMs, { label: loopEvent.label, detail: loopDetail });

    setError("");
    setLoopIndicator("Running");
  } catch (error) {
    setError(error.message);
    setLoopIndicator("Running with errors");
    if (batchState?.activeRun) {
      await handleBatchCommandError(error, { label: "run_loop" });
    }
  } finally {
    isTicking = false;
  }
}

async function bootstrap() {
  document.body.addEventListener("click", handleButtonClick);
  updateBatchUi();
  setManualControlsDisabled(false);

  try {
    await createNewMapSession();
    const status = await requestJson("/status");
    renderStatus(status);
  } catch (error) {
    setError(error.message);
  }

  setInterval(() => {
    runLoop(performance.now());
  }, LOOP_INTERVAL_MS);
}

bootstrap();
