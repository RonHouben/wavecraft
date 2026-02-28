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
        id="tone_filter_mode"
        name="Mode"
        value={1}
        options={['Lowpass', 'Bandpass', 'Highpass']}
        onChange={mockSetValue}
      />
    );

    expect(screen.getByText('Mode')).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Lowpass' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Bandpass' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Highpass' })).toBeInTheDocument();
  });

  it('displays current value as selected option', () => {
    render(
      <ParameterSelect
        id="tone_filter_mode"
        name="Mode"
        value={1}
        options={['Lowpass', 'Bandpass', 'Highpass']}
        onChange={mockSetValue}
      />
    );

    const select = screen.getByRole('combobox') as HTMLSelectElement;
    expect(select.value).toBe('1');
  });

  it('calls setValue with numeric index on change', async () => {
    render(
      <ParameterSelect
        id="tone_filter_mode"
        name="Mode"
        value={1}
        options={['Lowpass', 'Bandpass', 'Highpass']}
        onChange={mockSetValue}
      />
    );

    const select = screen.getByRole('combobox');
    fireEvent.change(select, { target: { value: '2' } });

    expect(mockSetValue).toHaveBeenCalledWith(2);
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
