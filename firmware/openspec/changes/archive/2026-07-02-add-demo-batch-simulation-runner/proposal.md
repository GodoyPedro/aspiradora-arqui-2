## Why

El demo local ya permite probar una corrida individual con mapa aleatorio, sensores visuales, cobertura y dump por sesion. Pero hoy sigue dependiendo de intervencion manual y no genera evidencia repetible a escala para detectar patrones de falla, loops, coberturas pobres o problemas de docking entre multiples mapas.

Este cambio propone agregar un modo batch controlado por el frontend para ejecutar muchas simulaciones consecutivas sobre mapas generados, persistir un log por corrida y un resumen agregado del batch, y frenar cada corrida con condiciones de corte claras. El objetivo es producir evidencia exportable para analisis posterior del comportamiento del robot, sin ampliar el alcance hacia navegacion real, SLAM, hardware o cambios del contrato principal de comandos.

## What Changes

- Extender el demo local con una seccion `Batch Simulation` para configurar corridas multiples, condiciones de corte, velocidad de simulacion, opcion de `Return to Dock` y estado visible del batch en progreso.
- Agregar un runner batch demo-only en el frontend que genere mapa nuevo por corrida, cree `session_id` y `batch_id`, reinicie estado visual/local, arranque limpieza automaticamente, observe condiciones de corte y persista cada log sin intervenir el algoritmo base del robot.
- Incorporar deteccion batch-only de stuck o timeout para impedir corridas infinitas y clasificar la finalizacion como `STUCK_DIAGNOSTIC` o `DOCKING_FAILED_OR_TIMEOUT` segun el contexto.
- Extender el formato de log de simulacion con metadata de batch, labels de eventos batch y un resumen compacto por corrida para analisis posterior.
- Agregar persistencia demo-only para `batch-summary.json` y compatibilidad de rutas para guardar logs manuales o batch sin romper el endpoint existente `POST /simulation/log-dump`.
- Documentar el flujo batch, las metricas agregadas y la ubicacion de los artefactos exportables.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `command-protocol`: documentar helpers demo-only de persistencia batch bajo `/simulation/*`, sanitizacion de `batch_id` y compatibilidad entre logs manuales y batch.
- `simulation-and-testing`: documentar el runner batch del demo, su ciclo de vida, condiciones de corte, eventos de timeline, metricas de resumen y validaciones manuales/backend requeridas.

## Impact

- `src/api/assets/demo.html`
- `src/api/assets/demo.css`
- `src/api/assets/demo.js`
- `src/api/models.rs`
- `src/api/routes.rs`
- `tests/http_api.rs`
- `README.md`
- `openspec/changes/add-demo-batch-simulation-runner/*`
