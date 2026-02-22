export const focusRingClass =
  'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-accent-light focus-visible:ring-offset-2 focus-visible:ring-offset-plugin-dark';

export const interactionStateClass =
  'motion-safe:transition-all motion-safe:duration-150 hover:brightness-110 active:scale-[0.98] disabled:cursor-not-allowed disabled:opacity-50';

export const surfaceCardClass = 'rounded-lg border border-plugin-border bg-plugin-surface p-4';

export const sectionHeadingClass = 'text-sm font-semibold uppercase tracking-wider text-gray-400';

export const parameterListClass = 'space-y-3';

export function mergeClassNames(...classes: Array<string | undefined | null | false>): string {
  return classes.filter(Boolean).join(' ');
}
