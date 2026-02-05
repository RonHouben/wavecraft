#!/usr/bin/env bash
# Check for broken markdown links in documentation
# Excludes _archive/ directory (historical records don't need fixing)

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$REPO_ROOT"

BROKEN_LINKS_FILE=$(mktemp)
trap "rm -f $BROKEN_LINKS_FILE" EXIT

echo "🔍 Checking markdown links (excluding _archive/)..."
echo ""

# Find all markdown files, excluding _archive/
find docs -name "*.md" -not -path "*/_archive/*" | while read -r file; do
    # Extract markdown links: [text](path) or [text](path#anchor)
    # Skip external links (http/https) and anchor-only links (#section)
    grep -oE '\[([^\]]+)\]\(([^)]+)\)' "$file" 2>/dev/null | while read -r link; do
        # Extract the path part (everything between parentheses)
        link_path=$(echo "$link" | sed -E 's/.*\(([^)]+)\).*/\1/')
        
        # Skip external links (http/https)
        if [[ "$link_path" =~ ^https?:// ]]; then
            continue
        fi
        
        # Skip anchor-only links (#section)
        if [[ "$link_path" =~ ^# ]]; then
            continue
        fi
        
        # Strip anchor from path if present (path#anchor -> path)
        link_file="${link_path%%#*}"
        
        # Skip empty paths
        if [[ -z "$link_file" ]]; then
            continue
        fi
        
        # Resolve relative path from the file's directory
        file_dir="$(dirname "$file")"
        
        # If link starts with /, it's relative to repo root
        if [[ "$link_file" =~ ^/ ]]; then
            target_path="${REPO_ROOT}${link_file}"
        else
            # Otherwise, relative to the file's directory
            target_path="${file_dir}/${link_file}"
        fi
        
        # Check if target exists
        if [[ ! -e "$target_path" ]]; then
            echo "❌ Broken link in $file" | tee -a "$BROKEN_LINKS_FILE"
            echo "   Link: $link_path" | tee -a "$BROKEN_LINKS_FILE"
            echo "   Resolved to: $target_path" | tee -a "$BROKEN_LINKS_FILE"
            echo "" | tee -a "$BROKEN_LINKS_FILE"
        fi
    done || true  # Continue even if grep finds no matches
done

CHECKED_FILES=$(find docs -name "*.md" -not -path "*/_archive/*" | wc -l)
CHECKED_FILES=$(echo "$CHECKED_FILES" | tr -d ' \n')

if [[ -s "$BROKEN_LINKS_FILE" ]]; then
    BROKEN_LINKS=$(grep -c "^❌" "$BROKEN_LINKS_FILE" || echo "0")
    BROKEN_LINKS=$(echo "$BROKEN_LINKS" | tr -d ' \n')
else
    BROKEN_LINKS=0
fi

echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "📊 Summary:"
echo "   Files checked: $CHECKED_FILES"
echo "   Broken links: $BROKEN_LINKS"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [[ $BROKEN_LINKS -gt 0 ]]; then
    echo ""
    echo "❌ Found $BROKEN_LINKS broken link(s). Please fix them."
    exit 1
fi

echo ""
echo "✅ All links valid!"
exit 0
