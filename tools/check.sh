#!/usr/bin/env bash
# The checks of docs/development/code-rules.md ("The checks"), in that order, stopping at the first
# failure. Run from any folder: the repository root is taken from this script's own path.
#
#   bash tools/check.sh              format, clippy, tests, size, lint, front-end check and tests
#   bash tools/check.sh --coverage   the same, then the coverage compare
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

COVERAGE=0
for arg in "$@"; do
  case "$arg" in
    --coverage) COVERAGE=1 ;;
    *) echo "usage: tools/check.sh [--coverage]" >&2; exit 2 ;;
  esac
done

step() { printf '\n== %s\n' "$*"; }

step "cargo fmt --all --check"
cargo fmt --all --check

# The parameter limit is a plain deny; the cognitive-complexity warning is counted by
# tools/check-lint.mjs, not by `-D warnings`.
step "cargo clippy --workspace --all-targets -- -D warnings"
cargo clippy --workspace --all-targets -- \
  -D warnings -D clippy::too_many_arguments -A clippy::cognitive_complexity

step "cargo test --workspace"
cargo test --workspace

step "node tools/check-size.mjs"
node tools/check-size.mjs

step "node tools/check-lint.mjs"
node tools/check-lint.mjs

step "node tools/check-comments.mjs"
node tools/check-comments.mjs

step "cd app && npm run check"
(cd app && npm run check)

step "cd app && npm test"
(cd app && npm test)

if [ "$COVERAGE" = 1 ]; then
  step "node tools/check-coverage.mjs"
  node tools/check-coverage.mjs
fi

printf '\nall checks passed\n'
