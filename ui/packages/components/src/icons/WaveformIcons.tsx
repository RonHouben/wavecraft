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
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
      data-waveform-icon="sine"
      className={mergeClassNames(baseWaveformIconClassName, className)}
      {...rest}
    >
      <path d="M1 8C2.5 4 4.5 4 6 8S9.5 12 11 8 13.5 4 15 8" />
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
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
      data-waveform-icon="square"
      className={mergeClassNames(baseWaveformIconClassName, className)}
      {...rest}
    >
      <path d="M1 11.5V4.5h5.5v7h5.5v-7H15" />
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
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
      data-waveform-icon="saw"
      className={mergeClassNames(baseWaveformIconClassName, className)}
      {...rest}
    >
      <path d="M1 11.5 6.5 4.5v7L12 4.5v7" />
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
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
      data-waveform-icon="triangle"
      className={mergeClassNames(baseWaveformIconClassName, className)}
      {...rest}
    >
      <path d="M1 10.5 4.5 4.5 8 10.5 11.5 4.5 15 10.5" />
    </svg>
  );
}
