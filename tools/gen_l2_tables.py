"""Generate L2 node + edge markdown tables from tools/edges.json, grouped by crate/module."""
import json, io
from collections import defaultdict

d = json.load(io.open('tools/edges.json', encoding='utf-8'))
fns = {f['id']: f for f in d['fns']}
edges = d['edges']

def istest(f):
    return '::tests::' in f['id'] or f['module'].endswith('::tests') \
        or '::test_' in f['id'] or f['module'].endswith('::test')

prod_ids = set(fid for fid, f in fns.items() if not istest(f))

# group prod fns by crate then module-file
by_crate_mod = defaultdict(lambda: defaultdict(list))
for fid in prod_ids:
    f = fns[fid]
    by_crate_mod[f['crate']][f['file']].append(f)

# out-edges per fn (prod->prod only), as short names
out_edges = defaultdict(list)
for s, t in edges:
    if s in prod_ids and t in prod_ids:
        out_edges[s].append(t)

def short(fid):
    # drop crate prefix for readability inside tables
    return fid

CRATE_ORDER = ['antigen', 'antigen-attestation', 'antigen-fingerprint', 'antigen-macros', 'cargo-antigen']

lines = []
node_count = 0
edge_count = 0
for crate in CRATE_ORDER:
    mods = by_crate_mod.get(crate, {})
    if not mods:
        continue
    lines.append(f"\n### L2 · `{crate}`\n")
    for modfile in sorted(mods.keys()):
        fl = sorted(mods[modfile], key=lambda x: (x['line'] or 0))
        lines.append(f"\n#### `{modfile}`\n")
        lines.append("| id | kind | line | impl |")
        lines.append("|---|---|---|---|")
        for f in fl:
            node_count += 1
            impl = f['impl'] or ''
            lines.append(f"| `{f['id']}` | fn | {f['line']} | {impl} |")

# edge table (prod->prod), grouped by crate of source
lines.append("\n\n## L2 Call Edges (production functions)\n")
lines.append("`src → dst` (relation: calls). Test-function edges omitted; see Coverage note.\n")
for crate in CRATE_ORDER:
    crate_edges = sorted((s, t) for s in out_edges for t in out_edges[s]
                         if False)  # placeholder
# rebuild edge listing properly
edge_rows = []
for s in sorted(out_edges):
    if fns[s]['crate'] not in CRATE_ORDER:
        continue
    for t in sorted(set(out_edges[s])):
        edge_rows.append((s, t))
edge_count = len(edge_rows)

for crate in CRATE_ORDER:
    rows = [(s, t) for (s, t) in edge_rows if fns[s]['crate'] == crate]
    if not rows:
        continue
    lines.append(f"\n### edges · `{crate}`\n")
    lines.append("| src_id | → | dst_id |")
    lines.append("|---|---|---|")
    for s, t in rows:
        lines.append(f"| `{s}` | calls | `{t}` |")

open('tools/l2_tables.md', 'w', encoding='utf-8').write('\n'.join(lines))
print('prod nodes:', node_count)
print('prod call edges:', edge_count)
# test counts for honesty note
test_ids = [fid for fid in fns if fid not in prod_ids]
print('test nodes (excluded from L2 tables):', len(test_ids))
tcross = sum(1 for s, t in edges if s not in prod_ids or t not in prod_ids)
print('edges touching a test fn (excluded):', tcross)
