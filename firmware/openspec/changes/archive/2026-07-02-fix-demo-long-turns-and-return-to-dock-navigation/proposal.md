## Why

El demo ya resolvio varios problemas de sensores y rollback, pero el log mas reciente todavia muestra dos fallas claras:

1. Durante `AUTO`, el robot puede quedar mucho tiempo girando en el mismo lugar cerca de una pared u obstaculo. En la timeline `x/y` quedan congelados mientras los wheel speeds alternan patrones de giro y `obstacle_detected` sigue reapareciendo durante el mismo episodio.
2. En `RETURNING_TO_DOCK`, el backend entra correctamente al estado, pero el robot visual no navega hacia la base. Solo mantiene `30/30`, sigue su heading actual y termina chocando contra una pared sin acercarse al dock.

Este cambio propone una refinacion acotada del simulador/demo para limitar giros largos en sitio, agregar una guarda de escape visible en el log y hacer que el retorno a base use una navegacion visual simple hacia `docking_station.x/y`.

## What Changes

- Agregar una guarda de giro largo en el demo para que el robot no permanezca girando en el mismo `x/y` por demasiado tiempo sin un evento de escape o reintento de avance.
- Incorporar angular substepping acotado para giros en sitio a alta velocidad, evitando saltos grandes de heading y toggles artificiales de sensores durante el mismo giro.
- Hacer que `obstacle_detected` no reinicie indefinidamente nuevos turns mientras ya hay un giro activo; un nuevo giro solo podra arrancar despues de un intento real de avance y los eventos deberan pertenecer a episodios de giro explicitamente trazables.
- Hacer explicito que el boton `Return to Dock` debe llamar `POST /commands/return-to-dock`, consumir el status devuelto y reflejar inmediatamente `RETURNING_TO_DOCK` o el error correspondiente en UI/log.
- Implementar homing visual demo-only para `RETURNING_TO_DOCK`: orientar hacia el docking station, alinear, avanzar, desacelerar cerca del dock y reintentar con un desvio simple si hay bloqueo, con prioridad sobre el movimiento normal `AUTO`.
- Agregar eventos y diagnosticos de log para giros largos, escape, docking targeting, bloqueo de dock y progreso hacia la base, incluyendo diferencia entre wheel speeds backend y guidance espacial del demo.
- Actualizar README, tareas OpenSpec y validaciones manuales para cubrir la diferencia entre estado backend de docking y guiado espacial del frontend.
- Reforzar el manejo de rechazo del boton `Return to Dock`: si backend rechaza el comando, el frontend no debe arrancar homing local y debe loguear `RETURN_TO_DOCK_REJECTED` con detalle de error.
- Mantener timestamps de timeline no decrecientes para que el dump sea analizable de forma confiable.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `simulation-and-testing`: reforzar el demo local para evitar giros largos silenciosos, mejorar precision angular a alta velocidad, rastrear episodios de giro, y agregar homing visual simple hacia el dock con diagnosticos de log que distingan guidance del demo frente a wheel speeds backend.

## Impact

- `src/api/assets/demo.js`
- `src/api/assets/demo.html` si se necesita exponer diagnosticos de docking
- `src/api/assets/demo.css` si se necesita styling adicional
- `src/application/controller.rs` solo si hace falta un ajuste minimo para `RETURNING_TO_DOCK`
- `tests/robot_controller.rs`
- `tests/http_api.rs`
- `README.md`
- `openspec/changes/fix-demo-long-turns-and-return-to-dock-navigation/*`
- deltas OpenSpec en `simulation-and-testing`
- delta OpenSpec adicional en `robot-state-machine` para expectativas de transicion backend
