import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import {
  SawWaveIcon,
  SineWaveIcon,
  SquareWaveIcon,
  TriangleWaveIcon,
  getWaveformIcon,
} from './icons';

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

  it('maps known waveform labels to matching icon components', () => {
    render(
      <div>
        {getWaveformIcon('Sine')}
        {getWaveformIcon('Square')}
        {getWaveformIcon('Saw')}
        {getWaveformIcon('Triangle')}
      </div>
    );

    expect(document.querySelector('[data-waveform-icon="sine"]')).toBeInTheDocument();
    expect(document.querySelector('[data-waveform-icon="square"]')).toBeInTheDocument();
    expect(document.querySelector('[data-waveform-icon="saw"]')).toBeInTheDocument();
    expect(document.querySelector('[data-waveform-icon="triangle"]')).toBeInTheDocument();
  });

  it('returns null for unsupported waveform labels', () => {
    render(<div>{getWaveformIcon('Pulse')}</div>);

    expect(screen.queryByRole('img')).not.toBeInTheDocument();
    expect(document.querySelector('[data-waveform-icon]')).not.toBeInTheDocument();
  });
});
