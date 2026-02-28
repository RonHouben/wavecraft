import { omit } from 'lodash';
import React, { useId } from 'react';
import { Row } from './Row';
import { PolymorphicProps } from './types';
import { focusRingClass, mergeClassNames } from './utils/classNames';
import { getControlStateClass } from './utils/controlStates';

type RadioGroupValue = string | number;

export interface RadioGroupOption<T extends RadioGroupValue> {
  readonly value: T;
  readonly label: string;
  readonly disabled?: boolean;
}

export type RadioGroupOwnProps<T extends RadioGroupValue> = {
  name: string;
  options: readonly PolymorphicProps<React.ElementType, RadioGroupOption<T>>[];
  value: T;
  onChange: (value: T) => void;
  label?: string;
  disabled?: boolean;
  className?: string;
  orientation?: 'horizontal' | 'vertical';
};

export function RadioGroup<T extends RadioGroupValue>(
  props: Readonly<RadioGroupOwnProps<T>>
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
    <div
      className={mergeClassNames(
        props.orientation === 'vertical'
          ? 'flex w-full flex-col gap-2'
          : 'inline-flex flex-col gap-2',
        props.className
      )}
    >
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
          'inline-flex gap-3',
          props.orientation === 'vertical' ? 'flex-col items-center' : 'items-center'
        )}
      >
        {props.options.map((option, index) => {
          const isChecked = option.value === props.value;
          const isDisabled = props.disabled || option.disabled;
          const optionKey = `${props.name}-${String(option.value)}`;

          const className = mergeClassNames(
            'inline-flex min-w-[72px] items-center justify-center rounded-md border border-plugin-border px-3 py-2 text-type-sm shadow-control',
            focusRingClass,
            getControlStateClass({ disabled: isDisabled }),
            isChecked
              ? 'border-accent-light bg-gradient-to-b from-accent/30 to-accent/15 font-semibold text-accent ring-1 ring-inset ring-accent/45'
              : 'bg-plugin-surface text-plugin-text-primary'
          );

          const commonProps = {
            ...omit(option, 'as', 'renderOptionsAs'),
            'aria-checked': isChecked,
            'aria-label': option.label,
            'aria-disabled': isDisabled || undefined,
            className,
            disabled: isDisabled,
            tabIndex: -1,
          };

          if (option.as) {
            return React.createElement(option.as, {
              ...commonProps,
              key: optionKey,
              role: 'radio',
              onClick: () => selectOption(index),
              size: 'sm',
            });
          }

          const inputId = `input-${index}-${generatedId}`;

          return (
            <Row key={optionKey} className="text-xs">
              <label htmlFor={inputId}>{option.label}</label>
              <input
                id={inputId}
                name={option.label}
                checked={isChecked}
                {...commonProps}
                type="radio"
                onKeyDown={(event): void => {
                  if (
                    event.key === ' ' ||
                    event.key === 'Enter' ||
                    event.key === 'ArrowRight' ||
                    event.key === 'ArrowLeft' ||
                    event.key === 'ArrowUp' ||
                    event.key === 'ArrowDown'
                  ) {
                    event.preventDefault();
                    event.stopPropagation();
                  }
                }}
                onChange={() => selectOption(index)}
              />
            </Row>
          );
        })}
      </div>
    </div>
  );
}
