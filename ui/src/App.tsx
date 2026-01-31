/**
 * Main App component
 */

import React, { useEffect } from 'react';
import { ParameterSlider } from './components/ParameterSlider';
import { LatencyMonitor } from './components/LatencyMonitor';
import { Meter } from './components/Meter';
import { ResizeHandle } from './components/ResizeHandle';
import { requestResize } from './lib/vstkit-ipc';

function App(): React.JSX.Element {
  // Handle native window resize
  useEffect(() => {
    const handleResize = (): void => {
      const width = window.innerWidth;
      const height = window.innerHeight;

      // Notify host of the new size
      requestResize(width, height).catch((err) => {
        console.error('Failed to notify host of resize:', err);
      });
    };

    // Listen for window resize events
    window.addEventListener('resize', handleResize);

    return (): void => {
      window.removeEventListener('resize', handleResize);
    };
  }, []);

  return (
    <div className="flex min-h-full flex-col">
      <header className="border-b-2 border-plugin-border bg-gradient-to-br from-plugin-surface to-plugin-dark p-8 text-center">
        <h1 className="mb-2 bg-gradient-to-r from-accent to-accent-light bg-clip-text text-2xl text-transparent">
          VstKit — Plugin UI
        </h1>
        <p className="text-sm text-gray-500">WebView ↔ Rust IPC Demo</p>
      </header>

      <main className="mx-auto w-full max-w-3xl flex-1 p-8">
        <section className="mb-8">
          <h2 className="mb-4 border-b-2 border-plugin-border pb-2 text-xl text-gray-200">
            Parameters
          </h2>
          <ParameterSlider id="gain" />
        </section>

        <section className="mb-8">
          <h2 className="mb-4 border-b-2 border-plugin-border pb-2 text-xl text-gray-200">
            Meters
          </h2>
          <Meter />
        </section>

        <section className="mb-8">
          <h2 className="mb-4 border-b-2 border-plugin-border pb-2 text-xl text-gray-200">
            Diagnostics
          </h2>
          <LatencyMonitor />
        </section>
      </main>

      <footer className="border-t border-plugin-border bg-plugin-surface p-4 text-center text-sm text-gray-500">
        <p>VstKit Audio Plugin | React + WKWebView</p>
      </footer>

      <ResizeHandle />
    </div>
  );
}

export default App;
