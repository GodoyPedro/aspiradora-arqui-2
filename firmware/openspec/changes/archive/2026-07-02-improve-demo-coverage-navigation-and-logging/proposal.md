## Why

El demo local ya muestra movimiento, sensores y obstaculos, pero sigue quedando corto para explicar un comportamiento de aspiradora realista. Hoy el AUTO evita demasiado temprano, puede dejar la sensacion de atasco o cobertura pobre, no visualiza el piso limpiado y no genera un dump completo para analizar una corrida despues.

Este cambio propone una mejora puntual del simulador local y de su demo web para que el robot avance por defecto, use `bumper_pressed` como trigger principal de reaccion por colision normal, solo use `obstacle_detected` como proximidad frontal de muy corto alcance, gire in-place sin retroceder, evite loops obvios con una heuristica simple, pinte la cobertura limpiada y permita exportar una sesion completa con mapa reconstruible, timeline, sensores y resumen.

## What Changes

- Refinar el comportamiento AUTO del firmware/simulador para que avance por defecto, mantenga `state = CLEANING` ante `bumper_pressed` o obstaculo frontal muy cercano, gire en el lugar sin retroceder, retome avance automaticamente y nunca quede detenido indefinidamente por un evento normal de obstaculo.
- Extender el demo frontend para recalcular colisiones de forma local, permitir contacto real con obstaculos, usar `bumper_pressed` como trigger principal, pintar cobertura del piso, mostrar siempre el `session_id` activo, acelerar o frenar la simulacion, resetear la corrida sin cambiar sesion y generar nuevos mapas con nueva sesion.
- Agregar endpoints demo-only bajo `/simulation/*` para resetear la simulacion y persistir log dumps completos por sesion/mapa en una carpeta local segura con una ruta fija por sesion del tipo `demo-logs/<session_id>/log.json`.
- Ampliar los tests de backend e integracion HTTP para cubrir recuperacion AUTO, ausencia de freeze por obstaculos normales, reset, log dump y validaciones de payload.
- Actualizar la documentacion para explicar navegacion AUTO, diferencia entre `obstacle_detected` y `bumper_pressed`, cobertura, velocidad, reset, mapas y ubicacion de logs.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `demo-frontend`: agregar cobertura visual, control de velocidad, reset, nuevo mapa, exportacion de log dump y modelo local de colision/contacto mas realista.
- `sensor-actuator-control`: refinar el loop AUTO para avanzar por defecto, girar in-place ante bumper/obstaculo frontal y evitar loops obvios sin agregar path planning.
- `command-protocol`: documentar los helpers demo-only `/simulation/reset` y `/simulation/log-dump` como endpoints locales fuera del contrato principal.
- `simulation-and-testing`: ampliar las expectativas de cobertura de tests y del contenido persistido en los dumps de simulacion.

## Impact

- `src/application/controller.rs`
- `src/domain/models.rs`
- `src/api/demo.rs` o modulo equivalente de rutas de simulacion
- `src/api/assets/demo.html`
- `src/api/assets/demo.css`
- `src/api/assets/demo.js`
- `tests/robot_controller.rs`
- `tests/http_api.rs`
- `README.md`
- `.gitignore`
- deltas OpenSpec para demo, control y testing
