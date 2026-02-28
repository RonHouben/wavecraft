import { Button, ButtonProps } from './Button';
import { Icon, IconProps } from './Icon';

export interface IconButtonProps extends Omit<ButtonProps, 'children'>, IconProps {}

export function IconButton({ icon, ...buttonProps }: IconButtonProps) {
  return (
    <Button {...buttonProps}>
      <Icon icon={icon} />
    </Button>
  );
}
