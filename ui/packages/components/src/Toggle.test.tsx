import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Toggle } from './Toggle';

describe('Toggle', () => {
  it('uses switch semantics and toggles value on click', () => {
    const onChange = vi.fn();

    render(<Toggle checked={false} label="Bypass" onChange={onChange} />);

    const toggle = screen.getByRole('switch', { name: 'Bypass' });
    fireEvent.click(toggle);

    expect(onChange).toHaveBeenCalledWith(true);
  });

  it('forces off state when bypassed', () => {
    render(<Toggle checked label="Bypass" pluginState="bypassed" onChange={vi.fn()} />);

    const toggle = screen.getByRole('switch', { name: 'Bypass' });
    expect(toggle).toHaveAttribute('aria-checked', 'false');
    expect(screen.getByText('BYP')).toBeInTheDocument();
  });
});
