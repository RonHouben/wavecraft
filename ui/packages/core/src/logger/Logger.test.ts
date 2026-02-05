import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { Logger, LogLevel } from './Logger';

describe('Logger', () => {
  let consoleDebugSpy: ReturnType<typeof vi.spyOn>;
  let consoleInfoSpy: ReturnType<typeof vi.spyOn>;
  let consoleWarnSpy: ReturnType<typeof vi.spyOn>;
  let consoleErrorSpy: ReturnType<typeof vi.spyOn>;

  beforeEach(() => {
    consoleDebugSpy = vi.spyOn(console, 'debug').mockImplementation(() => {});
    consoleInfoSpy = vi.spyOn(console, 'info').mockImplementation(() => {});
    consoleWarnSpy = vi.spyOn(console, 'warn').mockImplementation(() => {});
    consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it('logs debug messages when minLevel is DEBUG', () => {
    const logger = new Logger({ minLevel: LogLevel.DEBUG });
    logger.debug('Test debug', { key: 'value' });

    expect(consoleDebugSpy).toHaveBeenCalledWith('[DEBUG] Test debug', { key: 'value' });
  });

  it('does not log debug messages when minLevel is INFO', () => {
    const logger = new Logger({ minLevel: LogLevel.INFO });
    logger.debug('Test debug');

    expect(consoleDebugSpy).not.toHaveBeenCalled();
  });

  it('logs info messages when minLevel is INFO', () => {
    const logger = new Logger({ minLevel: LogLevel.INFO });
    logger.info('Test info', { key: 'value' });

    expect(consoleInfoSpy).toHaveBeenCalledWith('[INFO] Test info', { key: 'value' });
  });

  it('logs warn messages when minLevel is INFO', () => {
    const logger = new Logger({ minLevel: LogLevel.INFO });
    logger.warn('Test warning', { error: 'something' });

    expect(consoleWarnSpy).toHaveBeenCalledWith('[WARN] Test warning', { error: 'something' });
  });

  it('logs error messages at all levels', () => {
    const logger = new Logger({ minLevel: LogLevel.ERROR });
    logger.error('Test error', { code: 500 });

    expect(consoleErrorSpy).toHaveBeenCalledWith('[ERROR] Test error', { code: 500 });
  });

  it('allows changing minLevel at runtime', () => {
    const logger = new Logger({ minLevel: LogLevel.DEBUG });

    logger.debug('Should log');
    expect(consoleDebugSpy).toHaveBeenCalledTimes(1);

    logger.setMinLevel(LogLevel.INFO);
    logger.debug('Should not log');
    expect(consoleDebugSpy).toHaveBeenCalledTimes(1); // Still 1, not 2
  });

  it('defaults to DEBUG level when no minLevel is specified', () => {
    const logger = new Logger();
    logger.debug('Test debug');

    expect(consoleDebugSpy).toHaveBeenCalled();
  });

  it('handles missing context parameter', () => {
    const logger = new Logger({ minLevel: LogLevel.INFO });
    logger.info('No context');

    expect(consoleInfoSpy).toHaveBeenCalledWith('[INFO] No context', {});
  });
});
