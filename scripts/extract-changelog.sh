#!/usr/bin/env bash
# Extract the Keep a Changelog section for VERSION from FILE (default CHANGELOG.md).
# Usage: extract-changelog.sh VERSION [FILE]
set -euo pipefail

VERSION="${1:?usage: extract-changelog.sh VERSION [FILE]}"
FILE="${2:-CHANGELOG.md}"

if [[ ! -f "$FILE" ]]; then
  echo "Changelog file not found: $FILE" >&2
  exit 1
fi

awk -v ver="$VERSION" '
  /^## \[/ {
    if (found) exit
    if ($0 ~ "^## \\[" ver "\\]") found = 1
    next
  }
  found { print }
' "$FILE" | sed -e :a -e '/^\n*$/{$d;N;ba' -e '}' | sed '/^---$/d' | sed -e :a -e '/^\n*$/{$d;N;ba' -e '}'
