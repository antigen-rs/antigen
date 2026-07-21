#!/usr/bin/env bash
# Recursion-web scoreboard: call-graph feedback structure across every indexed corpus.
#
# Runs the dep-free `hodge_scc` example on antigen's own index plus every `R:/corpora/*/index.scip`,
# extracts (nodes, feedback %, largest-SCC), and prints one sorted table. This is the reproducible form of
# the near-DAG / recursion-web investigation: near-DAG is NOT universal, it is grammar-complexity-driven —
# the largest SCC tracks the domain's structural recursion (lisp 15 < rhai 24 << syn 299). One command so
# adding a corpus is `git clone && rust-analyzer scip . && this`.
#
# Usage: scripts/feedback-scoreboard.sh
set -uo pipefail
cd "$(dirname "$0")/.."

row() {
  local name="$1" scip="$2"
  [ -f "$scip" ] || { printf "%-18s %8s\n" "$name" "NO INDEX"; return; }
  local o n fb lg
  o=$(cargo run -q -p antigen-stroma --example hodge_scc -- "$scip" 2>/dev/null)
  n=$(printf '%s\n' "$o"  | sed -n 's/.*nodes: \([0-9]*\).*/\1/p' | head -1)
  fb=$(printf '%s\n' "$o" | sed -n 's/.*feedback SCC: [0-9]* *(\([0-9.]*\)%.*/\1/p')
  lg=$(printf '%s\n' "$o" | sed -n 's/.*largest SCC: \([0-9]*\) nodes *(\([0-9.]*\)%.*/\1 (\2%)/p')
  printf "%-18s %8s %9s%% %16s\n" "$name" "${n:-?}" "${fb:-?}" "${lg:-?}"
}

printf "%-18s %8s %10s %16s\n" "corpus" "nodes" "feedback%" "largest-SCC"
printf -- "------------------------------------------------------------\n"
row "antigen" "target/antigen.scip"
for d in R:/corpora/*/; do
  [ -f "$d/index.scip" ] || continue
  row "$(basename "$d")" "$d/index.scip"
done
