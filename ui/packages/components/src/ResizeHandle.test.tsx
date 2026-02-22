import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { ResizeHandle } from './ResizeHandle';

const mockRequestResize = vi.hoisted(() => vi.fn());

describe('ResizeHandle', () => {
  beforeEach(() => {
    mockRequestResize.mockReset();
    mockRequestResize.mockResolvedValue(undefined);

    Object.defineProperty(window, 'innerWidth', {
      configurable: true,
      writable: true,
      value: 1000,
    });
    Object.defineProperty(window, 'innerHeight', {
      configurable: true,
      writable: true,
      value: 700,
    });
  });

  it('renders with a visible anchored handle class', () => {
    render(<ResizeHandle onRequestResize={mockRequestResize} />);

    const handle = screen.getByTestId('resize-handle');
    expect(handle).toHaveClass('bottom-2');
    expect(handle).toHaveClass('right-2');
    expect(handle).toHaveClass('bg-plugin-surface');
    expect(handle).toHaveClass('focus-visible:ring-2');
    expect(handle).toHaveClass('focus-visible:ring-accent-light');
  });

  it('requests resize while dragging', () => {
    render(<ResizeHandle onRequestResize={mockRequestResize} />);
    const handle = screen.getByTestId('resize-handle');

    fireEvent.mouseDown(handle, { clientX: 100, clientY: 100 });
    fireEvent.mouseMove(document, { clientX: 150, clientY: 120 });
    fireEvent.mouseUp(document);

    expect(mockRequestResize).toHaveBeenCalledTimes(1);

    const [width, height] = mockRequestResize.mock.calls[0] as [number, number];
    expect(width).toBeGreaterThanOrEqual(400);
    expect(height).toBeGreaterThanOrEqual(300);
  });

  it('supports keyboard resize with arrow keys', () => {
    render(<ResizeHandle onRequestResize={mockRequestResize} />);
    const handle = screen.getByTestId('resize-handle');

    fireEvent.keyDown(handle, { key: 'ArrowRight' });
    fireEvent.keyDown(handle, { key: 'ArrowDown', shiftKey: true });

    expect(mockRequestResize).toHaveBeenNthCalledWith(1, 1024, 700);
    expect(mockRequestResize).toHaveBeenNthCalledWith(2, 1000, 764);
  });
});
