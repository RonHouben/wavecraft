/// <reference types="vite/client" />

/**
 * Additional type definitions for Wavecraft
 */

interface Window {
  ipc?: {
    postMessage(message: string): void;
  };
}

/**
 * Build-time constant injected by Vite define.
 * Contains the plugin version from engine/Cargo.toml.
 */
declare const __APP_VERSION__: string;
