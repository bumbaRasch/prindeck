#!/bin/sh
set -eu

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

sources=$(find src-tauri/src ui/src scripts -type f \( \
  -name '*.rs' -o -name '*.ts' -o -name '*.mjs' -o -name '*.sh' -o -name '*.css' \
\) | sort)
sources="$sources ui/index.html"

failures=0

for path in $sources; do
  case "$path" in
    ui/src/locales/*) continue ;;
  esac

  hits=$(grep -nP '[\x{0400}-\x{04FF}]' "$path" || true)
  if [ -n "$hits" ]; then
    printf '%s\n' "$hits" | sed "s|^|$path:|"
    failures=$((failures + 1))
  fi
done

if [ "$failures" -ne 0 ]; then
  printf '\ncode must be English: UI strings live only in ui/src/locales/\n'
  exit 1
fi

echo "code language ok"
