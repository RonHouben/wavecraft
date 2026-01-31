/**
 * ResizeControls component - Demonstrates window resizing functionality
 */

import { useState } from 'react';
import { useRequestResize } from '../lib/vstkit-ipc';
import './ResizeControls.css';

const PRESET_SIZES = [
  { name: 'Small', width: 600, height: 400 },
  { name: 'Medium', width: 800, height: 600 },
  { name: 'Large', width: 1024, height: 768 },
  { name: 'Extra Large', width: 1280, height: 960 },
];

export function ResizeControls() {
  const requestResize = useRequestResize();
  const [status, setStatus] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);

  const handleResize = async (width: number, height: number) => {
    setIsLoading(true);
    setStatus(`Requesting ${width}x${height}...`);
    
    try {
      const accepted = await requestResize(width, height);
      if (accepted) {
        setStatus(`✓ Resized to ${width}x${height}`);
      } else {
        setStatus(`✗ Host rejected ${width}x${height}`);
      }
    } catch (error) {
      setStatus(`✗ Error: ${error}`);
    } finally {
      setIsLoading(false);
    }
  };

  return (
    <div className="resize-controls">
      <h3>Window Size</h3>
      <div className="resize-presets">
        {PRESET_SIZES.map((preset) => (
          <button
            key={preset.name}
            onClick={() => handleResize(preset.width, preset.height)}
            disabled={isLoading}
            className="resize-button"
          >
            {preset.name}
            <span className="size-label">
              {preset.width} × {preset.height}
            </span>
          </button>
        ))}
      </div>
      {status && (
        <div className={`resize-status ${status.startsWith('✓') ? 'success' : status.startsWith('✗') ? 'error' : ''}`}>
          {status}
        </div>
      )}
    </div>
  );
}
