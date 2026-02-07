/**
 * VersionBadge Component
 *
 * Displays the plugin version in a clearly visible badge.
 * Version is injected at build time from engine/Cargo.toml.
 */

import React from 'react';

export function VersionBadge(): React.JSX.Element {
  return (
    <span data-testid="version-badge" className="text-sm font-medium text-accent">
      v{__APP_VERSION__}
    </span>
  );
}
