import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import {
  IconComponentMap,
  SawWaveIcon,
  SineWaveIcon,
  SquareWaveIcon,
  TriangleWaveIcon,
} from './icons/WaveformIcons';

describe('Waveform icons', () => {
  it('renders dedicated icons with waveform markers', () => {
    render(
      <div>
        <SineWaveIcon />
        <SquareWaveIcon />
        <SawWaveIcon />
        <TriangleWaveIcon />
      </div>
    );

    expect(document.querySelector('[data-waveform-icon="sine"]')).toBeInTheDocument();
    expect(document.querySelector('[data-waveform-icon="square"]')).toBeInTheDocument();
    expect(document.querySelector('[data-waveform-icon="saw"]')).toBeInTheDocument();
    expect(document.querySelector('[data-waveform-icon="triangle"]')).toBeInTheDocument();
  });

  it('maps waveform icon keys to matching icon components', () => {
    const SineIcon = IconComponentMap['waveform-sine'];
    const SquareIcon = IconComponentMap['waveform-square'];
    const SawIcon = IconComponentMap['waveform-saw'];
    const TriangleIcon = IconComponentMap['waveform-triangle'];

    render(
      <div>
        <SineIcon />
        <SquareIcon />
        <SawIcon />
        <TriangleIcon />
      </div>
    );

    expect(document.querySelector('[data-waveform-icon="sine"]')).toBeInTheDocument();
    expect(document.querySelector('[data-waveform-icon="square"]')).toBeInTheDocument();
    expect(document.querySelector('[data-waveform-icon="saw"]')).toBeInTheDocument();
    expect(document.querySelector('[data-waveform-icon="triangle"]')).toBeInTheDocument();
  });

  it('keeps saw and sawtooth aliases mapped to the same icon component', () => {
    expect(IconComponentMap['waveform-sawtooth']).toBe(IconComponentMap['waveform-saw']);

    const SawtoothAliasIcon = IconComponentMap['waveform-sawtooth'];

    render(
      <div>
        <SawtoothAliasIcon data-waveform-icon="sawtooth-alias" />
      </div>
    );

    expect(screen.queryByRole('img')).not.toBeInTheDocument();
    expect(document.querySelector('[data-waveform-icon="sawtooth-alias"]')).toBeInTheDocument();
  });
});
