import type { HTMLAttributes, ReactNode } from 'react';
import { mergeClassNames } from './utils/classNames';

export interface ColProps extends Omit<HTMLAttributes<HTMLDivElement>, 'children'> {
  readonly children: ReactNode;
}

const BASE_COL_CLASS_NAME = 'flex flex-col gap-2';

export function Col({ children, className, ...props }: Readonly<ColProps>): React.JSX.Element {
  return (
    <div className={mergeClassNames(BASE_COL_CLASS_NAME, className)} {...props}>
      {children}
    </div>
  );
}
