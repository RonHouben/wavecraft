import type { HTMLAttributes, ReactNode } from 'react';
import { mergeClassNames, surfaceCardClass } from './utils/classNames';

type NativeDivProps = Omit<HTMLAttributes<HTMLDivElement>, 'children'>;

export interface CardProps extends NativeDivProps {
  readonly children: ReactNode;
}

interface CardHeaderProps extends NativeDivProps {
  readonly children: ReactNode;
}

type NativeHeadingProps = Omit<HTMLAttributes<HTMLHeadingElement>, 'children'>;

interface CardTitleProps extends NativeHeadingProps {
  readonly children: ReactNode;
}

interface CardDescriptionProps extends NativeDivProps {
  readonly children: ReactNode;
}

interface CardContentProps extends NativeDivProps {
  readonly children: ReactNode;
}

interface CardFooterProps extends NativeDivProps {
  readonly children: ReactNode;
}

export function CardHeader({
  children,
  className,
  ...props
}: Readonly<CardHeaderProps>): React.JSX.Element {
  return (
    <div
      className={mergeClassNames('flex flex-row flex-wrap justify-between gap-1.5', className)}
      {...props}
    >
      {children}
    </div>
  );
}

export function CardTitle({
  children,
  className,
  ...props
}: Readonly<CardTitleProps>): React.JSX.Element {
  return (
    <h3
      className={mergeClassNames('text-type-md font-semibold text-plugin-text-primary', className)}
      {...props}
    >
      {children}
    </h3>
  );
}

export function CardDescription({
  children,
  className,
  ...props
}: Readonly<CardDescriptionProps>): React.JSX.Element {
  return (
    <div
      className={mergeClassNames('text-type-sm text-plugin-text-secondary', className)}
      {...props}
    >
      {children}
    </div>
  );
}

export function CardContent({
  children,
  className,
  ...props
}: Readonly<CardContentProps>): React.JSX.Element {
  return (
    <div className={mergeClassNames('pt-4', className)} {...props}>
      {children}
    </div>
  );
}

export function CardFooter({
  children,
  className,
  ...props
}: Readonly<CardFooterProps>): React.JSX.Element {
  return (
    <div className={mergeClassNames('flex items-center gap-2 pt-4', className)} {...props}>
      {children}
    </div>
  );
}

function CardRoot({ children, className, ...props }: Readonly<CardProps>): React.JSX.Element {
  return (
    <div
      className={mergeClassNames(surfaceCardClass, 'text-plugin-text-primary', className)}
      {...props}
    >
      {children}
    </div>
  );
}

type CardCompoundComponent = ((props: Readonly<CardProps>) => React.JSX.Element) & {
  readonly Header: typeof CardHeader;
  readonly Title: typeof CardTitle;
  readonly Description: typeof CardDescription;
  readonly Content: typeof CardContent;
  readonly Footer: typeof CardFooter;
};

export const Card: CardCompoundComponent = Object.assign(CardRoot, {
  Header: CardHeader,
  Title: CardTitle,
  Description: CardDescription,
  Content: CardContent,
  Footer: CardFooter,
});
