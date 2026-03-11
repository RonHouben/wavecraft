import type {
  GetHardwareInputSelectionResult,
  HardwareInputSelectionChangedNotification,
  SetHardwareInputSelectionParams,
} from '../types/hardware-input';
import { IpcBridge } from './IpcBridge';
import { IpcEvents, IpcMethods } from './constants';

type HardwareInputSelectionChangedCallback = (
  selection: HardwareInputSelectionChangedNotification
) => void;

export class HardwareInputClient {
  private static instance: HardwareInputClient | null = null;
  private readonly bridge: IpcBridge;

  private constructor() {
    this.bridge = IpcBridge.getInstance();
  }

  public static getInstance(): HardwareInputClient {
    HardwareInputClient.instance ??= new HardwareInputClient();
    return HardwareInputClient.instance;
  }

  public async getHardwareInputSelection(): Promise<GetHardwareInputSelectionResult> {
    return this.bridge.invoke<GetHardwareInputSelectionResult>(
      IpcMethods.GET_HARDWARE_INPUT_SELECTION
    );
  }

  public async setHardwareInputSelection(params: SetHardwareInputSelectionParams): Promise<void> {
    await this.bridge.invoke<Record<string, never>>(
      IpcMethods.SET_HARDWARE_INPUT_SELECTION,
      params
    );
  }

  public onHardwareInputSelectionChanged(
    callback: HardwareInputSelectionChangedCallback
  ): () => void {
    return this.bridge.on<HardwareInputSelectionChangedNotification>(
      IpcEvents.HARDWARE_INPUT_SELECTION_CHANGED,
      (data) => {
        if (data && typeof data === 'object') {
          callback(data);
        }
      }
    );
  }
}
