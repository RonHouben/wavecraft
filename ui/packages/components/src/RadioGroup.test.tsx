import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';
import { Button } from './Button';
import { RadioGroup, type RadioGroupOption } from './RadioGroup';

type Mode = 'clean' | 'drive' | 'fuzz';

const modeOptions: readonly RadioGroupOption<Mode>[] = [
  { value: 'clean', label: 'Clean' },
  { value: 'drive', label: 'Drive' },
  { value: 'fuzz', label: 'Fuzz' },
];

describe('RadioGroup', () => {
  it('renders radiogroup semantics and checked state', () => {
    render(
      <RadioGroup name="mode" label="Mode" value="drive" options={modeOptions} onChange={vi.fn()} />
    );

    expect(screen.getByRole('radiogroup', { name: 'Mode' })).toBeInTheDocument();
    expect(screen.getByRole('radio', { name: 'Drive' })).toHaveAttribute('aria-checked', 'true');
    expect(screen.getByRole('radio', { name: 'Clean' })).toHaveAttribute('aria-checked', 'false');
  });

  it('calls onChange when selecting by click', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <RadioGroup
        name="mode"
        label="Mode"
        value="clean"
        options={modeOptions}
        onChange={onChange}
      />
    );

    await user.click(screen.getByRole('radio', { name: 'Fuzz' }));

    expect(onChange).toHaveBeenCalledWith('fuzz');
  });

  it('ignores keyboard arrow navigation', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <RadioGroup
        name="mode"
        label="Mode"
        value="clean"
        options={modeOptions}
        onChange={onChange}
      />
    );

    const clean = screen.getByRole('radio', { name: 'Clean' });
    const drive = screen.getByRole('radio', { name: 'Drive' });
    clean.focus();

    await user.keyboard('{ArrowRight}');
    await user.keyboard('{ArrowDown}');
    await user.keyboard('{ArrowLeft}');
    await user.keyboard('{ArrowUp}');

    expect(onChange).not.toHaveBeenCalled();
    expect(clean).toHaveAttribute('aria-checked', 'true');
    expect(drive).toHaveAttribute('aria-checked', 'false');
  });

  it('ignores keyboard activation with Space and Enter', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <RadioGroup
        name="mode"
        label="Mode"
        value="clean"
        options={modeOptions}
        onChange={onChange}
      />
    );

    const drive = screen.getByRole('radio', { name: 'Drive' });
    drive.focus();

    await user.keyboard(' ');
    await user.keyboard('{Enter}');

    expect(onChange).not.toHaveBeenCalled();
    expect(screen.getByRole('radio', { name: 'Clean' })).toHaveAttribute('aria-checked', 'true');
    expect(drive).toHaveAttribute('aria-checked', 'false');
  });

  it('prevents interaction when group is disabled', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <RadioGroup
        name="mode"
        label="Mode"
        value="clean"
        options={modeOptions}
        disabled
        onChange={onChange}
      />
    );

    const clean = screen.getByRole('radio', { name: 'Clean' });
    const drive = screen.getByRole('radio', { name: 'Drive' });

    expect(clean).toBeDisabled();
    expect(drive).toBeDisabled();

    await user.click(drive);
    clean.focus();
    await user.keyboard('{ArrowRight}');
    await user.keyboard('{Enter}');
    await user.keyboard(' ');

    expect(onChange).not.toHaveBeenCalled();
    expect(screen.getByRole('radio', { name: 'Clean' })).toHaveAttribute('aria-checked', 'true');
    expect(screen.getByRole('radio', { name: 'Drive' })).toHaveAttribute('aria-checked', 'false');
  });

  it('keeps disabled options non-interactive on click', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();

    render(
      <RadioGroup
        name="mode"
        label="Mode"
        value="clean"
        options={[
          { value: 'clean', label: 'Clean' },
          { value: 'drive', label: 'Drive', disabled: true },
          { value: 'fuzz', label: 'Fuzz' },
        ]}
        onChange={onChange}
      />
    );

    const drive = screen.getByRole('radio', { name: 'Drive' });

    await user.click(drive);

    expect(onChange).not.toHaveBeenCalled();
    expect(screen.getByRole('radio', { name: 'Clean' })).toHaveAttribute('aria-checked', 'true');
    expect(drive).toHaveAttribute('aria-checked', 'false');
  });

  it('uses non-tabbable radio options', () => {
    render(
      <RadioGroup name="mode" label="Mode" value="clean" options={modeOptions} onChange={vi.fn()} />
    );

    expect(screen.getByRole('radio', { name: 'Clean' })).toHaveAttribute('tabindex', '-1');
    expect(screen.getByRole('radio', { name: 'Drive' })).toHaveAttribute('tabindex', '-1');
    expect(screen.getByRole('radio', { name: 'Fuzz' })).toHaveAttribute('tabindex', '-1');
  });

  it('does not forward renderOptionsAs to DOM when rendering options as Button', () => {
    render(
      <RadioGroup
        name="mode"
        label="Mode"
        value="clean"
        options={[
          {
            value: 'clean',
            label: 'Clean',
            as: Button,
            renderOptionsAs: Button,
            size: 'sm',
          },
          {
            value: 'drive',
            label: 'Drive',
            as: Button,
            renderOptionsAs: Button,
            size: 'sm',
          },
        ] as unknown as readonly RadioGroupOption<Mode>[]}
        onChange={vi.fn()}
      />
    );

    const clean = screen.getByRole('radio', { name: 'Clean' });
    expect(clean).not.toHaveAttribute('renderOptionsAs');
    expect(clean).not.toHaveAttribute('renderoptionsas');
  });
});
