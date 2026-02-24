import { render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { ParameterGroup } from './ParameterGroup';
import type { ProcessorParameter } from './types';

describe('ParameterGroup rendering', () => {
  it('renders mixed parameter controls via presentational props', () => {
    const params: ProcessorParameter[] = [
      {
        id: 'test_tone_enabled',
        name: 'Enabled',
        type: 'bool',
        value: true,
        default: true,
        min: 0,
        max: 1,
        onChange: vi.fn(),
      },
      {
        id: 'test_tone_frequency',
        name: 'Frequency',
        type: 'float',
        value: 440,
        default: 440,
        min: 20,
        max: 20000,
        unit: 'Hz',
        onChange: vi.fn(),
      },
      {
        id: 'test_tone_level',
        name: 'Level',
        type: 'float',
        value: 0.5,
        default: 0.5,
        min: 0,
        max: 1,
        unit: '%',
        onChange: vi.fn(),
      },
    ];

    render(<ParameterGroup group={{ name: 'Test Tone', parameters: params }} />);

    expect(screen.getByRole('heading', { level: 3, name: 'Test Tone' })).toBeInTheDocument();
    expect(screen.getByLabelText('Enabled')).toBeInTheDocument();
    expect(screen.getByLabelText('Frequency')).toBeInTheDocument();
    expect(screen.getByLabelText('Level')).toBeInTheDocument();
  });
});
