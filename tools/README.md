# `tools/` — archived call-graph extraction POC

These Python scripts are an **archived proof-of-concept**, kept for provenance, not as
maintained tooling. They extract a function/type inventory + a heuristic call-graph from the
crates and render Markdown architecture tables.

**Why they're here / why they're archived:** this regex-and-name-matching extractor is exactly
the heuristic ADR-067 §E names as *"the seed of the cost / drift / detection failure-trinity"* —
the cheap approximation the council used to motivate the decision to **compose installed
rust-analyzer** for real resolution instead. The scripts are preserved so that citation stays
grounded in something readable; the design that supersedes them lives in
`research/notebooks/007-adr-067-sovereign-stroma-builder.md`.

## Pipeline

```
extract_edges.py   →  edges.json        (FN nodes + heuristic call edges, from *.rs)
extract_nodes.py   →  nodes.json        (structs/enums/fns/traits + docs, from target/doc/*.json)
gen_l1_lines.py    →  (stdout)          (62 curated "L1" spine nodes → path:line table)
gen_l2_tables.py   →  l2_tables.md      (production nodes + call edges, grouped by crate/module)
gen_test_appendix.py → test_appendix.md (per-file test-fn counts)
```

## Outputs are NOT tracked

`edges.json`, `nodes.json`, `l2_tables.md`, `test_appendix.md` are **derived, regenerable
snapshots** (~1 MB total) — gitignored. Regenerate by running the scripts; don't commit them.
