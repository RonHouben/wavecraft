import {
  useMeterSignalActivity,
  type PassthroughBypassParameterId as CorePassthroughBypassParameterId,
  type PassthroughProcessorId as CorePassthroughProcessorId,
} from '@wavecraft/core';
import { type JSX } from 'react';
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

export function PassthroughProcessor({
  processorId,
  title,
  subtitle,
  hideWhenNotInSignalChain,
  className,
}: Readonly<PassthroughProcessorProps>): JSX.Element | null {
  const { isSignalActive } = useMeterSignalActivity({
    smoothing: {
      enabled: true,
    },
  });

  return (
    <ProcessorCard
      processorId={processorId}
      hideWhenNotInSignalChain={hideWhenNotInSignalChain}
      title={title ?? 'Passthrough'}
      subtitle={subtitle ?? 'Bypass'}
      className={mergeClassNames('h-full w-full', className)}
    >
      <div className="flex flex-1 items-center justify-center">
        <div
          aria-hidden="true"
          data-testid="passthrough-eye"
          data-signal-active={isSignalActive ? 'true' : 'false'}
          className="relative flex h-20 w-20 items-center justify-center rounded-full border border-plugin-border-strong bg-plugin-canvas shadow-control"
        >
          <div
            className={mergeClassNames(
              'absolute inset-0 rounded-full bg-meter-clip-dark blur-md motion-safe:transition-opacity motion-safe:duration-700 motion-reduce:transition-none',
              isSignalActive ? 'opacity-70' : 'opacity-0'
            )}
          />
          <div
            className={mergeClassNames(
              'absolute inset-2 rounded-full bg-meter-clip motion-safe:transition-[opacity,transform] motion-safe:duration-500 motion-reduce:transition-none',
              isSignalActive ? 'scale-100 opacity-90' : 'scale-95 opacity-35'
            )}
          />
          <div
            className={mergeClassNames(
              'absolute h-4 w-4 rounded-full bg-plugin-text-primary motion-safe:transition-opacity motion-safe:duration-500 motion-reduce:transition-none',
              isSignalActive ? 'opacity-90' : 'opacity-60'
            )}
          />
        </div>
      </div>
    </ProcessorCard>
  );
}
