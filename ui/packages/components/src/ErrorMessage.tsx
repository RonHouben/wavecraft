import React, { type HTMLAttributes } from 'react';
import { mergeClassNames } from './utils/classNames';

type NativeParagraphProps = Omit<HTMLAttributes<HTMLParagraphElement>, 'children'>;

export interface ErrorMessageProps extends NativeParagraphProps {
  readonly message: string;
}

const BASE_ERROR_CLASS_NAME =
  'rounded-md border border-meter-clip bg-meter-clip/10 px-3 py-2 text-type-sm text-meter-clip';

export function ErrorMessage({
  className,
  message,
  ...props
}: Readonly<ErrorMessageProps>): React.JSX.Element {
  return (
    <p
      role="alert"
      aria-live="assertive"
      aria-atomic="true"
      className={mergeClassNames(BASE_ERROR_CLASS_NAME, className)}
      {...props}
    >
      {message}
    </p>
  );
}
