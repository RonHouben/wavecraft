import type { HTMLAttributes, ReactNode } from 'react';
import { mergeClassNames } from './utils/classNames';

export interface RowProps extends Omit<HTMLAttributes<HTMLDivElement>, 'children'> {
  readonly children: ReactNode;
}

const BASE_ROW_CLASS_NAME = 'flex flex-wrap w-full items-center gap-2';

export function Row({ children, className, ...props }: Readonly<RowProps>): React.JSX.Element {
  return (
    <div className={mergeClassNames(BASE_ROW_CLASS_NAME, className)} {...props}>
      {children}
    </div>
  );
}
