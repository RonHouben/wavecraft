import { fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import { Button } from './Button';

describe('Button', () => {
  it('renders label and calls onClick', () => {
    const onClick = vi.fn();

    render(<Button onClick={onClick}>Bypass</Button>);

    const button = screen.getByRole('button', { name: 'Bypass' });
    fireEvent.click(button);

    expect(onClick).toHaveBeenCalledTimes(1);
  });

  it('renders loading state with busy semantics', () => {
    render(<Button state="loading">Save</Button>);

    const button = screen.getByRole('button', { name: 'Save' });
    expect(button).toHaveAttribute('aria-busy', 'true');
    expect(button).toBeDisabled();
  });

  it('renders plugin state badge', () => {
    render(<Button pluginState="mapped">Gain</Button>);

    expect(screen.getByText('MAP')).toBeInTheDocument();
  });

  it('disables interaction for disabled visual state', () => {
    render(<Button state="disabled">Bypass</Button>);

    const button = screen.getByRole('button', { name: 'Bypass' });
    expect(button).toBeDisabled();
    expect(button).toHaveAttribute('data-state', 'disabled');
  });

  it('applies error semantics when state is error', () => {
    render(<Button state="error">Save</Button>);

    const button = screen.getByRole('button', { name: 'Save' });
    expect(button).toHaveAttribute('aria-invalid', 'true');
  });

  it('renders explicit active state semantics and indicator', () => {
    render(<Button active>Mode A</Button>);

    const button = screen.getByRole('button', { name: 'Mode A' });
    expect(button).toHaveAttribute('aria-pressed', 'true');
    expect(button).toHaveAttribute('data-active', 'true');
    expect(screen.getByText('✓')).toBeInTheDocument();
  });

  it('keeps legacy pressed behavior for backwards compatibility', () => {
    render(<Button pressed>Legacy Toggle</Button>);

    const button = screen.getByRole('button', { name: 'Legacy Toggle' });
    expect(button).toHaveAttribute('aria-pressed', 'true');
    expect(button).toHaveAttribute('data-active', 'true');
  });

  it('keeps legacy isActive behavior for backwards compatibility', () => {
    render(<Button isActive>Legacy Active</Button>);

    const button = screen.getByRole('button', { name: 'Legacy Active' });
    expect(button).toHaveAttribute('aria-pressed', 'true');
    expect(button).toHaveAttribute('data-active', 'true');
  });

  it('gives explicit active prop precedence over isActive and pressed', () => {
    render(
      <Button active={false} isActive pressed>
        Explicit Off
      </Button>
    );

    const button = screen.getByRole('button', { name: 'Explicit Off' });
    expect(button).toHaveAttribute('aria-pressed', 'false');
    expect(button).toHaveAttribute('data-active', 'false');
    expect(screen.queryByText('✓')).not.toBeInTheDocument();
  });
});
