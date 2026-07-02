## Context

El backend ya hace lo correcto cuando recibe `bumper_pressed = true`: mantiene `CLEANING`, cambia wheel speeds a un patron de giro y luego reanuda avance. El problema aparece antes, en la capa visual del demo. El navegador calcula la proxima pose, detecta que pisaria pared u obstaculo, hace rollback o clamping a la ultima posicion valida, pero deja los sensores reportados como si el robot siguiera libre.

Eso genera una contradiccion en la timeline:
- wheel speeds del backend siguen en `60/60`;
- `obstacle_detected = false`;
- `bumper_pressed = false`;
- `x/y` quedan congelados porque el frontend no puede avanzar visualmente.

La correccion debe vivir principalmente en el frontend del demo, sin introducir dependencias ni rediseñar la navegacion del firmware.

## Goals / Non-Goals

**Goals:**
- Reportar como contacto real todo avance visual rechazado por colision.
- Evitar congelamientos largos con `60/60` y `bumper_pressed = false`.
- Hacer mas robusto el movimiento visual a alta velocidad usando substeps simples.
- Reflejar en el log cuando hubo rollback de colision, contacto, giro y reanudacion de avance.
- Hacer explicita la secuencia frontend/backend para que el demo no espere varios frames antes de convertir un bloqueo visual en un giro backend.

**Non-Goals:**
- Cambiar la API principal `/commands/*`.
- Rehacer la logica `AUTO` del backend.
- Introducir path planning, SLAM, memoria de mapa o nuevas librerias frontend.

## Decisions

### 1. Rollback de movimiento equivale a bumper/contacto
- **Decision:** si el frontend propone un substep o movimiento hacia adelante y lo rechaza porque pisaria una pared u obstaculo, ese frame se tratara como contacto local. El demo mantendra la ultima pose valida, marcara `bumper_pressed = true`, hara `POST /simulation/sensors` con ese estado, llamara inmediatamente a `POST /simulation/tick` con el `delta_ms` actual, y usara el `RobotStatus` devuelto para permitir el cambio inmediato de `60/60` al patron de giro `AUTO`.
- **Rationale:** desde el punto de vista del firmware, un avance frustrado por contacto fisico debe disparar la misma reaccion que un bumper real.
- **Alternatives considered:** dejar el rollback como detalle visual interno. Se descarta porque mantiene la divergencia entre estado backend y pose visible.

### 2. Deteccion explicita del estado "forward but blocked"
- **Decision:** el frontend comparara wheel speeds, resultado del movimiento y cambio real de pose. Si el backend ordena avance (`60/60` o patron recto), pero el paso propuesto es rechazado y `x/y` no cambian, se registrara un evento como `COLLISION_ROLLBACK` o `BUMPER_CONTACT`. Ese evento podra incluir metadata opcional y simple como `collision_target = "wall" | "obstacle"` y `event_detail.obstacle_id` cuando aplique.
- **Rationale:** hace visible en el dump la causa del bloqueo en lugar de dejar solo una secuencia silenciosa de poses repetidas.
- **Alternatives considered:** inferirlo despues offline desde `x/y`. Se descarta porque no deja intencion explicita en la timeline.

### 3. Movimiento visual con substeps acotados en alta velocidad
- **Decision:** el frontend dividira el delta de avance en varios substeps pequenos antes de probar colision, especialmente cuando la velocidad de simulacion multiplica el desplazamiento por frame. Cada substep tendra una distancia maxima acotada, por ejemplo `4px` a `8px`, y el numero total de substeps por frame tambien tendra un cap explicito. Si el cap se alcanza, el resto del movimiento de ese frame podra descartarse de forma segura.
- **Rationale:** reduce tunneling y evita que el robot salte demasiado entre una pose valida y una colision profunda, sin abrir un loop caro a `10x`.
- **Alternatives considered:** bajar la velocidad maxima o confiar en un solo collision test por frame. Se descartan porque no corrigen el problema de precision a `10x`.

### 4. `bumper_pressed` debe limpiarse cuando el bloqueo desaparece
- **Decision:** el demo no limpiara `bumper_pressed` solo porque el robot este girando. Lo mantendra mientras siga fisicamente bloqueado o solapado. Solo se limpiara cuando el robot ya no este realmente presionado contra la pared/obstaculo y un futuro avance hacia adelante desde la nueva orientacion vuelva a ser valido.
- **Rationale:** evita loops de avoidance permanentes por sensores stale y tambien evita limpiar bumper demasiado temprano mientras el robot sigue apoyado contra la colision.
- **Alternatives considered:** limpiar bumper solo en Stop/Reset. Se descarta porque deja una condicion de contacto falsa durante el movimiento normal.

### 5. No spamear eventos de bumper mientras se espera reaccion
- **Decision:** una vez que el frontend reporto un bloqueo como bumper/contacto, no volvera a emitir eventos identicos en cada frame mientras espera la reaccion backend. Puede mantener `bumper_pressed = true`, pero si el backend sigue devolviendo `60/60` despues del reporte, registrara un evento diagnostico como `BUMPER_REPORTED_WAITING_FOR_TURN`.
- **Rationale:** evita ruido en la timeline y hace visible un retraso backend anomalo sin ocultarlo.
- **Alternatives considered:** seguir registrando `BUMPER_CONTACT` cada frame. Se descarta porque infla el log sin agregar informacion nueva.

### 6. Invariante fuerte contra freeze silencioso
- **Decision:** el cambio documentara que el log no debe contener mas de un numero pequeno de frames consecutivos, por ejemplo `3` a `5`, donde al mismo tiempo el estado siga en `CLEANING`, los wheel speeds indiquen avance, `x/y` no cambien, `obstacle_detected = false`, `bumper_pressed = false`, y no exista un evento diagnostico de collision/rollback.
- **Rationale:** esa combinacion exacta describe el failure mode que el cambio quiere eliminar.
- **Alternatives considered:** depender solo de inspeccion visual manual. Se descarta porque no deja un criterio objetivo de aceptacion sobre el dump.

## Risks / Trade-offs

- **[Substeps demasiado chicos cargan mas el loop]** -> usar una distancia maxima fija y un cap de substeps por frame; si sobra movimiento, descartar el remanente de ese frame.
- **[BUMPER_CONTACT y COLLISION_ROLLBACK pueden duplicarse]** -> definir una sola transicion por frame bloqueado y documentar cual evento tiene prioridad.
- **[Limpieza prematura de bumper]** -> limpiar solo cuando un futuro substep hacia adelante vuelva a ser valido y el robot ya no siga fisicamente bloqueado.
