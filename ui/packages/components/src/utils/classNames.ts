import { clsx, type ClassValue } from 'clsx';
import { extendTailwindMerge } from 'tailwind-merge';

const wavecraftTwMerge = extendTailwindMerge({
  extend: {
    classGroups: {
      'font-size': [
        {
          text: ['type-2xs', 'type-xs', 'type-sm', 'type-md', 'type-lg'],
        },
      ],
      'text-color': [
        {
          text: [
            'plugin-text-primary',
            'plugin-text-secondary',
            'plugin-text-muted',
            'accent',
            'accent-light',
            'state-danger',
          ],
        },
      ],
    },
  },
});

export const focusRingClass =
  'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-light focus-visible:ring-offset-2 focus-visible:ring-offset-plugin-dark';

export const interactionStateClass =
  'motion-safe:transition-all motion-safe:duration-150 hover:brightness-110 active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-50';

export const surfaceCardClass = 'rounded-lg border border-plugin-border bg-plugin-surface p-4';

export const elevatedCardClass =
  'rounded-xl border border-plugin-border bg-plugin-surface-1 p-3 shadow-panel';

export const insetSurfaceClass = 'rounded-lg border border-plugin-border bg-plugin-dark';

export const statusChipClass =
  'rounded-md border bg-plugin-surface px-2 py-1 text-type-2xs font-medium uppercase tracking-wide';

export const sectionHeadingClass = 'text-sm font-semibold uppercase tracking-wider text-gray-400';

export const parameterListClass = 'space-y-3';

export function mergeClassNames(...classes: ClassValue[]): string {
  return wavecraftTwMerge(clsx(classes));
}
