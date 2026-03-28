export interface HeaderProps {
  title: string;
  children?: React.ReactNode;
}

export function Header(props: Readonly<HeaderProps>) {
  return (
    <div className="relative z-50 flex items-start justify-between gap-4">
      <h1 className="min-w-0 flex-1 text-2xl font-bold leading-tight tracking-tight text-plugin-text-primary">
        {props.title}
      </h1>
      {props.children ? (
        <div className="flex shrink-0 items-center gap-2">{props.children}</div>
      ) : null}
    </div>
  );
}
