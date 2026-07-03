## Why

El demo local ya permite visualizar y controlar el simulador, pero hoy tiene defectos visibles: el robot puede quedar girando indefinidamente despues de evitar un obstaculo, la simulacion visual se mueve demasiado lento y la deteccion local del frontend puede sostener flags de obstaculo o collision mas tiempo del debido. Refinar este comportamiento mejora la credibilidad del demo y alinea mejor el AUTO cleaning con la expectativa de "evitar y continuar".

## What Changes

- Refinar el comportamiento AUTO para que, tras una maniobra de evitacion, el robot vuelva a avanzar sin que `tick()` resetee ciegamente cualquier wheel speed en `CLEANING`.
- Ajustar la simulacion del navegador para usar deteccion frontal basada en heading para `obstacle_detected` y colision real para `bumper_pressed`.
- Evitar penetracion visual en obstaculos o paredes mediante rollback o clamping a la ultima posicion valida.
- Cambiar la duracion por defecto de los controles manuales del demo a 2 segundos por click, sin spam repetido de comandos.
- Aumentar la velocidad visual del robot en el demo con una constante explicita de frontend.
- Actualizar tests de backend, tests HTTP y README para reflejar la recuperacion AUTO y el refinamiento del demo local.

## Capabilities

### New Capabilities

None.

### Modified Capabilities
- `demo-frontend`: refinar el loop visual, la deteccion frontal por heading, la prevencion de overlap, la duracion de comandos manuales del demo y la representacion de sensores activos.
- `sensor-actuator-control`: cambiar el comportamiento del `tick()` para restaurar movimiento AUTO hacia adelante solo despues de una evitacion previa o una condicion equivalente, sin sobreescribir movimientos no relacionados.
- `simulation-and-testing`: ampliar la cobertura para validar recuperacion AUTO, limpieza de flags, comportamiento HTTP visible y semantica del demo.

## Impact

- `src/application/controller.rs`
- `src/api/assets/demo.js`
- `tests/robot_controller.rs`
- `tests/http_api.rs`
- `README.md`
- deltas de OpenSpec para demo, control de actuadores y testing
