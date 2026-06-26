import { RobotPosition, RobotStatus } from '../models/robot-status';

export interface ViewTransform {
  originX: number;
  originY: number;
  scale: number;
}

export function extractRobotPosition(status: RobotStatus | null): RobotPosition | null {
  if (!status) {
    return null;
  }

  const source = status.position ?? status.pose ?? status;
  const x = toFiniteNumber(source.x);
  const y = toFiniteNumber(source.y);

  if (x === null || y === null) {
    return null;
  }

  return {
    x,
    y,
    heading:
      toFiniteNumber(source.heading) ??
      toFiniteNumber(status.heading) ??
      toFiniteNumber(status.heading_degrees) ??
      toFiniteNumber(status.orientation) ??
      undefined
  };
}

export function projectPosition(position: RobotPosition, transform: ViewTransform): { x: number; y: number } {
  return {
    x: transform.originX + position.x * transform.scale,
    y: transform.originY - position.y * transform.scale
  };
}

function toFiniteNumber(value: unknown): number | null {
  return typeof value === 'number' && Number.isFinite(value) ? value : null;
}
