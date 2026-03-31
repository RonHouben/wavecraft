import {
  getMeterClipWarningIntensity,
  usePassthroughMeterSignalActivity,
  type PassthroughBypassParameterId as CorePassthroughBypassParameterId,
  type PassthroughProcessorId as CorePassthroughProcessorId,
} from '@wavecraft/core';
import { dbToLinear } from '@wavecraft/core/meters';
import { useEffect, useState, type JSX } from 'react';
import { mergeClassNames } from '../utils/classNames';
import { ProcessorCard } from './ProcessorCard';

export type PassthroughProcessorId = CorePassthroughProcessorId;
export type PassthroughBypassParameterId = CorePassthroughBypassParameterId;

export interface PassthroughProcessorProps {
  readonly processorId: PassthroughProcessorId;
  readonly title?: string;
  readonly subtitle?: string;
  readonly hideWhenNotInSignalChain?: boolean;
  readonly className?: string;
}

const PASSTHROUGH_SIGNAL_INTENSITY_FLOOR_DB = -54;
const PASSTHROUGH_SIGNAL_INTENSITY_CEILING_DB = -6;
const PASSTHROUGH_CLIP_WARNING_THRESHOLD_DB = -1;
const PASSTHROUGH_CLIP_WARNING_RELEASE_DB = -1.5;
const PASSTHROUGH_CLIP_WARNING_CEILING_DB = 0;
const PASSTHROUGH_CLIP_WARNING_THRESHOLD_LEVEL = dbToLinear(PASSTHROUGH_CLIP_WARNING_THRESHOLD_DB);
const PASSTHROUGH_CLIP_WARNING_RELEASE_LEVEL = dbToLinear(PASSTHROUGH_CLIP_WARNING_RELEASE_DB);

export function PassthroughProcessor({
  processorId,
  title,
  subtitle,
  hideWhenNotInSignalChain,
  className,
}: Readonly<PassthroughProcessorProps>): JSX.Element | null {
  const { isSignalActive, signalIntensity, signalLevel } = usePassthroughMeterSignalActivity({
    smoothing: {
      enabled: true,
    },
    intensityRange: {
      floorDb: PASSTHROUGH_SIGNAL_INTENSITY_FLOOR_DB,
      ceilingDb: PASSTHROUGH_SIGNAL_INTENSITY_CEILING_DB,
    },
  });
  const [isClipWarningActive, setIsClipWarningActive] = useState(false);

  useEffect(() => {
    if (signalLevel >= PASSTHROUGH_CLIP_WARNING_THRESHOLD_LEVEL) {
      setIsClipWarningActive(true);
      return;
    }

    if (signalLevel <= PASSTHROUGH_CLIP_WARNING_RELEASE_LEVEL) {
      setIsClipWarningActive(false);
    }
  }, [signalLevel]);

  const outerGlowOpacity = isSignalActive ? clamp(0.08 + signalIntensity * 0.72, 0, 0.8) : 0;
  const clipWarningIntensity = isClipWarningActive
    ? getMeterClipWarningIntensity(signalLevel, {
        thresholdDb: PASSTHROUGH_CLIP_WARNING_RELEASE_DB,
        ceilingDb: PASSTHROUGH_CLIP_WARNING_CEILING_DB,
      })
    : 0;
  const clipOuterGlowOpacity = clamp(clipWarningIntensity * 0.9, 0, 0.9);
  const clipOuterRingOpacity = isClipWarningActive
    ? clamp(0.25 + clipWarningIntensity * 0.75, 0.25, 1)
    : 0;
  const innerGlowOpacity = clamp(0.35 + signalIntensity * 0.55, 0.35, 0.9);
  const pupilOpacity = clamp(0.6 + signalIntensity * 0.3, 0.6, 0.9);
  const eyeScale = 0.95 + signalIntensity * 0.05;
  const clipAuraColorClass = isClipWarningActive ? 'bg-meter-clip' : 'bg-accent';
  const clipRingColorClass = isClipWarningActive ? 'border-meter-clip' : 'border-accent-light';
  const innerGlowColorClass = isClipWarningActive ? 'bg-meter-clip' : 'bg-accent';
  const outerGlowColorClass = isClipWarningActive
    ? 'bg-meter-clip-dark'
    : 'bg-plugin-border-strong';

  return (
    <ProcessorCard
      processorId={processorId}
      hideWhenNotInSignalChain={hideWhenNotInSignalChain}
      title={title ?? 'Passthrough'}
      subtitle={subtitle ?? 'Bypass'}
      className={mergeClassNames(
        'flex h-full w-full flex-col [&>div:last-child]:flex-1',
        className
      )}
    >
      <div className="flex h-full w-full flex-1 items-center justify-center">
        <div
          aria-hidden="true"
          data-testid="passthrough-eye"
          data-signal-active={isSignalActive ? 'true' : 'false'}
          data-signal-intensity={signalIntensity.toFixed(3)}
          data-clip-warning-active={isClipWarningActive ? 'true' : 'false'}
          data-clip-warning-intensity={clipWarningIntensity.toFixed(3)}
          className="relative flex h-20 w-20 items-center justify-center rounded-full border border-plugin-border-strong bg-plugin-canvas shadow-control"
        >
          <div
            data-testid="passthrough-eye-outer-glow"
            style={{ opacity: outerGlowOpacity }}
            className={mergeClassNames(
              'absolute inset-0 rounded-full blur-md motion-safe:transition-opacity motion-safe:duration-700 motion-reduce:transition-none',
              outerGlowColorClass
            )}
          />
          <div
            data-testid="passthrough-eye-clip-aura"
            style={{
              opacity: clipOuterGlowOpacity,
              transform: `scale(${1 + clipWarningIntensity * 0.04})`,
            }}
            className={mergeClassNames(
              'pointer-events-none absolute -inset-1 rounded-full blur-lg motion-safe:transition-[opacity,transform] motion-safe:duration-200 motion-reduce:transition-none',
              clipAuraColorClass
            )}
          />
          <div
            data-testid="passthrough-eye-clip-ring"
            style={{ opacity: clipOuterRingOpacity }}
            className={mergeClassNames(
              'pointer-events-none absolute inset-0 rounded-full border motion-safe:transition-opacity motion-safe:duration-200 motion-reduce:transition-none',
              clipRingColorClass
            )}
          />
          <div
            data-testid="passthrough-eye-inner-glow"
            style={{ opacity: innerGlowOpacity, transform: `scale(${eyeScale})` }}
            className={mergeClassNames(
              'absolute inset-2 rounded-full motion-safe:transition-[opacity,transform] motion-safe:duration-500 motion-reduce:transition-none',
              innerGlowColorClass
            )}
          />
          <div
            data-testid="passthrough-eye-pupil"
            style={{ opacity: pupilOpacity }}
            className="absolute h-4 w-4 rounded-full bg-plugin-text-primary motion-safe:transition-opacity motion-safe:duration-500 motion-reduce:transition-none"
          />
        </div>
      </div>
    </ProcessorCard>
  );
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
