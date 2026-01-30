/**
 * ParameterToggle - Toggle control for boolean parameters
 */

import { useParameter } from '@vstkit/ipc';
import './ParameterToggle.css';

interface ParameterToggleProps {
  id: string;
}

export function ParameterToggle({ id }: ParameterToggleProps) {
  const { param, setValue, isLoading, error } = useParameter(id);

  if (isLoading) {
    return <div className="parameter-toggle loading">Loading {id}...</div>;
  }

  if (error || !param) {
    return (
      <div className="parameter-toggle error">
        Error: {error?.message || 'Parameter not found'}
      </div>
    );
  }

  const isOn = param.value >= 0.5;

  const handleToggle = () => {
    const newValue = isOn ? 0.0 : 1.0;
    setValue(newValue).catch((err) => {
      console.error('Failed to set parameter:', err);
    });
  };

  return (
    <div className="parameter-toggle">
      <label htmlFor={`toggle-${id}`}>{param.name}</label>
      <button
        id={`toggle-${id}`}
        className={`toggle-button ${isOn ? 'on' : 'off'}`}
        onClick={handleToggle}
        aria-pressed={isOn}
      >
        <span className="toggle-indicator"></span>
      </button>
    </div>
  );
}
