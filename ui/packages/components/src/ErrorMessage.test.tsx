import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { ErrorMessage } from './ErrorMessage';

describe('ErrorMessage', () => {
  it('renders the provided error message', () => {
    render(<ErrorMessage message="Failed to load processor" />);

    expect(screen.getByText('Failed to load processor')).toBeInTheDocument();
  });

  it('uses alert live-region semantics for accessibility', () => {
    render(<ErrorMessage message="Connection lost" />);

    const alert = screen.getByRole('alert');
    expect(alert).toHaveAttribute('aria-live', 'assertive');
    expect(alert).toHaveAttribute('aria-atomic', 'true');
  });

  it('uses existing error tokens for styling', () => {
    render(<ErrorMessage message="Invalid parameter value" />);

    const alert = screen.getByRole('alert');
    expect(alert).toHaveClass(
      'border-meter-clip',
      'bg-meter-clip/10',
      'text-meter-clip',
      'text-type-sm'
    );
  });
});
