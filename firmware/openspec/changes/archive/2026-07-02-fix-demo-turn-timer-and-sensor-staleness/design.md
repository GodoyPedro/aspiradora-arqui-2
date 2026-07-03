## Context

El firmware ya implementa timers internos para giros `AUTO` y deadlines de `MANUAL_MOVE` usando `SimulatedClock`. El problema es que el demo del navegador corre su loop con tiempo real, pero si el backend no recibe y aplica un `delta_ms` por tick, el clock simulado queda quieto. En ese escenario, `auto_turn_until_ms` y `manual_move_deadline_ms` nunca expiran.

El log abierto confirma ese sintoma: el robot arranca, entra en `CLEANING`, detecta obstaculo, queda con `42/-42`, cambia heading en muchos frames y no recupera movimiento translacional. A la vez, la timeline etiqueta `TURN_START` y `ANTI_LOOP_ESCAPE` repetidamente, lo que sugiere que el frontend tambien esta sobrerreportando transiciones de giro.

## Goals / Non-Goals

**Goals:**
- Hacer que `/simulation/tick` avance tiempo simulado de forma explicita y segura.
- Garantizar que los giros `AUTO` y movimientos manuales expiren cuando el tiempo simulado avance.
- Evitar que sensores stale o labels de frontend reinicien un turn timer o aparenten anti-loop en cada frame.
- Limpiar flags stale en Stop, Reset y Generate New Map.
- Mejorar el log para que muestre un `TURN_START` finito, frames de giro normales y luego avance real con `x/y` cambiando.

**Non-Goals:**
- Replantear la arquitectura general del demo.
- Cambiar la API principal `/commands/*`.
- Introducir planning, SLAM, memoria de mapa o nuevas dependencias frontend.

## Decisions

### 1. `/simulation/tick` recibe `delta_ms` demo-only
- **Decision:** `POST /simulation/tick` aceptara opcionalmente `{ "delta_ms": <n> }`. Si llega, el backend validara `0 < delta_ms <= 1000`, avanzara `SimulatedClock` y luego ejecutara `controller.tick()`. Si no llega payload, usara un default seguro como `100ms` para preservar compatibilidad.
- **Rationale:** resuelve la causa raiz de timers que nunca expiran y mantiene el endpoint util para tests viejos o clientes demo existentes.
- **Alternatives considered:** avanzar el clock implicitamente dentro de `controller.tick()` o depender de tiempo real del servidor. Se descartan porque mezclan concerns del dominio con detalles del demo/testing.

### 2. El controller no debe reprogamar un timed turn en cada frame
- **Decision:** mientras un turn window siga activo, `obstacle_detected` stale no debera arrancar un turno nuevo frame a frame. Solo al expirar el window se reevaluaran sensores y, si el robot sigue bloqueado, se permitira un turn adicional.
- **Rationale:** evita loops artificiales donde el timer se resetea constantemente antes de vencer.
- **Alternatives considered:** ignorar sensores durante todo el giro sin reevaluacion posterior. Se descarta porque impediria reaccionar correctamente si el robot sigue realmente bloqueado.

### 3. El frontend enviara tiempo simulado significativo en cada ciclo
- **Decision:** el loop del demo calculara `elapsedRealMs`, lo multiplicara por el speed factor, lo clamp a un max seguro y lo enviara a `/simulation/tick` como `delta_ms`.
- **Rationale:** alinea tiempo visual y tiempo del backend para que los timers expiren de forma consistente con la velocidad elegida por el usuario.
- **Alternatives considered:** dejar default fijo siempre. Se descarta porque rompe la relacion con la velocidad de simulacion elegida.

### 4. Labels de giro basados en transiciones, no en wheel speeds distintos
- **Decision:** `TURN_START` se emitira solo cuando se pase de avance/recta a giro. `ANTI_LOOP_ESCAPE` solo se emitira cuando haya una condicion explicita de escape anti-loop. Los frames normales de giro podran etiquetarse como `TURNING` o quedar sin evento especial.
- **Rationale:** evita ruido en la timeline y hace mas legible el log para analisis posterior.
- **Alternatives considered:** seguir derivando eventos solo de `left != right`. Se descarta porque produce falsos positivos, como ya muestra el log actual.

### 5. Stop/Reset/New Map deben limpiar sensores stale
- **Decision:** al detener, resetear o generar mapa nuevo, el frontend limpiara `obstacle_detected`, `bumper_pressed` y su estado local de colision, y enviara esa limpieza al backend si hace falta.
- **Rationale:** evita que el log termine con sensores activos que ya no representan una condicion real.
- **Alternatives considered:** confiar en que el siguiente frame visual lo arregle solo. Se descarta porque deja ventanas de inconsistencia visibles en UI y dump.

## Risks / Trade-offs

- **[`delta_ms` demasiado grande rompe la simulacion]** -> validar y clamp con un max explicito como `1000`.
- **[Default de compatibilidad en `/simulation/tick` oculta bugs de frontend]** -> mantenerlo solo como fallback y documentar que el demo debe enviar `delta_ms` siempre.
- **[Evitacion adicional tras expiry todavia puede girar varias veces]** -> permitir solo reprogrmar tras una reevaluacion real y mejorar labels para distinguir `TURN_START` de frames `TURNING`.
