import { type JSX } from 'react';
import { mergeClassNames } from '../utils/classNames';
import { ProcessorCard } from './ProcessorCard';
import { PassthroughProcessorId } from '@wavecraft/core';

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
  return (
    <ProcessorCard
      processorId={processorId}
      hideWhenNotInSignalChain={hideWhenNotInSignalChain}
      title={title ?? ''}
      subtitle={subtitle ?? ''}
      className={mergeClassNames('h-full w-full', className)}
    ></ProcessorCard>
  );
}
