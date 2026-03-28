import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Button } from './Button';
import { Sidebar } from './Sidebar';

describe('Sidebar', () => {
  it('does not render when closed', () => {
    render(
      <Sidebar open={false} onClose={vi.fn()}>
        <Button size="sm">Settings</Button>
      </Sidebar>
    );

    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('renders an accessible dialog with its content when open', () => {
    render(
      <Sidebar open onClose={vi.fn()} title="Main menu">
        <Button size="sm">Settings</Button>
      </Sidebar>
    );

    expect(screen.getByRole('dialog', { name: 'Main menu' })).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Settings' })).toBeInTheDocument();
  });

  it('calls onClose when the backdrop is clicked', () => {
    const onClose = vi.fn();

    render(
      <Sidebar open onClose={onClose}>
        <Button size="sm">Settings</Button>
      </Sidebar>
    );

    fireEvent.click(screen.getByRole('button', { name: 'Dismiss sidebar' }));

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('calls onClose when Escape is pressed', () => {
    const onClose = vi.fn();

    render(
      <Sidebar open onClose={onClose}>
        <Button size="sm">Settings</Button>
      </Sidebar>
    );

    fireEvent.keyDown(globalThis, { key: 'Escape' });

    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('supports className overrides on the panel', () => {
    render(
      <Sidebar open onClose={vi.fn()} className="custom-sidebar" data-testid="sidebar-panel">
        <Button size="sm">Settings</Button>
      </Sidebar>
    );

    expect(screen.getByTestId('sidebar-panel')).toHaveClass('custom-sidebar');
  });
});
