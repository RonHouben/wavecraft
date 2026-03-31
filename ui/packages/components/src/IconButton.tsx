import { Button, ButtonProps } from './Button';
import { Icon, IconProps } from './Icon';
import { mergeClassNames } from './utils/classNames';

export interface IconButtonProps extends Omit<ButtonProps, 'children'>, IconProps {}

const iconButtonSizeClassMap: Record<NonNullable<ButtonProps['size']>, string> = {
  sm: 'h-8 w-8 min-w-0 px-0',
  md: 'h-10 w-10 min-w-0 px-0',
  lg: 'h-11 w-11 min-w-0 px-0',
};

export function IconButton({ icon, className, size = 'md', ...buttonProps }: IconButtonProps) {
  return (
    <Button
      {...buttonProps}
      size={size}
      className={mergeClassNames(
        'text-plugin-text-secondary',
        iconButtonSizeClassMap[size],
        className
      )}
    >
      <Icon icon={icon} />
    </Button>
  );
}
