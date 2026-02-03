import { Meter } from './components/Meter';
import { ParameterSlider } from './components/ParameterSlider';
import { VersionBadge } from './components/VersionBadge';
import { LatencyMonitor } from './components/LatencyMonitor';

export function App() {
  return (
    <div className="flex h-screen flex-col gap-4 bg-plugin-dark p-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-gray-100">My Plugin</h1>
        <VersionBadge />
      </div>

      {/* Main Content */}
      <div className="flex flex-1 flex-col gap-6">
        {/* Parameters Section */}
        <div className="rounded-lg border border-plugin-border bg-plugin-surface p-4">
          <h2 className="mb-3 text-base font-semibold text-gray-200">
            Parameters
          </h2>
          <div className="space-y-3">
            <ParameterSlider id="gain" />
          </div>
        </div>

        {/* Metering Section */}
        <div className="rounded-lg border border-plugin-border bg-plugin-surface p-4">
          <h2 className="mb-3 text-base font-semibold text-gray-200">
            Output Metering
          </h2>
          <Meter />
        </div>

        {/* Info Section */}
        <div className="rounded-lg border border-plugin-border bg-plugin-surface p-4">
          <h2 className="mb-3 text-base font-semibold text-gray-200">Info</h2>
          <LatencyMonitor />
        </div>
      </div>
    </div>
  );
}
