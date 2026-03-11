import { act, renderHook, waitFor } from '@testing-library/react';
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';

import { HardwareInputClient } from '../ipc/HardwareInputClient';
import { IpcBridge } from '../ipc/IpcBridge';
import * as transportsModule from '../transports';
import { MockTransport } from '../transports/MockTransport';
import * as environmentModule from '../utils/environment';
import { useHardwareInputSelection } from './useHardwareInputSelection';

describe('useHardwareInputSelection', () => {
  let mockTransport: MockTransport;

  beforeEach(() => {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (IpcBridge as any).instance = null;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (HardwareInputClient as any).instance = null;

    mockTransport = new MockTransport();
    vi.spyOn(transportsModule, 'getTransport').mockReturnValue(mockTransport);
    vi.spyOn(environmentModule, 'isWebViewEnvironment').mockReturnValue(false);
  });

  afterEach(() => {
    mockTransport?.dispose();
    vi.restoreAllMocks();
  });

  it('returns fetched hardware input selection when connected', async () => {
    mockTransport.setConnected(true);

    vi.spyOn(HardwareInputClient.prototype, 'getHardwareInputSelection').mockResolvedValue({
      selected_device_id: 'input-device:0',
      available_devices: [{ id: 'input-device:0', label: 'Built-in Input', channel_count: 2 }],
      selected_channel_id: 'stereo:0:1',
      available_channels: [{ id: 'stereo:0:1', label: 'Inputs 1 + 2 (stereo)' }],
    });

    const { result } = renderHook(() => useHardwareInputSelection());

    await waitFor(() => {
      expect(result.current.selectedDeviceId).toBe('input-device:0');
      expect(result.current.selectedChannelId).toBe('stereo:0:1');
      expect(result.current.selectedDevice?.label).toBe('Built-in Input');
      expect(result.current.availableChannels).toHaveLength(1);
      expect(result.current.isLoading).toBe(false);
    });
  });

  it('tracks hardwareInputSelectionChanged notifications', async () => {
    mockTransport.setConnected(true);

    vi.spyOn(HardwareInputClient.prototype, 'getHardwareInputSelection').mockResolvedValue({
      selected_device_id: 'input-device:0',
      available_devices: [{ id: 'input-device:0', label: 'Built-in Input', channel_count: 2 }],
      selected_channel_id: 'stereo:0:1',
      available_channels: [{ id: 'stereo:0:1', label: 'Inputs 1 + 2 (stereo)' }],
    });

    const { result } = renderHook(() => useHardwareInputSelection());

    await waitFor(() => {
      expect(result.current.selectedChannelId).toBe('stereo:0:1');
    });

    await act(async () => {
      mockTransport.simulateNotification({
        jsonrpc: '2.0',
        method: 'hardwareInputSelectionChanged',
        params: {
          selected_device_id: 'input-device:0',
          available_devices: [{ id: 'input-device:0', label: 'Built-in Input', channel_count: 2 }],
          selected_channel_id: 'mono:0',
          available_channels: [
            { id: 'stereo:0:1', label: 'Inputs 1 + 2 (stereo)' },
            { id: 'mono:0', label: 'Input 1 (mono → dual mono)' },
          ],
        },
      });
    });

    await waitFor(() => {
      expect(result.current.selectedChannelId).toBe('mono:0');
      expect(result.current.availableChannels).toHaveLength(2);
    });
  });

  it('rolls back optimistic channel selection when update fails', async () => {
    mockTransport.setConnected(true);

    vi.spyOn(HardwareInputClient.prototype, 'getHardwareInputSelection').mockResolvedValue({
      selected_device_id: 'input-device:0',
      available_devices: [{ id: 'input-device:0', label: 'Built-in Input', channel_count: 2 }],
      selected_channel_id: 'stereo:0:1',
      available_channels: [
        { id: 'stereo:0:1', label: 'Inputs 1 + 2 (stereo)' },
        { id: 'mono:0', label: 'Input 1 (mono → dual mono)' },
      ],
    });
    vi.spyOn(HardwareInputClient.prototype, 'setHardwareInputSelection').mockRejectedValue(
      new Error('restart required')
    );

    const { result } = renderHook(() => useHardwareInputSelection());

    await waitFor(() => {
      expect(result.current.selectedChannelId).toBe('stereo:0:1');
    });

    await expect(result.current.setSelectedChannel('mono:0')).rejects.toThrow('restart required');

    await waitFor(() => {
      expect(result.current.selectedChannelId).toBe('stereo:0:1');
      expect(result.current.error?.message).toContain('restart required');
    });
  });
});
