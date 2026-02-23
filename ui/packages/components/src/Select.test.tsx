import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Select } from './Select';

describe('Select', () => {
  it('renders provided options', () => {
    render(
      <Select
        label="Processor"
        value="osc"
        options={[
          { label: 'Oscillator', value: 'osc' },
          { label: 'Filter', value: 'filter' },
          { label: 'Limiter', value: 'limiter' },
        ]}
        onChange={vi.fn()}
      />
    );

    expect(screen.getByRole('combobox')).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Oscillator' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Filter' })).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Limiter' })).toBeInTheDocument();
  });

  it('reflects the current value as selected option', () => {
    render(
      <Select
        value="filter"
        options={[
          { label: 'Oscillator', value: 'osc' },
          { label: 'Filter', value: 'filter' },
          { label: 'Limiter', value: 'limiter' },
        ]}
        onChange={vi.fn()}
      />
    );

    const select = screen.getByRole('combobox') as HTMLSelectElement;
    expect(select.value).toBe('1');
  });

  it('emits the selected numeric value with correct type', () => {
    const onChange = vi.fn<(value: number) => void>();

    render(
      <Select<number>
        value={100}
        options={[
          { label: 'Dry', value: 0 },
          { label: 'Mix 50%', value: 50 },
          { label: 'Wet', value: 100 },
        ]}
        onChange={onChange}
      />
    );

    fireEvent.change(screen.getByRole('combobox'), { target: { value: '1' } });

    expect(onChange).toHaveBeenCalledWith(50);
  });

  it('emits the selected string value with correct type', () => {
    const onChange = vi.fn<(value: string) => void>();

    render(
      <Select
        value="osc"
        options={[
          { label: 'Oscillator', value: 'osc' },
          { label: 'Filter', value: 'filter' },
          { label: 'Limiter', value: 'limiter' },
        ]}
        onChange={onChange}
      />
    );

    fireEvent.change(screen.getByRole('combobox'), { target: { value: '2' } });

    expect(onChange).toHaveBeenCalledWith('limiter');
  });

  it('supports disabled behavior', () => {
    const onChange = vi.fn();

    render(
      <Select
        disabled
        value="osc"
        options={[
          { label: 'Oscillator', value: 'osc' },
          { label: 'Filter', value: 'filter' },
        ]}
        onChange={onChange}
      />
    );

    const select = screen.getByRole('combobox');
    expect(select).toBeDisabled();
  });

  it('connects label and id accessibly', () => {
    render(
      <Select
        id="processor-select"
        label="Processor Type"
        value="osc"
        options={[
          { label: 'Oscillator', value: 'osc' },
          { label: 'Filter', value: 'filter' },
        ]}
        onChange={vi.fn()}
      />
    );

    const select = screen.getByLabelText('Processor Type');
    expect(select).toHaveAttribute('id', 'processor-select');
  });
});
