import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { OscillatorControl } from './OscillatorControl';

describe('OscillatorControl', () => {
  it('shows producing status and on state when active', () => {
    render(<OscillatorControl isProducing={true} isOn={true} onToggle={vi.fn()} />);

    expect(screen.getByText('Oscillator signal')).toBeInTheDocument();
    expect(screen.getByText('Producing')).toBeInTheDocument();
    expect(screen.getByText('Oscillator output: On')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Toggle oscillator output' })).toHaveAttribute(
      'aria-pressed',
      'true'
    );
  });

  it('shows no signal status and off state when inactive', () => {
    render(<OscillatorControl isProducing={false} isOn={false} onToggle={vi.fn()} />);

    expect(screen.getByText('No signal')).toBeInTheDocument();
    expect(screen.getByText('Oscillator output: Off')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Toggle oscillator output' })).toHaveAttribute(
      'aria-pressed',
      'false'
    );
  });

  it('invokes toggle handler when pressed', () => {
    const onToggle = vi.fn();
    render(<OscillatorControl isProducing={true} isOn={true} onToggle={onToggle} />);

    fireEvent.click(screen.getByRole('button', { name: 'Toggle oscillator output' }));

    expect(onToggle).toHaveBeenCalledTimes(1);
  });
});
