/**
 * Logger - Structured logging abstraction for the UI.
 *
 * Wraps browser console API with severity levels and structured context.
 * In production builds, logs can be filtered by level at runtime.
 */

export enum LogLevel {
  DEBUG = 0,
  INFO = 1,
  WARN = 2,
  ERROR = 3,
}

export interface LogContext {
  [key: string]: unknown;
}

/**
 * Logger class providing structured logging with severity levels.
 *
 * Example usage:
 * ```typescript
 * const logger = new Logger({ minLevel: LogLevel.INFO });
 * logger.info('Parameter updated', { id: 'gain', value: 0.5 });
 * logger.error('IPC failed', { method: 'getParameter', error });
 * ```
 */
export class Logger {
  private minLevel: LogLevel;

  constructor(options: { minLevel?: LogLevel } = {}) {
    this.minLevel = options.minLevel ?? LogLevel.DEBUG;
  }

  /**
   * Set the minimum log level at runtime.
   */
  setMinLevel(level: LogLevel): void {
    this.minLevel = level;
  }

  /**
   * Log debug message (verbose tracing).
   */
  debug(message: string, context?: LogContext): void {
    if (this.minLevel <= LogLevel.DEBUG) {
      console.debug(`[DEBUG] ${message}`, context ?? {});
    }
  }

  /**
   * Log informational message.
   */
  info(message: string, context?: LogContext): void {
    if (this.minLevel <= LogLevel.INFO) {
      console.info(`[INFO] ${message}`, context ?? {});
    }
  }

  /**
   * Log warning message.
   */
  warn(message: string, context?: LogContext): void {
    if (this.minLevel <= LogLevel.WARN) {
      console.warn(`[WARN] ${message}`, context ?? {});
    }
  }

  /**
   * Log error message.
   */
  error(message: string, context?: LogContext): void {
    if (this.minLevel <= LogLevel.ERROR) {
      console.error(`[ERROR] ${message}`, context ?? {});
    }
  }
}

/**
 * Global logger instance for the UI.
 * Configure once at app startup, use throughout the codebase.
 */
export const logger = new Logger({
  minLevel: import.meta.env.DEV ? LogLevel.DEBUG : LogLevel.INFO,
});
