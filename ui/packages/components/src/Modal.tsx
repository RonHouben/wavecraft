import {
  createContext,
  type HTMLAttributes,
  type ReactNode,
  useContext,
  useEffect,
  useId,
  useMemo,
  useRef,
  useState,
} from 'react';
import { Button } from '.';
import { ButtonProps } from './Button';
import { IconButton } from './IconButton';
import { elevatedCardClass, focusRingClass, mergeClassNames } from './utils/classNames';

type NativeDivProps = Omit<HTMLAttributes<HTMLDivElement>, 'children'>;

const MODAL_TRANSITION_MS = 150;

interface ModalContextValue {
  readonly descriptionId: string;
  readonly registerDescription: () => void;
  readonly registerTitle: () => void;
  readonly titleId: string;
  readonly unregisterDescription: () => void;
  readonly unregisterTitle: () => void;
  readonly openModal: () => void;
}

const ModalContext = createContext<ModalContextValue | null>(null);

export interface ModalProps extends NativeDivProps {
  readonly children?: ReactNode;
  readonly onClose: () => void;
  readonly open: boolean;
}

interface ModalHeaderProps extends NativeDivProps {
  readonly children: ReactNode;
}

type NativeHeadingProps = Omit<HTMLAttributes<HTMLHeadingElement>, 'children'>;

interface ModalTitleProps extends NativeHeadingProps {
  readonly children: ReactNode;
}

interface ModalDescriptionProps extends NativeDivProps {
  readonly children: ReactNode;
}

interface ModalContentProps extends NativeDivProps {
  readonly children: ReactNode;
}

interface ModalFooterProps extends NativeDivProps {
  readonly children: ReactNode;
}

type ModalButtonProps = ButtonProps;

export function ModalButton({ title, ...rest }: Readonly<ModalButtonProps>) {
  const modalContext = useModalContext();

  return (
    <Button onClick={modalContext?.openModal} {...rest}>
      {title}
    </Button>
  );
}

export function ModalHeader({
  children,
  className,
  ...props
}: Readonly<ModalHeaderProps>): React.JSX.Element {
  return (
    <div
      className={mergeClassNames(
        'flex flex-col gap-1.5 border-b border-plugin-border pb-3 pr-10',
        className
      )}
      {...props}
    >
      {children}
    </div>
  );
}

export function ModalTitle({
  children,
  className,
  ...props
}: Readonly<ModalTitleProps>): React.JSX.Element {
  const modalContext = useModalContext();

  useEffect(() => {
    modalContext?.registerTitle();

    return () => {
      modalContext?.unregisterTitle();
    };
  }, [modalContext]);

  return (
    <h2
      id={modalContext?.titleId}
      className={mergeClassNames('text-type-lg font-semibold text-plugin-text-primary', className)}
      {...props}
    >
      {children}
    </h2>
  );
}

export function ModalDescription({
  children,
  className,
  ...props
}: Readonly<ModalDescriptionProps>): React.JSX.Element {
  const modalContext = useModalContext();

  useEffect(() => {
    modalContext?.registerDescription();

    return () => {
      modalContext?.unregisterDescription();
    };
  }, [modalContext]);

  return (
    <div
      id={modalContext?.descriptionId}
      className={mergeClassNames('text-type-sm text-plugin-text-secondary', className)}
      {...props}
    >
      {children}
    </div>
  );
}

export function ModalContent({
  children,
  className,
  ...props
}: Readonly<ModalContentProps>): React.JSX.Element {
  return (
    <div className={mergeClassNames('min-h-0 flex-1 overflow-y-auto', className)} {...props}>
      {children}
    </div>
  );
}

export function ModalFooter({
  children,
  className,
  ...props
}: Readonly<ModalFooterProps>): React.JSX.Element {
  return (
    <div
      className={mergeClassNames(
        'mt-auto flex flex-row flex-wrap items-center justify-end gap-2 border-t border-plugin-border pt-4',
        className
      )}
      {...props}
    >
      {children}
    </div>
  );
}

function ModalRoot({
  children,
  className,
  onClose,
  open,
  ...props
}: Readonly<ModalProps>): React.JSX.Element | null {
  const panelRef = useRef<HTMLDivElement | null>(null);
  const [descriptionCount, setDescriptionCount] = useState(0);
  const [isRendered, setIsRendered] = useState(open);
  const [titleCount, setTitleCount] = useState(0);
  const titleId = useId();
  const descriptionId = useId();

  const hasDescription = descriptionCount > 0;
  const hasTitle = titleCount > 0;

  const modalContextValue = useMemo<ModalContextValue>(
    () => ({
      descriptionId,
      registerDescription: () => {
        setDescriptionCount((count) => count + 1);
      },
      registerTitle: () => {
        setTitleCount((count) => count + 1);
      },
      titleId,
      unregisterDescription: () => {
        setDescriptionCount((count) => Math.max(0, count - 1));
      },
      unregisterTitle: () => {
        setTitleCount((count) => Math.max(0, count - 1));
      },
      openModal: () => setIsRendered(true),
    }),
    [descriptionId, titleId]
  );

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
    }, MODAL_TRANSITION_MS);

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
        event.preventDefault();
        event.stopImmediatePropagation();
        onClose();
        return;
      }

      if (event.key !== 'Tab' || event.defaultPrevented || panelRef.current === null) {
        return;
      }

      trapFocus(event, panelRef.current);
    };

    globalThis.addEventListener('keydown', handleKeyDown, true);

    return () => {
      globalThis.removeEventListener('keydown', handleKeyDown, true);
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
    <ModalContext.Provider value={modalContextValue}>
      <div
        className="fixed inset-0 z-40 flex items-center justify-center p-4"
        data-open={String(open)}
      >
        <button
          type="button"
          aria-label="Dismiss modal"
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
          aria-labelledby={hasTitle ? titleId : undefined}
          aria-describedby={hasDescription ? descriptionId : undefined}
          tabIndex={-1}
          className={mergeClassNames(
            elevatedCardClass,
            focusRingClass,
            'relative z-10 flex max-h-[min(36rem,calc(100vh-2rem))] w-full max-w-lg flex-col gap-4 overflow-hidden p-4 text-plugin-text-primary motion-safe:transition-all motion-safe:duration-150',
            open ? 'translate-y-0 scale-100 opacity-100' : 'translate-y-2 scale-95 opacity-0',
            className
          )}
          {...props}
        >
          <IconButton
            icon="close"
            size="sm"
            aria-label="Close modal"
            className="absolute right-3 top-3"
            onClick={onClose}
          />
          {children}
        </div>
      </div>
    </ModalContext.Provider>
  );
}

type ModalCompoundComponent = ((props: Readonly<ModalProps>) => React.JSX.Element | null) & {
  readonly Content: typeof ModalContent;
  readonly Description: typeof ModalDescription;
  readonly Footer: typeof ModalFooter;
  readonly Header: typeof ModalHeader;
  readonly Title: typeof ModalTitle;
  readonly Button: typeof ModalButton;
};

export const Modal: ModalCompoundComponent = Object.assign(ModalRoot, {
  Content: ModalContent,
  Description: ModalDescription,
  Footer: ModalFooter,
  Header: ModalHeader,
  Title: ModalTitle,
  Button: ModalButton,
});

function useModalContext(): ModalContextValue | null {
  return useContext(ModalContext);
}

function trapFocus(event: KeyboardEvent, panel: HTMLDivElement): void {
  const focusableElements = getFocusableElements(panel);

  if (focusableElements.length === 0) {
    event.preventDefault();
    panel.focus();
    return;
  }

  const firstFocusableElement = focusableElements[0];
  const lastFocusableElement = focusableElements[focusableElements.length - 1];

  if (firstFocusableElement === undefined || lastFocusableElement === undefined) {
    event.preventDefault();
    panel.focus();
    return;
  }

  const activeElement = globalThis.document.activeElement;

  if (!(activeElement instanceof HTMLElement) || !panel.contains(activeElement)) {
    event.preventDefault();
    firstFocusableElement.focus();
    return;
  }

  if (!event.shiftKey && activeElement === panel) {
    event.preventDefault();
    firstFocusableElement.focus();
    return;
  }

  if (event.shiftKey) {
    if (activeElement === firstFocusableElement || activeElement === panel) {
      event.preventDefault();
      lastFocusableElement.focus();
    }

    return;
  }

  if (activeElement === lastFocusableElement) {
    event.preventDefault();
    firstFocusableElement.focus();
  }
}

function getFocusableElements(panel: HTMLDivElement): HTMLElement[] {
  const focusableSelector = [
    'a[href]',
    'button:not([disabled])',
    'input:not([disabled])',
    'select:not([disabled])',
    'textarea:not([disabled])',
    '[tabindex]:not([tabindex="-1"])',
  ].join(',');

  return Array.from(panel.querySelectorAll<HTMLElement>(focusableSelector)).filter(
    (element) => !element.hasAttribute('hidden') && element.getAttribute('aria-hidden') !== 'true'
  );
}
