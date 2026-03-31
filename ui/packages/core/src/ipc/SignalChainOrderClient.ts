/**
 * SignalChainOrderClient - High-level typed API for unified signal chain order
 *
 * Provides typed methods for getting, setting, and subscribing to
 * the runtime signal-chain slot order (processors + taps).
 */

import type { SignalChainOrder } from '../types/signal-chain';
import { IpcBridge } from './IpcBridge';
import { IpcEvents, IpcMethods } from './constants';

export interface GetSignalChainOrderResult {
  slots: SignalChainOrder[];
}

export interface SetSignalChainOrderParams {
  slots: SignalChainOrder[];
}

export interface SignalChainOrderChangedNotification {
  slots: SignalChainOrder[];
}

type SignalChainOrderChangedCallback = (slots: SignalChainOrder[]) => void;

export class SignalChainOrderClient {
  private static instance: SignalChainOrderClient | null = null;
  private readonly bridge: IpcBridge;

  private constructor() {
    this.bridge = IpcBridge.getInstance();
  }

  /**
   * Get singleton instance
   */
  public static getInstance(): SignalChainOrderClient {
    SignalChainOrderClient.instance ??= new SignalChainOrderClient();
    return SignalChainOrderClient.instance;
  }

  /**
   * Get the current signal chain slot order (processors + taps)
   */
  public async getSignalChainOrder(): Promise<SignalChainOrder[]> {
    const result = await this.bridge.invoke<GetSignalChainOrderResult>(
      IpcMethods.GET_SIGNAL_CHAIN_ORDER
    );
    return result.slots;
  }

  /**
   * Set the signal chain slot order (processors + taps)
   */
  public async setSignalChainOrder(slots: SignalChainOrder[]): Promise<void> {
    await this.bridge.invoke<Record<string, never>>(IpcMethods.SET_SIGNAL_CHAIN_ORDER, {
      slots,
    } satisfies SetSignalChainOrderParams);
  }

  /**
   * Subscribe to signal chain order change notifications
   * @returns Unsubscribe function
   */
  public onSignalChainOrderChanged(callback: SignalChainOrderChangedCallback): () => void {
    return this.bridge.on<SignalChainOrderChangedNotification>(
      IpcEvents.SIGNAL_CHAIN_ORDER_CHANGED,
      (data) => {
        if (data && typeof data === 'object' && 'slots' in data && Array.isArray(data.slots)) {
          callback(data.slots as SignalChainOrder[]);
        }
      }
    );
  }
}
