import json, io

CRATES = {
  'antigen': 'target/doc/antigen.json',
  'antigen-attestation': 'target/doc/antigen_attestation.json',
  'antigen-fingerprint': 'target/doc/antigen_fingerprint.json',
  'antigen-macros': 'target/doc/antigen_macros.json',
}

def load(p):
    return json.load(io.open(p, encoding='utf-8'))

def first_line(s):
    if not s: return ''
    s = s.strip().split('\n')[0].strip()
    return s[:90]

allnodes = {}
for crate, path in CRATES.items():
    d = load(path)
    idx = d['index']
    paths = d.get('paths', {})

    def fullpath(item_id, name):
        p = paths.get(str(item_id)) or paths.get(item_id)
        if p and p.get('path'):
            return '::'.join(p['path'])
        return name

    for k, v in idx.items():
        inner = v.get('inner', {})
        if not inner:
            continue
        kind = list(inner.keys())[0]
        if kind not in ('function', 'struct', 'enum', 'module', 'trait', 'type_alias'):
            continue
        name = v.get('name')
        if not name:
            continue
        span = v.get('span') or {}
        fn = span.get('filename', '')
        begin = (span.get('begin') or [None])[0]
        fp = fullpath(k, name)
        doc = first_line(v.get('docs') or '')
        allnodes.setdefault(crate, []).append({
            'kind': kind, 'name': name, 'path': fp,
            'file': fn.replace('\\', '/'), 'line': begin, 'doc': doc, 'id': k
        })

json.dump(allnodes, io.open('tools/nodes.json', 'w', encoding='utf-8'))
for c, ns in allnodes.items():
    kc = {}
    for n in ns:
        kc[n['kind']] = kc.get(n['kind'], 0) + 1
    print(c, kc, 'total', len(ns))
