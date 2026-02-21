import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ParameterGroup } from './ParameterGroup';
import type { ProcessorParameter } from './Processor';

describe('ParameterGroup rendering', () => {
  it('renders mixed parameter controls via presentational props', () => {
    const params: ProcessorParameter[] = [
      {
        id: 'oscillator_enabled',
        name: 'Enabled',
        type: 'bool',
        value: true,
        default: true,
        min: 0,
        max: 1,
        onChange: vi.fn(),
      },
      {
        id: 'oscillator_waveform',
        name: 'Waveform',
        type: 'enum',
        value: 0,
        default: 0,
        min: 0,
        max: 3,
        variants: ['Sine', 'Square', 'Saw', 'Triangle'],
        onChange: vi.fn(),
      },
      {
        id: 'oscillator_frequency',
        name: 'Frequency',
        type: 'float',
        value: 440,
        default: 440,
        min: 20,
        max: 5000,
        unit: 'Hz',
        onChange: vi.fn(),
      },
    ];

    render(<ParameterGroup group={{ name: 'Oscillator', parameters: params }} />);

    expect(screen.getByRole('heading', { level: 3, name: 'Oscillator' })).toBeInTheDocument();
    expect(screen.getByLabelText('Enabled')).toBeInTheDocument();
    expect(screen.getByLabelText('Waveform')).toBeInTheDocument();
    expect(screen.getByLabelText('Frequency')).toBeInTheDocument();
  });
});
