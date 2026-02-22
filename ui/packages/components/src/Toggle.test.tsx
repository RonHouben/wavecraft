import { fireEvent, render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
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

  it('uses associated label text for non-string label nodes', () => {
    render(<Toggle checked={false} label={<span>Processor Enabled</span>} onChange={vi.fn()} />);

    expect(screen.getByRole('switch', { name: 'Processor Enabled' })).toBeInTheDocument();
  });

  it('disables interaction for loading and disabled visual states', () => {
    const { rerender } = render(
      <Toggle checked={false} label="Bypass" state="loading" onChange={vi.fn()} />
    );

    const loadingToggle = screen.getByRole('switch', { name: 'Bypass' });
    expect(loadingToggle).toBeDisabled();
    expect(loadingToggle).toHaveAttribute('aria-busy', 'true');

    rerender(<Toggle checked={false} label="Bypass" state="disabled" onChange={vi.fn()} />);

    expect(screen.getByRole('switch', { name: 'Bypass' })).toBeDisabled();
  });

  it('activates with Enter key and calls onChange with next checked state', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(<Toggle checked={false} label="Bypass" onChange={onChange} />);

    const toggle = screen.getByRole('switch', { name: 'Bypass' });
    toggle.focus();
    await user.keyboard('{Enter}');

    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange).toHaveBeenCalledWith(true);
  });

  it('activates with Space key and calls onChange with next checked state', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(<Toggle checked={false} label="Bypass" onChange={onChange} />);

    const toggle = screen.getByRole('switch', { name: 'Bypass' });
    toggle.focus();
    await user.keyboard(' ');

    expect(onChange).toHaveBeenCalledTimes(1);
    expect(onChange).toHaveBeenCalledWith(true);
  });
});
