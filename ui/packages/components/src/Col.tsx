import type { HTMLAttributes, ReactNode } from 'react';
import { mergeClassNames } from './utils/classNames';

export interface ColProps extends Omit<HTMLAttributes<HTMLDivElement>, 'children'> {
  readonly children: ReactNode;
}

export function Col({ children, className, ...props }: Readonly<ColProps>): React.JSX.Element {
  return (
    <div className={mergeClassNames('grid grid-cols-12', className)} {...props}>
      {children}
    </div>
  );
}
