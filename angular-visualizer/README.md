# Angular Visualizer

Angular + PixiJS frontend for the robot vacuum firmware API.

## Run

Start the firmware from the repository root:

```bash
cargo run
```

Then run the visualizer:

```bash
cd angular-visualizer
npm install
npm start
```

The Angular dev proxy forwards `/status` and `/commands/*` to `http://127.0.0.1:3000`, so the browser can use same-origin requests during development.

## Notes

The current firmware response does not include robot coordinates. The canvas therefore shows a fallback indicator until `RobotStatus` includes `position`, `pose`, or top-level `x`/`y` values. Heading is honored when supplied as `heading`, `heading_degrees`, or `orientation`.
