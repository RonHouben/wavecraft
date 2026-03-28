import {
  useAudioStatus,
  useConnectionStatus,
  useHardwareInputSelection,
  useInputSource,
} from '@wavecraft/core';
import { ConnectionStatus } from '../ConnectionStatus';
import { Select } from '../Select';

export interface SettingsProps {}

export function Settings(_props: Readonly<SettingsProps>) {
  const { connected, transport } = useConnectionStatus();
  const { phase, isReady, isDegraded, diagnostic } = useAudioStatus();
  const {
    selected: selectedInputSource,
    available: availableInputSources,
    setSelected: setSelectedInputSource,
    isLoading: isInputSourceLoading,
  } = useInputSource();

  const {
    selectedChannelId,
    availableChannels,
    setSelectedChannel,
    isLoading: isHardwareInputSelectionLoading,
  } = useHardwareInputSelection();

  return (
    <>
      <Select
        label="Input Source"
        size="sm"
        value={selectedInputSource}
        options={availableInputSources.map((source) => ({
          label: source.label,
          value: source.id,
        }))}
        disabled={!connected || availableInputSources.length === 0}
        state={isInputSourceLoading ? 'loading' : 'default'}
        onChange={setSelectedInputSource}
      />

      <Select
        label="Input Channels"
        size="sm"
        value={selectedChannelId}
        options={availableChannels.map((channel) => ({
          label: channel.label,
          value: channel.id,
        }))}
        disabled={
          !connected || selectedInputSource !== 'hardwareInput' || availableChannels.length === 0
        }
        state={isHardwareInputSelectionLoading ? 'loading' : 'default'}
        onChange={setSelectedChannel}
      />
      <ConnectionStatus
        connected={connected}
        transport={transport}
        phase={phase}
        isReady={isReady}
        isDegraded={isDegraded}
        diagnostic={diagnostic}
      />
    </>
  );
}
