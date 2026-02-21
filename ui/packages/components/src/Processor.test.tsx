import { render, screen } from '@testing-library/react';
import type { ParameterInfo } from '@wavecraft/core';
import { describe, expect, it, vi } from 'vitest';

const mockUseAllParametersFor = vi.hoisted(() => vi.fn());

vi.mock('@wavecraft/core', async () => {
  const actual = await vi.importActual<typeof import('@wavecraft/core')>('@wavecraft/core');

  return {
    ...actual,
    useAllParametersFor: mockUseAllParametersFor,
  };
});

vi.mock('./ParameterToggle', () => ({
  ParameterToggle: ({ id }: { id: string }) => <div data-testid={`toggle-${id}`} />,
}));

vi.mock('./ParameterSlider', () => ({
  ParameterSlider: ({ id }: { id: string }) => <div data-testid={`slider-${id}`} />,
}));

vi.mock('./ParameterSelect', () => ({
  ParameterSelect: ({ id }: { id: string }) => <div data-testid={`select-${id}`} />,
}));

import { Processor } from './Processor';

type ParameterGroup = {
  name: string;
  parameters: ParameterInfo[];
};

describe('Processor', () => {
  it('renders bypass and processor parameters in a single processor section', () => {
    mockUseAllParametersFor.mockReturnValue({
      params: [
        {
          id: 'tone_filter_bypass',
          name: 'Bypass',
          type: 'bool',
          value: false,
          default: false,
          min: 0,
          max: 1,
        },
        {
          id: 'tone_filter_cutoff_hz',
          name: 'Cutoff',
          type: 'float',
          value: 0.5,
          default: 0.5,
          min: 0,
          max: 1,
        },
      ],
      processorId: 'tone_filter',
      isLoading: false,
      error: null,
      reload: vi.fn(),
    });

    const group: ParameterGroup = {
      name: 'Tone Filter',
      parameters: [
        {
          id: 'tone_filter_cutoff_hz',
          name: 'Cutoff',
          type: 'float',
          value: 0.5,
          default: 0.5,
          min: 0,
          max: 1,
        },
        {
          id: 'tone_filter_bypass',
          name: 'Bypass',
          type: 'bool',
          value: false,
          default: false,
          min: 0,
          max: 1,
        },
      ],
    };

    const { container } = render(<Processor id={'tone_filter'} />);

    expect(screen.getByRole('heading', { level: 3, name: 'tone_filter' })).toBeInTheDocument();
    expect(screen.getByTestId('toggle-tone_filter_bypass')).toBeInTheDocument();
    expect(screen.getByTestId('slider-tone_filter_cutoff_hz')).toBeInTheDocument();

    const renderedControls = container.querySelectorAll('[data-testid]');
    expect(renderedControls[0]).toHaveAttribute('data-testid', 'toggle-tone_filter_bypass');
    expect(renderedControls[1]).toHaveAttribute('data-testid', 'slider-tone_filter_cutoff_hz');

    void group;
  });
});
