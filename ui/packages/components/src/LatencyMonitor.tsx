/**
 * LatencyMonitor - Displays IPC roundtrip latency metrics
 */

import React from 'react';
import { Card } from './Card';
import {
  elevatedCardClass,
  insetSurfaceClass,
  mergeClassNames,
  statusChipClass,
} from './utils/classNames';

export interface LatencyMonitorProps {
  readonly latency: number | null;
  readonly avg: number;
  readonly max: number;
  readonly count: number;
  readonly className?: string;
}

export function LatencyMonitor({
  latency,
  avg,
  max,
  count,
  className,
}: Readonly<LatencyMonitorProps>): React.JSX.Element {
  const metrics = [
    {
      label: 'Current',
      value: latency === null ? '—' : `${latency.toFixed(2)} ms`,
    },
    {
      label: 'Average',
      value: avg > 0 ? `${avg.toFixed(2)} ms` : '—',
    },
    {
      label: 'Max',
      value: max > 0 ? `${max.toFixed(2)} ms` : '—',
    },
    {
      label: 'Samples',
      value: count.toString(),
    },
  ];

  return (
    <Card data-testid="latency-monitor" className={mergeClassNames(elevatedCardClass, className)}>
      <Card.Header>
        <Card.Title>IPC Latency</Card.Title>
        <span
          data-testid="latency-monitor-status"
          className={mergeClassNames(statusChipClass, getLatencyStatusClass(avg, count))}
        >
          {getLatencyStatusLabel(avg, count)}
        </span>
      </Card.Header>

      <Card.Content>
        <dl className="grid grid-cols-2 gap-3">
          {metrics.map((metric) => (
            <div
              key={metric.label}
              className={mergeClassNames(insetSurfaceClass, 'flex flex-col gap-1 px-3 py-2')}
            >
              <dt className="text-type-2xs uppercase tracking-wide text-plugin-text-secondary">
                {metric.label}
              </dt>
              <dd className="m-0 font-mono text-type-sm text-plugin-text-primary">
                {metric.value}
              </dd>
            </div>
          ))}
        </dl>
      </Card.Content>
    </Card>
  );
}

function getLatencyStatusClass(avg: number, count: number): string {
  if (count === 0 || avg <= 0) {
    return 'border-plugin-border text-plugin-text-secondary';
  }

  if (avg < 5) {
    return 'border-state-success/60 text-state-success';
  }

  if (avg < 10) {
    return 'border-state-warning/60 text-state-warning';
  }

  return 'border-state-danger/60 text-state-danger';
}

function getLatencyStatusLabel(avg: number, count: number): string {
  if (count === 0 || avg <= 0) {
    return 'Idle';
  }

  if (avg < 5) {
    return 'Excellent';
  }

  if (avg < 10) {
    return 'Fair';
  }

  return 'Poor';
}
