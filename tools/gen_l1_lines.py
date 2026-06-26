"""Resolve path:line for the curated L1 node ids (types + key fns)."""
import json, io
d = json.load(io.open('tools/edges.json', encoding='utf-8'))
fns = {f['id']: f for f in d['fns']}
nodes = json.load(io.open('tools/nodes.json', encoding='utf-8'))

# type id -> (file,line)
type_loc = {}
for crate, ns in nodes.items():
    for n in ns:
        if n['kind'] in ('struct', 'enum', 'trait', 'type_alias'):
            # build a crate::module::Name id from file
            parts = n['file'].split('/')
            # file like antigen/src/learn/adwin.rs
            rel = parts[parts.index('src')+1:]
            rel[-1] = rel[-1][:-3]
            if rel[-1] in ('mod', 'lib', 'main'):
                rel = rel[:-1]
            tid = '::'.join([crate] + rel + [n['name']])
            type_loc[tid] = (n['file'], n['line'])
            type_loc.setdefault(crate + '::' + n['name'], (n['file'], n['line']))

def loc_of(nid):
    if nid in fns:
        f = fns[nid]
        return (f['crate'] + '/src/' + f['file'], f['line'])
    if nid in type_loc:
        return type_loc[nid]
    return (None, None)

# Curated L1 ids (resolved below); print resolution so I can paste exact lines.
L1 = [
    # pipeline / scan spine
    'antigen::pipeline::run', 'antigen::scan::walk::scan_workspace',
    'antigen::scan::parse::ScanVisitor', 'antigen::scan::types::AntigenDeclaration',
    'antigen::finding::Finding', 'antigen::scan::catalog_match::catalog_match_findings_with_source',
    'antigen::scan::finalize::finalize_report_with_catalog', 'antigen::scan::synthesis::synthesis_pass',
    'antigen::scan::multi_crate::scan_workspace_multi_crate',
    # audit
    'antigen::audit::orchestrate', 'antigen::audit::immunity',
    'antigen::audit::types::WitnessTier',
    # learn organ
    'antigen::learn::discriminator::fused_classify', 'antigen::learn::discriminator::classify',
    'antigen::learn::adwin::detect', 'antigen::learn::adwin::fuse_channels',
    'antigen::learn::reader::silent_status', 'antigen::learn::self_tolerance::promote_if_safe',
    'antigen::learn::self_tolerance::is_near_miss', 'antigen::learn::propose::propose',
    'antigen::learn::maturation::mature', 'antigen::learn::curate::curate',
    'antigen::learn::curate::apply', 'antigen::learn::affinity::Affinity',
    'antigen::learn::life_record::LifeRecord', 'antigen::learn::szz::mine',
    # supply chain
    'antigen::supply_chain::evaluate::eval_supply_chain_predicate',
    'antigen::supply_chain::manifest::read_manifest_deps',
    # attestation
    'antigen-attestation::predicate::Predicate', 'antigen-attestation::evaluate::evaluate_predicate',
    'antigen-attestation::schema::Ratification', 'antigen-attestation::tier::WitnessTier',
    # fingerprint
    'antigen-fingerprint::Fingerprint', 'antigen-fingerprint::Constraint',
    'antigen-fingerprint::matcher::match_constraint', 'antigen-fingerprint::parser::parse_constraint',
    'antigen-fingerprint::digest::structural_digest', 'antigen-fingerprint::serialize::to_antigen_attr',
    # cli
    'cargo-antigen::run_audit', 'cargo-antigen::run_scan', 'cargo-antigen::mine::mine_repo',
]
for nid in L1:
    f, l = loc_of(nid)
    print(f"{nid}\t{f}\t{l}")
