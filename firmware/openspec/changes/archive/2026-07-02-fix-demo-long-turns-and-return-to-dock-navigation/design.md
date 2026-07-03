## Context

La posicion y orientacion del robot en el demo viven en el frontend, no en el firmware. El backend conoce estados (`CLEANING`, `RETURNING_TO_DOCK`, `CHARGING`) y drivers simulados, pero no posee localizacion real del cuarto. Por eso los problemas actuales tambien son del layer visual:

- un giro `AUTO` puede quedar encadenado con retriggers de proximidad cerca de una pared;
- una orden `RETURN_TO_DOCK` cambia el estado backend, pero no orienta la pose visual hacia el docking station.

La solucion debe seguir siendo demo-only, sin memoria de mapa ni planning avanzado, usando heuristicas deterministicas locales y un log suficientemente rico para diagnosticar si hubo progreso real.

En este repo no existe hoy un spec base `demo-frontend`. Por eso los requisitos funcionales del frontend seguiran documentados en este cambio bajo `simulation-and-testing`, mientras que las expectativas de transicion backend se agregaran como delta pequeno en `robot-state-machine`.

## Goals / Non-Goals

**Goals:**
- Evitar secuencias largas de `TURNING` con `x/y` fijos y sin eventos de guard/escape.
- Mejorar la precision angular a alta velocidad para que el heading no salte demasiado por frame.
- Asegurar que `RETURNING_TO_DOCK` apunte al dock, se acerque a el y no se limite a avanzar en linea recta hasta chocar.
- Agregar eventos de log que muestren claramente guardas de giro, intentos de escape y fases de docking.
- Hacer explicita la diferencia entre el estado backend/wheel speeds y el guidance espacial demo-only del frontend.

**Non-Goals:**
- Agregar SLAM, memoria de mapa, path planning o avoidance avanzado.
- Convertir el frontend en firmware real de docking.
- Cambiar la API principal `/commands/*`.

## Decisions

### 1. Gira largo continuo tendra una guarda explicita
- **Decision:** el demo medira cuanto tiempo continuo pasa el robot girando en el mismo lugar durante `CLEANING`. Si supera un umbral demo-only, por ejemplo `2000ms` a `3000ms`, se activara una guarda de escape.
- **Rationale:** evita que la timeline quede llena de `TURNING` con `x/y` fijos sin una intervencion visible.
- **Alternatives considered:** dejar solo la logica actual de timed turns del backend. Se descarta porque ya mostro loops largos cerca de pared.

### 2. La guarda de giro largo forzara una salida observable
- **Decision:** cuando se active la guarda, el demo tratara de limpiar proximidad stale si ya no hay bloqueo fisico, preferira un intento corto de avance si un probe frontal es valido, y si sigue bloqueado ejecutara un escape turn deterministico seguido de un intento forzado de avance. Despues de `LONG_TURN_GUARD`, no se permitira que otro turn nuevo empiece hasta evaluar el resultado de ese forward attempt.
- **Rationale:** el robot necesita una accion concreta y legible en el log, no solo seguir girando con otra secuencia de turns iguales.
- **Alternatives considered:** solo resetear sensores. Se descarta porque no garantiza movimiento real.

### 3. Los sensores no deben reiniciar turns durante el mismo giro
- **Decision:** `obstacle_detected` podra seguir recalculandose para UI/log, pero no debera reiniciar un nuevo turn mientras el giro actual siga activo. Un nuevo turn solo podra empezar despues de que el giro termino y hubo un intento real de avance bloqueado o una nueva deteccion fresca.
- **Rationale:** rompe el loop `turn -> obstacle true -> turn -> obstacle true`.
- **Alternatives considered:** desactivar la deteccion visual durante giros. Se descarta porque quitaria observabilidad innecesariamente.

### 4. El frontend llevara episodios de giro explicitos
- **Decision:** el demo llevara un `turn_episode_id` o equivalente local. `TURN_START` abrira un episodio nuevo, `TURNING` pertenecera a ese episodio y `TURN_END` lo cerrara. Cambios de `obstacle_detected` durante un episodio abierto no podran crear un episodio nuevo.
- **Rationale:** permite distinguir entre un giro muy largo y muchos giros retriggered artificialmente.
- **Alternatives considered:** inferir episodios solo por wheel speeds. Se descarta porque complica el analisis posterior del log.

### 5. El demo usara angular substepping acotado
- **Decision:** cuando el robot rote en sitio o tenga un delta angular grande, el frontend dividira esa rotacion en substeps angulares con un limite, por ejemplo `0.15` a `0.30` rad por substep.
- **Rationale:** con `10x`, un solo frame puede saltar demasiado el heading y hacer aliasing de sensores cerca de paredes.
- **Alternatives considered:** seguir usando substeps derivados solo de distancia lineal. Se descarta porque rotaciones puras tienen distancia lineal casi cero.

### 6. El boton Return to Dock debe reflejar el estado backend inmediatamente
- **Decision:** el boton `Return to Dock` del demo hara `POST /commands/return-to-dock`, consumira el `RobotStatus` devuelto, registrara `RETURN_TO_DOCK` en el log cuando el comando sea aceptado y mostrara el error devuelto por backend si el comando falla. Si backend rechaza el comando, el frontend registrara `RETURN_TO_DOCK_REJECTED` con detalle de error y NO iniciara homing local.
- **Rationale:** evita que el usuario vea un boton que parece no hacer nada o que el homing arranque sin confirmacion del estado backend.
- **Alternatives considered:** disparar solo un cambio local de modo en el frontend. Se descarta porque rompe la fuente de verdad del backend.

### 7. RETURNING_TO_DOCK tendra homing visual demo-only con prioridad
- **Decision:** cuando el backend este en `RETURNING_TO_DOCK`, el frontend computara el vector al dock, el heading error y aplicara una heuristica simple:
  - girar si el error angular es grande;
  - avanzar cuando este razonablemente alineado;
  - reducir velocidad al acercarse;
  - marcar `dock_detected = true` cuando haya proximidad suficiente.
- **Decision complementaria:** durante `RETURNING_TO_DOCK`, el movimiento visual no usara ciegamente `30/30` como avance recto. El frontend usara guidance demo-only derivado de `pose`, `dock position`, `heading error`, `distance to dock` y bloqueo local. Ese guidance tendra prioridad sobre el movimiento `AUTO` normal.
- **Rationale:** el frontend ya posee `x/y/heading` y la geometria del room; es el lugar correcto para este guiado de demo.
- **Alternatives considered:** mover esa inteligencia al backend. Se descarta porque el backend no posee localizacion real del mapa en este simulador.

### 8. El docking bloqueado tendra recovery simple, no planning
- **Decision:** si el acercamiento al dock queda bloqueado por pared u obstaculo, el demo reportara el bloqueo, ejecutara un pequeno giro o detour local, movera una pequena distancia si puede, y luego volvera a apuntar al dock. En este modo se usaran eventos especificos de docking como `DOCK_BLOCKED`, `DOCK_RECOVERY_TURN` y `DOCK_STUCK_DIAGNOSTIC`, en lugar de reutilizar eventos `AUTO` normales.
- **Rationale:** alcanza para un simulador local y evita quedarse indefinidamente en `30/30` contra una pared.
- **Alternatives considered:** agregar planeamiento o memoria de obstaculos. Se descarta por fuera de alcance.

### 9. El log debe probar intervencion y progreso
- **Decision:** el dump incluira eventos como `LONG_TURN_GUARD`, `ESCAPE_FORWARD_ATTEMPT`, `ESCAPE_TURN`, `DOCK_TARGETING`, `DOCK_ALIGNING`, `DOCK_APPROACH`, `DOCK_BLOCKED`, `DOCK_RECOVERY_TURN`, `DOCK_DETECTED`, `CHARGING_STARTED` y `DOCK_STUCK_DIAGNOSTIC`, con `event_detail` simple cuando haga falta, por ejemplo `turn_episode_id`, `backend_left_wheel_speed`, `backend_right_wheel_speed`, `demo_docking_phase`, `demo_target_heading`, `heading_error` y `distance_to_dock`.
- **Rationale:** la aceptacion de este cambio depende de ver si el robot realmente deja de girar demasiado y se acerca al dock en el tiempo.
- **Alternatives considered:** confiar solo en wheel speeds y estado. Se descarta porque no prueba progreso espacial.

### 10. Los timestamps del timeline deben ser monotonicos
- **Decision:** los frames del dump mantendran `timestamp_ms` no decreciente en toda la timeline.
- **Rationale:** hubo logs previos con pequenas regresiones de timestamp cerca de eventos de arranque; eso complica analisis posteriores.
- **Alternatives considered:** tolerar pequenos retrocesos. Se descarta porque no agrega valor y dificulta validaciones.

### 11. Los invariantes de aceptacion deben ser explicitos
- **Decision:** despues de `LONG_TURN_GUARD`, los siguientes frames deberan mostrar cambio de `x/y` o un evento de bloqueo con detalle de target. Durante `RETURNING_TO_DOCK`, no se aceptaran periodos largos sin reduccion de distancia al dock, con `x/y` clavados o alejandose, si no existe un evento de recovery o diagnostico.
- **Rationale:** evita que la guarda o el homing queden solo documentados sin impacto real observable.
- **Alternatives considered:** dejar la aceptacion solo a criterio visual general. Se descarta por ambigua.

## Risks / Trade-offs

- **[La guarda de giro largo podria intervenir demasiado pronto]** -> usar un umbral claro y documentado en tiempo demo-only.
- **[Angular substepping puede aumentar el costo del loop]** -> aplicar un maximo de substeps angulares por frame.
- **[Docking homing simple puede seguir fallando en mapas complicados]** -> aceptar detours locales simples y agregar `DOCK_STUCK_DIAGNOSTIC` en lugar de fingir navegacion avanzada.
- **[Frontend y backend pueden divergir si ambos intentan resolver docking]** -> documentar claramente que el backend maneja estado y el frontend maneja guiado espacial visual.
