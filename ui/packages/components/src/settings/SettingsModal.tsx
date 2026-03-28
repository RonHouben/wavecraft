import { useState } from 'react';
import { Modal } from '../Modal';
import { Settings } from './Settings';

export interface SettingsModalProps {}

export function SettingsModal(props: Readonly<SettingsModalProps>) {
  const { closeSettingsModal, isSettingsModalOpen } = useSettingsModal();

  return (
    <Modal open={isSettingsModalOpen} onClose={closeSettingsModal}>
      <Modal.Title>Plugin settings</Modal.Title>
      <Modal.Description>Configure your plugin&apos;s audio input.</Modal.Description>
      <Modal.Content className="flex flex-col gap-4">
        <Settings />
      </Modal.Content>
    </Modal>
  );
}

export function useSettingsModal() {
  const [isSettingsModalOpen, setIsSettingsModalOpen] = useState(false);

  return {
    isSettingsModalOpen,
    openSettingsModal: () => setIsSettingsModalOpen(true),
    closeSettingsModal: () => setIsSettingsModalOpen(false),
  };
}
