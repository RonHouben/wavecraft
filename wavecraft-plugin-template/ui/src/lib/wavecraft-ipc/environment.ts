/**
 * Environment Detection
 *
 * Determines if the code is running in WKWebView (production)
 * or a browser (development).
 */

/**
 * Check if running in a WKWebView environment (production)
 * @returns true if globalThis.__WAVECRAFT_IPC__ IPC primitives are available
 */
export function isWebViewEnvironment(): boolean {
  return globalThis.__WAVECRAFT_IPC__ !== undefined;
}

/**
 * Check if running in a browser environment (development)
 * @returns true if IPC primitives are NOT available
 */
export function isBrowserEnvironment(): boolean {
  return !isWebViewEnvironment();
}
