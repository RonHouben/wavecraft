import * as icons from './icons/WaveformIcons';

type Icon = keyof typeof icons.IconComponentMap;

export interface IconProps {
  icon: Icon;
}

export function Icon({ icon }: Readonly<IconProps>) {
  const IconComponent = icons.IconComponentMap[icon];

  return IconComponent ? <IconComponent /> : null;
}
