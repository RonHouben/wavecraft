import React, { useId, type ReactNode } from 'react';
import { focusRingClass, mergeClassNames } from './utils/classNames';
import { getControlStateClass } from './utils/controlStates';
import { PolymorphicProps } from './types';
import { omit } from 'lodash';

type inputType = string | number | undefined;

export interface RadioGroupOption<T extends inputType> {
  readonly value: T;
  readonly label: ReactNode;
  readonly disabled?: boolean;
}

export type RadioGroupOwnProps<T extends inputType, C extends React.ElementType> = {
  name: string;
  options: readonly PolymorphicProps<C, RadioGroupOption<T>>[];
  value: T;
  onChange: (value: T) => void;
  label?: React.ReactNode;
  disabled?: boolean;
  className?: string;
  orientation?: 'horizontal' | 'vertical';
};

export function RadioGroup<T extends inputType, C extends React.ElementType>(
  props: Readonly<RadioGroupOwnProps<T, C>>
): React.JSX.Element {
  const generatedId = useId();
  const labelId = props.label ? `${generatedId}-label` : undefined;

  const selectOption = (index: number): void => {
    const option = props.options[index];
    if (!option || props.disabled || option.disabled) {
      return;
    }

    if (option.value !== props.value) {
      props.onChange(option.value);
    }
  };

  return (
    <div className={mergeClassNames('inline-flex flex-col gap-2', props.className)}>
      {props.label ? (
        <span id={labelId} className="text-type-sm text-plugin-text-secondary">
          {props.label}
        </span>
      ) : null}

      <div
        role="radiogroup"
        aria-labelledby={labelId}
        aria-disabled={props.disabled || undefined}
        aria-orientation={props.orientation}
        className={mergeClassNames(
          'inline-flex gap-2',
          props.orientation === 'vertical' ? 'flex-col items-start' : 'items-center'
        )}
      >
        {props.options.map((option, index) => {
          const isChecked = option.value === props.value;
          const isDisabled = props.disabled || option.disabled;

          const className = mergeClassNames(
            'inline-flex min-w-[72px] items-center justify-center rounded-md border border-plugin-border px-3 py-2 text-type-sm shadow-control',
            focusRingClass,
            getControlStateClass({ disabled: isDisabled }),
            isChecked
              ? 'border-accent-light bg-gradient-to-b from-accent/30 to-accent/15 font-semibold text-accent ring-1 ring-inset ring-accent/45'
              : 'bg-plugin-surface text-plugin-text-primary'
          );

          const commonProps = {
            ...omit(option, 'as'),
            'aria-checked': isChecked,
            'aria-disabled': isDisabled || undefined,
            key: `${props.name}-${option.value}`,
            id: `${generatedId}-${index}`,
            className,
            disable: props.disabled,
            onClick: () => selectOption(index),
          };

          if (option.as) {
            return React.createElement(option.as, {
              ...commonProps,
              role: 'radio',
            });
          }

          return <input {...commonProps} type="radio" />;
        })}
      </div>
    </div>
  );
}
