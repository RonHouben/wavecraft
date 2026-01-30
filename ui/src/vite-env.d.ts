/// <reference types="vite/client" />

/**
 * Additional type definitions for VstKit
 */

interface Window {
  ipc?: {
    postMessage(message: string): void;
  };
}

