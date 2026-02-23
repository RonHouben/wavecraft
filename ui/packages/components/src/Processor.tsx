/**
 * Processor component - displays all controls for a single processor.
 */

import React from 'react';
import { Card } from './Card';
import type { ParameterInfo } from './types';
import { parameterListClass, sectionHeadingClass } from './utils/classNames';
import { renderParameter } from './utils/renderParameter';

export interface ProcessorParameter extends ParameterInfo {
  readonly onChange: (value: number | boolean) => void | Promise<void>;
  readonly disabled?: boolean;
}

export interface ProcessorProps {
  readonly id: string;
  readonly title?: string;
  readonly parameters: ProcessorParameter[];
}

function isBypassParameter(param: Pick<ProcessorParameter, 'id'>): boolean {
  return param.id.endsWith('_bypass');
}

export function Processor({ id, title, parameters }: Readonly<ProcessorProps>): React.JSX.Element {
  const bypassParameters = parameters.filter((param) => isBypassParameter(param));
  const regularParameters = parameters.filter((param) => !isBypassParameter(param));
  const orderedParameters = [...bypassParameters, ...regularParameters];

  return (
    <Card className="space-y-2">
      <h3 className={sectionHeadingClass}>{title ?? id}</h3>

      <div className={parameterListClass}>
        {orderedParameters.map((param) => renderParameter(param, param.id))}
      </div>
    </Card>
  );
}
