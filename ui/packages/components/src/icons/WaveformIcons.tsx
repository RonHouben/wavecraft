import type { SVGProps } from 'react';
import { mergeClassNames } from '../utils/classNames';

export const IconComponentMap = {
  'waveform-sine': SineWaveIcon,
  'waveform-square': SquareWaveIcon,
  'waveform-saw': SawWaveIcon,
  'waveform-sawtooth': SawWaveIcon,
  'waveform-triangle': TriangleWaveIcon,
};

export interface WaveformIconProps extends SVGProps<SVGSVGElement> {
  readonly className?: string;
}

const baseWaveformIconClassName = 'h-3.5 w-3.5 shrink-0';

export function SineWaveIcon({
  className,
  ...rest
}: Readonly<WaveformIconProps>): React.JSX.Element {
  return (
    <svg
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      aria-hidden="true"
      focusable="false"
      data-waveform-icon="sine"
      className={mergeClassNames(baseWaveformIconClassName, className)}
      {...rest}
    >
      <path d="M1 8c1.75 0 1.75-6 3.5-6s1.75 12 3.5 12 1.75-12 3.5-12 1.75 6 3.5 6" />
    </svg>
  );
}

export function SquareWaveIcon({
  className,
  ...rest
}: Readonly<WaveformIconProps>): React.JSX.Element {
  return (
    <svg
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      aria-hidden="true"
      focusable="false"
      data-waveform-icon="square"
      className={mergeClassNames(baseWaveformIconClassName, className)}
      {...rest}
    >
      <path d="M1 12V4h7v8h7" />
    </svg>
  );
}

export function SawWaveIcon({
  className,
  ...rest
}: Readonly<WaveformIconProps>): React.JSX.Element {
  return (
    <svg
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      aria-hidden="true"
      focusable="false"
      data-waveform-icon="saw"
      className={mergeClassNames(baseWaveformIconClassName, className)}
      {...rest}
    >
      <path d="M1 12 11 4v8h4" />
    </svg>
  );
}

export function TriangleWaveIcon({
  className,
  ...rest
}: Readonly<WaveformIconProps>): React.JSX.Element {
  return (
    <svg
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.6"
      aria-hidden="true"
      focusable="false"
      data-waveform-icon="triangle"
      className={mergeClassNames(baseWaveformIconClassName, className)}
      {...rest}
    >
      <path d="M1 12 5.5 4 10 12l4.5-8" />
    </svg>
  );
}
