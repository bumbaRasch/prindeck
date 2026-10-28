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
    ui/index.html|ui/src/labels/*) continue ;;
  esac

  hits=$(grep -nP '[\x{0400}-\x{04FF}]' "$path" | grep -vE '^[0-9]+:const [A-Z0-9_]+_LABEL: &str = ' || true)
  if [ -n "$hits" ]; then
    printf '%s\n' "$hits" | sed "s|^|$path:|"
    failures=$((failures + 1))
  fi
done

if [ "$failures" -ne 0 ]; then
  printf '\ncode must be English: only ui/index.html, ui/src/labels/ and *_LABEL constants may contain UI copy\n'
  exit 1
fi

echo "code language ok"
