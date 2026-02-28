/**
 * ProcessorOrderClient - High-level typed API for processor order operations
 *
 * Provides typed methods for getting, setting, and subscribing to
 * the runtime signal-chain processor order.
 */

import { IpcBridge } from './IpcBridge';
import { IpcEvents, IpcMethods } from './constants';

export interface GetProcessorOrderResult {
  order: string[];
}

export interface SetProcessorOrderParams {
  order: string[];
}

export interface ProcessorOrderChangedNotification {
  order: string[];
}

type ProcessorOrderChangedCallback = (order: string[]) => void;

export class ProcessorOrderClient {
  private static instance: ProcessorOrderClient | null = null;
  private readonly bridge: IpcBridge;

  private constructor() {
    this.bridge = IpcBridge.getInstance();
  }

  /**
   * Get singleton instance
   */
  public static getInstance(): ProcessorOrderClient {
    ProcessorOrderClient.instance ??= new ProcessorOrderClient();
    return ProcessorOrderClient.instance;
  }

  /**
   * Get the current processor order (slot indices as strings)
   */
  public async getProcessorOrder(): Promise<string[]> {
    const result = await this.bridge.invoke<GetProcessorOrderResult>(
      IpcMethods.GET_PROCESSOR_ORDER
    );
    return result.order;
  }

  /**
   * Set the processor order (slot indices as strings)
   */
  public async setProcessorOrder(order: string[]): Promise<void> {
    await this.bridge.invoke<Record<string, never>>(IpcMethods.SET_PROCESSOR_ORDER, {
      order,
    } satisfies SetProcessorOrderParams);
  }

  /**
   * Subscribe to processor order change notifications
   * @returns Unsubscribe function
   */
  public onProcessorOrderChanged(callback: ProcessorOrderChangedCallback): () => void {
    return this.bridge.on<ProcessorOrderChangedNotification>(
      IpcEvents.PROCESSOR_ORDER_CHANGED,
      (data) => {
        if (data && typeof data === 'object' && 'order' in data && Array.isArray(data.order)) {
          callback(data.order);
        }
      }
    );
  }
}
