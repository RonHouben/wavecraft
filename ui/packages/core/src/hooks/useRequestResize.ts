/**
 * useRequestResize - Hook for requesting window resize
 */

import { requestResize } from './useWindowResizeSync';

/**
 * React hook for requesting window resize
 *
 * @returns Function to request resize
 *
 * @example
 * ```tsx
 * function MyComponent() {
 *   const resize = useRequestResize();
 *
 *   const handleExpand = async () => {
 *     const accepted = await resize(1200, 900);
 *     if (!accepted) {
 *       alert('Host rejected resize request');
 *     }
 *   };
 *
 *   return <button onClick={handleExpand}>Expand</button>;
 * }
 * ```
 */
export function useRequestResize(): (width: number, height: number) => Promise<boolean> {
  return requestResize;
}
