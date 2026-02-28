import type { HTMLAttributes, ReactNode } from 'react';
import { mergeClassNames } from './utils/classNames';

export interface RowProps extends Omit<HTMLAttributes<HTMLDivElement>, 'children'> {
  readonly children: ReactNode;
}

export function Row({ children, className, ...props }: Readonly<RowProps>): React.JSX.Element {
  return (
    <div className={mergeClassNames('col-span-12 grid grid-cols-12', className)} {...props}>
      {children}
    </div>
  );
}
