## Context

El proyecto ya implementa un demo local servido por el backend y un comportamiento AUTO simple donde `tick()` aplica una maniobra de evitacion si `obstacle_detected` o `bumper_pressed` esta activo. Hoy esa logica no restablece el avance automatico cuando el obstaculo desaparece, y el frontend puede sostener flags de obstaculo demasiado tiempo o dejar que el robot penetre visualmente un obstaculo, generando un giro infinito aparente.

El objetivo no es redisenar la arquitectura ni agregar navegacion avanzada, sino hacer que el comportamiento actual sea mas estable y demostrable sin introducir hardware real, Android, Bluetooth ni nuevas dependencias grandes.

## Goals / Non-Goals

**Goals:**
- Restaurar movimiento AUTO hacia adelante cuando ya no hay obstaculo ni bumper, sin resetear ciegamente otros wheel speeds en `CLEANING`.
- Evitar que el demo mantenga `obstacle_detected` o `bumper_pressed` stale despues de iniciar un giro.
- Basar `obstacle_detected` en deteccion frontal segun heading y `bumper_pressed` en colision real.
- Evitar overlap visual del robot con obstaculos o paredes.
- Aumentar la velocidad visual del demo sin alterar la semantica de wheel speeds del backend.
- Fijar en 2 segundos la duracion por defecto de los botones manuales del demo con un solo request por click.
- Extender la cobertura de tests para la recuperacion AUTO y el comportamiento HTTP visible.

**Non-Goals:**
- Introducir path planning, SLAM o navegacion avanzada.
- Convertir el frontend en una fuente de verdad del firmware.
- Cambiar el contrato principal `/commands/*` o agregar nuevos endpoints.
- Cambiar la validacion backend de `MANUAL_MOVE` mas alla de usar un `duration_ms` distinto desde la UI.

## Decisions

### 1. Restaurar avance AUTO solo despues de una evitacion previa o una condicion equivalente
- **Decision:** la recuperacion AUTO no se definira como "todo `tick()` en `CLEANING` sin obstaculo pone 60/60", sino como recuperacion despues de una evitacion previa o cuando el robot este en limpieza AUTO normal y no ejecutando otro comportamiento explicito. La implementacion puede usar un flag/cooldown interno simple o una condicion basada en detectar el patron actual de wheel speeds de evitacion.
- **Rationale:** evita que `tick()` sobreescriba innecesariamente wheel speeds intencionales ajenos a la evitacion y sigue resolviendo el giro infinito.
- **Alternatives considered:** resetear siempre a 60/60 o mover toda la recuperacion al frontend. Se descarto porque la primera opcion es demasiado agresiva y la segunda deja una responsabilidad funcional critica fuera del firmware.

### 2. Mantener la correccion de obstaculos en el frontend como recomputacion continua orientada por heading
- **Decision:** el demo recalculara proximidad/colision en cada ciclo y usara una deteccion frontal basada en heading, cono o rayo para `obstacle_detected`. `bumper_pressed` se activara solo por colision u overlap real.
- **Rationale:** si el robot solo esta cerca de un obstaculo pero ya no lo tiene enfrente, `obstacle_detected` debe limpiarse para permitir la recuperacion AUTO.
- **Alternatives considered:** basar ambos flags solo en proximidad espacial. Se descarto porque perpetua giros artificiales aun cuando el robot ya esta rotado hacia una direccion libre.

### 3. Prevenir penetracion visual con rollback o clamping
- **Decision:** el frontend conservara la ultima posicion valida y, ante colision con obstaculo o pared, hara rollback o clamping a una posicion no penetrante.
- **Rationale:** evita que el robot quede "dentro" de un obstaculo y mantenga `bumper_pressed = true` indefinidamente.
- **Alternatives considered:** permitir overlap y depender solo de limpiar flags luego. Se descarto porque genera un demo visualmente incorrecto y propenso a loops.

### 4. Permitir un cooldown de evitacion solo del lado frontend
- **Decision:** el demo puede usar un cooldown local simple de 500 a 1000 ms antes de volver a reportar obstaculo, solo para darle tiempo visual al giro.
- **Rationale:** suaviza la experiencia del demo sin contaminar el modelo del firmware con conceptos visuales.
- **Alternatives considered:** agregar cooldown en `RobotController`. Se descarto porque el requisito lo posiciona como concern de demo local, no del firmware base.

### 5. Aumentar velocidad visual y hacerla explicita en `demo.js`
- **Decision:** usar una constante clara como `VISUAL_SPEED_SCALE` o `PIXELS_PER_SPEED_UNIT` con un valor mayor al actual.
- **Rationale:** la sala 2D es puramente visual; acelerar el desplazamiento mejora la demostracion sin tocar las velocidades reales del backend.
- **Alternatives considered:** subir `left_wheel_speed` y `right_wheel_speed` del backend. Se descarto porque altera semantica funcional en vez de solo la visualizacion.

### 6. Fijar `duration_ms = 2000` con un request manual por click
- **Decision:** los botones Manual Forward, Backward, Left, Right y Stop usaran 2000 ms por defecto y emitiran un solo `MANUAL_MOVE` por click, no una rafaga continua mientras el boton este presionado o mientras el loop corre.
- **Rationale:** mejora la usabilidad del demo y preserva el comportamiento donde la continuidad visual viene del estado backend hasta que expire el deadline manual.
- **Alternatives considered:** reenviar comandos manuales continuamente o exponer una configuracion editable en la UI. Se descarto por innecesario y porque mezclaria input UI con simulacion continua.

## Risks / Trade-offs

- **[Recuperacion AUTO todavia demasiado agresiva]** -> Limitar la restauracion a un contexto claro de evitacion previa o patron de wheel speeds de evitacion.
- **[Deteccion frontal demasiado sensible o demasiado laxa]** -> Mantener una heuristica simple de cono/rayo con constantes visibles y ajustables.
- **[Demo demasiado rapido o dificil de controlar]** -> Ajustar la constante visual a un valor moderado y dejarla explicita para futuros cambios.
- **[Desalineacion entre tests viejos y comportamiento nuevo]** -> Actualizar tests unitarios y HTTP para afirmar tanto evitacion como recuperacion sin afectar otras rutas.
- **[Confusion entre logica funcional y visual]** -> Mantener el cooldown, la escala de velocidad y el rollback exclusivamente en `demo.js`, mientras la recuperacion AUTO minima se implementa en `tick()`.
