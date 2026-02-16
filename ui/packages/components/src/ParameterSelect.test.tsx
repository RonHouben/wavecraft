/**
 * ParameterSelect component tests
 */

import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { ParameterSelect } from './ParameterSelect';
import type { ParameterInfo } from '@wavecraft/core';

const mockUseParameter = vi.hoisted(() => vi.fn());
const mockLoggerError = vi.hoisted(() => vi.fn());
const mockLoggerWarn = vi.hoisted(() => vi.fn());
const mockSetValue = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', () => ({
  useParameter: mockUseParameter,
  logger: {
    error: mockLoggerError,
    warn: mockLoggerWarn,
  },
}));

function enumParameter(value: number): ParameterInfo {
  return {
    id: 'oscillator_waveform',
    name: 'Waveform',
    type: 'enum',
    value,
    default: 0,
    min: 0,
    max: 3,
    variants: ['Sine', 'Square', 'Saw', 'Triangle'],
  };
}

describe('ParameterSelect', () => {
  beforeEach(() => {
    mockSetValue.mockReset();
    mockSetValue.mockResolvedValue(undefined);
    mockLoggerWarn.mockReset();
    mockLoggerError.mockReset();

    mockUseParameter.mockReturnValue({
      param: enumParameter(1),
      setValue: mockSetValue,
      isLoading: false,
      error: null,
    });
  });

  it('renders dropdown with enum variant labels', () => {
    render(<ParameterSelect id="oscillator_waveform" />);

    expect(screen.getByText('Waveform')).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Sine' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Square' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Saw' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Triangle' })).toBeInTheDocument();
  });

  it('displays current value as selected option', () => {
    render(<ParameterSelect id="oscillator_waveform" />);

    const select = screen.getByRole('combobox') as HTMLSelectElement;
    expect(select.value).toBe('1');
  });

  it('calls setValue with numeric index on change', async () => {
    render(<ParameterSelect id="oscillator_waveform" />);

    const select = screen.getByRole('combobox');
    fireEvent.change(select, { target: { value: '3' } });

    expect(mockSetValue).toHaveBeenCalledWith(3);
  });

  it('shows loading state while parameter is loading', () => {
    mockUseParameter.mockReturnValue({
      param: null,
      setValue: mockSetValue,
      isLoading: true,
      error: null,
    });

    render(<ParameterSelect id="oscillator_waveform" />);

    expect(screen.getByText('Loading oscillator_waveform...')).toBeInTheDocument();
  });

  it('shows error when parameter does not exist', () => {
    mockUseParameter.mockReturnValue({
      param: null,
      setValue: mockSetValue,
      isLoading: false,
      error: new Error('Parameter not found: nonexistent'),
    });

    render(<ParameterSelect id="nonexistent" />);

    expect(screen.getByText(/Error:/)).toBeInTheDocument();
    expect(screen.getByText(/Parameter not found/)).toBeInTheDocument();
  });

  it('renders a disabled select with helper text when enum variants are missing', () => {
    mockUseParameter.mockReturnValue({
      param: {
        id: 'processor_mode',
        name: 'Mode',
        type: 'enum',
        value: 2,
        default: 1,
        min: 1,
        max: 3,
      } satisfies ParameterInfo,
      setValue: mockSetValue,
      isLoading: false,
      error: null,
    });

    render(<ParameterSelect id="processor_mode" />);

    const select = screen.getByRole('combobox') as HTMLSelectElement;
    expect(select).toBeDisabled();
    expect(select.querySelectorAll('option')).toHaveLength(0);
    expect(screen.getByText('No variants available')).toBeInTheDocument();
    expect(mockLoggerWarn).toHaveBeenCalledWith('Enum parameter has no variants', {
      parameterId: 'processor_mode',
    });
    expect(mockLoggerWarn).toHaveBeenCalledTimes(1);
  });

  it('does not spam missing-variants warning on rerender', () => {
    mockUseParameter.mockReturnValue({
      param: {
        id: 'processor_mode',
        name: 'Mode',
        type: 'enum',
        value: 2,
        default: 1,
        min: 1,
        max: 3,
      } satisfies ParameterInfo,
      setValue: mockSetValue,
      isLoading: false,
      error: null,
    });

    const { rerender } = render(<ParameterSelect id="processor_mode" />);
    rerender(<ParameterSelect id="processor_mode" />);
    rerender(<ParameterSelect id="processor_mode" />);

    expect(mockLoggerWarn).toHaveBeenCalledTimes(1);
  });
});
