import { useSettingsModal } from '@wavecraft/core';

import { Modal } from '../Modal';
import { Settings } from './Settings';

export type SettingsModalProps = Record<string, never>;

export function SettingsModal(_props: Readonly<SettingsModalProps>) {
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
