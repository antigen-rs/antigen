//! # `syn_ingest` — the syntactic (`syn`) witness producer (the SECOND afferent instrument)
//!
//! A `syn` walk that corroborates SCIP `Call` edges by bare name — the second afferent instrument.
//!
//! The stroma's ingest is SCIP-only today (`scip.rs`), so every `EdgeFact`'s witness set is a
//! singleton and corroboration / conflict (H0 / H1) can never fire in production — the sheaf has one
//! section (nb 029 §1).
//!
//! This module promotes the measured `CallVisitor` from
//! `examples/dual_tier_barcode.rs` (itself ported from the syntactic-vs-SCIP falsifier) into a reusable
//! library producer plus the syn **corroboration primitive**: it adds the [`WitnessSource::Syn`] witness
//! to the `Call` edges the parser also saw, turning a `{ScipResolved}` singleton into
//! `{Syn, ScipResolved}` — the first non-degenerate witness set on the real graph.
//!
//! ## What `syn` sees, and its blind spot (the point — see `base::facts::WitnessSource`)
//!
//! `syn` parses EVERY `cfg` branch but resolves nothing: it yields a call by **bare name** (`foo` calls
//! `bar`), not *which* `bar`. So it corroborates a SCIP edge by **bare-name projection** ([`bare_fn_name`]) —
//! two distinct `bar`s collapse onto one name. That coarsening is the honest first version, and it is
//! exactly the complementary blind spot that makes the syn-vs-SCIP *disagreement* a signal. Full-symbol
//! reconciliation (lifting a `syn` edge to a SCIP symbol via the symbol table) is the frontier. `syn`
//! also requires **parseable** code (a parse error yields nothing) — its home is the `save` lifecycle
//! tier; the keystroke / always-live tier wants an error-resilient parser (tree-sitter).
//!
//! **Scope (build-ahead-of-consumer):** this is the producer + the corroboration primitive, with unit
//! tests. Wiring it into the `constitute` / scan ingest path (so every organ sees corroborated edges),
//! and representing `syn`-only *phantom* pairs (a call `syn` saw that SCIP did not resolve — the H1
//! conflict side), are the next slices.

use std::collections::HashSet;

use syn::visit::Visit;

use crate::base::facts::{EdgeFact, EdgeKind, WitnessSource};

// ── Syntactic extraction (promoted verbatim from `examples/dual_tier_barcode.rs`) ─────────────────

/// A `syn` visitor recording `(caller_fn, callee_name)` bare-name call edges.
struct CallVisitor {
    /// Enclosing-function name stack (for nested fns / closures).
    fn_stack: Vec<String>,
    /// Collected `(caller, callee)` bare-name pairs.
    edges: Vec<(String, String)>,
}

impl CallVisitor {
    const fn new() -> Self {
        Self {
            fn_stack: Vec::new(),
            edges: Vec::new(),
        }
    }

    fn current_fn(&self) -> Option<&str> {
        self.fn_stack.last().map(String::as_str)
    }
}

impl<'ast> Visit<'ast> for CallVisitor {
    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        self.fn_stack.push(node.sig.ident.to_string());
        syn::visit::visit_item_fn(self, node);
        self.fn_stack.pop();
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        self.fn_stack.push(node.sig.ident.to_string());
        syn::visit::visit_impl_item_fn(self, node);
        self.fn_stack.pop();
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        self.fn_stack.push(node.sig.ident.to_string());
        syn::visit::visit_trait_item_fn(self, node);
        self.fn_stack.pop();
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let (Some(caller), Some(callee)) = (self.current_fn(), extract_call_name(&node.func)) {
            self.edges.push((caller.to_string(), callee));
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast syn::ExprMethodCall) {
        if let Some(caller) = self.current_fn() {
            self.edges
                .push((caller.to_string(), node.method.to_string()));
        }
        syn::visit::visit_expr_method_call(self, node);
    }
}

/// Extract the bare callee name from a call expression's function part.
fn extract_call_name(expr: &syn::Expr) -> Option<String> {
    match expr {
        // `foo()`, `Foo::bar()`, `crate::m::fn()` — last path segment is the name.
        syn::Expr::Path(ep) => ep.path.segments.last().map(|s| s.ident.to_string()),
        // `self.field()` (call through a field) — the field name.
        syn::Expr::Field(ef) => match &ef.member {
            syn::Member::Named(n) => Some(n.to_string()),
            syn::Member::Unnamed(_) => None,
        },
        // Closures, macro calls, etc. — not extractable as a bare name.
        _ => None,
    }
}

/// Extract bare-name `(caller, callee)` call pairs from one Rust source string.
///
/// Returns empty on a parse error — `syn` requires **parseable** code (see the module docs; the
/// error-resilient live tier is a different instrument).
#[must_use]
pub fn syn_call_pairs_from_str(src: &str) -> Vec<(String, String)> {
    let Ok(parsed) = syn::parse_file(src) else {
        return Vec::new();
    };
    let mut visitor = CallVisitor::new();
    visitor.visit_file(&parsed);
    visitor.edges
}

/// Reduce a SCIP symbol string to its bare fn name (`…/scan_file().` → `scan_file`) — the projection
/// that lets a bare-name `syn` edge meet a full-symbol SCIP edge. `None` for a non-method symbol.
#[must_use]
pub fn bare_fn_name(symbol: &str) -> Option<&str> {
    let without_suffix = symbol.strip_suffix(").")?;
    let paren_pos = without_suffix.rfind('(')?;
    let before_paren = &without_suffix[..paren_pos];
    let segment = before_paren
        .rsplit(['/', '#'])
        .next()
        .filter(|s| !s.is_empty())?;
    // `[`TypeName`]method` → `method` (strip a backtick-escaped type qualifier).
    if let Some(bracket_end) = segment.rfind(']') {
        let after = &segment[bracket_end + 1..];
        if !after.is_empty() {
            return Some(after);
        }
    }
    // `` `name` `` → `name` (strip backtick escaping of a plain identifier).
    if segment.starts_with('`') && segment.ends_with('`') && segment.len() > 2 {
        return Some(&segment[1..segment.len() - 1]);
    }
    Some(segment)
}

// ── The corroboration primitive ───────────────────────────────────────────────────────────────

/// The outcome of a `syn` corroboration pass over a graph's `Call` edges.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SynCorroboration {
    /// `Call` edges that GAINED the `Syn` witness (their bare-name projection was in the `syn` set):
    /// `{ScipResolved}` → `{Syn, ScipResolved}`, an H0 agreement across independent instruments.
    pub corroborated: usize,
    /// `Call` edges `syn` did NOT see — either genuinely SCIP-only, or whose endpoints did not reduce
    /// to a bare-name pair. The SCIP-only remainder.
    pub scip_only: usize,
}

/// The corroboration primitive: add the `Syn` witness to the `Call` edges `syn` also saw.
///
/// Enriches a graph's `Call` `EdgeFact`s IN PLACE with the [`WitnessSource::Syn`] witness wherever the
/// edge's bare-name projection is present in `syn_pairs`; non-`Call` edges are left untouched.
///
/// The witness set (and thus the *derived* tier) is rebuilt through [`EdgeFact::new`], never stamped, so
/// the tier-is-a-projection invariant holds: `{Syn, ScipResolved}` still projects to `Resolved`
/// (adding `Syn` to a resolved edge never demotes it, and `syn` alone never lifts — the
/// lower-never-corroborates-up law lives in the read layer, not here).
pub fn corroborate_with_syn<S: std::hash::BuildHasher>(
    edges: &mut [EdgeFact],
    syn_pairs: &HashSet<(String, String), S>,
) -> SynCorroboration {
    let mut out = SynCorroboration::default();
    for e in edges.iter_mut() {
        if e.kind != EdgeKind::Call {
            continue;
        }
        let seen_by_syn = matches!(
            (bare_fn_name(&e.src.fq_path.path), bare_fn_name(&e.dst.fq_path.path)),
            (Some(c), Some(d)) if syn_pairs.contains(&(c.to_string(), d.to_string()))
        );
        if seen_by_syn {
            if !e.witnesses.contains(&WitnessSource::Syn) {
                let mut w = e.witnesses.clone();
                w.insert(WitnessSource::Syn);
                let (src, dst, kind) = (e.src.clone(), e.dst.clone(), e.kind);
                *e = EdgeFact::new(src, dst, kind, w);
            }
            out.corroborated += 1;
        } else {
            out.scip_only += 1;
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};

    use super::{SynCorroboration, bare_fn_name, corroborate_with_syn, syn_call_pairs_from_str};
    use crate::base::facts::{EdgeFact, EdgeKind, WitnessSource};
    use crate::node::id::FqPath;
    use crate::node::{CfgSet, IdentityDigest, StromaNodeId};
    use crate::read::ResolutionTier;

    /// A node keyed on a raw SCIP-style symbol (mirrors `scip.rs::node_from_symbol` — the real edge's
    /// `fq_path.path` IS the SCIP symbol string).
    fn node(symbol: &str) -> StromaNodeId {
        StromaNodeId {
            fq_path: FqPath {
                path: symbol.to_string(),
                tier: ResolutionTier::Syntactic,
            },
            identity_digest: IdentityDigest::of_tokens(symbol.as_bytes()),
            cfg_set: CfgSet::default(),
        }
    }

    fn scip_call(src_sym: &str, dst_sym: &str) -> EdgeFact {
        EdgeFact::new(
            node(src_sym),
            node(dst_sym),
            EdgeKind::Call,
            BTreeSet::from([WitnessSource::ScipResolved]),
        )
    }

    #[test]
    fn syn_extracts_caller_callee_pairs_from_source() {
        let src = "fn caller() { callee(); self.method_call(); Foo::assoc(); }";
        let pairs = syn_call_pairs_from_str(src);
        assert!(pairs.contains(&("caller".to_string(), "callee".to_string())));
        assert!(pairs.contains(&("caller".to_string(), "method_call".to_string())));
        assert!(pairs.contains(&("caller".to_string(), "assoc".to_string())));
    }

    #[test]
    fn unparseable_source_yields_no_pairs() {
        // syn requires parseable code — a broken file is silently empty, not a panic.
        assert!(syn_call_pairs_from_str("fn caller( { callee(").is_empty());
    }

    #[test]
    fn bare_fn_name_reduces_scip_symbols() {
        assert_eq!(
            bare_fn_name("ra cargo antigen 0.6.0 scan/scan_file()."),
            Some("scan_file")
        );
        assert_eq!(
            bare_fn_name("ra cargo c 0.1 m/impl#[`Foo`]method()."),
            Some("method")
        );
        assert_eq!(bare_fn_name("ra cargo c 0.1 m/Type#"), None); // not a method symbol
    }

    #[test]
    fn corroboration_adds_the_syn_witness_on_a_bare_name_match() {
        let mut edges = vec![scip_call(
            "x cargo c 0.1 m/caller().",
            "x cargo c 0.1 n/callee().",
        )];
        let syn_pairs: HashSet<(String, String)> =
            HashSet::from([("caller".to_string(), "callee".to_string())]);

        let stats = corroborate_with_syn(&mut edges, &syn_pairs);

        assert_eq!(
            stats,
            SynCorroboration {
                corroborated: 1,
                scip_only: 0
            }
        );
        // {ScipResolved} → {Syn, ScipResolved} — the first non-degenerate witness set.
        assert!(edges[0].witnesses.contains(&WitnessSource::Syn));
        assert!(edges[0].witnesses.contains(&WitnessSource::ScipResolved));
        // …and the derived tier is STILL Resolved (adding Syn never demotes a resolved edge).
        assert_eq!(edges[0].tier, ResolutionTier::Resolved);
    }

    #[test]
    fn an_edge_syn_never_saw_stays_scip_only() {
        let mut edges = vec![scip_call(
            "x cargo c 0.1 m/caller().",
            "x cargo c 0.1 n/callee().",
        )];
        let syn_pairs: HashSet<(String, String)> =
            HashSet::from([("other".to_string(), "thing".to_string())]);

        let stats = corroborate_with_syn(&mut edges, &syn_pairs);

        assert_eq!(
            stats,
            SynCorroboration {
                corroborated: 0,
                scip_only: 1
            }
        );
        assert!(!edges[0].witnesses.contains(&WitnessSource::Syn));
    }

    #[test]
    fn non_call_edges_are_untouched() {
        // A Construct edge is never corroborated by the call-graph syn producer, even if its bare
        // names happen to match — the producer is Call-kind-scoped.
        let mut edges = vec![EdgeFact::new(
            node("x cargo c 0.1 m/caller()."),
            node("x cargo c 0.1 n/callee()."),
            EdgeKind::Construct,
            BTreeSet::from([WitnessSource::ScipResolved]),
        )];
        let syn_pairs: HashSet<(String, String)> =
            HashSet::from([("caller".to_string(), "callee".to_string())]);

        let stats = corroborate_with_syn(&mut edges, &syn_pairs);

        assert_eq!(stats, SynCorroboration::default()); // neither corroborated nor scip_only
        assert!(!edges[0].witnesses.contains(&WitnessSource::Syn));
    }
}
