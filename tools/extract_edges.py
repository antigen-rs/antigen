"""
Source-level function + call-edge extractor for the antigen workspace.

Strategy: a brace-aware single-pass scanner per file.
- Tracks a context stack of (kind, name) frames: mod / impl / fn.
- Emits one FN node per `fn` definition, with a stable id:
    crate::module_path[::ImplType]::fn_name   (de-duplicated with #N on collision)
- Within each fn body, collects called identifiers (foo(  /  Type::foo(  /  .foo()).
- Resolves call targets in a SECOND pass against the known-fn name table,
  preferring same-impl, then same-module, then crate-wide unique matches.
Comments and strings are stripped first so braces/parens inside them don't break nesting.
"""
import re, io, os, json, sys

ROOT = "."
CRATE_DIRS = {
    "antigen": "antigen/src",
    "antigen-attestation": "antigen-attestation/src",
    "antigen-fingerprint": "antigen-fingerprint/src",
    "antigen-macros": "antigen-macros/src",
    "cargo-antigen": "cargo-antigen/src",
}

def strip_comments_strings(src):
    """Replace string/char/comment contents with spaces, preserving newlines & length-ish."""
    out = []
    i = 0
    n = len(src)
    while i < n:
        c = src[i]
        # line comment
        if c == '/' and i+1 < n and src[i+1] == '/':
            j = src.find('\n', i)
            if j == -1: j = n
            out.append(' ' * (j - i))
            i = j
            continue
        # block comment
        if c == '/' and i+1 < n and src[i+1] == '*':
            j = src.find('*/', i+2)
            if j == -1: j = n
            else: j += 2
            seg = src[i:j]
            out.append(''.join(ch if ch == '\n' else ' ' for ch in seg))
            i = j
            continue
        # raw string r#"..."#
        if c == 'r' and i+1 < n and src[i+1] in '#"':
            m = re.match(r'r(#*)"', src[i:])
            if m:
                hashes = m.group(1)
                close = '"' + hashes
                j = src.find(close, i + m.end())
                if j == -1: j = n
                else: j += len(close)
                seg = src[i:j]
                out.append(''.join(ch if ch == '\n' else ' ' for ch in seg))
                i = j
                continue
        # normal string
        if c == '"':
            j = i + 1
            while j < n:
                if src[j] == '\\': j += 2; continue
                if src[j] == '"': j += 1; break
                j += 1
            seg = src[i:j]
            out.append(''.join(ch if ch == '\n' else ' ' for ch in seg))
            i = j
            continue
        # char literal 'a' or lifetime 'a  -> only treat as char if closing quote near
        if c == "'":
            m = re.match(r"'(\\.|.)'", src[i:])
            if m:
                out.append(' ' * (m.end()))
                i += m.end()
                continue
        out.append(c)
        i += 1
    return ''.join(out)

FN_RE = re.compile(r'\bfn\s+([A-Za-z_][A-Za-z0-9_]*)')
MOD_RE = re.compile(r'\bmod\s+([A-Za-z_][A-Za-z0-9_]*)\s*\{')
IMPL_RE = re.compile(r'\bimpl\b(?:\s*<[^>]*>)?\s+(?:([A-Za-z_][A-Za-z0-9_:]*)\s+for\s+)?([A-Za-z_][A-Za-z0-9_:]*)')

def module_path_for(crate, relpath):
    # relpath like 'audit/types.rs' -> ['audit','types']; mod.rs / lib.rs / main.rs collapse
    parts = relpath.replace('\\','/').split('/')
    parts[-1] = parts[-1][:-3]  # drop .rs
    if parts[-1] in ('mod','lib','main'):
        parts = parts[:-1]
    return parts

def line_of(src, pos):
    return src.count('\n', 0, pos) + 1

def parse_file(crate, abspath, relpath):
    raw = io.open(abspath, encoding='utf-8').read()
    clean = strip_comments_strings(raw)
    modparts = module_path_for(crate, relpath)
    base_mod = '::'.join([crate] + modparts) if modparts else crate

    fns = []   # dicts: id,name,impl,module,line,body_start,body_end
    # Walk char by char tracking brace depth and a stack of frames.
    # frame = {'kind','name','depth_at_open'}
    stack = []
    depth = 0
    i = 0
    n = len(clean)
    pending = None  # a frame waiting for its opening brace
    while i < n:
        ch = clean[i]
        if ch == '{':
            if pending:
                pending['open_depth'] = depth
                pending['body_start'] = i+1
                stack.append(pending)
                pending = None
            depth += 1
            i += 1
            continue
        if ch == '}':
            depth -= 1
            # close frames opened at this depth
            while stack and stack[-1]['open_depth'] == depth:
                fr = stack.pop()
                fr['body_end'] = i
                if fr['kind'] == 'fn':
                    fns.append(fr)
            i += 1
            continue
        # try to match a declaration starting here (only at identifier boundaries)
        if ch.isalpha() or ch == '_':
            # module path frame string
            cur_mod = base_mod
            cur_impl = None
            for fr in stack:
                if fr['kind']=='mod': cur_mod = cur_mod + '::' + fr['name']
                if fr['kind']=='impl': cur_impl = fr['name']
            m = MOD_RE.match(clean, i)
            if m:
                pending_frame = {'kind':'mod','name':m.group(1),'open_depth':None}
                # the brace is at m.end()-1; handle by setting pending and jumping to before brace
                stack_pending = pending_frame
                # advance to the '{'
                pending = pending_frame
                i = m.end()-1
                continue
            mi = IMPL_RE.match(clean, i)
            if mi:
                impl_name = mi.group(2)
                # find the next '{' that opens this impl (skip where clauses/generics)
                bracepos = clean.find('{', mi.end())
                if bracepos != -1:
                    pending = {'kind':'impl','name':impl_name.split('::')[-1],'open_depth':None}
                    i = bracepos
                    continue
            mf = FN_RE.match(clean, i)
            if mf:
                fname = mf.group(1)
                fline = line_of(raw, i)
                bracepos = clean.find('{', mf.end())
                semipos = clean.find(';', mf.end())
                # trait fn decl without body (semicolon before brace) -> still a node, no body
                if semipos != -1 and (bracepos == -1 or semipos < bracepos):
                    fns.append({'kind':'fn','name':fname,'module':cur_mod,'impl':cur_impl,
                                'line':fline,'body_start':None,'body_end':None})
                    i = semipos+1
                    continue
                if bracepos != -1:
                    pending = {'kind':'fn','name':fname,'module':cur_mod,'impl':cur_impl,'line':fline,'open_depth':None}
                    i = bracepos
                    continue
            # skip this identifier token wholesale
            j = i
            while j < n and (clean[j].isalnum() or clean[j]=='_'):
                j += 1
            i = j
            continue
        i += 1

    # assign ids + extract call tokens from each fn body
    return raw, clean, fns

# Free / type-qualified call:  foo(  or  Type::foo(
# Negative lookbehind on '.' so method calls (a.len()) are NOT caught here
# (those belong to METHOD_RE, which feeds the std-method filter).
CALL_RE = re.compile(r'(?<![.\w])(?:([A-Za-z_][A-Za-z0-9_]*)\s*::\s*)?([A-Za-z_][A-Za-z0-9_]*)\s*\(')
METHOD_RE = re.compile(r'\.\s*([A-Za-z_][A-Za-z0-9_]*)\s*\(')

KW = set("if while for match return fn let mut as in else loop move ref where impl dyn pub use mod struct enum trait const static unsafe async await self Self super crate type".split())

# Ubiquitous std/library method names: a `.foo()` call to these is almost
# always a stdlib collection/Option/Result/iterator method, NOT a project fn.
# We refuse to resolve method-style calls to these even if a same-named project
# fn exists, unless the call is type-qualified (Type::foo).
STD_METHODS = set("""
len push pop new clone is_empty iter into_iter iter_mut to_string as_str as_ref
as_mut insert remove get get_mut contains contains_key keys values entry extend
collect map filter filter_map flat_map fold for_each find any all count sum min max
unwrap unwrap_or unwrap_or_else unwrap_or_default expect ok_or ok_or_else map_err
and_then or_else take replace borrow borrow_mut lock read write send recv next
join split splitn split_whitespace trim trim_end trim_start starts_with ends_with
to_owned to_vec sort sort_by sort_unstable dedup reverse last first chars bytes
push_str write_str write_all flush parent file_name extension to_path_buf display
clamp abs min_by max_by saturating_sub saturating_add checked_add checked_sub
floor ceil round sqrt powi powf ln log abs_diff signum rev enumerate zip skip
take_while skip_while peekable cloned copied flatten chain step_by windows chunks
retain drain truncate resize with_capacity capacity reserve append clear swap
position rposition partition unzip product cycle by_ref nth last_mut split_at
to_lowercase to_uppercase replace matches lines repeat from_iter
""".split())

def main():
    all_fns = []  # global list with ids
    file_clean = {}
    for crate, d in CRATE_DIRS.items():
        for dirpath, _, files in os.walk(d):
            for f in files:
                if not f.endswith('.rs'): continue
                abspath = os.path.join(dirpath, f)
                relpath = os.path.relpath(abspath, d)
                raw, clean, fns = parse_file(crate, abspath, relpath)
                file_clean[abspath] = clean
                for fr in fns:
                    base = fr['module']
                    if fr['impl']:
                        fid = base + '::' + fr['impl'] + '::' + fr['name']
                    else:
                        fid = base + '::' + fr['name']
                    fr['id'] = fid
                    fr['crate'] = crate
                    fr['file'] = relpath.replace('\\','/')
                    fr['filefull'] = (d + '/' + relpath.replace('\\','/'))
                    fr['abspath'] = abspath
                    all_fns.append(fr)

    # de-dup ids
    seen = {}
    for fr in all_fns:
        c = seen.get(fr['id'],0)
        seen[fr['id']] = c+1
        if c>0:
            fr['id'] = fr['id'] + ('#%d'%c)

    # name -> list of fns (for resolution)
    by_name = {}
    for fr in all_fns:
        by_name.setdefault(fr['name'], []).append(fr)

    # call edges
    edges = set()
    for fr in all_fns:
        if fr['body_start'] is None: continue
        clean = file_clean[fr['abspath']]
        body = clean[fr['body_start']:fr['body_end']]
        targets = set()  # (qual, name, is_method)
        for m in CALL_RE.finditer(body):
            qual, name = m.group(1), m.group(2)
            if name in KW: continue
            targets.add((qual, name, False))
        for m in METHOD_RE.finditer(body):
            name = m.group(1)
            if name in KW: continue
            targets.add((None, name, True))
        for qual, name, is_method in targets:
            # refuse method-style calls to ubiquitous std method names
            if is_method and not qual and name in STD_METHODS:
                continue
            cands = by_name.get(name)
            if not cands: continue
            chosen = None
            # 1. qualified by impl type
            if qual:
                q = [c for c in cands if c['impl']==qual]
                if len(q)>=1: chosen = q[0]
            if not chosen:
                # same impl
                q = [c for c in cands if c['impl']==fr['impl'] and c['impl'] is not None]
                if len(q)==1: chosen = q[0]
            if not chosen:
                # same module
                q = [c for c in cands if c['module']==fr['module']]
                if len(q)==1: chosen = q[0]
            if not chosen and len(cands)==1:
                chosen = cands[0]
            if chosen and chosen['id'] != fr['id']:
                edges.add((fr['id'], chosen['id']))

    out = {
        'fns': [{k:fr[k] for k in ('id','name','crate','file','line','impl','module')} for fr in all_fns],
        'edges': sorted(list(edges)),
    }
    json.dump(out, io.open('tools/edges.json','w',encoding='utf-8'))
    print('total fns:', len(all_fns))
    percrate={}
    for fr in all_fns: percrate[fr['crate']]=percrate.get(fr['crate'],0)+1
    print('per crate:', percrate)
    print('call edges:', len(edges))

if __name__ == '__main__':
    main()
