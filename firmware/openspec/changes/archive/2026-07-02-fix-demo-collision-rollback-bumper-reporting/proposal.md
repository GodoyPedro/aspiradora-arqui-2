## Why

El ultimo log del demo muestra una nueva inconsistencia distinta al bug anterior: despues de girar y volver a avanzar, el backend sigue reportando `state = CLEANING`, `left_wheel_speed = 60`, `right_wheel_speed = 60`, `obstacle_detected = false` y `bumper_pressed = false`, pero la pose visual `x/y` deja de cambiar durante muchos frames. Eso indica que el frontend esta rechazando o rollbackeando el avance por colision local, pero no esta informando esa colision como contacto al backend.

Mientras el backend cree que el robot sigue avanzando libremente, nunca activa la reaccion `AUTO` por bumper. El resultado visible es que el robot queda congelado en pantalla con velocidades de avance, sin un nuevo giro de escape.

## What Changes

- Ajustar el demo frontend para que todo rollback o rechazo de movimiento por colision contra pared/obstaculo se reporte como `bumper_pressed = true` siguiendo una secuencia explicita: mantener ultima pose valida, `POST /simulation/sensors`, `POST /simulation/tick`, y consumir el status devuelto en el mismo ciclo.
- Detectar explicitamente el caso "backend ordena avanzar, el frontend rechaza el paso y `x/y` no cambian" y registrarlo como `BUMPER_CONTACT` o `COLLISION_ROLLBACK`, con detalle simple de origen (`wall`, `obstacle`, `obstacle_id`).
- Aplicar substeps internos de movimiento cuando la simulacion corre a alta velocidad, con distancia maxima por substep y un cap de substeps por frame para evitar loops caros.
- Evitar spam de eventos de bumper duplicados mientras ya se esta esperando la reaccion del backend; si el backend sigue devolviendo `60/60`, registrar un evento diagnostico explicito.
- Limpiar `bumper_pressed` solo cuando el robot ya no este realmente bloqueado y un futuro avance vuelva a ser valido, para no dejar avoidance loops permanentes ni limpiarlo demasiado temprano.
- Actualizar la documentacion del demo y las tareas/specs de validacion manual para cubrir este caso de rollback con contacto.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `simulation-and-testing`: refinar la semantica del demo local para que un avance visual bloqueado se traduzca en contacto/bumper, use substeps acotados a alta velocidad, respete una secuencia explicita `sensors -> tick -> status`, y deje evidencia clara en el log timeline.

## Impact

- `src/api/assets/demo.js`
- `README.md`
- `openspec/changes/fix-demo-collision-rollback-bumper-reporting/*`
- deltas OpenSpec en `simulation-and-testing`
- tests solo si hiciera falta reforzar cobertura del comportamiento existente por bumper
