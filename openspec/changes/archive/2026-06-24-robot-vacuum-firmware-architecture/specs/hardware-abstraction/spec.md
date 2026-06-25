## ADDED Requirements

### Requirement: Firmware shall depend on trait-based hardware abstractions
The application controller and domain controllers SHALL depend on Rust traits named `WheelMotorDriver`, `SuctionDriver`, `BrushDriver`, `SensorReader`, `BatteryDriver`, `DockingDriver`, and `Clock` rather than on concrete simulated driver implementations.

#### Scenario: Controller uses trait implementation
- **WHEN** firmware logic commands wheel movement
- **THEN** it SHALL do so through the `WheelMotorDriver` trait interface rather than by accessing simulator-specific fields directly

### Requirement: Traits shall separate capabilities by responsibility
Each hardware trait SHALL represent a focused capability boundary so tests and future production drivers can implement only the behavior relevant to that device type.

#### Scenario: Sensor access remains isolated
- **WHEN** the safety manager evaluates the environment
- **THEN** it SHALL obtain a `SensorSnapshot` through `SensorReader` without requiring wheel, brush, or suction driver access

#### Scenario: Dock detection remains isolated
- **WHEN** the firmware evaluates whether `ReturningToDock` should transition to `Charging`
- **THEN** it SHALL obtain dock detection through `DockingDriver` rather than through `SensorSnapshot`

### Requirement: Clock access shall be abstracted
The firmware SHALL use a `Clock` abstraction for time-dependent behaviors such as evaluating manual movement duration or periodic control-loop timing.

#### Scenario: Manual movement timeout
- **WHEN** a manual movement command includes a finite `duration_ms`
- **THEN** the firmware SHALL use the `Clock` abstraction to determine when the manual movement should stop
