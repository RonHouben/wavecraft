/**
 * IPC Mock Module for Testing
 *
 * Provides mock implementations of IPC hooks and utilities
 * that allow testing components without the Rust engine.
 */

import { useState, useCallback } from 'react';
import type {
  ParameterInfo,
  MeterFrame,
  UseParameterResult,
  UseAllParametersResult,
  ConnectionStatus,
} from '@wavecraft/core';

// Re-export types
export type {
  ParameterInfo,
  ParameterType,
  MeterFrame,
  UseParameterResult,
  UseAllParametersResult,
  ConnectionStatus,
} from '@wavecraft/core';

// ============================================================================
// Mock State
// ============================================================================

const mockParameters = new Map<string, ParameterInfo>();
let mockMeterFrame: MeterFrame | null = null;

// ============================================================================
// Test Control API
// ============================================================================

/**
 * Set a mock parameter value for testing
 */
export function setMockParameter(id: string, info: Partial<ParameterInfo>): void {
  const existing = mockParameters.get(id);
  const fullInfo: ParameterInfo = {
    id,
    name: info.name ?? existing?.name ?? id,
    value: info.value ?? existing?.value ?? 0,
    default: info.default ?? existing?.default ?? 0,
    unit: info.unit ?? existing?.unit,
    type: info.type ?? existing?.type ?? 'float',
    ...info,
  };
  mockParameters.set(id, fullInfo);
}

/**
 * Set mock meter frame data for testing
 */
export function setMockMeterFrame(frame: MeterFrame): void {
  mockMeterFrame = frame;
}

/**
 * Get current mock parameter value
 */
export function getMockParameter(id: string): ParameterInfo | undefined {
  return mockParameters.get(id);
}

/**
 * Clear all mock state
 */
export function resetMocks(): void {
  mockParameters.clear();
  mockMeterFrame = null;
}

// ============================================================================
// Mock Hook Implementations
// ============================================================================

/**
 * Mock implementation of useParameter hook
 */
export function useParameter(id: string): UseParameterResult {
  // Initialize state directly from mockParameters without useEffect
  const mockParam = mockParameters.get(id);
  const [param, setParam] = useState<ParameterInfo | null>(mockParam ?? null);
  const [isLoading] = useState(false); // Mock is never loading
  const [error] = useState<Error | null>(
    mockParam ? null : new Error(`Parameter not found: ${id}`)
  );

  const setValue = useCallback(
    async (value: number): Promise<void> => {
      const existing = mockParameters.get(id);
      if (existing) {
        const updated = { ...existing, value };
        mockParameters.set(id, updated);
        setParam(updated);
      } else {
        throw new Error(`Parameter not found: ${id}`);
      }
    },
    [id]
  );

  return { param, setValue, isLoading, error };
}

/**
 * Mock implementation of useAllParameters hook
 */
export function useAllParameters(): UseAllParametersResult {
  const [params] = useState<ParameterInfo[]>(Array.from(mockParameters.values()));
  const [isLoading] = useState(false);
  const [error] = useState<Error | null>(null);

  const reload = useCallback(async (): Promise<void> => {
    // In tests, params are set via setMockParameter
    // No need to actually reload
  }, []);

  return { params, isLoading, error, reload };
}

// ============================================================================
// Mock Utility Functions
// ============================================================================

/**
 * Mock implementation of getMeterFrame
 */
export async function getMeterFrame(): Promise<MeterFrame | null> {
  return mockMeterFrame;
}

/**
 * Convert linear amplitude to decibels (not mocked - pure function)
 */
export function linearToDb(linear: number, floor: number = -60): number {
  if (linear <= 0) {
    return floor;
  }
  const db = 20 * Math.log10(linear);
  return Math.max(db, floor);
}

/**
 * Convert decibels to linear amplitude (not mocked - pure function)
 */
export function dbToLinear(db: number): number {
  return Math.pow(10, db / 20);
}

/**
 * Mock implementation of useConnectionStatus
 * Always returns connected in test environment
 */
export function useConnectionStatus(): ConnectionStatus {
  return {
    connected: true,
    transport: 'native',
  };
}
