## Context

El demo local ya corre una simulacion visual con mapa aleatorio, sensores frontend-owned, coverage painting, session logs y helpers `/simulation/*`. Eso alcanza para inspeccionar una corrida, pero no para juntar evidencia repetible sobre muchos mapas sin intervencion manual.

La necesidad nueva no es mejorar la navegacion del robot ni agregar inteligencia espacial real. La necesidad es operar el simulador actual en modo batch para recolectar logs comparables, con condiciones de fin claras y resumenes compactos que despues puedan enviarse a otro analisis.

Ademas, este repo todavia no tiene un spec canonico `demo-frontend` en `openspec/specs`. Por eso los requisitos funcionales del runner batch del demo deben seguir documentados bajo `simulation-and-testing`, mientras que los endpoints demo-only y sus invariantes de persistencia van en `command-protocol`.

## Goals / Non-Goals

**Goals:**
- Permitir ejecutar N corridas consecutivas desde `/demo` sin controles manuales entre una y otra.
- Hacer que cada corrida produzca un `log.json` compatible con el dump existente, mas metadata batch suficiente para correlacionar resultados.
- Persistir un `batch-summary.json` independiente del UI con resumen por corrida y metricas agregadas.
- Garantizar que ninguna corrida batch quede ejecutandose para siempre gracias a condiciones de corte y stuck detection demo-only.
- Mantener intacto el contrato principal `/commands/*` y limitar los cambios nuevos a `/simulation/*` y al frontend estatico actual.

**Non-Goals:**
- Mejorar el algoritmo de limpieza, coverage o docking del firmware.
- Agregar SLAM, path planning, map memory, localizacion, camaras, lidar o avoidance avanzado.
- Agregar Android, Bluetooth, WiFi, ESP32, RTOS, GPIO o drivers reales.
- Introducir npm, React, Vue, build pipeline o dependencias frontend externas.
- Convertir el batch runner en una API externa o de produccion.

## Decisions

### 1. El batch runner vivira en el frontend del demo
- **Decision:** la orquestacion de corridas multiples se implementara en `demo.js`, reutilizando el loop actual, la generacion de mapas, la pose visual local y los endpoints ya existentes.
- **Rationale:** el frontend ya posee `x/y/heading`, coverage local y ownership del mapa visual; mover esta orquestacion al backend agregaria estado innecesario y acoplaria demasiado la simulacion.
- **Alternatives considered:** un scheduler backend persistente. Se descarta porque el pedido es demo/simulation-only y el runner necesita controlar estado visual local.

### 2. Cada corrida tendra un ciclo de vida explicito y repetible
- **Decision:** cada run seguira exactamente la secuencia generar mapa, crear `session_id`, asociar `batch_id`, resetear estado frontend, resetear backend por el flujo ya existente, disparar `POST /commands/start`, correr hasta una condicion de fin, opcionalmente disparar `POST /commands/return-to-dock`, registrar fin, guardar log y agregar run summary al batch.
- **Rationale:** un ciclo fijo facilita comparabilidad entre corridas y evita estados residuales entre mapas.
- **Alternatives considered:** reusar parcialmente el estado entre runs. Se descarta porque contaminaria coverage, timeline, sensores y diagnosticos.

### 3. Las condiciones de corte seran configurables pero con un solo `finish_reason` final
- **Decision:** el batch permitira cortar por tiempo, coverage, docking exitoso, stuck diagnostico, cancelacion o error de comando, y cada corrida cerrara con exactamente un `finish_reason`.
- **Rationale:** el analisis posterior necesita clasificacion consistente y no una mezcla ambigua de causas de finalizacion.
- **Alternatives considered:** multiples razones acumuladas por run. Se descarta porque complica el resumen y dificulta ranking de peores corridas.

### 4. `Return to Dock` sera una segunda fase opcional del run
- **Decision:** si la opcion esta activa, alcanzar el tiempo o coverage no cierra inmediatamente la corrida; primero se emite `POST /commands/return-to-dock` y el run continua hasta `CHARGING`, timeout/stuck de docking o cancelacion.
- **Rationale:** permite evaluar el comportamiento de docking como parte del batch sin cambiar la semantica del comando principal.
- **Alternatives considered:** cerrar siempre al llegar a tiempo/coverage. Se descarta porque perderia un caso de analisis explicitamente pedido.

### 5. El batch tendra stuck detection solo del lado simulador
- **Decision:** el frontend batch medira falta de movimiento con ruedas no nulas, repeticion excesiva de `LONG_TURN_GUARD`, repeticion excesiva de `DOCK_STUCK_DIAGNOSTIC` o `DOCK_BLOCKED`, y falta de progreso hacia el dock durante `RETURNING_TO_DOCK`.
- **Rationale:** el batch no puede depender de supervision humana; necesita una salida automatica cuando la simulacion deja de avanzar.
- **Alternatives considered:** introducir un estado de error de firmware para stuck. Se descarta porque el requerimiento pide una condicion demo-only, no una nueva semantica de produccion.

### 6. Los logs seguiran siendo compatibles y sumaran metadata batch
- **Decision:** cada `log.json` mantendra el formato actual y agregara `batch_id`, `run_index`, `batch_total_runs`, `batch_mode = true`, configuracion aplicada y nuevos eventos batch como `BATCH_RUN_START`, `BATCH_STOP_TIME_LIMIT` o `BATCH_STOP_DOCKED`.
- **Rationale:** mantener compatibilidad evita romper tooling existente y a la vez permite filtrar corridas batch para analisis posterior.
- **Alternatives considered:** un formato de log completamente nuevo para batch. Se descarta porque duplicaria payloads y aumentaria trabajo de compatibilidad.

### 7. La persistencia batch usara carpetas por `batch_id` con sanitizacion estricta
- **Decision:** el backend agregara `POST /simulation/batch-summary` y, si hace falta, extendera `POST /simulation/log-dump` para aceptar rutas batch del tipo `demo-logs/<batch_id>/<session_id>/log.json`, validando `batch_id` y `session_id` contra traversal o caracteres inseguros.
- **Rationale:** el layout por batch deja juntos el resumen y los logs individuales y simplifica compartir evidencia.
- **Alternatives considered:** guardar todos los archivos en una carpeta plana. Se descarta porque hace mas dificil correlacionar artefactos y aumenta riesgo de colisiones.

### 8. El resumen batch sera autocontenido y util sin UI
- **Decision:** `batch-summary.json` incluira configuracion usada, lista completa de run summaries, metricas agregadas y listas de peores corridas por coverage, contactos, guards, stucks y streaks de no movimiento.
- **Rationale:** el usuario quiere poder zippear y enviar los resultados sin depender de reproducir el estado del navegador.
- **Alternatives considered:** persistir solo contadores agregados. Se descarta porque perderia trazabilidad por run y rutas a logs.

### 9. Los controles manuales quedaran aislados mientras el batch esta activo
- **Decision:** durante el batch, los controles manuales se deshabilitaran o se marcaran como no disponibles para evitar corromper una corrida activa; al finalizar o cancelar, vuelven a funcionar normalmente.
- **Rationale:** el runner necesita exclusividad sobre el estado visual y de comandos durante una corrida batch.
- **Alternatives considered:** permitir controles manuales y solo avisar. Se descarta porque habilita corrupcion del experimento y resultados no reproducibles.

## Risks / Trade-offs

- **[El batch puede acoplar demasiado UI y simulacion]** -> mantenerlo como capa de orquestacion frontend sin cambiar el contrato principal ni la logica central del firmware.
- **[La deteccion de stuck puede cortar falsos positivos]** -> hacer umbrales configurables o al menos explicitamente documentados en la configuracion batch.
- **[Persistir muchos logs puede generar carpetas grandes]** -> mantener JSON pretty pero compacto y documentar como zippear solo el `batch_id` relevante.
- **[La falta de spec canonico `demo-frontend` deja requisitos frontend fuera de su capability ideal]** -> documentar el runner batch bajo `simulation-and-testing` hasta que ese spec exista y se sincronice.
