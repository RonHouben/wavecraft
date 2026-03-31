import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

const mockUseSettingsModal = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', () => ({
  useSettingsModal: mockUseSettingsModal,
}));

vi.mock('./Settings', () => ({
  Settings: () => <div>Settings content</div>,
}));

import { SettingsModal } from './SettingsModal';

describe('SettingsModal', () => {
  it('does not render when closed', () => {
    mockUseSettingsModal.mockReturnValue({
      isSettingsModalOpen: false,
      openSettingsModal: vi.fn(),
      closeSettingsModal: vi.fn(),
    });

    render(<SettingsModal />);

    expect(screen.queryByRole('dialog', { name: 'Plugin settings' })).not.toBeInTheDocument();
  });

  it('renders an accessible dialog when open', () => {
    mockUseSettingsModal.mockReturnValue({
      isSettingsModalOpen: true,
      openSettingsModal: vi.fn(),
      closeSettingsModal: vi.fn(),
    });

    render(<SettingsModal />);

    const dialog = screen.getByRole('dialog', { name: 'Plugin settings' });

    expect(dialog).toBeInTheDocument();
    expect(dialog).toHaveAccessibleDescription("Configure your plugin's audio input.");
    expect(screen.getByText('Settings content')).toBeInTheDocument();
  });

  it('routes close affordances and Escape to closeSettingsModal', () => {
    const closeSettingsModal = vi.fn();

    mockUseSettingsModal.mockReturnValue({
      isSettingsModalOpen: true,
      openSettingsModal: vi.fn(),
      closeSettingsModal,
    });

    render(<SettingsModal />);

    fireEvent.click(screen.getByRole('button', { name: 'Dismiss modal' }));
    fireEvent.click(screen.getByRole('button', { name: 'Close modal' }));
    fireEvent.keyDown(globalThis.window, { key: 'Escape' });

    expect(closeSettingsModal).toHaveBeenCalledTimes(3);
  });
});
