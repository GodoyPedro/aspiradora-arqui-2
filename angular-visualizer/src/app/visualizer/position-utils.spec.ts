import { RobotStatus } from '../models/robot-status';
import { extractRobotPosition, projectPosition } from './position-utils';

const baseStatus: RobotStatus = {
  state: 'STANDBY',
  cleaning_mode: 'AUTO',
  battery_percent: 80,
  is_charging: false,
  suction_enabled: false,
  brushes_enabled: false,
  left_wheel_speed: 0,
  right_wheel_speed: 0,
  current_error: null,
  sensors: {
    obstacle_detected: false,
    drop_off_detected: false,
    bumper_pressed: false,
    dust_container_full: false,
    wheel_stuck: false,
    brush_stuck: false,
    top_cover_open: false
  }
};

describe('position utils', () => {
  it('extracts nested finite coordinates', () => {
    expect(extractRobotPosition({ ...baseStatus, position: { x: 3, y: 4, heading: 90 } })).toEqual({
      x: 3,
      y: 4,
      heading: 90
    });
  });

  it('returns null for missing or malformed coordinates', () => {
    expect(extractRobotPosition(baseStatus)).toBeNull();
    expect(extractRobotPosition({ ...baseStatus, position: { x: Number.NaN, y: 1 } })).toBeNull();
  });

  it('projects robot coordinates with origin and scale', () => {
    expect(projectPosition({ x: 2, y: 3 }, { originX: 100, originY: 80, scale: 10 })).toEqual({ x: 120, y: 50 });
  });
});
