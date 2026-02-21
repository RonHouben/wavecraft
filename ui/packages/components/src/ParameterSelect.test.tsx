/**
 * ParameterSelect component tests
 */

import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { ParameterSelect } from './ParameterSelect';
const mockSetValue = vi.hoisted(() => vi.fn());

describe('ParameterSelect', () => {
  beforeEach(() => {
    mockSetValue.mockReset();
    mockSetValue.mockResolvedValue(undefined);
  });

  it('renders dropdown with enum variant labels', () => {
    render(
      <ParameterSelect
        id="oscillator_waveform"
        name="Waveform"
        value={1}
        options={['Sine', 'Square', 'Saw', 'Triangle']}
        onChange={mockSetValue}
      />
    );

    expect(screen.getByText('Waveform')).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Sine' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Square' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Saw' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Triangle' })).toBeInTheDocument();
  });

  it('displays current value as selected option', () => {
    render(
      <ParameterSelect
        id="oscillator_waveform"
        name="Waveform"
        value={1}
        options={['Sine', 'Square', 'Saw', 'Triangle']}
        onChange={mockSetValue}
      />
    );

    const select = screen.getByRole('combobox') as HTMLSelectElement;
    expect(select.value).toBe('1');
  });

  it('calls setValue with numeric index on change', async () => {
    render(
      <ParameterSelect
        id="oscillator_waveform"
        name="Waveform"
        value={1}
        options={['Sine', 'Square', 'Saw', 'Triangle']}
        onChange={mockSetValue}
      />
    );

    const select = screen.getByRole('combobox');
    fireEvent.change(select, { target: { value: '3' } });

    expect(mockSetValue).toHaveBeenCalledWith(3);
  });

  it('renders a disabled select with helper text when enum variants are missing', () => {
    render(
      <ParameterSelect
        id="processor_mode"
        name="Mode"
        value={2}
        options={[]}
        onChange={mockSetValue}
      />
    );

    const select = screen.getByRole('combobox') as HTMLSelectElement;
    expect(select).toBeDisabled();
    expect(select.querySelectorAll('option')).toHaveLength(0);
    expect(screen.getByText('No variants available')).toBeInTheDocument();
  });
});
