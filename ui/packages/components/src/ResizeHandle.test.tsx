import { fireEvent, render, screen } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { ResizeHandle } from './ResizeHandle';

const mockRequestResize = vi.hoisted(() => vi.fn());

describe('ResizeHandle', () => {
  beforeEach(() => {
    mockRequestResize.mockReset();
    mockRequestResize.mockResolvedValue(undefined);
  });

  it('renders with a visible anchored handle class', () => {
    render(<ResizeHandle onRequestResize={mockRequestResize} />);

    const handle = screen.getByTestId('resize-handle');
    expect(handle).toHaveClass('bottom-2');
    expect(handle).toHaveClass('right-2');
    expect(handle).toHaveClass('bg-black/40');
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
});
