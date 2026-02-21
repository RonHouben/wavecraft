/**
 * Processor component - displays all controls for a single processor.
 */

import React from 'react';
import { useAllParametersFor, useHasProcessorInSignalChain } from '@wavecraft/core';
import { ParameterSlider } from './ParameterSlider';
import { ParameterSelect } from './ParameterSelect';
import { logger } from '@wavecraft/core';
import { ProcessorId } from '@wavecraft/core';
import { ParameterToggle } from './ParameterToggle';

export interface ProcessorProps {
  readonly id: ProcessorId;
  readonly hideWhenNotInSignalChain?: boolean;
}

export function Processor({
  id,
  hideWhenNotInSignalChain,
}: Readonly<ProcessorProps>): React.JSX.Element | null {
  const hasProcessorInSignalChain = useHasProcessorInSignalChain(id);
  const { params } = useAllParametersFor(id);

  if ((hideWhenNotInSignalChain && !hasProcessorInSignalChain) || !params) {
    return null;
  }

  return (
    <div className="border-green-200! border-spacing-300 bg-blue-100">
      <h3 className="text-sm font-semibold uppercase tracking-wider text-gray-400">{id}</h3>

      <div className="space-y-3">
        {params.map((param) => {
          switch (param.type) {
            case 'bool':
              return <ParameterToggle key={param.id} id={param.id} />;
            case 'enum':
              return <ParameterSelect key={param.id} id={param.id} />;
            case 'float':
              return <ParameterSlider key={param.id} id={param.id} />;
            default: {
              const msg = `Unknown processor parameter type ${param.type}`;

              logger.error(msg, {
                processor: id,
                parameterId: param.id,
                parameterType: param.type,
              });

              throw new Error(msg);
            }
          }
        })}
      </div>
    </div>
  );
}
