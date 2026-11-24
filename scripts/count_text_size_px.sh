#!/usr/bin/env bash
# Count remaining literal `text_size(px(...))` calls in the Rust workspace.
#
# Scope and exclusions are fixed here so the number is reproducible:
#   * only crates/**/*.rs
#   * skip files under tests/ or benches/, and files named test_*.rs / tests.rs
#   * skip lines carrying the `// dense-data` marker (known dense data grids)
#
# Usage:
#   scripts/count_text_size_px.sh            # print the current count
#   scripts/count_text_size_px.sh --check    # compare with the recorded baseline
#
# This is a migration tracker only; no CI job runs it.
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
cd "$root"

# A search with no matches is a valid empty stream (including the final zero).
# Preserve real grep errors instead of hiding them behind `|| true`.
grep_or_empty() {
  grep "$@" || {
    status=$?
    if [[ "$status" -ne 1 ]]; then
      return "$status"
    fi
  }
}

count=$(grep_or_empty -rn --include='*.rs' -E 'text_size\(px\(' crates \
  | grep_or_empty -vE '/(tests|benches)/' \
  | grep_or_empty -vE '^(.*/)?(test_[^/]*|tests)\.rs:[0-9]+:' \
  | grep_or_empty -v '// dense-data' \
  | sed -E 's|^[^:]+:[0-9]+:||' \
  | grep_or_empty -o 'text_size(px(' \
  | wc -l \
  | tr -d '[:space:]')

baseline_file="scripts/text_size_px_baseline.txt"

if [[ "${1:-}" == "--check" ]]; then
  recorded=$(sed -n 's/^baseline=\([0-9][0-9]*\)$/\1/p' "$baseline_file")
  if [[ -z "$recorded" ]]; then
    echo "No baseline= line in $baseline_file" >&2
    exit 2
  fi
  echo "current=$count baseline=$recorded"
  if [[ "$count" != "$recorded" ]]; then
    echo "text_size(px(...)) drifted from the baseline; update the migration tracker." >&2
    exit 1
  fi
else
  echo "$count"
fi
