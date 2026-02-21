import React from 'react';

import { Processor } from './Processor';
import type { ProcessorProps } from './Processor';

type CompatProcessorProps = Omit<ProcessorProps, 'id'>;

/** @deprecated Use <Processor id="input_trim" .../> directly. */
export function InputTrimProcessor(props: Readonly<CompatProcessorProps>): React.JSX.Element {
  return React.createElement(Processor, { id: 'input_trim', ...props });
}

/** @deprecated Use <Processor id="output_gain" .../> directly. */
export function OutputGainProcessor(props: Readonly<CompatProcessorProps>): React.JSX.Element {
  return React.createElement(Processor, { id: 'output_gain', ...props });
}

/** @deprecated Use <Processor id="soft_clip" .../> directly. */
export function SoftClipProcessor(props: Readonly<CompatProcessorProps>): React.JSX.Element {
  return React.createElement(Processor, { id: 'soft_clip', ...props });
}

/** @deprecated Use <Processor id="tone_filter" .../> directly. */
export function ToneFilterProcessor(props: Readonly<CompatProcessorProps>): React.JSX.Element {
  return React.createElement(Processor, { id: 'tone_filter', ...props });
}

/** @deprecated Use <Processor id="oscillator" .../> directly. */
export function OscillatorProcessor(props: Readonly<CompatProcessorProps>): React.JSX.Element {
  return React.createElement(Processor, { id: 'oscillator', ...props });
}
