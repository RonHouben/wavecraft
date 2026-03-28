import { type HTMLAttributes, type ReactNode, useEffect, useId, useRef, useState } from 'react';
import { IconButton } from './IconButton';
import { mergeClassNames } from './utils/classNames';

type NativeDivProps = Omit<HTMLAttributes<HTMLDivElement>, 'children'>;

const SIDEBAR_TRANSITION_MS = 150;

type DefaultSidebarActions = ReadonlyArray<'show-settings'>;

export interface SidebarProps extends NativeDivProps {
  readonly children?: ReactNode;
  readonly description?: string;
  readonly onClose: () => void;
  readonly open: boolean;
  readonly title?: string;
  readonly defaultActions: DefaultSidebarActions;
}

export function Sidebar({
  children,
  className,
  defaultActions,
  description,
  onClose,
  open,
  title,
  ...props
}: Readonly<SidebarProps>): React.JSX.Element | null {
  const panelRef = useRef<HTMLDivElement | null>(null);
  const [isRendered, setIsRendered] = useState(open);
  const titleId = useId();
  const descriptionId = useId();

  useEffect(() => {
    if (open) {
      setIsRendered(true);
      return undefined;
    }

    if (!isRendered) {
      return undefined;
    }

    const timeoutId = globalThis.setTimeout(() => {
      setIsRendered(false);
    }, SIDEBAR_TRANSITION_MS);

    return () => {
      globalThis.clearTimeout(timeoutId);
    };
  }, [isRendered, open]);

  useEffect(() => {
    if (!open) {
      return undefined;
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === 'Escape') {
        onClose();
      }
    };

    globalThis.addEventListener('keydown', handleKeyDown);

    return () => {
      globalThis.removeEventListener('keydown', handleKeyDown);
    };
  }, [onClose, open]);

  useEffect(() => {
    if (open) {
      panelRef.current?.focus();
    }
  }, [open]);

  if (!isRendered) {
    return null;
  }

  return (
    <div
      className="fixed inset-0 z-40 flex justify-end"
      data-testid="sidebar-overlay"
      data-open={String(open)}
    >
      <button
        type="button"
        aria-label="Dismiss sidebar"
        className={mergeClassNames(
          'absolute inset-0 bg-plugin-dark/70 motion-safe:transition-opacity motion-safe:duration-150',
          open ? 'opacity-100' : 'opacity-0'
        )}
        onClick={onClose}
      />

      <div
        ref={panelRef}
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        aria-describedby={description ? descriptionId : undefined}
        tabIndex={-1}
        className={mergeClassNames(
          'relative z-10 flex h-full w-full max-w-80 flex-col gap-4 border-l border-plugin-border bg-plugin-surface-1 p-4 text-plugin-text-primary shadow-panel motion-safe:transition-all motion-safe:duration-150',
          open ? 'translate-x-0 opacity-100' : 'translate-x-full opacity-0',
          className
        )}
        {...props}
      >
        <div className="flex items-start justify-between gap-3 border-b border-plugin-border pb-3">
          <div className="flex min-w-0 flex-1 flex-col gap-1">
            <h2 id={titleId} className="text-type-lg font-semibold text-plugin-text-primary">
              {title}
            </h2>
            {description ? (
              <p id={descriptionId} className="text-type-sm text-plugin-text-secondary">
                {description}
              </p>
            ) : null}
          </div>

          <IconButton icon="close" size="sm" aria-label="Close sidebar" onClick={onClose} />
        </div>

        <div className="flex flex-1 flex-col gap-3 overflow-y-auto">
          <div className="text-type-xs font-semibold uppercase tracking-wider text-plugin-text-muted">
            Quick actions
          </div>
          <nav aria-label="Sidebar quick actions" className="flex flex-col gap-2">
            {children}
          </nav>
        </div>

        <div className="border-t border-plugin-border pt-3 text-type-xs text-plugin-text-muted">
          Press{' '}
          <kbd className="rounded border border-plugin-border bg-plugin-surface px-1.5 py-0.5">
            Esc
          </kbd>{' '}
          to close
        </div>
      </div>
    </div>
  );
}
