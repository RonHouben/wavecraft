/**
 * Main App component
 */

import { ParameterSlider } from './components/ParameterSlider';
import { ParameterToggle } from './components/ParameterToggle';
import { LatencyMonitor } from './components/LatencyMonitor';
import './App.css';

function App() {
  return (
    <div className="app">
      <header className="app-header">
        <h1>VstKit Desktop POC</h1>
        <p>WebView ↔ Rust IPC Demo</p>
      </header>

      <main className="app-main">
        <section className="parameters">
          <h2>Parameters</h2>
          <ParameterSlider id="gain" />
          <ParameterSlider id="mix" />
          <ParameterToggle id="bypass" />
        </section>

        <section className="diagnostics">
          <h2>Diagnostics</h2>
          <LatencyMonitor />
        </section>
      </main>

      <footer className="app-footer">
        <p>Built with React + TypeScript + Vite | Powered by wry WebView</p>
      </footer>
    </div>
  );
}

export default App;
