import { CommonModule } from '@angular/common';
import { AfterViewInit, Component, ElementRef, OnDestroy, OnInit, ViewChild } from '@angular/core';
import { FormsModule } from '@angular/forms';
import { Application, Container, Graphics, Text } from 'pixi.js';
import { EMPTY, Subscription, timer } from 'rxjs';
import { catchError, switchMap } from 'rxjs/operators';
import { ManualDirection, ManualMoveRequest, RobotStatus } from './models/robot-status';
import { RobotApiService } from './services/robot-api.service';
import { extractRobotPosition, projectPosition, ViewTransform } from './visualizer/position-utils';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './app.component.html',
  styleUrl: './app.component.scss'
})
export class AppComponent implements OnInit, AfterViewInit, OnDestroy {
  @ViewChild('canvasHost', { static: true }) private readonly canvasHost!: ElementRef<HTMLElement>;

  status: RobotStatus | null = null;
  connectionError = '';
  commandMessage = '';
  lastUpdated: Date | null = null;
  pollIntervalMs = 2000;

  readonly directions: ManualDirection[] = ['FORWARD', 'BACKWARD', 'LEFT', 'RIGHT', 'STOP'];
  manualMove: ManualMoveRequest = {
    direction: 'FORWARD',
    speed: 40,
    duration_ms: 1500
  };

  private pollSubscription?: Subscription;
  private pixi?: Application;
  private resizeObserver?: ResizeObserver;
  private readonly gridLayer = new Graphics();
  private readonly robotLayer = new Container();
  private readonly robotBody = new Graphics();
  private readonly robotHeading = new Graphics();
  private readonly placeholderText = new Text({
    text: 'Sin coordenadas',
    style: {
      fill: 0x5d6f65,
      fontFamily: 'Inter, Arial, sans-serif',
      fontSize: 18,
      fontWeight: '600'
    }
  });

  constructor(private readonly api: RobotApiService) {
    this.robotLayer.addChild(this.robotBody, this.robotHeading);
  }

  ngOnInit(): void {
    this.startPolling();
  }

  ngAfterViewInit(): void {
    void this.initializeCanvas();
  }

  ngOnDestroy(): void {
    this.pollSubscription?.unsubscribe();
    this.resizeObserver?.disconnect();
    this.pixi?.destroy(true, { children: true });
  }

  restartPolling(): void {
    this.pollIntervalMs = Math.max(500, Math.floor(this.pollIntervalMs || 2000));
    this.startPolling();
  }

  runCommand(command: 'start' | 'stop' | 'pause' | 'dock' | 'clear' | 'auto'): void {
    this.commandMessage = '';
    const request = {
      start: this.api.start(),
      stop: this.api.stop(),
      pause: this.api.pause(),
      dock: this.api.returnToDock(),
      clear: this.api.clearError(),
      auto: this.api.setAutoMode()
    }[command];

    request.subscribe({
      next: (status) => {
        this.applyStatus(status);
        this.commandMessage = 'Comando aplicado';
      },
      error: (error: Error) => {
        this.commandMessage = error.message;
      }
    });
  }

  sendManualMove(): void {
    const validation = this.validateManualMove(this.manualMove);
    if (validation) {
      this.commandMessage = validation;
      return;
    }

    this.api.manualMove(this.manualMove).subscribe({
      next: (status) => {
        this.applyStatus(status);
        this.commandMessage = 'Movimiento enviado';
      },
      error: (error: Error) => {
        this.commandMessage = error.message;
      }
    });
  }

  get statusTone(): string {
    const state = this.status?.state ?? 'UNKNOWN';
    if (state === 'ERROR') {
      return 'danger';
    }
    if (state === 'CLEANING' || state === 'MANUAL_CONTROL') {
      return 'active';
    }
    if (state === 'RETURNING_TO_DOCK' || state === 'CHARGING') {
      return 'transit';
    }
    return 'idle';
  }

  get activeSensors(): string[] {
    const sensors = this.status?.sensors;
    if (!sensors) {
      return [];
    }

    return Object.entries(sensors)
      .filter(([, active]) => active)
      .map(([name]) => name.replaceAll('_', ' ').toUpperCase());
  }

  private startPolling(): void {
    this.pollSubscription?.unsubscribe();
    this.pollSubscription = timer(0, this.pollIntervalMs)
      .pipe(
        switchMap(() =>
          this.api.getStatus().pipe(
            catchError((error: Error) => {
              this.connectionError = error.message;
              return EMPTY;
            })
          )
        )
      )
      .subscribe({
        next: (status) => {
          this.connectionError = '';
          this.applyStatus(status);
        }
      });
  }

  private applyStatus(status: RobotStatus): void {
    this.status = status;
    this.lastUpdated = new Date();
    this.renderCanvas();
  }

  private validateManualMove(payload: ManualMoveRequest): string {
    if (!Number.isInteger(payload.duration_ms) || payload.duration_ms <= 0) {
      return 'duration_ms debe ser mayor que 0';
    }

    if (!Number.isInteger(payload.speed) || payload.speed < 0 || payload.speed > 100) {
      return 'speed debe estar entre 0 y 100';
    }

    if (payload.direction !== 'STOP' && payload.speed === 0) {
      return 'speed debe ser mayor que 0 para avanzar, retroceder o girar';
    }

    return '';
  }

  private async initializeCanvas(): Promise<void> {
    const host = this.canvasHost.nativeElement;
    this.pixi = new Application();
    await this.pixi.init({
      resizeTo: host,
      backgroundColor: 0xf8fbf7,
      antialias: true,
      autoDensity: true,
      resolution: window.devicePixelRatio || 1
    });

    host.appendChild(this.pixi.canvas);
    this.pixi.stage.addChild(this.gridLayer, this.robotLayer, this.placeholderText);
    this.resizeObserver = new ResizeObserver(() => this.renderCanvas());
    this.resizeObserver.observe(host);
    this.renderCanvas();
  }

  private renderCanvas(): void {
    if (!this.pixi) {
      return;
    }

    const width = this.pixi.renderer.width / this.pixi.renderer.resolution;
    const height = this.pixi.renderer.height / this.pixi.renderer.resolution;
    const transform: ViewTransform = {
      originX: width / 2,
      originY: height / 2,
      scale: Math.max(12, Math.min(width, height) / 12)
    };

    this.drawGrid(width, height);
    this.drawRobot(transform, width, height);
  }

  private drawGrid(width: number, height: number): void {
    this.gridLayer.clear();
    this.gridLayer.rect(0, 0, width, height).fill({ color: 0xf8fbf7 });

    const step = Math.max(32, Math.min(width, height) / 10);
    for (let x = 0; x <= width; x += step) {
      this.gridLayer.moveTo(x, 0).lineTo(x, height).stroke({ color: 0xdce8df, width: 1 });
    }
    for (let y = 0; y <= height; y += step) {
      this.gridLayer.moveTo(0, y).lineTo(width, y).stroke({ color: 0xdce8df, width: 1 });
    }

    this.gridLayer.moveTo(width / 2, 0).lineTo(width / 2, height).stroke({ color: 0x91a99a, width: 1, alpha: 0.7 });
    this.gridLayer.moveTo(0, height / 2).lineTo(width, height / 2).stroke({ color: 0x91a99a, width: 1, alpha: 0.7 });
  }

  private drawRobot(transform: ViewTransform, width: number, height: number): void {
    const position = extractRobotPosition(this.status);
    const hasPosition = Boolean(position);
    this.robotLayer.visible = hasPosition;
    this.placeholderText.visible = !hasPosition;
    this.placeholderText.anchor.set(0.5);
    this.placeholderText.position.set(width / 2, height / 2);

    if (!position) {
      return;
    }

    const projected = projectPosition(position, transform);
    const clampedX = Math.max(28, Math.min(width - 28, projected.x));
    const clampedY = Math.max(28, Math.min(height - 28, projected.y));
    const color = this.statusTone === 'danger' ? 0xc94343 : this.statusTone === 'active' ? 0x2e8b57 : 0x386f8f;

    this.robotLayer.position.set(clampedX, clampedY);
    this.robotLayer.rotation = ((position.heading ?? 0) * Math.PI) / 180;
    this.robotBody.clear();
    this.robotBody.circle(0, 0, 24).fill({ color }).stroke({ color: 0xffffff, width: 4 });
    this.robotBody.circle(0, 0, 8).fill({ color: 0xffffff, alpha: 0.92 });
    this.robotHeading.clear();
    this.robotHeading.poly([0, -34, 10, -12, -10, -12]).fill({ color: 0x18352a });
  }
}
