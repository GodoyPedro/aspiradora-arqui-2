## Why

El firmware ya expone una API HTTP/JSON util para tests y clientes externos, pero hoy no ofrece una forma visual y local de demostrar el comportamiento del robot simulado. Un demo browser-based acelera revisiones academicas y validacion manual sin cambiar el alcance hacia Android ni hardware real.

## What Changes

- Agregar un demo frontend local servido por el backend en `GET /demo` con assets estaticos embebidos en `/static/*`.
- Incorporar una visualizacion 2D simple del robot, la sala, obstaculos, base de carga y senales sensoriales usando HTML, CSS y JavaScript sin dependencias externas.
- Agregar endpoints demo-only bajo `/simulation/*` para mutar sensores, bateria, docking y ejecutar `tick()` desde el navegador, devolviendo `RobotStatus` y usando el formato de error estructurado existente.
- Mantener el backend como fuente de verdad para `RobotStatus`, estado del robot, actuadores, bateria y errores, mientras el navegador mantiene solo la posicion visual local.
- Extender tests HTTP y documentacion para cubrir el demo local y dejar explicito que `/simulation/*` no forma parte de la API principal de firmware.

## Capabilities

### New Capabilities
- `demo-frontend`: UI local servida por el backend para visualizar el robot simulado, enviar comandos y renderizar el estado del firmware en una sala 2D.

### Modified Capabilities
- `command-protocol`: agregar serving del demo local y endpoints `/simulation/*` documentados como helpers de demo/testing sin alterar el contrato principal `/commands/*` y `/status`.
- `simulation-and-testing`: ampliar helpers y cobertura para soportar mutacion simulada via HTTP para demo/tests y validar los nuevos endpoints.

## Impact

- Codigo backend en `src/api` o modulo equivalente para rutas `/demo`, `/static/*` y `/simulation/*`.
- Assets HTML, CSS y JavaScript embebidos en tiempo de compilacion para no depender del directorio de ejecucion.
- Nuevos tests HTTP para endpoints de simulacion, validaciones de payload y compatibilidad con la API existente.
- README actualizado con instrucciones de uso del demo y alcance de los endpoints de simulacion.
