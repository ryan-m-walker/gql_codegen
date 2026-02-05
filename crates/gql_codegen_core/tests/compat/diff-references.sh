#!/usr/bin/env bash
# Compare our insta snapshots against JS @graphql-codegen reference output.
# This is informational — it shows where our output differs from the JS codegen.
#
# Usage: bash diff-references.sh

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SNAPSHOTS_DIR="$SCRIPT_DIR/snapshots"
REFERENCES_DIR="$SCRIPT_DIR/references"

if [ ! -d "$REFERENCES_DIR" ] || [ -z "$(ls -A "$REFERENCES_DIR" 2>/dev/null)" ]; then
  echo "No references found. Run 'pnpm generate' first."
  exit 1
fi

matches=0
diffs=0
missing=0

for ref_file in "$REFERENCES_DIR"/**/*.ts; do
  # Extract plugin dir and test name from reference path
  rel="${ref_file#$REFERENCES_DIR/}"
  plugin_dir="$(dirname "$rel")"
  test_name="$(basename "$rel" .ts)"

  # Find matching snapshot: compat__<test_name>@<test_name>.json.snap
  snap_file="$SNAPSHOTS_DIR/compat__${test_name}@${test_name}.json.snap"

  if [ ! -f "$snap_file" ]; then
    echo "  ? $plugin_dir/$test_name — no snapshot found"
    ((missing++)) || true
    continue
  fi

  # Extract snapshot content (everything after the second --- line)
  snap_content=$(awk 'BEGIN{n=0} /^---$/{n++; next} n>=2{print}' "$snap_file")

  # Compare (ignoring trailing whitespace)
  if diff -q -B -b <(echo "$snap_content") "$ref_file" > /dev/null 2>&1; then
    echo "  ✓ $plugin_dir/$test_name — matches"
    ((matches++)) || true
  else
    echo "  ✗ $plugin_dir/$test_name — differs"
    diff -u --label "our output" --label "js reference" \
      <(echo "$snap_content") "$ref_file" | head -50
    echo ""
    ((diffs++)) || true
  fi
done

echo ""
echo "Results: $matches matching, $diffs differing, $missing missing snapshots"
