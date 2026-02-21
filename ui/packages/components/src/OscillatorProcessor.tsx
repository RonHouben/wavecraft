/**
 * OscillatorProcessor - Displays oscillator signal status and oscillator-specific controls.
 */

import React from 'react';
import { Processor } from './Processor';

interface OscillatorProcessorProps {
  hideWhenNotInSignalChain?: boolean;
}

export function OscillatorProcessor({
  hideWhenNotInSignalChain,
}: Readonly<OscillatorProcessorProps>): React.JSX.Element {
  return <Processor id="oscillator" hideWhenNotInSignalChain={hideWhenNotInSignalChain} />;
}
