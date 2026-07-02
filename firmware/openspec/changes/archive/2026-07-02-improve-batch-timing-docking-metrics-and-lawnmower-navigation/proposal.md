## Why

El demo local ya tiene batch simulation, coverage, log dumps y docking visual, pero todavia mezcla conceptos inconsistentes: el batch usa tiempos y metricas que no siempre coinciden entre frontend, backend y logs; el docking puede fallar por falsos positivos de stuck; y el modo `AUTO` sigue comportandose demasiado reactivo y frenetico para evaluar cobertura de manera estable.

Este cambio propone corregir esos problemas de forma coordinada. Por un lado, el batch debe usar tiempo simulado consistente, metricas coherentes y criterios de docking/timeout mas confiables. Por otro lado, el `AUTO` del demo debe abandonar el rebote pseudo-random y pasar a un patron lawn-mower / boustrophedon simple, deterministico y mayormente guiado por contacto real, para producir corridas mas ordenadas y comparables.

## What Changes

- Corregir la base de tiempo del batch para que el demo, los logs, las ventanas de stuck y los stop conditions usen el mismo tiempo simulado por corrida.
- Refinar el guidance de `RETURNING_TO_DOCK` con steering proporcional, mejor criterio de progreso angular/posicional y cleanup consistente al entrar en `CHARGING`.
- Normalizar semantica de eventos y metricas de turn, anti-loop, docking y summaries para que run logs, run summaries y batch summaries usen definiciones consistentes.
- Reemplazar el `AUTO` demasiado reactivo por un patron lawn-mower demo-only con lanes, lane shifts, obstacle bypass controlado y wall handling predecible.
- Extender logs, summaries, README y validaciones manuales/backend para reflejar el nuevo comportamiento estructurado.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `sensor-actuator-control`: acotar las responsabilidades del backend a estado alto nivel, safety transitions, actuator/status reporting y transicion de docking/charging sin fingir navegacion espacial lane-based.
- `simulation-and-testing`: documentar tiempo simulado consistente, controller visual `AUTO` frontend-owned, state machine lane-based del demo, metric/event semantics y validaciones del demo/batch.

## Impact

- `src/api/assets/demo.js`
- `src/api/assets/demo.html`
- `src/api/assets/demo.css`
- `src/api/models.rs` only if log metrics need expansion
- `src/api/routes.rs` only if sensor cleanup or persistence behavior changes
- `src/application/controller.rs` only if minimal backend docking cleanup is needed
- `tests/http_api.rs`
- `tests/robot_controller.rs`
- `README.md`
- `openspec/changes/improve-batch-timing-docking-metrics-and-lawnmower-navigation/*`
