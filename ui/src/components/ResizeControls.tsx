/**
 * ResizeControls component - Demonstrates window resizing functionality
 */

import React, { useState } from 'react';
import { useRequestResize } from '../lib/wavecraft-ipc';

const PRESET_SIZES = [
  { name: 'Small', width: 600, height: 400 },
  { name: 'Medium', width: 800, height: 600 },
  { name: 'Large', width: 1024, height: 768 },
  { name: 'Extra Large', width: 1280, height: 960 },
];

export function ResizeControls(): React.JSX.Element {
  const requestResize = useRequestResize();
  const [status, setStatus] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);

  const handleResize = async (width: number, height: number): Promise<void> => {
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
    <div className="rounded-lg bg-black/5 p-5">
      <h3 className="m-0 mb-4 text-sm font-semibold uppercase tracking-wide text-black/70">
        Window Size
      </h3>
      <div className="mb-4 grid grid-cols-2 gap-2.5">
        {PRESET_SIZES.map((preset) => (
          <button
            key={preset.name}
            onClick={() => handleResize(preset.width, preset.height)}
            disabled={isLoading}
            className="flex cursor-pointer flex-col items-center justify-center rounded-md border border-gray-300 bg-white p-3 font-medium text-gray-800 transition-all duration-200 hover:-translate-y-px hover:border-blue-500 hover:bg-gray-100 hover:shadow-md disabled:cursor-not-allowed disabled:opacity-50"
          >
            {preset.name}
            <span className="mt-1 text-[11px] text-gray-500">
              {preset.width} × {preset.height}
            </span>
          </button>
        ))}
      </div>
      {status && (
        <div
          className={`rounded px-3 py-2 text-center text-sm ${((): string => {
            if (status.startsWith('✓')) return 'bg-green-500/10 text-green-600';
            if (status.startsWith('✗')) return 'bg-red-500/10 text-red-500';
            return 'bg-black/5 text-gray-500';
          })()}`}
        >
          {status}
        </div>
      )}
    </div>
  );
}
