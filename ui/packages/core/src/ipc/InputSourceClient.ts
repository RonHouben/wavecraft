import type {
  GetInputSourceResult,
  InputSourceChangedNotification,
  InputSourceKind,
  InputSourceOption,
  SetInputSourceParams,
} from '../types/input-source';
import { IpcBridge } from './IpcBridge';
import { IpcEvents, IpcMethods } from './constants';

type InputSourceChangedCallback = (selected: InputSourceKind) => void;

export class InputSourceClient {
  private static instance: InputSourceClient | null = null;
  private readonly bridge: IpcBridge;

  private constructor() {
    this.bridge = IpcBridge.getInstance();
  }

  public static getInstance(): InputSourceClient {
    InputSourceClient.instance ??= new InputSourceClient();
    return InputSourceClient.instance;
  }

  public async getInputSource(): Promise<GetInputSourceResult> {
    return this.bridge.invoke<GetInputSourceResult>(IpcMethods.GET_INPUT_SOURCE);
  }

  public async setInputSource(selected: InputSourceKind): Promise<void> {
    await this.bridge.invoke<Record<string, never>>(IpcMethods.SET_INPUT_SOURCE, {
      selected,
    } satisfies SetInputSourceParams);
  }

  public onInputSourceChanged(callback: InputSourceChangedCallback): () => void {
    return this.bridge.on<InputSourceChangedNotification>(
      IpcEvents.INPUT_SOURCE_CHANGED,
      (data) => {
        if (data && typeof data === 'object' && 'selected' in data) {
          callback(data.selected);
        }
      }
    );
  }

  public async getAvailableInputSources(): Promise<InputSourceOption[]> {
    const result = await this.getInputSource();
    return result.available;
  }
}
