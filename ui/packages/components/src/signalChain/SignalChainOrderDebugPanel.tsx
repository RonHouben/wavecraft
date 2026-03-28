import { useSignalChainOrder } from '@wavecraft/core';
import React from 'react';

import { Card } from '../Card';
import { ErrorMessage } from '../ErrorMessage';
import {
  elevatedCardClass,
  insetSurfaceClass,
  mergeClassNames,
  statusChipClass,
} from '../utils/classNames';

export interface SignalChainOrderDebugPanelProps {
  readonly className?: string;
}

export function SignalChainOrderDebugPanel({
  className,
}: Readonly<SignalChainOrderDebugPanelProps>): React.JSX.Element {
  const { order, isLoading, error } = useSignalChainOrder();

  return (
    <Card
      data-testid="signal-chain-order-debug-panel"
      className={mergeClassNames('w-full', elevatedCardClass, className)}
    >
      <Card.Header>
        <div>
          <Card.Title>Backend signal chain</Card.Title>
          <Card.Description>
            Live backend-reported slot order from JSON-RPC notifications.
          </Card.Description>
        </div>
        <div
          className={mergeClassNames(
            statusChipClass,
            'border-plugin-border text-plugin-text-secondary'
          )}
        >
          {order.length} slots
        </div>
      </Card.Header>

      <Card.Content className="flex flex-col gap-3 pt-3">
        {error ? (
          <ErrorMessage
            data-testid="signal-chain-order-debug-error"
            message={`Unable to read backend signal chain: ${error.message}`}
          />
        ) : null}

        {!error && isLoading ? (
          <div
            data-testid="signal-chain-order-debug-loading"
            className={mergeClassNames(
              insetSurfaceClass,
              'px-3 py-3 text-type-sm text-plugin-text-secondary'
            )}
          >
            Waiting for backend order…
          </div>
        ) : null}

        {!error && !isLoading && order.length === 0 ? (
          <div
            data-testid="signal-chain-order-debug-empty"
            className={mergeClassNames(
              insetSurfaceClass,
              'px-3 py-3 text-type-sm text-plugin-text-secondary'
            )}
          >
            Backend has not reported any slots yet.
          </div>
        ) : null}

        {!error && !isLoading && order.length > 0 ? (
          <ol
            data-testid="signal-chain-order-debug-list"
            className="flex list-decimal flex-col gap-3 pl-5 marker:text-plugin-text-muted"
          >
            {order.map((slot, index) => (
              <li key={`${slot.type}:${slot.id}`} className="pl-1">
                <div
                  className={mergeClassNames(
                    insetSurfaceClass,
                    'flex items-center justify-between gap-3 px-3 py-2'
                  )}
                >
                  <div className="flex min-w-0 flex-col gap-0.5">
                    <span className="text-type-2xs uppercase tracking-wide text-plugin-text-secondary">
                      Slot {index + 1}
                    </span>
                    <code className="truncate font-mono text-type-sm text-plugin-text-primary">
                      {slot.id}
                    </code>
                  </div>
                  <span
                    className={mergeClassNames(
                      statusChipClass,
                      'border-plugin-border text-plugin-text-secondary'
                    )}
                  >
                    {slot.type}
                  </span>
                </div>
              </li>
            ))}
          </ol>
        ) : null}
      </Card.Content>
    </Card>
  );
}
