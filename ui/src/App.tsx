/**
 * Main App component
 */

import React, { useEffect } from 'react';
import { ParameterSlider } from './components/ParameterSlider';
import { LatencyMonitor } from './components/LatencyMonitor';
import { Meter } from './components/Meter';
import { ResizeHandle } from './components/ResizeHandle';
import { VersionBadge } from './components/VersionBadge';
import { ConnectionStatus } from './components/ConnectionStatus';
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
    <div data-testid="app-root" className="flex min-h-full flex-col bg-plugin-dark">
      <header className="border-b-2 border-plugin-border bg-gradient-to-br from-plugin-surface to-plugin-dark p-8">
        <div className="mx-auto flex max-w-3xl items-center justify-between">
          <div className="flex-1 text-center">
            <h1 className="mb-2 bg-gradient-to-r from-accent to-accent-light bg-clip-text text-2xl text-transparent">
              VstKit — Plugin UI Test
            </h1>
            <p className="text-sm text-gray-400">WebView ↔ Rust IPC Demo</p>
          </div>
          <ConnectionStatus />
        </div>
      </header>

      <main className="mx-auto w-full max-w-3xl flex-1 p-8">
        <section className="mb-8">
          <h2 className="mb-4 border-b-2 border-plugin-border pb-2 text-xl text-gray-100">
            Parameters
          </h2>
          <ParameterSlider id="gain" />
        </section>

        <section className="mb-8">
          <h2 className="mb-4 border-b-2 border-plugin-border pb-2 text-xl text-gray-100">
            Meters
          </h2>
          <Meter />
        </section>

        <section className="mb-8">
          <h2 className="mb-4 border-b-2 border-plugin-border pb-2 text-xl text-gray-100">
            Diagnostics
          </h2>
          <LatencyMonitor />
        </section>
      </main>

      <footer className="border-t border-plugin-border bg-plugin-surface p-4 text-center text-sm text-gray-400">
        <p>
          VstKit Audio Plugin <VersionBadge /> | React + WKWebView
        </p>
      </footer>

      <ResizeHandle />
    </div>
  );
}

export default App;
