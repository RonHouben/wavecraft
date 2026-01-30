/**
 * Main App component
 */

import { ParameterSlider } from './components/ParameterSlider';
import { LatencyMonitor } from './components/LatencyMonitor';
import { Meter } from './components/Meter';
import { ResizeControls } from './components/ResizeControls';
import './App.css';
import { useEffect } from 'react';
import { requestResize } from './lib/vstkit-ipc';

function App() {
  // Handle native window resize
  useEffect(() => {
    const handleResize = () => {
      const width = window.innerWidth;
      const height = window.innerHeight;
      
      // Notify host of the new size
      requestResize(width, height).catch((err) => {
        console.error('Failed to notify host of resize:', err);
      });
    };

    // Listen for window resize events
    window.addEventListener('resize', handleResize);

    return () => {
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

        <section className="resize-section">
          <h2>Window Resize</h2>
          <ResizeControls />
        </section>
      </main>

      <footer className="app-footer">
        <p>VstKit Audio Plugin | React + WKWebView</p>
      </footer>
    </div>
  );
}

export default App;
