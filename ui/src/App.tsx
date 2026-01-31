/**
 * Main App component
 */

import React, { useEffect } from 'react';
import { ParameterSlider } from './components/ParameterSlider';
import { LatencyMonitor } from './components/LatencyMonitor';
import { Meter } from './components/Meter';
import { ResizeHandle } from './components/ResizeHandle';
import './App.css';
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
    <div className="app">
      <header className="app-header">
        <h1>VstKit — Plugin UI</h1>
        <p>WebView ↔ Rust IPC Demo</p>
      </header>

      <main className="app-main">
        <section className="parameters">
          <h2>Parameters</h2>
          <ParameterSlider id="gain" />
        </section>

        <section className="meters">
          <h2>Meters</h2>
          <Meter />
        </section>

        <section className="diagnostics">
          <h2>Diagnostics</h2>
          <LatencyMonitor />
        </section>
      </main>

      <footer className="app-footer">
        <p>VstKit Audio Plugin | React + WKWebView</p>
      </footer>

      <ResizeHandle />
    </div>
  );
}

export default App;
