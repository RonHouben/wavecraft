export interface HeaderProps {
  title: string;
  children?: React.ReactNode;
}

export function Header(props: Readonly<HeaderProps>) {
  return (
    <div className="flex gap-2">
      <h1 className="text-2xl font-bold text-gray-100">{props.title}</h1>
      {props.children}
    </div>
  );
}
