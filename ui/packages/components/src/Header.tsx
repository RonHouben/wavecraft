export interface HeaderProps {
  title: string;
  children?: React.ReactNode;
}

export function Header(props: Readonly<HeaderProps>) {
  return (
    <div className="relative z-50 flex items-center justify-between gap-3">
      <h1 className="text-2xl font-bold text-gray-100">{props.title}</h1>
      {props.children ? <div className="flex items-center gap-2">{props.children}</div> : null}
    </div>
  );
}
