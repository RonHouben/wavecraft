#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TEMPLATE_DIR="$REPO_ROOT/sdk-template"

if [ ! -d "$TEMPLATE_DIR" ]; then
  echo "❌ Missing sdk-template/ at repository root"
  exit 1
fi

echo "🔧 Setting up sdk-template for SDK development..."

# 1) Materialize all .template files
while IFS= read -r template_file; do
  output_file="${template_file%.template}"
  cp "$template_file" "$output_file"
  echo "  • generated $(realpath --relative-to="$REPO_ROOT" "$output_file" 2>/dev/null || echo "$output_file")"
done < <(find "$TEMPLATE_DIR" -type f -name "*.template" | sort)

CURRENT_YEAR="$(date +%Y)"

# 2) Replace template variables with dev defaults in generated Cargo files
# (all 9 variables are handled, even if some are not currently present)
for generated_file in \
  "$TEMPLATE_DIR/Cargo.toml" \
  "$TEMPLATE_DIR/engine/Cargo.toml" \
  "$TEMPLATE_DIR/engine/xtask/Cargo.toml"
do
  if [ -f "$generated_file" ]; then
    sed -i.bak \
      -e 's|{{[[:space:]]*plugin_name[[:space:]]*}}|wavecraft-dev-template|g' \
      -e 's|{{[[:space:]]*plugin_name_snake[[:space:]]*}}|wavecraft_dev_template|g' \
      -e 's|{{[[:space:]]*plugin_name_pascal[[:space:]]*}}|WavecraftDevTemplate|g' \
      -e 's|{{[[:space:]]*plugin_name_title[[:space:]]*}}|Wavecraft Dev Template|g' \
      -e 's|{{[[:space:]]*author_name[[:space:]]*}}|Wavecraft SDK|g' \
      -e 's|{{[[:space:]]*author_email[[:space:]]*}}|dev@wavecraft.dev|g' \
      -e 's|{{[[:space:]]*homepage[[:space:]]*}}|https://github.com/RonHouben/wavecraft|g' \
      -e 's|{{[[:space:]]*sdk_version[[:space:]]*}}|dev|g' \
      -e "s|{{[[:space:]]*year[[:space:]]*}}|$CURRENT_YEAR|g" \
      "$generated_file"
    rm -f "${generated_file}.bak"
  fi
done

ENGINE_CARGO="$TEMPLATE_DIR/engine/Cargo.toml"
if [ ! -f "$ENGINE_CARGO" ]; then
  echo "❌ Expected generated file missing: sdk-template/engine/Cargo.toml"
  echo "   Run this script from the Wavecraft repository root or ensure sdk-template exists."
  exit 1
fi

# 3) Replace Wavecraft git dependencies with local path dependencies for SDK mode
sed -i.bak \
  -e 's|wavecraft = { package = "wavecraft-nih_plug", git = "https://github.com/RonHouben/wavecraft", tag = "[^"]*" }|wavecraft = { package = "wavecraft-nih_plug", path = "../../engine/crates/wavecraft-nih_plug" }|g' \
  -e 's|wavecraft-dsp = { git = "https://github.com/RonHouben/wavecraft", tag = "[^"]*", optional = true }|wavecraft-dsp = { path = "../../engine/crates/wavecraft-dsp", optional = true }|g' \
  -e 's|wavecraft-dev-server = { package = "wavecraft-dev-server", git = "https://github.com/RonHouben/wavecraft", tag = "[^"]*", features = \["audio"\], optional = true }|wavecraft-dev-server = { package = "wavecraft-dev-server", path = "../../dev-server", features = ["audio"], optional = true }|g' \
  "$ENGINE_CARGO"
rm -f "$ENGINE_CARGO.bak"

required_path_entries=(
  'wavecraft = { package = "wavecraft-nih_plug", path = "../../engine/crates/wavecraft-nih_plug" }'
  'wavecraft-dsp = { path = "../../engine/crates/wavecraft-dsp", optional = true }'
  'wavecraft-dev-server = { package = "wavecraft-dev-server", path = "../../dev-server", features = ["audio"], optional = true }'
)

for required_entry in "${required_path_entries[@]}"; do
  if ! grep -Fq "$required_entry" "$ENGINE_CARGO"; then
    echo "❌ Failed to rewrite SDK git dependency to path dependency in sdk-template/engine/Cargo.toml"
    echo "   Missing expected entry: $required_entry"
    echo "   This likely means sdk-template/engine/Cargo.toml.template format drifted from setup script expectations."
    echo "   Update scripts/setup-dev-template.sh rewrite patterns to match current template formatting."
    exit 1
  fi
done

# 4) Inject tsconfig paths for SDK development mode
TSCONFIG="$TEMPLATE_DIR/ui/tsconfig.json"
if [ -f "$TSCONFIG" ]; then
  if ! grep -Fq '"@wavecraft/core": ["../../ui/packages/core/src/index.ts"]' "$TSCONFIG"; then
    echo "🔧 Injecting tsconfig paths for SDK development mode..."
    sed -i.bak \
      's/"noFallthroughCasesInSwitch": true/"noFallthroughCasesInSwitch": true,\n\n    \/* SDK development — resolve @wavecraft packages from monorepo source *\/\n    "baseUrl": ".",\n    "paths": {\n      "@wavecraft\/core": ["..\/..\/ui\/packages\/core\/src\/index.ts"],\n      "@wavecraft\/core\/*": ["..\/..\/ui\/packages\/core\/src\/*"],\n      "@wavecraft\/components": ["..\/..\/ui\/packages\/components\/src\/index.ts"],\n      "@wavecraft\/components\/*": ["..\/..\/ui\/packages\/components\/src\/*"]\n    }/' \
      "$TSCONFIG"
    rm -f "$TSCONFIG.bak"
  fi

  if ! grep -Fq '"@wavecraft/core": ["../../ui/packages/core/src/index.ts"]' "$TSCONFIG"; then
    echo "❌ Failed to inject tsconfig paths for SDK development mode"
    echo "   Missing expected @wavecraft/core paths entry in sdk-template/ui/tsconfig.json"
    exit 1
  fi
fi

# 5) Install UI dependencies for sdk-template project
if [ -d "$TEMPLATE_DIR/ui" ]; then
  echo "📦 Installing sdk-template UI dependencies..."
  (
    cd "$TEMPLATE_DIR/ui"
    npm install
  )
else
  echo "❌ Missing sdk-template/ui directory"
  exit 1
fi

echo "✅ sdk-template setup complete"
