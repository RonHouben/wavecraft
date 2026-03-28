import { act, fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';
import {
  Modal,
  ModalContent,
  ModalDescription,
  ModalFooter,
  ModalHeader,
  ModalTitle,
} from './Modal';

describe('Modal', () => {
  it('does not render when closed', () => {
    render(
      <Modal open={false} onClose={vi.fn()}>
        <Modal.Content>Hidden content</Modal.Content>
      </Modal>
    );

    expect(screen.queryByRole('dialog')).not.toBeInTheDocument();
  });

  it('supports compound component API with static subcomponents', () => {
    render(
      <Modal open onClose={vi.fn()}>
        <Modal.Header>
          <Modal.Title>Delete preset</Modal.Title>
          <Modal.Description>This action cannot be undone.</Modal.Description>
        </Modal.Header>
        <Modal.Content>Preset deletion will remove saved settings.</Modal.Content>
        <Modal.Footer>
          <button type="button">Cancel</button>
        </Modal.Footer>
      </Modal>
    );

    const dialog = screen.getByRole('dialog', { name: 'Delete preset' });

    expect(dialog).toBeInTheDocument();
    expect(dialog).toHaveAccessibleDescription('This action cannot be undone.');
    expect(screen.getByText('Preset deletion will remove saved settings.')).toBeInTheDocument();
    expect(screen.getByRole('button', { name: 'Cancel' })).toBeInTheDocument();
  });

  it('renders an accessible dialog and focuses it when opened', () => {
    render(
      <Modal open onClose={vi.fn()}>
        <Modal.Header>
          <Modal.Title>Signal chain options</Modal.Title>
          <Modal.Description>Choose how to continue editing the chain.</Modal.Description>
        </Modal.Header>
        <Modal.Content>
          <p>Body</p>
        </Modal.Content>
        <Modal.Footer>
          <button type="button">Done</button>
        </Modal.Footer>
      </Modal>
    );

    const dialog = screen.getByRole('dialog', { name: 'Signal chain options' });

    expect(dialog).toHaveFocus();
    expect(screen.getByRole('button', { name: 'Close modal' })).toBeInTheDocument();
    expect(screen.getByText('Choose how to continue editing the chain.')).toBeInTheDocument();
  });

  it('calls onClose when the backdrop is clicked or Escape is pressed', () => {
    const onClose = vi.fn();

    render(
      <Modal open onClose={onClose}>
        <Modal.Header>
          <Modal.Title>Close me</Modal.Title>
        </Modal.Header>
        <Modal.Content>Body</Modal.Content>
      </Modal>
    );

    fireEvent.click(screen.getByRole('button', { name: 'Dismiss modal' }));
    fireEvent.keyDown(globalThis.window, { key: 'Escape' });

    expect(onClose).toHaveBeenCalledTimes(2);
  });

  it('cycles focus within the modal when tabbing', () => {
    render(
      <Modal open onClose={vi.fn()}>
        <Modal.Header>
          <Modal.Title>Keyboard flow</Modal.Title>
        </Modal.Header>
        <Modal.Content>
          <button type="button">Secondary action</button>
        </Modal.Content>
        <Modal.Footer>
          <button type="button">Cancel</button>
          <button type="button">Confirm</button>
        </Modal.Footer>
      </Modal>
    );

    const dialog = screen.getByRole('dialog', { name: 'Keyboard flow' });
    const closeButton = screen.getByRole('button', { name: 'Close modal' });
    const confirmButton = screen.getByRole('button', { name: 'Confirm' });

    expect(dialog).toHaveFocus();

    fireEvent.keyDown(globalThis.window, { key: 'Tab' });
    expect(closeButton).toHaveFocus();

    fireEvent.keyDown(globalThis.window, { key: 'Tab', shiftKey: true });
    expect(confirmButton).toHaveFocus();
  });

  it('supports custom className overrides on the root and subcomponents', () => {
    render(
      <Modal open onClose={vi.fn()} className="custom-modal" data-testid="modal-panel">
        <ModalHeader className="custom-header" data-testid="header">
          <ModalTitle className="custom-title">Title</ModalTitle>
          <ModalDescription className="custom-description">Description</ModalDescription>
        </ModalHeader>
        <ModalContent className="custom-content" data-testid="content">
          Content
        </ModalContent>
        <ModalFooter className="custom-footer" data-testid="footer">
          Footer
        </ModalFooter>
      </Modal>
    );

    expect(screen.getByTestId('modal-panel')).toHaveClass('custom-modal');
    expect(screen.getByTestId('header')).toHaveClass('custom-header');
    expect(screen.getByText('Title')).toHaveClass('custom-title');
    expect(screen.getByText('Description')).toHaveClass('custom-description');
    expect(screen.getByTestId('content')).toHaveClass('custom-content');
    expect(screen.getByTestId('footer')).toHaveClass('custom-footer');
  });

  it('keeps the modal mounted briefly for exit animation before unmounting', () => {
    vi.useFakeTimers();

    const { rerender } = render(
      <Modal open onClose={vi.fn()}>
        <Modal.Header>
          <Modal.Title>Animated modal</Modal.Title>
        </Modal.Header>
        <Modal.Content>Content</Modal.Content>
      </Modal>
    );

    rerender(
      <Modal open={false} onClose={vi.fn()}>
        <Modal.Header>
          <Modal.Title>Animated modal</Modal.Title>
        </Modal.Header>
        <Modal.Content>Content</Modal.Content>
      </Modal>
    );

    expect(screen.getByRole('dialog', { name: 'Animated modal' })).toHaveClass('scale-95');

    act(() => {
      vi.advanceTimersByTime(151);
    });

    expect(screen.queryByRole('dialog', { name: 'Animated modal' })).not.toBeInTheDocument();
    vi.useRealTimers();
  });
});
