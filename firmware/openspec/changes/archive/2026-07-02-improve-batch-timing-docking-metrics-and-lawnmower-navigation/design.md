## Context

El repo ya tiene un demo local con frontend estatico, sensores calculados en navegador, persisted logs por sesion y un batch runner demo-only. Esa capa sirve para analizar comportamiento espacial, pero hoy mezcla varias responsabilidades de forma inconsistente:

- el batch puede usar una base temporal distinta de la del tick y de los timestamps del timeline;
- el docking stuck detection castiga casos donde el robot esta alineando heading correctamente aunque `x/y` casi no cambie;
- el guidance de retorno puede sobrecorregir y oscilar a alta velocidad;
- las metricas de `turns`, `ANTI_LOOP_ESCAPE` y eventos de docking no son suficientemente canonicas;
- el `AUTO` sigue demasiado cercano a un bounce/avoidance reactivo y no genera cobertura ordenada.

En este repo sigue sin existir un spec canonico `demo-frontend` en `openspec/specs`, por lo que los requisitos funcionales del navegador continuaran documentados en `simulation-and-testing`. `sensor-actuator-control` debe seguir describiendo comportamiento backend-safe de estado, safety, actuators y docking, sin implicar que el firmware conoce lanes, geometria espacial ni bypass paths del demo.

## Goals / Non-Goals

**Goals:**
- Hacer que batch stop conditions, stuck windows, timeline timestamps y `total_simulated_time_ms` usen una sola nocion de tiempo simulado por run.
- Evitar falsos positivos de docking stuck cuando hay progreso angular real hacia el dock.
- Suavizar el docking con steering proporcional y recovery bounded.
- Unificar definiciones de turns, anti-loop events y docking-turn semantics para summaries y README.
- Reemplazar el `AUTO` frenetico por un patron lawn-mower simple, deterministico y principalmente contact-driven.
- Mantener el cambio dentro del simulador/demo, sin convertirlo en navegacion real ni planning global.

**Non-Goals:**
- Agregar memoria de mapa, path planning real, localizacion, SLAM, lidar, camara o avoidance avanzado.
- Exponer nuevos contratos de API productiva fuera de `/simulation/*`.
- Llevar la logica de lane pattern al firmware como si fuera movimiento real del robot fisico.
- Introducir frameworks frontend o dependencias externas.

## Decisions

### 1. El tiempo batch debe ser estrictamente simulado y unico
- **Decision:** cada run mantendra un contador `simulated_elapsed_ms` que se incrementa con el mismo `delta_ms` usado para `/simulation/tick`, movimiento visual, `timestamp_ms`, ventanas de stuck y `total_simulated_time_ms`.
- **Rationale:** usar tiempo real para unas cosas y tiempo simulado para otras rompe comparabilidad y hace que el batch corte de forma engañosa.
- **Alternatives considered:** seguir usando tiempo real para el cutoff visible y tiempo simulado para el backend. Se descarta por inconsistente.

### 2. El loop visual y el tick backend deben compartir la misma base temporal
- **Decision:** el movimiento visual, la actualizacion de sensores y la llamada a `/simulation/tick` se calcularan sobre el mismo `delta_ms` simulado. Si un frame visual implica demasiado tiempo, el frontend puede subdividir internamente para mantener coherencia.
- **Rationale:** a `10x`, el demo necesita que pose, wheel speeds y tick sigan el mismo reloj logico.
- **Alternatives considered:** dejar que el backend corra con `delta_ms` simulado pero que el frontend siga usando `performance.now()` como base de timeline. Se descarta porque ya genero desalineacion.

### 3. Docking stuck detection debe medir progreso angular y posicional
- **Decision:** durante `RETURNING_TO_DOCK`, rotar in-place con heading mejorando hacia el dock cuenta como progreso valido. El diagnostico de stuck solo disparara si no mejora ni distancia, ni heading error, ni fase, o si se repiten recuperaciones/bloqueos por demasiado tiempo simulado.
- **Rationale:** alinearse antes de avanzar es un comportamiento esperado de docking, no una falla.
- **Alternatives considered:** seguir midiendo stuck solo por `x/y` inmovil con ruedas no nulas. Se descarta porque castiga alineaciones legitimas.

### 4. El guidance de docking usara steering proporcional capped
- **Decision:** el frontend calculara `target heading`, `heading error` y aplicara steering proporcional: giro capped para error grande, differential steering lento para error intermedio y avance controlado para error chico, desacelerando cerca del dock.
- **Rationale:** reduce overshoot y oscilacion respecto de giros fijos agresivos, especialmente a `10x`.
- **Alternatives considered:** seguir con thresholds y turn speeds fijos. Se descarta por sobrecorreccion visible.

### 5. El cleanup de `CHARGING` debe limpiar sensores transientes
- **Decision:** al detectar dock y entrar en `CHARGING`, el demo detendra movimiento visual, reflejara `0/0` wheel speeds y limpiara `obstacle_detected` y `bumper_pressed`, registrando `DOCK_DETECTED` y `CHARGING_STARTED`.
- **Rationale:** evita arrastrar señales de contacto stale hacia el siguiente estado o siguiente corrida.
- **Alternatives considered:** dejar los sensores como estaban hasta el siguiente reset. Se descarta porque ensucia logs y puede sesgar batch.

### 6. Las metricas y eventos de giro deben tener una semantica canonica
- **Decision:** `turns` sera el numero de episodios de giro abiertos por `TURN_START` o equivalente de cleaning turn. `TURNING` no contara como turn nuevo. `ANTI_LOOP_ESCAPE` quedara reservado a intervenciones reales, no a giros izquierdos normales. Los giros de docking se distinguiran de los giros de cleaning mediante event names o event detail.
- **Rationale:** batch summaries y log summaries no pueden comparar peras con manzanas.
- **Alternatives considered:** seguir contando segun heuristicas implícitas del timeline. Se descarta por ambiguedad.

### 7. El patron `AUTO` lawn-mower vivira en el frontend del demo
- **Decision:** la geometria lane-based, los lane transitions, lane spacing y obstacle bypass seguiran siendo demo-only y frontend-owned, porque el navegador ya posee `x/y/heading`, room geometry y collision ownership. El backend sigue gobernando `CLEANING`, `RETURNING_TO_DOCK`, `CHARGING` y `ERROR`.
- **Rationale:** el backend no tiene localizacion real del room en este simulador; el frontend si.
- **Alternatives considered:** mover el lawn-mower pattern al backend. Se descarta porque implicaria fingir capacidades de localization/planning que el firmware real no tiene.

### 8. El lawn-mower pattern debe ser deterministico y calmado
- **Decision:** `AUTO` seguira lanes largos; al tocar pared se tratara como lane end con secuencia turn-shift-turn; al tocar obstaculo se hara un bypass bounded y luego se retomara la lane o se marcara el lane segment como bloqueado para recovery controlado.
- **Rationale:** el objetivo es cobertura mas ordenada, menos wall hits repetidos y menos secuencias freneticas.
- **Alternatives considered:** conservar random bounce con tweaks menores. Se descarta porque no mejora suficientemente la comparabilidad entre corridas.

### 9. El avoidance debe ser principalmente contact-driven
- **Decision:** `obstacle_detected` quedara de muy corto alcance. El trigger normal principal sera `bumper_pressed` o colision real. La respuesta sera bypass controlado o lane recovery, no un giro preemptivo largo a distancia.
- **Rationale:** el comportamiento actual sobreanticipa y parece nervioso, lo que empeora cobertura y logs.
- **Alternatives considered:** mantener avoidance anticipatorio con thresholds largos. Se descarta por frenetico.

### 10. El frontend usara un controller visual explicito para `AUTO`
- **Decision:** mientras el backend reporte `Cleaning` y el modo activo sea `AUTO`, el frontend manejara un `demo_navigation_phase` y producira `demo_left_wheel_speed` y `demo_right_wheel_speed` para el movimiento visual. Ese controller no correra en `ReturningToDock`, `Charging`, `Paused`, `Error` ni `ManualControl`.
- **Rationale:** separa claramente el estado alto nivel del backend respecto de la navegacion espacial del demo.
- **Alternatives considered:** derivar todo el movimiento visual directamente de wheel speeds backend. Se descarta porque el backend no posee suficiente contexto espacial del mapa.

### 11. La maquina de estados lawn-mower debe ser concreta y acotada
- **Decision:** el controller frontend usara fases explicitas `LANE_DRIVE`, `LANE_TURN_TO_SHIFT`, `LANE_SHIFT`, `LANE_TURN_TO_REVERSE`, `OBSTACLE_BYPASS_TURN`, `OBSTACLE_BYPASS_OFFSET`, `OBSTACLE_BYPASS_REJOIN` y `PATTERN_RECOVERY`.
- **Rationale:** evita heuristicas implÃ­citas y facilita logs, metricas y depuracion de comportamiento frenetico.
- **Alternatives considered:** mantener una sola fase de cleaning con flags sueltos. Se descarta por ambiguedad.

### 12. Las constantes del patron deben ser estables y documentadas
- **Decision:** el demo definira constantes documentadas para `lane_spacing_px`, heading inicial, `lane_shift_side`, tolerancia angular y tolerancia de distancia. El default propuesto es `lane_spacing_px = robot_radius * 1.6` salvo que el cleaning width local existente sugiera mejor valor.
- **Rationale:** el usuario necesita comportamiento predecible y comparable entre corridas.
- **Alternatives considered:** dejar distancias y tolerancias como valores emergentes no documentados. Se descarta por underspecified.

### 13. Las metricas de patron son obligatorias
- **Decision:** los run summaries incluiran siempre `lane_starts`, `lane_ends`, `lane_shifts`, `lane_blocked_events`, `obstacle_bypass_attempts`, `obstacle_bypass_failures` y `pattern_recoveries`. Los batch aggregate metrics incluiran siempre sus totales correspondientes.
- **Rationale:** sin esos campos, el patron no se puede comparar ni validar de forma consistente.
- **Alternatives considered:** hacerlas opcionales si el pattern no emitio eventos. Se descarta porque obliga a formas JSON variables.

## Risks / Trade-offs

- **[El lawn-mower pattern sigue siendo simplificado y puede no cubrir casos raros]** -> aceptar cobertura imperfecta pero exigir trazas mas estables y menos reactividad erratica.
- **[Mas estado frontend implica mas complejidad del demo]** -> documentar claramente fases, lane state y metric definitions.
- **[Separar cleaning turns de docking turns aumenta costo analitico]** -> compensa con summaries y README mas claros.
- **[Substepping extra puede aumentar costo del loop]** -> mantener caps y aplicar subdivision solo cuando el delta simulado lo requiera.
