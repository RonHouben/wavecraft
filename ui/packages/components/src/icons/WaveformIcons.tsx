import type { SVGProps } from 'react';
import { mergeClassNames } from '../utils/classNames';

// eslint-disable-next-line react-refresh/only-export-components
export const IconComponentMap = {
  'chevron-right': ChevronRightIcon,
  close: CloseIcon,
  menu: MenuIcon,
  settings: SettingsIcon,
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

export function CloseIcon({ className, ...rest }: Readonly<WaveformIconProps>): React.JSX.Element {
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
      data-waveform-icon="close"
      className={mergeClassNames(baseWaveformIconClassName, className)}
      {...rest}
    >
      <path d="M4 4 12 12M12 4 4 12" />
    </svg>
  );
}

export function ChevronRightIcon({
  className,
  ...rest
}: Readonly<WaveformIconProps>): React.JSX.Element {
  return (
    <svg
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.8"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
      data-waveform-icon="chevron-right"
      className={mergeClassNames(baseWaveformIconClassName, className)}
      {...rest}
    >
      <path d="M6 4.5 9.5 8 6 11.5" />
    </svg>
  );
}

export function MenuIcon({ className, ...rest }: Readonly<WaveformIconProps>): React.JSX.Element {
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
      data-waveform-icon="menu"
      className={mergeClassNames(baseWaveformIconClassName, className)}
      {...rest}
    >
      <path d="M2 4.5h12M2 8h12M2 11.5h12" />
    </svg>
  );
}

export function SettingsIcon({
  className,
  ...rest
}: Readonly<WaveformIconProps>): React.JSX.Element {
  return (
    <svg
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.4"
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      focusable="false"
      data-waveform-icon="settings"
      className={mergeClassNames(baseWaveformIconClassName, className)}
      {...rest}
    >
      <path d="M6.65 1.9h2.7l.34 1.56c.34.1.67.24.98.41l1.39-.77 1.9 1.91-.78 1.38c.17.31.31.64.42.99l1.55.34v2.69l-1.55.34c-.11.35-.25.68-.42.99l.78 1.38-1.9 1.91-1.39-.77c-.31.17-.64.31-.98.41l-.34 1.56h-2.7l-.34-1.56a4.8 4.8 0 0 1-.98-.41l-1.39.77-1.9-1.91.78-1.38a4.8 4.8 0 0 1-.42-.99l-1.55-.34V6.72l1.55-.34c.11-.35.25-.68.42-.99l-.78-1.38 1.9-1.91 1.39.77c.31-.17.64-.31.98-.41L6.65 1.9Z" />
      <circle cx="8" cy="8" r="2.15" />
    </svg>
  );
}

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
