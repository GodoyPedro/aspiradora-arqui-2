import { HttpClient, HttpErrorResponse } from '@angular/common/http';
import { Injectable } from '@angular/core';
import { Observable, throwError } from 'rxjs';
import { catchError } from 'rxjs/operators';
import { ApiErrorBody, ManualMoveRequest, RobotStatus } from '../models/robot-status';

@Injectable({ providedIn: 'root' })
export class RobotApiService {
  constructor(private readonly http: HttpClient) {}

  getStatus(): Observable<RobotStatus> {
    return this.http.get<RobotStatus>('/status').pipe(catchError((error) => throwError(() => this.normalizeError(error))));
  }

  start(): Observable<RobotStatus> {
    return this.postEmpty('/commands/start');
  }

  stop(): Observable<RobotStatus> {
    return this.postEmpty('/commands/stop');
  }

  pause(): Observable<RobotStatus> {
    return this.postEmpty('/commands/pause');
  }

  returnToDock(): Observable<RobotStatus> {
    return this.postEmpty('/commands/return-to-dock');
  }

  clearError(): Observable<RobotStatus> {
    return this.postEmpty('/commands/clear-error');
  }

  manualMove(payload: ManualMoveRequest): Observable<RobotStatus> {
    return this.http
      .post<RobotStatus>('/commands/manual-move', payload)
      .pipe(catchError((error) => throwError(() => this.normalizeError(error))));
  }

  setAutoMode(): Observable<RobotStatus> {
    return this.http
      .post<RobotStatus>('/commands/mode', { mode: 'AUTO' })
      .pipe(catchError((error) => throwError(() => this.normalizeError(error))));
  }

  private postEmpty(url: string): Observable<RobotStatus> {
    return this.http.post<RobotStatus>(url, undefined).pipe(catchError((error) => throwError(() => this.normalizeError(error))));
  }

  private normalizeError(error: unknown): Error {
    if (error instanceof HttpErrorResponse) {
      const body = error.error as Partial<ApiErrorBody> | string | null;
      if (body && typeof body === 'object' && 'message' in body) {
        const code = body.code ? `${body.code}: ` : '';
        return new Error(`${code}${body.message ?? 'Command failed'}`);
      }

      return new Error(error.status ? `HTTP ${error.status}: ${error.statusText}` : 'Firmware is unreachable');
    }

    return error instanceof Error ? error : new Error('Unexpected API error');
  }
}
