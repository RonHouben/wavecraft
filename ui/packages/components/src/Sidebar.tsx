import { type HTMLAttributes, type ReactNode, useEffect, useId } from 'react';
import { mergeClassNames } from './utils/classNames';

type NativeDivProps = Omit<HTMLAttributes<HTMLDivElement>, 'children'>;

export interface SidebarProps extends NativeDivProps {
  readonly children: ReactNode;
  readonly onClose: () => void;
  readonly open: boolean;
  readonly title?: string;
}

export function Sidebar({
  children,
  className,
  onClose,
  open,
  title = 'Sidebar',
  ...props
}: Readonly<SidebarProps>): React.JSX.Element | null {
  const titleId = useId();

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

  if (!open) {
    return null;
  }

  return (
    <div className="fixed inset-0 z-40 flex justify-end" data-testid="sidebar-overlay">
      <button
        type="button"
        aria-label="Dismiss sidebar"
        className="absolute inset-0 bg-plugin-dark/70 motion-safe:transition-opacity motion-safe:duration-150"
        onClick={onClose}
      />

      <div
        role="dialog"
        aria-modal="true"
        aria-labelledby={titleId}
        className={mergeClassNames(
          'relative z-10 flex h-full w-full max-w-80 flex-col gap-4 border-l border-plugin-border bg-plugin-surface-1 p-4 text-plugin-text-primary shadow-panel motion-safe:transition-transform motion-safe:duration-150',
          className
        )}
        {...props}
      >
        <div className="flex items-center justify-between gap-3 border-b border-plugin-border pb-3">
          <h2 id={titleId} className="text-type-lg font-semibold text-plugin-text-primary">
            {title}
          </h2>
        </div>

        <div className="flex flex-1 flex-col gap-3 overflow-y-auto">{children}</div>
      </div>
    </div>
  );
}
