/**
 * Audio Math Utility Tests
 */

import { describe, it, expect, vi } from 'vitest';
import { linearToDb, dbToLinear } from '@wavecraft/core/meters';

// Mock the IPC module to avoid WebView dependency
vi.mock('./wavecraft-ipc', () => import('../test/mocks/ipc'));

describe('linearToDb', () => {
  it('converts 1.0 to 0 dB', () => {
    expect(linearToDb(1)).toBeCloseTo(0, 2);
  });

  it('converts 0.5 to approximately -6 dB', () => {
    expect(linearToDb(0.5)).toBeCloseTo(-6.02, 1);
  });

  it('converts 0.1 to approximately -20 dB', () => {
    expect(linearToDb(0.1)).toBeCloseTo(-20, 1);
  });

  it('returns floor for 0 amplitude', () => {
    expect(linearToDb(0)).toBe(-60);
  });

  it('returns floor for negative values', () => {
    expect(linearToDb(-0.5)).toBe(-60);
  });

  it('respects custom floor value', () => {
    expect(linearToDb(0, -80)).toBe(-80);
    expect(linearToDb(0.001, -80)).toBeCloseTo(-60, 0);
  });

  it('handles values above 1.0 (above 0 dB)', () => {
    expect(linearToDb(2)).toBeCloseTo(6.02, 1);
    expect(linearToDb(10)).toBeCloseTo(20, 1);
  });
});

describe('dbToLinear', () => {
  it('converts 0 dB to 1.0', () => {
    expect(dbToLinear(0)).toBeCloseTo(1, 5);
  });

  it('converts -6 dB to approximately 0.5', () => {
    expect(dbToLinear(-6)).toBeCloseTo(0.501, 2);
  });

  it('converts -20 dB to approximately 0.1', () => {
    expect(dbToLinear(-20)).toBeCloseTo(0.1, 3);
  });

  it('converts -40 dB to approximately 0.01', () => {
    expect(dbToLinear(-40)).toBeCloseTo(0.01, 4);
  });

  it('converts +6 dB to approximately 2.0', () => {
    expect(dbToLinear(6)).toBeCloseTo(2, 2);
  });

  it('handles very negative values', () => {
    const result = dbToLinear(-80);
    expect(result).toBeGreaterThan(0);
    expect(result).toBeLessThan(0.001);
  });
});

describe('linearToDb and dbToLinear roundtrip', () => {
  it('roundtrips for common values', () => {
    const values = [0.1, 0.5, 1, 2];
    for (const value of values) {
      const db = linearToDb(value);
      const back = dbToLinear(db);
      expect(back).toBeCloseTo(value, 3);
    }
  });

  it('roundtrips for dB values', () => {
    const dbValues = [-40, -20, -6, 0, 6, 12];
    for (const db of dbValues) {
      const linear = dbToLinear(db);
      const back = linearToDb(linear);
      expect(back).toBeCloseTo(db, 2);
    }
  });
});
