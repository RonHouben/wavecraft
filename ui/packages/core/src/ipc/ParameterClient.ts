/**
 * ParameterClient - High-level typed API for parameter operations
 *
 * Provides typed methods for interacting with plugin parameters.
 */

import { IpcBridge } from './IpcBridge';
import type {
  ParameterInfo,
  GetParameterResult,
  SetParameterResult,
  GetAllParametersResult,
  ParameterChangedNotification,
} from '../types/parameters';
import {
  METHOD_GET_PARAMETER,
  METHOD_SET_PARAMETER,
  METHOD_GET_ALL_PARAMETERS,
  NOTIFICATION_PARAMETER_CHANGED,
} from '../types/parameters';

type ParameterChangeCallback = (id: string, value: number) => void;

export class ParameterClient {
  private static instance: ParameterClient | null = null;
  private readonly bridge: IpcBridge;

  private constructor() {
    this.bridge = IpcBridge.getInstance();
  }

  /**
   * Get singleton instance
   */
  public static getInstance(): ParameterClient {
    ParameterClient.instance ??= new ParameterClient();
    return ParameterClient.instance;
  }

  /**
   * Get a single parameter's current value and metadata
   */
  public async getParameter(id: string): Promise<GetParameterResult> {
    return this.bridge.invoke<GetParameterResult>(METHOD_GET_PARAMETER, { id });
  }

  /**
   * Set a parameter's value
   * @param id Parameter ID
   * @param value Normalized value [0.0, 1.0]
   */
  public async setParameter(id: string, value: number): Promise<void> {
    await this.bridge.invoke<SetParameterResult>(METHOD_SET_PARAMETER, {
      id,
      value,
    });
  }

  /**
   * Get all parameters with their current values and metadata
   */
  public async getAllParameters(): Promise<ParameterInfo[]> {
    const result = await this.bridge.invoke<GetAllParametersResult>(METHOD_GET_ALL_PARAMETERS);
    return result.parameters;
  }

  /**
   * Test connectivity with Rust backend
   * @returns Roundtrip time in milliseconds
   */
  public async ping(): Promise<number> {
    const start = performance.now();
    await this.bridge.invoke('ping');
    const end = performance.now();
    return end - start;
  }

  /**
   * Subscribe to parameter change notifications
   * @returns Unsubscribe function
   */
  public onParameterChanged(callback: ParameterChangeCallback): () => void {
    return this.bridge.on<ParameterChangedNotification>(NOTIFICATION_PARAMETER_CHANGED, (data) => {
      if (data && typeof data === 'object' && 'id' in data && 'value' in data) {
        callback(data.id, data.value);
      }
    });
  }
}
