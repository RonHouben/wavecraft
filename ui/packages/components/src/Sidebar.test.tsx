import { act, fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Button } from './Button';
import { Sidebar } from './Sidebar';

describe('Sidebar', () => {
  it('does not render when closed', () => {
    render(
      <Sidebar open={false} onClose={vi.fn()} defaultActions={[]}>
        <Button size="sm">Settings</Button>
      </Sidebar>
    );

    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('renders an accessible dialog with its content when open', () => {
    render(
      <Sidebar
        open
        onClose={vi.fn()}
        title="Main menu"
        description="Quick actions live here."
        defaultActions={[]}
      >
        <Button size="sm">Settings</Button>
      </Sidebar>
    );

    const dialog = screen.getByRole('dialog', { name: 'Main menu' });

    expect(dialog).toBeInTheDocument();
    expect(dialog).toHaveFocus();
    expect(screen.getByText('Quick actions live here.')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Close sidebar' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Settings' })).toBeInTheDocument();
    expect(screen.getByText('Quick actions')).toBeInTheDocument();
    expect(screen.getByRole('navigation', { name: 'Sidebar quick actions' })).toBeInTheDocument();
    expect(screen.getByText('Esc')).toBeInTheDocument();
  });

  it('calls onClose when the backdrop is clicked', () => {
    const onClose = vi.fn();

    render(
      <Sidebar open onClose={onClose} defaultActions={[]}>
        <Button size="sm">Settings</Button>
      </Sidebar>
    );

    fireEvent.click(screen.getByRole('button', { name: 'Dismiss sidebar' }));

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('calls onClose when Escape is pressed', () => {
    const onClose = vi.fn();

    render(
      <Sidebar open onClose={onClose} defaultActions={[]}>
        <Button size="sm">Settings</Button>
      </Sidebar>
    );

    fireEvent.keyDown(globalThis.window, { key: 'Escape' });

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('supports className overrides on the panel', () => {
    render(
      <Sidebar
        open
        onClose={vi.fn()}
        className="custom-sidebar"
        data-testid="sidebar-panel"
        defaultActions={[]}
      >
        <Button size="sm">Settings</Button>
      </Sidebar>
    );

    expect(screen.getByTestId('sidebar-panel')).toHaveClass('custom-sidebar');
  });

  it('keeps the drawer mounted briefly for exit animation before unmounting', () => {
    vi.useFakeTimers();

    const { rerender } = render(
      <Sidebar open onClose={vi.fn()} title="Main menu" defaultActions={[]}>
        <Button size="sm">Settings</Button>
      </Sidebar>
    );

    rerender(
      <Sidebar open={false} onClose={vi.fn()} title="Main menu" defaultActions={[]}>
        <Button size="sm">Settings</Button>
      </Sidebar>
    );

    expect(screen.getByRole('dialog', { name: 'Main menu' })).toHaveClass('translate-x-full');

    act(() => {
      vi.advanceTimersByTime(151);
    });

    expect(screen.queryByRole('dialog', { name: 'Main menu' })).not.toBeInTheDocument();
    vi.useRealTimers();
  });
});
