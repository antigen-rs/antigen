#!/usr/bin/env bash
# Run the hold-all node-identity measurement on one indexed corpus.
#
# Usage:  scripts/holdall-corpus.sh <corpus-dir>
#   <corpus-dir> must contain an `index.scip` (produce it with `rust-analyzer scip .`, or
#   scripts/scale-probe.ps1 for an instrumented run).
#
# Finds the corpus's `src` directories and points the syn feeder at them alongside its SCIP index.
# The example's own COHERENCE GUARD aborts if the two describe different corpora (<2% edge overlap) —
# so a wrong src-dir guess fails loudly rather than printing an inflated "external" figure. This script
# exists so a cross-corpus run is one command, not a rewrite: the standing multi-corpus rule only sticks
# if obeying it is cheaper than skipping it.
set -euo pipefail

repo="${1:?usage: holdall-corpus.sh <corpus-dir>}"
scip="$repo/index.scip"
[ -f "$scip" ] || { echo "no index.scip in $repo — index it first (rust-analyzer scip .)"; exit 1; }

# All src/ dirs except build output; the guard catches over/under-inclusion.
mapfile -t srcs < <(find "$repo" -type d -name src 2>/dev/null | grep -v '/target/')
[ "${#srcs[@]}" -gt 0 ] || { echo "no src/ dirs found under $repo"; exit 1; }

echo "== $(basename "$repo") =="
( cd "$(dirname "$0")/.." && cargo run -q -p antigen-stroma --example holdall_nodes -- "$scip" "${srcs[@]}" )
