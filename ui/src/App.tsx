/**
 * Main App component
 */

import { ParameterSlider } from './components/ParameterSlider';
import { LatencyMonitor } from './components/LatencyMonitor';
import { Meter } from './components/Meter';
import './App.css';

function App() {
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
    </div>
  );
}

export default App;
