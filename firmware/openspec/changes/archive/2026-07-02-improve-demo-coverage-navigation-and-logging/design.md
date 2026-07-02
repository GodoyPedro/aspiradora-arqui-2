## Context

El backend Rust con `axum` ya expone el contrato principal `/commands/*` y `/status`, junto con helpers de simulacion usados por el demo local. El frontend del demo mantiene posicion y heading visuales en el navegador y transforma wheel speeds del backend en movimiento 2D.

El problema actual no es de arquitectura base sino de credibilidad del demo: el robot detecta obstaculos demasiado pronto, evita antes de tocar, puede quedarse girando alrededor de un obstaculo central, no deja rastro de cobertura y no guarda una corrida exportable para analisis posterior.

El objetivo de este cambio es mantener el sistema simple y explicable. No se agrega SLAM, memoria de mapa, lidar, camaras, dependencias frontend externas ni planning avanzado. Solo se mejora el simulador local para parecerse mas a una aspiradora sencilla real.

## Goals / Non-Goals

**Goals:**
- Hacer que `AUTO` avance hacia adelante por defecto y siga intentando limpiar.
- Hacer que `bumper_pressed` sea el trigger principal de reaccion por colision y dejar `obstacle_detected` como deteccion frontal de corto alcance solamente.
- Reemplazar el patron de "retroceder" por giro in-place y reanudacion automatica de avance.
- Evitar loops obvios con una heuristica deterministica simple.
- Pintar cobertura limpiada y poder exportarla junto con la timeline del movimiento.
- Agregar controles de velocidad, reset, nuevo mapa y generacion de log dump.
- Persistir un dump JSON por sesion/mapa en una carpeta local segura y mostrar siempre la sesion activa en la UI.

**Non-Goals:**
- Agregar navegacion inteligente, path planning, SLAM o localizacion real.
- Cambiar el contrato principal `/commands/*` o convertir `/simulation/*` en API publica.
- Incorporar hardware real, Android, Bluetooth, ESP32, GPIO o dependencias frontend externas.
- Relajar la reaccion de seguridad para `drop_off_detected`, `wheel_stuck`, `brush_stuck`, `top_cover_open` o `dust_container_full`.

## Decisions

### 1. AUTO avanza por defecto y gira in-place ante bloqueo frontal o contacto
- **Decision:** en `Cleaning/AUTO`, el comportamiento nominal sera `forward`. `bumper_pressed` sera el trigger principal para reaccionar a obstaculos normales. `obstacle_detected` solo se usara como proximidad frontal de muy corto alcance y no debera hacer que el robot quede frenado lejos del obstaculo. Si aparece `bumper_pressed` o un obstaculo frontal ya muy cercano, el controlador iniciara una maniobra de giro en el lugar durante una ventana fija y luego retomara avance automaticamente.
- **Rationale:** se acerca al comportamiento de una aspiradora basica real, evita frenadas prematuras y evita que una colision normal se trate como error o stop indefinido.
- **Alternatives considered:** seguir con evitacion temprana o retroceder antes de girar. Se descartan porque limpian peor y se sienten menos naturales en el demo.

### 2. Anti-loop deterministico y acotado
- **Decision:** la direccion de giro por defecto sera consistente, por ejemplo clockwise/right. La heuristica anti-loop solo intervendra despues de eventos repetidos que indiquen que el robot esta circulando o golpeando el mismo obstaculo varias veces dentro de una ventana corta. La salida podra ser un giro mas largo o un giro opuesto temporal una vez.
- **Rationale:** mantiene el comportamiento facil de explicar y evita loops obvios sin introducir memoria de mapa ni path planning.
- **Alternatives considered:** pseudo-random puro, memoria de mapa o path planning. Se descartan por ser innecesarios para un demo explicable. Esta heuristica no implica map memory ni planificacion, solo escape deterministico local.

### 3. El frontend sigue siendo la fuente de verdad visual local
- **Decision:** la posicion, heading, cobertura y timeline de la corrida seguiran viviendo en el frontend. El backend solo recibira un dump completo al final y expondra reset/log endpoints demo-only.
- **Rationale:** simplifica el backend y evita streaming de cada frame.
- **Alternatives considered:** registrar cada frame en backend en tiempo real. Se descarta por complejidad y por no aportar valor al objetivo del demo.

### 4. Colision visual con rollback y sensores recalculados en cada frame
- **Decision:** el navegador permitira acercamiento real a obstaculos, aplicara `obstacle_detected` solo en corto alcance frontal y activara `bumper_pressed` solo cuando la huella circular del robot choque con pared u obstaculo. Al detectar choque, el movimiento visual se clamp/rollbackea a la ultima posicion valida. Un obstaculo frontal no debera frenar al robot lejos del obstaculo; como mucho podra disparar un giro corto cuando ya este muy cerca.
- **Rationale:** evita que el robot se meta adentro de un obstaculo, mantiene sensores coherentes con lo que se ve y refuerza que el contacto sea la reaccion principal.
- **Alternatives considered:** proximidad amplia para ambos flags o overlap visual permitido. Se descartan porque generan falsos positivos y loops artificiales.

### 5. Cobertura basada en grilla simple exportable
- **Decision:** el frontend mantendra una grilla de cobertura de tamano fijo y marcara celdas cubiertas con el footprint circular del robot mientras se mueve. El dump exportara la grilla o lista compacta de celdas y un porcentaje acumulado final, y la timeline incluira el porcentaje de cobertura por frame o sample.
- **Rationale:** es barato, visible y suficiente para analizar calidad de limpieza sin cartografia real.
- **Alternatives considered:** raster fino pixel-a-pixel o poligonos complejos. Se descartan por sobreingenieria.

### 6. Un log session por mapa generado
- **Decision:** cada vez que se genere un nuevo mapa se abrira inmediatamente una nueva sesion con `session_id`, `map_id` si se separa, metadatos completos del mapa, timeline vacia y resumen reiniciado. Cada sesion tendra exactamente una ruta objetivo de dump, por ejemplo `demo-logs/<session_id>/log.json`. El frontend mostrara siempre el `session_id` activo. Al pulsar "Generate Log Dump", el frontend enviara el payload completo de esa sesion a `POST /simulation/log-dump` y el backend lo persistira en esa ruta objetivo.
- **Rationale:** el pedido necesita un archivo facil de mandar luego para analisis, asociado a un mapa concreto y visible para el usuario mientras corre el demo.
- **Alternatives considered:** usar un solo archivo global o descargar solo del lado cliente. Se descartan porque dificultan trazabilidad y persistencia local.

### 7. Reset demo-only explicito y separado de nuevo mapa
- **Decision:** `POST /simulation/reset` restaurara el controlador a un estado inicial seguro y devolvera `RobotStatus`. El frontend lo usara cuando el usuario resetee corrida/posicion, pero ese reset debera conservar el mapa y la sesion actual. Solo "Generate new map" creara nuevos obstaculos, nueva sesion y nueva ruta objetivo de dump.
- **Rationale:** evita resets parciales inconsistentes entre UI local y backend y deja clara la diferencia entre reiniciar la corrida y abrir una sesion nueva.
- **Alternatives considered:** reset solo del lado frontend. Se descarta porque deja el firmware con estado previo oculto.

## Risks / Trade-offs

- **[La heuristica anti-loop puede seguir siendo imperfecta]** -> se prioriza simplicidad deterministica sobre exhaustividad, con tests dirigidos a no quedar en giro permanente.
- **[Alta velocidad visual puede saltear colisiones]** -> el frontend debera subdividir pasos internos para mantener deteccion estable en 2x, 5x, 10x o equivalentes.
- **[El dump puede crecer demasiado]** -> se mantiene JSON simple con timeline razonable y resumen agregado; no se persisten binarios ni capturas.
- **[Capacidad `demo-frontend` aun no esta sincronizada en specs canonicas]** -> esta propuesta sigue usando esa capacidad en la delta del cambio para revision; el sync quedara para cuando el cambio se implemente y apruebe.
