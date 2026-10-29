#!/usr/bin/env bash

set -euo pipefail

out="${1:-/tmp/opencode/visual/$(date +%Y%m%d-%H%M%S).png}"
pictures="$(xdg-user-dir PICTURES 2>/dev/null || true)"
[ -n "$pictures" ] || pictures="$HOME/Pictures"
mkdir -p "$(dirname "$out")" "$pictures"

marker="$(mktemp)"
sleep 0.1

gdbus call --session \
  --dest org.freedesktop.portal.Desktop \
  --object-path /org/freedesktop/portal/desktop \
  --method org.freedesktop.portal.Screenshot.Screenshot \
  '' "{'interactive': <false>, 'modal': <true>}" >/dev/null

shot=""
for _ in $(seq 1 50); do
  candidate="$(find "$pictures" -maxdepth 1 -name '*.png' -newer "$marker" \
    -printf '%T@ %p\n' 2>/dev/null | sort -n | tail -1 | cut -d' ' -f2-)"
  if [ -n "$candidate" ]; then
    size_before="$(stat -c %s "$candidate" 2>/dev/null || echo 0)"
    sleep 0.4
    size_after="$(stat -c %s "$candidate" 2>/dev/null || echo 0)"
    if [ "$size_after" -gt 0 ] && [ "$size_before" = "$size_after" ]; then
      shot="$candidate"
      break
    fi
  else
    sleep 0.2
  fi
done
rm -f "$marker"

if [ -z "$shot" ]; then
  echo "screenshot: the portal returned no file within 12 s" >&2
  exit 1
fi

if [ "$shot" != "$out" ]; then
  mv -f "$shot" "$out"
fi

echo "$out"
