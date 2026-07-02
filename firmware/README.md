# aspiradora-arqui-2

Simulador de firmware para una aspiradora robotica domestica, implementado en Rust. El proyecto expone una API HTTP/JSON con `axum`, modela el hardware mediante traits y usa drivers simulados en memoria para poder desarrollar y probar la logica sin hardware fisico.

## Objetivo

El foco de este repositorio es la capa de firmware:

- recibir comandos externos;
- validar requests y estado actual del robot;
- coordinar movimiento, limpieza, bateria, docking y seguridad;
- exponer estado y telemetria;
- mantener una arquitectura testeable y desacoplada del hardware real.

La capa de firmware no incluye Bluetooth, GPIO real ni código específico de una placa embebida.
La app Android de control remoto vive separada en `android-app/` y consume la API HTTP.

## Diagrama General

![Diagrama general del proyecto](./docs/img/diagrama-general.png)

## Estado Actual Del Proyecto

La implementacion actual incluye:

- API HTTP/JSON con `axum`;
- `RobotController` como orquestador principal;
- maquina de estados del robot;
- traits de HAL para desacoplar la logica del hardware;
- drivers simulados en memoria;
- loop periodico `tick()`;
- modo `AUTO` y movimiento `MANUAL`;
- manejo de errores de seguridad;
- transicion por bateria baja hacia `RETURNING_TO_DOCK`;
- transicion de docking a `CHARGING`;
- demo local servido por el mismo backend en `/demo`;
- tests unitarios y tests HTTP de integracion.

## Arquitectura

La estructura principal del crate es:

- `src/api`: rutas HTTP, modelos JSON, assets del demo y mapeo de errores HTTP.
- `src/application`: `RobotController`, coordinacion de comandos y loop `tick()`.
- `src/domain`: comandos internos, estados, errores y modelos del robot.
- `src/controllers`: logica funcional separada por dominio.
- `src/hal`: traits para motores, sensores, bateria, docking y clock.
- `src/simulation`: drivers simulados y configuracion del entorno de ejecucion.
- `tests`: tests del controlador y tests HTTP contra el router de `axum`.
- `openspec`: especificaciones y cambios guiados por OpenSpec.

## Flujo De Ejecucion

1. Un cliente externo envia un request HTTP/JSON.
2. La capa `api` parsea el payload y lo transforma en un `RobotCommand`.
3. `RobotController` valida el comando segun el `RobotState` actual.
4. El controlador coordina los domain controllers y los drivers HAL.
5. `tick()` evalua sensores, seguridad, bateria y docking.
6. La API devuelve un `RobotStatus` o un error HTTP estructurado.

## Estado Inicial

La simulacion arranca en `STANDBY` con:

- ruedas detenidas;
- succion apagada;
- cepillos apagados;
- `current_error = None`;
- `cleaning_mode = AUTO`;
- bateria simulada configurable, con `80%` por defecto.

El estado `OFF` existe en el modelo, pero en esta etapa queda documentado como extension futura y no es alcanzable por la API HTTP.

## Maquina De Estados

Estados modelados:

- `OFF`
- `STANDBY`
- `CLEANING`
- `PAUSED`
- `MANUAL_CONTROL`
- `RETURNING_TO_DOCK`
- `CHARGING`
- `ERROR`

Comportamientos relevantes implementados:

- `START` desde `STANDBY` o `PAUSED` inicia limpieza `AUTO`.
- `STOP` detiene actuadores y vuelve a `STANDBY`.
- `PAUSE` detiene actuadores y pasa a `PAUSED`.
- `RETURN_TO_DOCK` inicia retorno a base.
- bateria `<= 15` durante `CLEANING` fuerza `RETURNING_TO_DOCK`.
- deteccion de dock durante `RETURNING_TO_DOCK` pasa a `CHARGING`.
- bateria `100` durante `CHARGING` vuelve a `STANDBY`.
- condiciones criticas de seguridad pasan a `ERROR`.

## Controladores De Dominio

El firmware separa responsabilidades en controladores especializados:

- `MotionController`: avance, retroceso, giros, stop y velocidades de ruedas.
- `CleaningController`: succion, cepillos laterales y detencion de limpieza.
- `BatteryManager`: lectura de bateria, bateria baja, carga activa y bateria llena.
- `DockingManager`: disponibilidad del dock, deteccion de base y retorno a carga.
- `SafetyManager`: deteccion de condiciones criticas y parada segura de actuadores.

## HAL Y Simulacion

La logica del firmware depende de traits, no de hardware concreto. Los contratos principales estan en `src/hal/traits.rs`:

- `WheelMotorDriver`
- `SuctionDriver`
- `BrushDriver`
- `SensorReader`
- `BatteryDriver`
- `DockingDriver`
- `Clock`

Las implementaciones actuales viven en `src/simulation/drivers.rs` y guardan estado en memoria.

## Comportamientos Implementados

### Limpieza y navegacion

- `AUTO` inicia limpieza con ruedas, succion y cepillos activos.
- En `AUTO`, el robot avanza por defecto.
- `bumper_pressed` se mantiene por compatibilidad con el backend, pero en el demo significa contacto virtual por proximidad extrema, no bumper fisico real.
- `obstacle_detected` queda reservado para proximidad frontal de corto alcance; no debe frenar al robot lejos del obstaculo.
- Si hay choques repetidos en poco tiempo, el controlador aplica una heuristica deterministica simple de escape con giro mas largo u oposicion temporal. No hay memoria de mapa ni path planning.
- `MANUAL_MOVE` permite mover el robot con duracion controlada por clock simulado.

### Seguridad

Las siguientes condiciones llevan al robot a `ERROR` y detienen ruedas, succion y cepillos:

- `drop_off_detected`
- `wheel_stuck`
- `brush_stuck`
- `top_cover_open`
- `dust_container_full`

### Modos de limpieza

En esta version solo se implementan:

- `AUTO`
- `MANUAL`

Los modos `ZIGZAG`, `WALL_FOLLOWING` y `SPOT` estan documentados como extensiones futuras y hoy se rechazan con `400 Bad Request` y codigo `UNSUPPORTED_MODE`.

## API HTTP Principal

### Endpoints

- `POST /commands/start`
- `POST /commands/stop`
- `POST /commands/pause`
- `POST /commands/return-to-dock`
- `POST /commands/manual-move`
- `POST /commands/mode`
- `POST /commands/clear-error`
- `GET /status`

### Reglas de transporte

- `GET /status` siempre devuelve el estado actual, incluso si el robot ya esta en `ERROR`.
- JSON malformado o payload invalido devuelve HTTP `400`.
- comandos validos pero incompatibles con el estado actual devuelven HTTP `409`.
- respuestas HTTP `400` y `409` incluyen al menos `code` y `message`.

### Ejemplos

Iniciar limpieza automatica:

```http
POST /commands/start
```

Mover manualmente durante 1.5 segundos:

```http
POST /commands/manual-move
Content-Type: application/json

{
  "direction": "FORWARD",
  "speed": 45,
  "duration_ms": 1500
}
```

Fijar modo de limpieza:

```http
POST /commands/mode
Content-Type: application/json

{
  "mode": "AUTO"
}
```

Consultar estado:

```http
GET /status
```

Ejemplo de `RobotStatus`:

```json
{
  "state": "STANDBY",
  "cleaning_mode": "AUTO",
  "battery_percent": 80,
  "is_charging": false,
  "suction_enabled": false,
  "brushes_enabled": false,
  "left_wheel_speed": 0,
  "right_wheel_speed": 0,
  "current_error": null,
  "sensors": {
    "obstacle_detected": false,
    "drop_off_detected": false,
    "bumper_pressed": false,
    "dust_container_full": false,
    "wheel_stuck": false,
    "brush_stuck": false,
    "top_cover_open": false
  }
}
```

Ejemplo de error estructurado:

```json
{
  "code": "COMMAND_NOT_ALLOWED",
  "message": "PAUSE_CLEANING is only allowed while actively moving",
  "current_state": "STANDBY"
}
```

## Demo Local En El Navegador

El backend ahora sirve un demo local en:

- `GET /demo`
- `GET /static/demo.css`
- `GET /static/demo.js`

El demo muestra:

- una sala 2D;
- el robot como objeto circular;
- obstaculos generados por mapa;
- base de carga;
- indicador de heading;
- pintura de cobertura limpiada;
- visualizacion de sensores;
- panel de `RobotStatus`;
- controles para comandos principales, reset, cambio de mapa, velocidad y log dump;
- `session_id` visible del mapa/log activo.
- una seccion `Batch Simulation` para correr multiples mapas en serie, guardar logs por corrida y emitir un `batch-summary.json`.

### Como ejecutarlo

Desde la carpeta `firmware/`:

```bash
cargo run
```

Luego abrir:

```text
http://127.0.0.1:3000/demo
```

### Modelo del demo

- La posicion visual del robot vive solo en el navegador.
- El backend sigue siendo la fuente de verdad para estado, actuadores, bateria, carga, errores y sensores.
- El frontend consulta `GET /status`, conserva la autoridad espacial de `x/y/heading`, calcula proximidad/colision y sincroniza sensores con el backend.
- El frontend envia `delta_ms` speed-adjusted a `POST /simulation/tick`, por lo que el clock simulado del backend avanza en cada ciclo del demo.
- La velocidad visual usa una escala del frontend y un selector `1x/2x/5x/10x`; esa misma escala tambien afecta el `delta_ms` del demo, no la API principal de comandos.
- Mientras el backend esta en `CLEANING` con modo `AUTO`, el frontend usa un controller visual reactivo propio y emite `demo_left_wheel_speed`, `demo_right_wheel_speed` y `demo_navigation_phase`; fuera de `CLEANING/AUTO` ese controller no corre.
- El backend sigue siendo la fuente de verdad para `Cleaning`, `ReturningToDock`, `Charging`, `Error`, safety stops, battery y docking detectado, pero no conoce las fases reactivas del demo ni sus heading targets intermedios.
- Si un substep visual hacia adelante queda bloqueado por pared u obstaculo, o el sensor frontal entra en umbral de contacto virtual, el demo mantiene la ultima pose valida, publica `bumper_pressed = true` a `POST /simulation/sensors` por compatibilidad, luego llama inmediatamente a `POST /simulation/tick` y usa el status devuelto para dejar que el backend pase de `60/60` al patron de giro `AUTO`.
- Los botones manuales envian un solo `MANUAL_MOVE` por click con `duration_ms = 2000`.
- El boton `Return to Dock` llama `POST /commands/return-to-dock`; si el backend lo acepta, la UI/log reflejan `RETURNING_TO_DOCK` de inmediato; si lo rechaza, la UI muestra el error y el timeline registra `RETURN_TO_DOCK_REJECTED`, sin arrancar homing local.
- `obstacle_detected` representa un obstaculo cercano delante del robot, dentro del cono frontal calculado segun el heading visual.
- `bumper_pressed` representa `proximity_contact` o contacto virtual por proximidad extrema; no representa un bumper fisico real.
- El controller reactivo usa fases explicitas `ROOM_CROSSING`, `CONTACT_BACKUP`, `WALL_ALIGN`, `WALL_FOLLOW`, `WALL_RELEASE`, `OBSTACLE_BACKUP`, `OBSTACLE_TURN_AWAY`, `OBSTACLE_ESCAPE_FORWARD` y `ANTI_LOOP_ESCAPE`.
- El avoidance normal de `AUTO` es contact-driven: `obstacle_detected` queda de muy corto alcance y no dispara giros lejanos por si solo; el cambio de fase ocurre por contacto virtual de proximidad, `forward blocked` real o anti-loop.
- Los timers de giro `AUTO` y de `MANUAL_MOVE` dependen del clock simulado, no del reloj real del navegador.
- El movimiento visual usa substeps internos acotados tanto lineales como angulares, con caps por frame. Es un mecanismo de precision de colision del demo, no odometria fisica ni odometria real.
- Si una actualizacion visual intentaria meter el robot dentro de una pared u obstaculo, el navegador hace rollback/clamping a la ultima posicion valida para evitar overlap persistente y registra el detalle simple de la colision en el timeline del dump.
- `bumper_pressed` no se limpia solo porque el robot este girando; se limpia cuando el robot ya no esta en umbral de contacto virtual y un futuro avance vuelve a ser valido.
- Si un giro local se prolonga demasiado sin progreso espacial, el demo activa un guard rail con `LONG_TURN_GUARD`, puede ejecutar `ESCAPE_TURN` o `ESCAPE_FORWARD_ATTEMPT`, y agrega `turn_episode_id` a los eventos de giro para que el dump diferencie un giro largo de varios re-triggers.
- Durante `RETURNING_TO_DOCK`, el frontend es la autoridad de movimiento visual: usa guidance local hacia la base, priorizada por sobre el `AUTO` normal, y registra campos como `backend_left_wheel_speed`, `backend_right_wheel_speed`, `demo_docking_phase`, `demo_target_heading`, `heading_error` y `distance_to_dock` para dejar explicita la diferencia entre backend y render.
- Si el docking visual queda bloqueado, el demo registra `DOCK_BLOCKED`, `DOCK_RECOVERY_TURN` o `DOCK_STUCK_DIAGNOSTIC` y vuelve a intentar llegar al dock en vez de seguir indefinidamente con `30/30`.
- El demo pinta cobertura del piso con una grilla local y exporta esa misma grilla en el dump.
- `Stop`, `Reset cleaning and position` y `Generate new map` limpian `obstacle_detected` y `bumper_pressed` para evitar sensores stale en la sesion siguiente.
- `Reset cleaning and position` conserva el mapa y el `session_id` actual, pero limpia coverage, historia local, pose y resetea el backend.
- `Generate new map` crea obstaculos nuevos, nuevo `session_id`, nueva historia local y nuevo target de dump.
- Durante un batch, los controles manuales quedan deshabilitados para no corromper la corrida activa; al terminar o cancelar, vuelven a habilitarse.

### Batch Simulation

El demo ahora permite ejecutar un batch automatico desde la misma pagina `/demo`.

Configuracion disponible:

- cantidad de corridas;
- tiempo maximo simulado por corrida en milisegundos;
- cobertura objetivo;
- velocidad del batch;
- opcion de pedir `Return to Dock` despues de llegar a tiempo/cobertura;
- condiciones de corte para tiempo, cobertura, docking exitoso y diagnostico de stuck;
- cantidad de obstaculos del mapa si se quiere fijar.

Controles:

- `Start Batch`
- `Stop Batch`
- `Clear Batch Results`

El panel muestra:

- `batch_id`;
- corrida actual y total;
- `session_id` de la corrida activa;
- tiempo simulado transcurrido;
- cobertura actual;
- estado de la corrida;
- motivo final de corte;
- corridas completadas;
- corridas fallidas/atascadas;
- ultimo log guardado;
- ruta del `batch-summary.json`.

Comportamiento del batch:

- cada corrida genera mapa nuevo y `session_id` nuevo;
- el frontend resetea pose, coverage, timeline, sensores locales, estado de docking y contadores de diagnostico;
- el backend se resetea via `POST /simulation/reset`;
- la corrida arranca con `POST /commands/start`;
- si se alcanza tiempo o cobertura y `Return to Dock` esta habilitado, el demo llama `POST /commands/return-to-dock` y espera `CHARGING`, timeout o stuck;
- cada corrida termina con exactamente un `finish_reason`;
- al pulsar `Stop Batch`, la corrida activa se cierra con `USER_CANCELLED`, se intenta guardar su log, se guarda un resumen parcial del batch y no se lanzan nuevas corridas.

## Endpoints `/simulation/*`

Para soportar el demo local y ciertos tests HTTP, el backend expone helpers adicionales:

- `POST /simulation/sensors`
- `POST /simulation/battery`
- `POST /simulation/docking`
- `POST /simulation/reset`
- `POST /simulation/batch-summary`
- `POST /simulation/log-dump`
- `POST /simulation/tick`

Estos endpoints:

- son solo para simulacion, demo y testing;
- no son endpoints de firmware productivo;
- no forman parte del contrato principal para Android u otros clientes externos;
- no reemplazan la API principal `/commands/*` y `/status`.

Reglas importantes:

- `POST /simulation/sensors` y `POST /simulation/docking` aceptan partial updates.
- `POST /simulation/battery` rechaza valores fuera de `0..=100` con `HTTP 400`.
- `POST /simulation/reset` devuelve el `RobotStatus` inicial seguro del simulador.
- `POST /simulation/batch-summary` exige un payload `{ "batch_id": "...", "summary": { ... } }`, valida que `summary.batch_id` coincida con el top-level, persiste solo `summary` como pretty JSON bajo `demo-logs/<batch_id>/batch-summary.json` y devuelve `{ "saved_path": "demo-logs/<batch_id>/batch-summary.json" }`.
- `POST /simulation/log-dump` sigue aceptando el payload manual existente y lo guarda en `demo-logs/<session_id>/log.json`, pero en batch tambien acepta `{ "batch_id": "...", "session_id": "...", "log": { ... } }`, valida ids top-level vs embebidos, persiste solo `log` bajo `demo-logs/<batch_id>/<session_id>/log.json` y devuelve `{ "saved_path": "demo-logs/..." }`.
- `POST /simulation/tick` acepta un payload opcional `{ "delta_ms": <1..=1000> }`, avanza el clock simulado y luego ejecuta `tick()`. Si no hay payload usa un default seguro.
- todos los endpoints `/simulation/*`, salvo `log-dump` y `batch-summary`, devuelven `RobotStatus`.
- JSON malformado o payload invalido devuelve `HTTP 400` con el mismo formato de error estructurado del resto de la API.

## Coverage y log dumps

- Cada mapa generado crea inmediatamente una nueva sesion de demo con `session_id` visible en la UI.
- Cada sesion tiene exactamente una ruta objetivo de dump: `demo-logs/<session_id>/log.json`.
- Cada corrida batch usa `demo-logs/<batch_id>/<session_id>/log.json`.
- Cada batch guarda un resumen agregado en `demo-logs/<batch_id>/batch-summary.json`.
- El frontend acumula timeline, coverage, eventos y metadata del mapa durante la corrida.
- Al pulsar `Generate Log Dump`, el navegador envia el payload completo al backend y este lo guarda en `demo-logs/`.
- En modo batch, el frontend agrega metadata `batch_id`, `run_index`, `batch_total_runs`, `batch_mode = true` y `run_config` a cada log antes de persistirlo.
- El dump incluye:
  - `session_id`, `map_id`, `created_at`;
  - dimensiones del room, radio del robot, pose inicial, dock y lista completa de obstaculos;
  - velocidad de simulacion, thresholds de sensores y configuracion de la grilla de coverage;
  - timeline por frame con pose, heading, wheel speeds backend, wheel speeds visuales del demo, `demo_navigation_phase`, estado backend, sensores, coverage, eventos y `event_detail` para distinguir contacto virtual de proximidad, pared vs obstaculo, `target_heading`, `wall_side`, `escape_side`, `obstacle_id`, episodios de giro y guidance de docking;
  - coverage compacta exportable y porcentaje final;
  - resumen con frames, tiempos real/simulado, contactos, turns y coverage final.
- El batch summary agrupa:
  - `config`;
  - `runs` con un run summary en snake_case por corrida;
  - `aggregate_metrics`;
  - `worst_runs`.
- Cada run summary incluye `room_crossing_segments`, `wall_follow_segments`, `proximity_contacts`, `wall_contacts`, `obstacle_contacts`, `obstacle_escape_attempts`, `obstacle_escape_successes`, `obstacle_escape_failures`, `anti_loop_escapes`, `backup_blocked_events`, `max_no_movement_ms`, `coverage_per_real_second` y `coverage_per_simulated_second`.
- `turns` cuenta giros intencionales del cleaning visual reactivo (`WALL_ALIGN`, `WALL_RELEASE`, `OBSTACLE_TURN_AWAY` y `ANTI_LOOP_TURN`), y excluye giros de docking y manuales.
- `aggregate_metrics` del batch incluye los totales de esos mismos campos reactivos, mas docking, stuck diagnostics y contactos.
- Los `timestamp_ms` del timeline se emiten en forma no decreciente para que el analisis posterior pueda confiar en orden y duraciones.
- `total_simulated_time_ms`, `total_real_time_ms` y `run_config.max_simulated_time` se expresan en milisegundos.
- En batch, el corte por tiempo usa el reloj real del navegador. `simulation_speed` acelera la simulacion interna pero no acorta la duracion real de cada run.

### Enviar evidencia para analisis

- Para una corrida manual, alcanza con compartir `demo-logs/<session_id>/log.json`.
- Para un batch, conviene zippear la carpeta completa `demo-logs/<batch_id>/`.
- Las metricas mas utiles para diagnosticar debilidades son `finish_reason`, cobertura final, contactos contra pared/obstaculos, `long_turn_guards`, `escape_turns`, `dock_blocked_events`, `dock_stuck_diagnostics` y las rachas largas sin movimiento.

Ese JSON esta pensado para mandarlo luego a ChatGPT u otra herramienta de analisis de comportamiento.

## Validaciones Importantes

- `MANUAL_MOVE` requiere `direction`, `speed` y `duration_ms`.
- `duration_ms` debe ser mayor que `0`.
- `speed` debe estar en el rango `0..=100`.
- para `FORWARD`, `BACKWARD`, `LEFT` y `RIGHT`, `speed` debe ser mayor que `0`.
- para `STOP`, `speed` puede ser `0`.
- `POST /commands/start` con bateria `<= 15` devuelve `409`.
- `POST /commands/start` mientras ya esta limpiando devuelve `409`.
- `POST /commands/stop` desde `STANDBY` devuelve `409`.
- `POST /commands/return-to-dock` desde `STANDBY` puede iniciar retorno si el dock esta disponible.
- `POST /commands/clear-error` fuera de `ERROR` devuelve `409`.

## Testing

La suite actual cubre:

- transiciones de estado principales;
- validacion de comandos;
- lectura de estado por HTTP;
- errores criticos de seguridad;
- bateria baja y docking;
- rechazo de modos no soportados;
- maniobras `AUTO` ante obstaculo o bumper;
- reanudacion `AUTO` tras turn window y escape anti-loop simple;
- endpoints `/simulation/*`;
- reset y persistencia de log dumps por sesion;
- serving de assets del demo;
- consistencia entre controladores y API HTTP.

Comandos utiles:

```bash
cargo test
cargo run
```

La aplicacion levanta por defecto en:

```text
http://127.0.0.1:3000
```

## Android Remote Control App

Este repositorio tambien incluye una app Android nativa en `android-app/`
para controlar la aspiradora por la API HTTP del firmware.

### Ejecutar firmware y app juntos

1. Inicia el firmware desde la raiz del repositorio:

```bash
cargo run
```

2. Abre `android-app/` en Android Studio.
3. Ejecuta la app `Vacuum Remote` en un emulador o dispositivo Android.
4. Usa el campo `Base URL` de la pantalla principal para apuntar al firmware.

Para el emulador Android, usa:

```text
http://10.0.2.2:3000
```

Para un dispositivo fisico en la misma red WiFi, usa la IP local de la
computadora que ejecuta el firmware:

```text
http://<ip-local-de-tu-pc>:3000
```

La app permite consultar estado, iniciar, detener, pausar, volver al dock,
limpiar errores, seleccionar modo `AUTO` y enviar movimientos manuales con
velocidad y duracion configurables. No implementa apagado/encendido real de
hardware porque el firmware actual no expone endpoints de power on/off ni hace
alcanzable el estado `OFF` por HTTP.

## Qué Conviene Versionar

En este repositorio tiene sentido subir:

- `src/`
- `tests/`
- `Cargo.toml`
- `Cargo.lock`
- `README.md`
- `openspec/`
- `.codex/skills/` si querés conservar el flujo de trabajo del proyecto con Codex/OpenSpec

No conviene subir:

- `target/`
- `.vscode/`
- exports locales como `aspiradora-arqui2.zip`

## Limitaciones Del Alcance

Quedan explícitamente fuera de esta entrega de firmware:

- Bluetooth;
- Wi-Fi embebido real;
- GPIO real;
- codigo especifico de una placa;
- SLAM;
- mapeo de habitaciones;
- camara o lidar;
- algoritmos avanzados de navegacion.
