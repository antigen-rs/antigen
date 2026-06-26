import json, io
from collections import defaultdict
d = json.load(io.open('tools/edges.json', encoding='utf-8'))
fns = {f['id']: f for f in d['fns']}
def istest(f):
    return '::tests::' in f['id'] or f['module'].endswith('::tests') \
        or '::test_' in f['id'] or f['module'].endswith('::test')
test = [f for f in fns.values() if istest(f)]
by = defaultdict(lambda: defaultdict(int))
for f in test:
    by[f['crate']][f['file']] += 1
lines = ["\n\n## Appendix · test functions (inventory only, not drawn)\n",
         f"839 test functions exist; they are nodes in the source but excluded from the L2 tables/edges above to keep navigation clean. Counts per file:\n",
         "| crate | file | test fns |", "|---|---|---|"]
for crate in ['antigen','antigen-attestation','antigen-fingerprint','antigen-macros','cargo-antigen']:
    for fl in sorted(by[crate]):
        lines.append(f"| {crate} | {fl} | {by[crate][fl]} |")
open('tools/test_appendix.md','w',encoding='utf-8').write('\n'.join(lines))
print('test files:', sum(len(by[c]) for c in by), 'total test fns:', len(test))
