## Context

El proyecto ya implementa el simulador de firmware en Rust con `axum`, `RobotController`, drivers simulados, loop `tick()` y tests HTTP. El demo pedido no debe redisenar la arquitectura existente ni introducir frameworks frontend, sino montar una UI local minima sobre el backend actual y exponer helpers de simulacion explicitamente separados de la API principal.

La principal restriccion es preservar el firmware como fuente de verdad del estado funcional del robot mientras la posicion 2D del demo vive en el navegador. Tambien hay que mantener separados los concerns de demo/testing respecto del contrato principal `/commands/*`.

## Goals / Non-Goals

**Goals:**
- Servir una pagina local en `/demo` que funcione con `cargo run` y el backend actual.
- Reutilizar la API de comandos existente como canal principal de control del robot.
- Agregar endpoints `/simulation/*` limitados a mutaciones de estado simulado y ejecucion de `tick()`.
- Mostrar en el navegador una sala 2D, robot, obstaculos, base, heading y panel de sensores/telemetria.
- Mantener la simulacion visual simple, deterministica y sin dependencias JS externas.
- Documentar con claridad que el demo local no es la app Android ni una API productiva adicional.

**Non-Goals:**
- Implementar una app Android real.
- Agregar Bluetooth, Wi-Fi embebido, GPIO, drivers reales o codigo de placa.
- Agregar SLAM, path planning avanzado, camara, lidar o mapping.
- Redisenar `RobotController`, HAL o la maquina de estados.
- Convertir la posicion 2D local del navegador en fuente de verdad del firmware.

## Decisions

### 1. Servir el demo como assets estaticos embebidos desde el backend
- **Decision:** exponer `GET /demo` para el HTML y `/static/demo.css`, `/static/demo.js` para assets estaticos embebidos en tiempo de compilacion mediante `include_str!` o equivalente.
- **Rationale:** mantiene el demo acoplado al binario/backend existente, evita tooling adicional y garantiza que `cargo run` desde el crate `firmware` sea suficiente sin depender del working directory del proceso.
- **Alternatives considered:** leer archivos del filesystem en runtime o incrustar todo el HTML/CSS/JS inline en una sola ruta. Se descarto porque la primera opcion introduce fragilidad operacional y la segunda complica mantenimiento y pruebas.

### 2. Mantener el demo frontend en JavaScript plano sin build step
- **Decision:** usar HTML, CSS y JavaScript vanilla, sin npm, bundlers, React ni TypeScript.
- **Rationale:** reduce complejidad, mantiene el cambio acotado y portable para una demo academica corta.
- **Alternatives considered:** usar un framework SPA. Se descarto por ser desproporcionado para el alcance y por introducir infraestructura no pedida.

### 3. Separar estrictamente la API principal de los helpers de simulacion
- **Decision:** agregar endpoints `/simulation/sensors`, `/simulation/battery`, `/simulation/docking` y `/simulation/tick` como helpers demo/testing-only.
- **Rationale:** el frontend necesita mutar estado simulado desde el navegador, pero eso no debe contaminar el contrato principal de firmware.
- **Alternatives considered:** reutilizar `/commands/*` para todo o exponer setters genericos de estado en la API principal. Se descarto porque mezcla concerns de demo con comandos reales del firmware.

### 4. Hacer que todos los endpoints `/simulation/*` devuelvan `RobotStatus`
- **Decision:** `POST /simulation/sensors`, `POST /simulation/battery` y `POST /simulation/docking` devolveran el `RobotStatus` actualizado inmediatamente, mientras `POST /simulation/tick` ejecutara `tick()` y devolvera el `RobotStatus` resultante.
- **Rationale:** simplifica el loop del frontend, evita requests extras para reflejar cambios y hace el contrato mas homogeneo para tests.
- **Alternatives considered:** devolver `204 No Content` o requerir luego un `GET /status`. Se descarto porque agrega latencia y complejidad innecesaria al demo local.

### 5. Mantener el backend como fuente de verdad funcional
- **Decision:** el navegador mantiene solo `x`, `y` y `heading` locales, mientras que el backend conserva `RobotStatus`, sensores persistidos, bateria, carga, actuadores y errores.
- **Rationale:** preserva la arquitectura actual y evita introducir un modelo hibrido dificil de razonar.
- **Alternatives considered:** persistir posicion/heading en el backend. Se descarto porque requiere redisenar dominio y controlador para una necesidad puramente visual.

### 6. Conducir la simulacion visual con el ciclo status -> sensores -> tick
- **Decision:** cada intervalo del demo hara `GET /status`, movera el robot visualmente segun `left_wheel_speed` y `right_wheel_speed`, calculara colisiones/proximidad locales, enviara `/simulation/sensors` y luego llamara a `/simulation/tick`. El loop usara un guard simple como `isTicking` para no disparar ciclos concurrentes y renderizara errores HTTP en la UI sin romper la animacion.
- **Rationale:** refleja de forma simple la interaccion entre mundo simulado local y firmware, y reutiliza el `tick()` existente como punto de evaluacion.
- **Alternatives considered:** llamar primero a `tick()` y luego calcular sensores, o empujar toda la fisica al backend. Se descarto porque rompe el modelo pedido donde el frontend "ve" el entorno y alimenta sensores.

### 7. Definir `/simulation/sensors` y `/simulation/docking` como partial updates
- **Decision:** los payloads de sensores y docking aceptaran campos booleanos opcionales y conservaran los valores existentes cuando un campo no sea enviado. `battery_percent` se validara de forma estricta y valores fuera de `0..=100` se rechazaran con HTTP `400`.
- **Rationale:** el frontend solo calcula algunos flags por frame y no debe tener que reenviar todo el snapshot cada vez; la bateria, en cambio, debe conservar un contrato simple y explicito.
- **Alternatives considered:** reemplazo completo de sensores/docking o clamping automatico de bateria. Se descarto porque el reemplazo completo hace mas fragil el loop del demo y el clamping oculta errores de payload.

### 8. Anadir tests HTTP especificos para los endpoints de simulacion
- **Decision:** agregar tests de integracion que usen el router de `axum` para validar `/simulation/*` y confirmar que no rompen la API actual.
- **Rationale:** el riesgo principal del cambio esta en la interaccion HTTP y en conservar el comportamiento existente.
- **Alternatives considered:** cubrirlo solo con tests unitarios. Se descarto porque estos endpoints son contrato HTTP y deben testearse como tal.

## Risks / Trade-offs

- **[Divergencia entre posicion visual y estado funcional]** -> Mantener explicito que la posicion del robot en el navegador es local y reseteable, mientras el backend solo conoce wheel speeds y sensores.
- **[Endpoints de simulacion confundidos con API productiva]** -> Aislarlos bajo `/simulation/*`, documentarlos como demo/testing-only y mantener `/commands/*` sin cambios semanticos.
- **[Payloads parciales malinterpretados]** -> Documentar claramente que sensores y docking son partial updates y agregar tests de validacion para payloads invalidos.
- **[Loop visual inestable o no deterministico]** -> Usar obstaculos estaticos, una sala simple, un intervalo fijo y reglas de colision basicas.
- **[Requests concurrentes a `tick()`]** -> Asegurar un guard `isTicking` o equivalente y mostrar errores en la UI sin detener el rendering.
- **[Complejidad extra en el router]** -> Encapsular las rutas del demo y de simulacion en un modulo dedicado en vez de mezclar logica con endpoints principales.
- **[Cobertura incompleta de UX local]** -> Validar backend y helpers por test HTTP, y dejar los detalles puramente visuales fuera de pruebas exhaustivas automaticas.

## Migration Plan

1. Agregar modulo/rutas del demo local y serving de assets estaticos embebidos.
2. Agregar modelos request/response y endpoints `/simulation/*` sobre el controlador simulado compartido.
3. Implementar HTML/CSS/JS del demo y conectar el loop de polling/render con guard contra concurrencia.
4. Extender tests HTTP para `/simulation/*`, assets estaticos y compatibilidad de la API existente.
5. Actualizar README con instrucciones de uso y aclaraciones de alcance.

No se requiere migracion de datos ni rollback complejo; el cambio es aditivo. Si fuera necesario revertirlo, alcanza con eliminar las rutas `/demo`, `/static/*` y `/simulation/*` junto con los assets estaticos asociados.
