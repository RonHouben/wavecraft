/**
 * VersionBadge Component
 *
 * Displays the plugin version in a small, unobtrusive badge.
 * Version is injected at build time from engine/Cargo.toml.
 */

import React from 'react';

export function VersionBadge(): React.JSX.Element {
  return (
    <span data-testid="version-badge" className="text-xs text-gray-500">
      v{__APP_VERSION__}
    </span>
  );
}
