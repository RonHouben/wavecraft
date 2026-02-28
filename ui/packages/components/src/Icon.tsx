import * as icons from './icons/WaveformIcons';

type IconName = keyof typeof icons.IconComponentMap;

export interface IconProps {
  icon: IconName;
}

export function Icon({ icon }: Readonly<IconProps>) {
  const IconComponent = icons.IconComponentMap[icon];

  return IconComponent ? <IconComponent /> : null;
}
