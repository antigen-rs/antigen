//! Store spike — project the stroma into an RDF quad store to find gaps BEFORE ratifying
//! `jbd/research/decisions/layered-derivation-store.md` (the "two-dimensions-one-slot problem":
//! named graph = (producer, tier), so the witness set becomes multi-graph membership of the same
//! triple — does that hold cleanly, and does it make the explicit `witnesses` field redundant?).
//!
//! Constitutes the real graph exactly as `light_the_wing.rs` (`ingest_scip` +
//! `populate_from_reconstructions`), reads the SCIP symbol table, projects every `EdgeFact` /
//! `SymbolFact` into RDF quads (named graph = producer×tier), loads them into a sophia in-memory
//! dataset, dumps N-Quads to `target/stroma.nq` (the co-native "you can `cat` and `git diff` your
//! stroma" win), round-trips through the parser and through redb, then runs the three query shapes
//! the design record calls out as HARD before: cross-witness (multi-graph membership), tier-filter
//! (facts at tier ≤ k), and a join (call-edges into a constructor symbol) — plus a Datalog
//! feasibility probe (hand-rolled fixpoint vs a real `ascent!` program, cross-checked).
//!
//! Usage: `cargo run -p antigen-stroma --example store_spike -- [path/to.scip]`
//! (defaults to `target/antigen.scip`, produced by `rust-analyzer scip . --output …`).

// A stats-display spike: usize→f64 precision loss on graph/quad counts is irrelevant here.
#![allow(clippy::cast_precision_loss)]
// `main` is a linear, narrated walkthrough of the spike's 6 numbered steps (same spirit as
// `light_the_wing.rs`/`dual_tier_barcode.rs`'s file-level allows) — splitting it into helper
// functions would scatter the narrative the report needs to read top-to-bottom.
#![allow(clippy::too_many_lines)]

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt::Write as _;
use std::path::Path;

use antigen_stroma::base::facts::{EdgeFact, SymbolFact, WitnessSource};
use antigen_stroma::db::StromaDb;
use antigen_stroma::engine::ReachabilityRelation;
use antigen_stroma::read::SnapshotHandle;
use antigen_stroma::scip::{
    EdgeReconstruction, ingest_scip, ingest_scip_symbols, populate_from_reconstructions,
};
use ascent::ascent;
use redb::{Database, ReadableTable, TableDefinition};
use sophia::api::prelude::*;
use sophia::api::term::{FromTerm, GraphName, SimpleTerm};
use sophia::inmem::dataset::FastDataset;
use sophia::iri::Iri;
use sophia::turtle::parser::nq;
use sophia::turtle::serializer::nq::NQuadsSerializer;

/// The one base namespace every term in this spike hangs off — bare local names underneath it
/// (mirrors `dual_tier_barcode.rs`'s bare-name projection style for readability).
const NS: &str = "urn:antigen:stroma#";

/// A term that is always an owned, checked-absolute IRI — no borrow lifetimes to juggle across the
/// ~24k-quad loop below, and `Iri::new` (checked, not `_unchecked`) doubles as a live IRI-validity
/// ATK on [`iri_safe`]: a bug in the escaper fails LOUD here instead of silently shipping a corrupt
/// N-Quads line.
type IriTerm = Iri<String>;

/// The SPOG (forward) redb table — one row per quad, S/P/O/G each a [`term_key`]-encoded string
/// (empty string for the default graph, since no real IRI in this spike is empty).
const SPOG: TableDefinition<(&str, &str, &str, &str), ()> = TableDefinition::new("spog");
/// The OSPG (reverse) redb table — the same quads, keyed object-first for the "what points at this
/// node" query direction.
const OSPG: TableDefinition<(&str, &str, &str, &str), ()> = TableDefinition::new("ospg");

/// Percent-encode any byte outside a small IRI-safe set. Applied to every local name (both the
/// hand-picked ASCII vocabulary words AND raw SCIP symbols, which carry spaces/`#`/`(`/`)`/`` ` ``/
/// `[`/`]`) so [`iri`] can never hand `Iri::new` an invalid IRI.
fn iri_safe(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b':' => {
                out.push(b as char);
            },
            _ => {
                let _ = write!(out, "%{b:02X}");
            },
        }
    }
    out
}

/// Build a term under the base namespace from a local name, percent-encoding it first.
fn iri(local: &str) -> IriTerm {
    Iri::new(format!("{NS}{}", iri_safe(local))).expect("iri_safe output is always IRI-legal")
}

/// A node term for a raw SCIP symbol — the PRECISE identity (percent-encoded, collision-free),
/// distinct from its bare-name [`label`] (display-only, see the doc comment on [`label`] for why
/// the two must not be conflated).
fn node_iri(full_symbol: &str) -> IriTerm {
    iri(&format!("sym/{full_symbol}"))
}

// ── Bare-name reduction (ported from examples/dual_tier_barcode.rs, LABEL-ONLY here) ───────────
//
// dual_tier_barcode coarsens node identity to the bare name because two INDEPENDENT feeders (syn +
// SCIP) need a common coordinate to agree on. This spike has ONE feeder (SCIP, per
// `light_the_wing.rs`), so there is no such need — and bare-naming would MERGE distinct symbols
// that happen to share a tail (measured below: `bare_name_collisions`). So the full percent-encoded
// symbol is the quad IDENTITY; the bare name is carried ONLY as a `:label` literal for the human
// reading the N-Quads dump — reusing the extraction, not the coarsening.

/// Extract the bare function name from a SCIP symbol string (ported, unchanged, from
/// `dual_tier_barcode.rs`) — e.g. `rust-analyzer cargo antigen 0.6.0 scan/scan_file().` →
/// `scan_file`. Returns `None` for symbols that aren't function-shaped (types, modules, consts…).
fn symbol_fn_name(symbol: &str) -> Option<&str> {
    let without_suffix = symbol.strip_suffix(").")?;
    let paren_pos = without_suffix.rfind('(')?;
    let before_paren = &without_suffix[..paren_pos];
    let segment = before_paren
        .rsplit(['/', '#'])
        .next()
        .filter(|s| !s.is_empty())?;
    if let Some(bracket_end) = segment.rfind(']') {
        let after_bracket = &segment[bracket_end + 1..];
        if !after_bracket.is_empty() {
            return Some(after_bracket);
        }
    }
    if segment.starts_with('`') && segment.ends_with('`') && segment.len() > 2 {
        return Some(&segment[1..segment.len() - 1]);
    }
    Some(segment)
}

/// The tail descriptor of a SCIP symbol (ported, unchanged, from `light_the_wing.rs`'s `short()`)
/// — the fallback label for non-fn-shaped symbols `symbol_fn_name` declines to reduce.
fn short_tail(symbol: &str) -> &str {
    symbol.rsplit(' ').next().unwrap_or(symbol)
}

/// A human-readable label for a raw SCIP symbol: the bare fn name where the symbol is fn-shaped,
/// else the tail descriptor, never the full space-delimited symbol.
fn label(full_symbol: &str) -> &str {
    symbol_fn_name(full_symbol).unwrap_or_else(|| short_tail(full_symbol))
}

/// The named-graph = (producer, tier) triple the design record's crux rests on.
struct GraphMeta {
    /// The graph node's own local name — `:g_scip_t0` etc.
    local: &'static str,
    /// `:producer` value — which instrument.
    producer: &'static str,
    /// `:tier` value — the resolution tier that producer's edges are stamped at.
    tier: &'static str,
}

const fn graph_meta(w: WitnessSource) -> GraphMeta {
    match w {
        WitnessSource::Syn => GraphMeta {
            local: "g_syn_t0",
            producer: "Syn",
            tier: "Syntactic",
        },
        WitnessSource::ScipResolved => GraphMeta {
            local: "g_scip_t0",
            producer: "ScipResolved",
            tier: "Resolved",
        },
        WitnessSource::ScipAmbiguous => GraphMeta {
            local: "g_scip_ambiguous",
            producer: "ScipAmbiguous",
            tier: "Syntactic",
        },
    }
}

/// Encode a borrowed [`Term`] to a stable string key — `I<iri>` for an IRI, `L<lexical>` for a
/// literal. Used both for the redb persistence keys and for in-memory triple grouping; the 1-byte
/// tag keeps the two term kinds from ever colliding in the same keyspace.
fn term_key<T: Term>(t: T) -> String {
    t.iri().map_or_else(
        || {
            t.lexical_form()
                .map_or_else(|| format!("?{:?}", t.kind()), |lex| format!("L{lex}"))
        },
        |i| format!("I{i}"),
    )
}

/// Decode a [`term_key`]-encoded string back into an [`IriTerm`] or a literal string — the redb
/// reload path. Returns `Err` for the `?...` escape hatch (not expected in this spike's data).
fn term_key_decode(k: &str) -> Result<SimpleTerm<'static>, String> {
    match k.as_bytes().first() {
        Some(b'I') => {
            let iri_term = Iri::new(k[1..].to_string()).map_err(|e| e.to_string())?;
            Ok(SimpleTerm::from_term(iri_term))
        },
        Some(b'L') => Ok(SimpleTerm::from_term(&k[1..])),
        _ => Err(format!("undecodable term key: {k}")),
    }
}

/// Note one raw SCIP symbol into the collision-measurement sets and the label map — called once
/// per edge endpoint / symbol-table entry.
fn note_symbol<'a>(
    sym: &'a str,
    labels: &mut BTreeMap<String, String>,
    distinct_full_symbols: &mut HashSet<&'a str>,
    distinct_bare_names: &mut HashSet<&'a str>,
) {
    distinct_full_symbols.insert(sym);
    distinct_bare_names.insert(label(sym));
    labels.insert(node_iri(sym).to_string(), label(sym).to_string());
}

/// The bare predicates every quad in this spike hangs off — bundled into one struct so
/// [`project_witnessed_edge`] stays under clippy's arg-count ceiling.
struct Vocab {
    calls: IriTerm,
    producer: IriTerm,
    tier: IriTerm,
}

/// Project one witnessed edge into quads — iterating `witnesses` and emitting into the graph each
/// one owns (declaring the graph's `:producer`/`:tier` description the first time it's seen).
/// Factored out so the SAME mechanism can be exercised on both the real data in `main` AND the
/// synthetic positive-control edge (see the "positive control" block there).
fn project_witnessed_edge(
    dataset: &mut FastDataset,
    declared_graphs: &mut HashSet<&'static str>,
    vocab: &Vocab,
    s: &IriTerm,
    o: &IriTerm,
    witnesses: &BTreeSet<WitnessSource>,
) -> Result<(), Box<dyn std::error::Error>> {
    for &w in witnesses {
        let gm = graph_meta(w);
        if declared_graphs.insert(gm.local) {
            let g = iri(gm.local);
            dataset.insert(
                g.clone(),
                vocab.producer.clone(),
                iri(gm.producer),
                None::<IriTerm>,
            )?;
            dataset.insert(g, vocab.tier.clone(), iri(gm.tier), None::<IriTerm>)?;
        }
        dataset.insert(
            s.clone(),
            vocab.calls.clone(),
            o.clone(),
            Some(iri(gm.local)),
        )?;
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let scip_path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "target/antigen.scip".to_string());
    println!("== Store spike: projecting the stroma into an RDF quad store ({scip_path}) ==\n");

    // 1. Constitute — exactly as light_the_wing.rs: ingest_scip + populate_from_reconstructions.
    let recons = ingest_scip(Path::new(&scip_path));
    let (mut n_resolved, mut n_ambiguous, mut n_unreconstructible) = (0usize, 0usize, 0usize);
    for r in &recons {
        match r {
            EdgeReconstruction::Resolved(_) => n_resolved += 1,
            EdgeReconstruction::Ambiguous(v) => n_ambiguous += v.len(),
            EdgeReconstruction::Unreconstructible => n_unreconstructible += 1,
        }
    }
    println!(
        "SCIP edge-reconstructions: {} (Resolved={n_resolved}  Ambiguous={n_ambiguous}  \
         Unreconstructible={n_unreconstructible})",
        recons.len()
    );
    let mut db = StromaDb::default();
    populate_from_reconstructions(&mut db, &recons);
    let snap = SnapshotHandle::open(&db);
    let edges: Vec<EdgeFact> = snap
        .db()
        .edge_facts()
        .map(|ef| ef.edges(snap.db()))
        .unwrap_or_default();
    let rel = ReachabilityRelation::from_edges(&edges);
    println!(
        "stroma constituted: {} nodes, {} edges (ReachabilityRelation baseline)\n",
        rel.len(),
        edges.len()
    );

    let symbols: Vec<SymbolFact> = ingest_scip_symbols(Path::new(&scip_path));
    println!("SCIP symbol-table entries: {}\n", symbols.len());

    // ── 2. Project facts → quads (named graph = producer×tier) ─────────────────────────────────
    let mut dataset = FastDataset::new();
    let mut declared_graphs: HashSet<&'static str> = HashSet::new();
    let mut labels: BTreeMap<String, String> = BTreeMap::new(); // node IRI string -> bare label
    let mut distinct_full_symbols: HashSet<&str> = HashSet::new();
    let mut distinct_bare_names: HashSet<&str> = HashSet::new();

    let vocab = Vocab {
        calls: iri("calls"),
        producer: iri("producer"),
        tier: iri("tier"),
    };
    let p_kind = iri("kind");
    let p_label = iri("label");

    for e in &edges {
        note_symbol(
            &e.src.fq_path.path,
            &mut labels,
            &mut distinct_full_symbols,
            &mut distinct_bare_names,
        );
        note_symbol(
            &e.dst.fq_path.path,
            &mut labels,
            &mut distinct_full_symbols,
            &mut distinct_bare_names,
        );
        let s = node_iri(&e.src.fq_path.path);
        let o = node_iri(&e.dst.fq_path.path);
        project_witnessed_edge(
            &mut dataset,
            &mut declared_graphs,
            &vocab,
            &s,
            &o,
            &e.witnesses,
        )?;
    }

    // SymbolFacts (design record step 2): `(:sym :kind :EnumMember)` etc., landing in `:g_scip_t0`
    // per the instruction — held, not enforced (an organ decides what a constructor-callee means).
    if declared_graphs.insert("g_scip_t0") {
        let g = iri("g_scip_t0");
        dataset.insert(
            g.clone(),
            vocab.producer.clone(),
            iri("ScipResolved"),
            None::<IriTerm>,
        )?;
        dataset.insert(g, vocab.tier.clone(), iri("Resolved"), None::<IriTerm>)?;
    }
    let g_scip_t0 = iri("g_scip_t0");
    for sf in &symbols {
        note_symbol(
            &sf.symbol,
            &mut labels,
            &mut distinct_full_symbols,
            &mut distinct_bare_names,
        );
        let sym = node_iri(&sf.symbol);
        let kind_local = format!("{:?}", sf.kind);
        dataset.insert(
            sym,
            p_kind.clone(),
            iri(&kind_local),
            Some(g_scip_t0.clone()),
        )?;
    }

    // Labels, once per distinct node, in the default graph — the human-readability win.
    for (node, lab) in &labels {
        dataset.insert(
            Iri::new(node.clone())?,
            p_label.clone(),
            lab.as_str(),
            None::<IriTerm>,
        )?;
    }

    let quad_count = dataset.quads().count();
    println!(
        "== Quad projection ==\n  {quad_count} quads across {} named graphs (+ default graph for \
         provenance/label triples)",
        declared_graphs.len()
    );
    println!(
        "  distinct full SCIP symbols: {}  distinct bare names: {}  collision ratio: {:.2}x \
         (why the bare name is a :label, not identity)\n",
        distinct_full_symbols.len(),
        distinct_bare_names.len(),
        distinct_full_symbols.len() as f64 / distinct_bare_names.len().max(1) as f64
    );

    // ── 3. Dump N-Quads — the co-native inspection win — then reload + assert round-trip ────────
    let nq_text = NQuadsSerializer::new_stringifier()
        .serialize_dataset(&dataset)
        .map_err(|e| format!("N-Quads serialize failed: {e}"))?
        .to_string();
    std::fs::write("target/stroma.nq", &nq_text)?;
    println!(
        "== N-Quads dump (target/stroma.nq, {} bytes) ==",
        nq_text.len()
    );
    for line in nq_text.lines().take(15) {
        println!("  {line}");
    }
    println!("  …\n");

    let reloaded: FastDataset = nq::parse_str(&nq_text)
        .collect_quads()
        .map_err(|e| format!("N-Quads parse failed: {e}"))?;
    let reloaded_count = reloaded.quads().count();
    println!(
        "N-Quads round-trip: {reloaded_count} quads reloaded ({})\n",
        if reloaded_count == quad_count {
            "MATCHES the in-memory dataset"
        } else {
            "MISMATCH — a real gap"
        }
    );

    // ── 4. Persist to redb + reload — SPOG forward, OSPG reverse ────────────────────────────────
    let redb_path = "target/stroma.redb";
    let _ = std::fs::remove_file(redb_path);
    let redb_db = Database::create(redb_path)?;
    {
        let txn = redb_db.begin_write()?;
        {
            let mut spog = txn.open_table(SPOG)?;
            let mut ospg = txn.open_table(OSPG)?;
            for q in dataset.quads() {
                let ([s, p, o], g) = q?.spog();
                let sk = term_key(s);
                let pk = term_key(p);
                let ok = term_key(o);
                let gk = g.map_or_else(String::new, term_key);
                spog.insert((sk.as_str(), pk.as_str(), ok.as_str(), gk.as_str()), ())?;
                ospg.insert((ok.as_str(), sk.as_str(), pk.as_str(), gk.as_str()), ())?;
            }
        }
        txn.commit()?;
    }
    let redb_read = redb_db.begin_read()?;
    let spog_table = redb_read.open_table(SPOG)?;
    let mut redb_reloaded = FastDataset::new();
    for entry in spog_table.iter()? {
        let (key, _val) = entry?;
        let (sk, pk, ok, gk) = key.value();
        let s = term_key_decode(sk).map_err(|e| format!("redb decode (s): {e}"))?;
        let p = term_key_decode(pk).map_err(|e| format!("redb decode (p): {e}"))?;
        let o = term_key_decode(ok).map_err(|e| format!("redb decode (o): {e}"))?;
        let g = if gk.is_empty() {
            None
        } else {
            Some(term_key_decode(gk).map_err(|e| format!("redb decode (g): {e}"))?)
        };
        redb_reloaded.insert(s, p, o, g)?;
    }
    let redb_count = redb_reloaded.quads().count();
    println!(
        "redb persistence round-trip: {redb_count} quads (SPOG + OSPG tables, {}) ({})\n",
        redb_path,
        if redb_count == quad_count {
            "MATCHES"
        } else {
            "MISMATCH — a real gap"
        }
    );

    // ── 5. The queries that were hard before ────────────────────────────────────────────────────

    // Positive control — proves the MECHANISM before trusting the real-data query below. This
    // codebase's current SCIP index measured `Ambiguous=0` (step 1) and this spike never feeds a
    // Syn witness (out of scope per the brief), so the real graph has exactly ONE producer graph
    // right now — Query 1 below would read 0 by ABSENCE OF OPPORTUNITY, which looks identical to
    // "the projection is broken" unless it's disentangled. A synthetic edge with 2 witnesses,
    // pushed through the SAME `project_witnessed_edge` used on real data, settles which one it is.
    {
        let mut control = FastDataset::new();
        let mut control_graphs: HashSet<&'static str> = HashSet::new();
        let cs = node_iri("control_src");
        let co = node_iri("control_dst");
        let w: BTreeSet<WitnessSource> = [WitnessSource::Syn, WitnessSource::ScipResolved]
            .into_iter()
            .collect();
        project_witnessed_edge(&mut control, &mut control_graphs, &vocab, &cs, &co, &w)?;
        let graphs_seen: BTreeSet<String> = control
            .quads_matching([&cs], [&vocab.calls], [&co], Any)
            .filter_map(|q| q.ok().and_then(|q| q.g().map(term_key)))
            .collect();
        println!("== Positive control: synthetic edge witnessed by {{Syn, ScipResolved}} ==");
        println!(
            "  landed in {} graph(s): {:?}  ({})\n",
            graphs_seen.len(),
            graphs_seen,
            if graphs_seen.len() == 2 {
                "CONFIRMS multi-graph membership = witness set, mechanically"
            } else {
                "MECHANISM GAP — investigate before trusting the crux"
            }
        );
    }

    // (a) Cross-witness: `:calls` triples appearing in ≥2 producer graphs.
    let mut by_triple: HashMap<(String, String), BTreeSet<String>> = HashMap::new();
    for q in dataset.quads() {
        let ([s, p, o], g) = q?.spog();
        if term_key(p) != term_key(vocab.calls.clone()) {
            continue;
        }
        let Some(g) = g else { continue };
        by_triple
            .entry((term_key(s), term_key(o)))
            .or_default()
            .insert(term_key(g));
    }
    let cross_witnessed: Vec<_> = by_triple
        .iter()
        .filter(|(_, graphs)| graphs.len() >= 2)
        .collect();
    println!("== Query 1: cross-witness (multi-graph membership on the same :calls triple) ==");
    println!(
        "  {} of {} distinct :calls triples appear in ≥2 producer graphs ({} producer graph(s) \
         populated this run — see the positive control above for the mechanism proof if this is 0)",
        cross_witnessed.len(),
        by_triple.len(),
        declared_graphs.len()
    );
    for ((s, o), graphs) in cross_witnessed.iter().take(3) {
        println!(
            "    {} → {}  (witnessed in: {:?})",
            &s[1..].rsplit("sym/").next().unwrap_or(s),
            &o[1..].rsplit("sym/").next().unwrap_or(o),
            graphs.iter().map(|g| &g[1..]).collect::<Vec<_>>()
        );
    }
    println!();

    // (b) Tier filter: all quads in graphs stamped `:tier :Resolved` — via `quads_matching` with a
    // Vec<GraphName<_>> matcher (the "facts at tier ≤ k" shape the design record calls out).
    let resolved_graphs: Vec<GraphName<IriTerm>> = declared_graphs
        .iter()
        .filter(|&&local| graph_of_tier(&dataset, local, "Resolved"))
        .map(|&local| Some(iri(local)))
        .collect();
    let tier_filtered = dataset
        .quads_matching(Any, Any, Any, resolved_graphs.as_slice())
        .count();
    println!("== Query 2: tier filter (quads_matching, graphs stamped :tier :Resolved) ==");
    println!(
        "  {tier_filtered} quads live in a Resolved-tier graph (of {quad_count} total; {} such \
         graphs of {})\n",
        resolved_graphs.len(),
        declared_graphs.len()
    );

    // (c) Join: call-edges whose callee is a constructor (`:kind` ∈ {EnumMember, Struct}).
    let constructor_kinds = [iri("EnumMember"), iri("Struct")];
    let constructor_syms: HashSet<String> = dataset
        .quads_matching(
            Any,
            [&p_kind],
            constructor_kinds.iter().collect::<Vec<_>>(),
            Any,
        )
        .filter_map(|q| q.ok().map(|q| term_key(q.s())))
        .collect();
    let construction_calls: Vec<_> = dataset
        .quads_matching(Any, [&vocab.calls], Any, Any)
        .filter_map(std::result::Result::ok)
        .filter(|q| constructor_syms.contains(&term_key(q.o())))
        .collect();
    println!("== Query 3: join (call-edges whose callee :kind is a constructor) ==");
    // If this comes back 0, check WHY before blaming the store: `ingest_scip`'s own `is_fn_symbol`
    // gate (only accepts reference occurrences whose SCIP symbol ends in `).`) may structurally
    // exclude constructor symbols from ever becoming an EdgeFact destination — a substrate finding,
    // not a query bug. Sample a few raw constructor symbols to check.
    let constructor_symbol_shapes: Vec<&str> = symbols
        .iter()
        .filter(|sf| sf.kind.is_constructor())
        .take(3)
        .map(|sf| sf.symbol.as_str())
        .collect();
    println!(
        "  {} call-edge(s) target a constructor symbol ({} distinct constructor symbols observed)",
        construction_calls.len(),
        constructor_syms.len()
    );
    for q in construction_calls.iter().take(3) {
        let s = term_key(q.s());
        let o = term_key(q.o());
        println!(
            "    {} → {}",
            labels.get(&s[1..]).map_or(s.as_str(), String::as_str),
            labels.get(&o[1..]).map_or(o.as_str(), String::as_str)
        );
    }
    if construction_calls.is_empty() {
        println!(
            "  (sample constructor symbols, to check whether `ingest_scip`'s `).`-suffix gate excludes them:)"
        );
        for sym in &constructor_symbol_shapes {
            println!("    {sym}  (ends with \").\": {})", sym.ends_with(")."));
        }
    }
    println!();

    // ── 6. Datalog feasibility — hand-rolled fixpoint vs a real ascent! program, cross-checked ──
    datalog_feasibility(&dataset, &vocab.calls, &labels)?;

    println!("== The store spike is complete. ==");
    Ok(())
}

/// Whether the named graph `local` carries `(:local :tier :want)` in the default graph.
fn graph_of_tier(dataset: &FastDataset, local: &str, want: &str) -> bool {
    dataset
        .quads_matching(
            [&iri(local)],
            [&iri("tier")],
            [&iri(want)],
            [None::<IriTerm>],
        )
        .next()
        .is_some()
}

/// Assess Datalog feasibility over the quad store: extract the `:calls` edge set, run a hand-rolled
/// worklist fixpoint AND a real `ascent!` transitive-closure program over the SAME interned facts,
/// and cross-check they agree — a real answer to "could ascent bind to a quad source", not just a
/// sketch. `ascent` is an existing workspace dep that (per `base/facts.rs`'s own comment) appears
/// ONLY in a comment today — this is its first real invocation in the crate.
fn datalog_feasibility(
    dataset: &FastDataset,
    p_calls: &IriTerm,
    labels: &BTreeMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Intern every node touched by a `:calls` triple to a small Copy index — ascent facts want
    // Copy/Eq/Hash, and the redb/sophia terms are heap strings.
    let mut intern: HashMap<String, u32> = HashMap::new();
    let mut rev: Vec<String> = Vec::new();
    let mut edge_pairs: Vec<(u32, u32)> = Vec::new();
    for q in dataset.quads_matching(Any, [p_calls], Any, Any) {
        let q = q?;
        let s = term_key(q.s());
        let o = term_key(q.o());
        let si = *intern.entry(s.clone()).or_insert_with(|| {
            rev.push(s);
            u32::try_from(rev.len() - 1).expect("edge count fits u32")
        });
        let oi = *intern.entry(o.clone()).or_insert_with(|| {
            rev.push(o);
            u32::try_from(rev.len() - 1).expect("edge count fits u32")
        });
        edge_pairs.push((si, oi));
    }
    edge_pairs.sort_unstable();
    edge_pairs.dedup();

    // (a) Hand-rolled worklist fixpoint — plain BFS closure per source (linear-rule shape:
    // `reaches(x,y) <-- calls(x,y).  reaches(x,z) <-- reaches(x,y), calls(y,z).`).
    let mut adjacency: HashMap<u32, Vec<u32>> = HashMap::new();
    for &(a, b) in &edge_pairs {
        adjacency.entry(a).or_default().push(b);
    }
    let mut hand_rolled: BTreeSet<(u32, u32)> = BTreeSet::new();
    for &start in intern.values() {
        let mut seen = HashSet::new();
        let mut stack = vec![start];
        while let Some(n) = stack.pop() {
            for &next in adjacency.get(&n).into_iter().flatten() {
                if seen.insert(next) {
                    hand_rolled.insert((start, next));
                    stack.push(next);
                }
            }
        }
    }

    // (b) A real ascent! program over the same interned facts.
    ascent! {
        relation calls(u32, u32);
        relation reaches(u32, u32);
        reaches(x, y) <-- calls(x, y);
        reaches(x, z) <-- reaches(x, y), calls(y, z);
    }
    let mut prog = AscentProgram {
        calls: edge_pairs.clone(),
        ..Default::default()
    };
    prog.run();
    let ascent_reaches: BTreeSet<(u32, u32)> = prog.reaches.into_iter().collect();

    println!("== Datalog feasibility (transitive :calls reachability) ==");
    println!(
        "  {} distinct :calls edges over {} interned nodes",
        edge_pairs.len(),
        rev.len()
    );
    println!(
        "  hand-rolled fixpoint: {} reachable pair(s)  ascent! program: {} reachable pair(s)  ({})",
        hand_rolled.len(),
        ascent_reaches.len(),
        if hand_rolled == ascent_reaches {
            "AGREE — ascent binds cleanly to a materialized quad source"
        } else {
            "DISAGREE — a real gap, investigate before trusting either"
        }
    );
    // Reach-count per source in one pass (the naive per-pair `filter` re-scan is O(n²) and this
    // graph is fat-tailed — light_the_wing.rs measured max ≫ mean blast-radius on this same
    // codebase — so a hub source can make that re-scan genuinely too slow).
    let mut reach_count: HashMap<u32, usize> = HashMap::new();
    for &(a, _) in &hand_rolled {
        *reach_count.entry(a).or_insert(0) += 1;
    }
    if let Some((&top, &reach_n)) = reach_count.iter().max_by_key(|&(_, &n)| n) {
        // `rev[top]` is a term_key-encoded string (`I<iri>` / `L<lexical>`); strip the 1-byte tag
        // to look it up in `labels` (keyed by the raw IRI string built in the projection step).
        let node_iri_str = &rev[top as usize][1..];
        println!(
            "  widest source: {} reaches {reach_n} node(s) transitively\n",
            labels
                .get(node_iri_str)
                .map_or(node_iri_str, String::as_str)
        );
    }
    Ok(())
}
