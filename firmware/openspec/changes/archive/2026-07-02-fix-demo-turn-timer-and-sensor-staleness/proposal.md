## Why

El log actual del demo muestra una falla concreta: despues de `START`, el robot entra en un patron de giro con wheel speeds `42/-42`, el heading sigue cambiando, pero `x/y` quedan congelados y la timeline repite eventos de giro durante muchos frames. Eso indica que el timer de giro `AUTO` no esta expirando con tiempo simulado real o que el frontend sigue reinyectando sensores/eventos stale en cada ciclo.

Este cambio propone una correccion acotada del simulador/demo para que `/simulation/tick` avance el clock simulado, los turn windows y deadlines manuales puedan expirar, y la UI/log reflejen una transicion limpia entre `TURN_START`, frames de giro normales y reanudacion de avance.

## What Changes

- Extender `POST /simulation/tick` para aceptar `delta_ms` demo-only, avanzar `SimulatedClock` y mantener compatibilidad con un default seguro cuando el payload no se envia.
- Asegurar que el comportamiento `AUTO` timed-turn expire correctamente, no se reinicie cada frame por sensores stale y solo reprograme un nuevo turn cuando el bloqueo siga presente despues de reevaluar.
- Ajustar el frontend para enviar `delta_ms` speed-adjusted en cada tick, limpiar flags stale tras stop/reset/new map, y corregir el etiquetado de eventos de giro para no registrar `TURN_START` o `ANTI_LOOP_ESCAPE` falsos en cada frame.
- Ampliar tests backend/HTTP para expiracion de timers `AUTO` y `MANUAL`, validacion de `delta_ms` y ausencia de spin infinito con `x/y` congelados.
- Actualizar README para documentar tiempo simulado, `delta_ms`, timers y limpieza de sensores stale.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `command-protocol`: permitir `delta_ms` demo-only en `/simulation/tick` sin tocar la API principal.
- `sensor-actuator-control`: reforzar expiracion de timed turns y evitar retrigger frame-a-frame por sensores stale.
- `demo-frontend`: enviar tiempo simulado por tick, limpiar flags stale, corregir labels de eventos y reanudar movimiento real despues del giro.
- `simulation-and-testing`: agregar cobertura para clock advancement, expiracion de turn windows/manual deadlines y validacion de logs/eventos del demo.

## Impact

- `src/api/models.rs`
- `src/api/routes.rs`
- `src/application/controller.rs`
- `src/api/assets/demo.js`
- `tests/http_api.rs`
- `tests/robot_controller.rs`
- `README.md`
- deltas OpenSpec para command protocol, demo frontend, sensor/actuator control y testing
