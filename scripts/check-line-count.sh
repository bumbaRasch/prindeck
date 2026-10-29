#!/usr/bin/env bash
set -u
cd "$(dirname "$0")/.."
limit=200
baseline=scripts/line-count-baseline.txt
status=0
while IFS= read -r file; do
	case "$file" in
	src-tauri/src/*|ui/*) ;;
	*) continue ;;
	esac
	case "$file" in
	*.rs|*.ts|*.tsx|*.js|*.mjs|*.css|*.html) ;;
	*) continue ;;
	esac
	lines=$(wc -l <"$file")
	[ "$lines" -le "$limit" ] && continue
	if [ -f "$baseline" ] && grep -Fqx -- "$file" "$baseline"; then continue; fi
	printf 'over %d lines: %s (%d)\n' "$limit" "$file" "$lines"
	status=1
done < <(git ls-files)
exit "$status"
