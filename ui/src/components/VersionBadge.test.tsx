/**
 * VersionBadge Component Tests
 */

import { describe, it, expect } from 'vitest';
import { render, screen } from '@testing-library/react';
import { VersionBadge } from './VersionBadge';

describe('VersionBadge', () => {
  it('renders without crashing', () => {
    render(<VersionBadge />);
    const badge = screen.getByText(/^v/);
    expect(badge).toBeInTheDocument();
  });

  it('displays version in correct format', () => {
    render(<VersionBadge />);
    const badge = screen.getByText(/^v/);

    // Should start with 'v' followed by version string
    expect(badge.textContent).toMatch(/^v.+/);
  });

  it('applies correct styling classes', () => {
    render(<VersionBadge />);
    const badge = screen.getByText(/^v/);

    // Check for expected Tailwind classes
    expect(badge).toHaveClass('text-xs', 'text-gray-500');
  });
});
