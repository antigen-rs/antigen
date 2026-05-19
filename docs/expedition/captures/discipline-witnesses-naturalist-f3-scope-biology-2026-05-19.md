# Capture — Naturalist on F3 Scope Biology

> **Date**: 2026-05-19
> **Author**: team-naturalist
> **Status**: append-only capture
> **What this addresses**: aristotle F3 frontier flag for naturalist — does
> biology have a discipline-scope axis matching site/file/module/crate/workspace?
> (Aristotle team-pass capture 2026-05-18, section "F3 — Per-site discipline →
> per-site substrate is NOT universally true")
> **Relation to framing-call capture**: independent; framing-call was about
> recognition-vs-evidence role distinction. This is about scope hierarchy.
> Both biology-questions; no architecture overreach.

---

## The question

Aristotle elevated discipline-scope to a first-class dimension: `site | file |
module | crate | workspace`, with substrate-currency following discipline-
scope. Aristotle's analysis stopped at: "scope is meaningful and multi-
granular; the primitive needs the granularity dimension named explicitly."

Open question carried for naturalist: does biology validate this taxonomy?
And if so, does it predict something more than aristotle named?

---

## Yes, biology has scope — and more

Immunity operates at multiple structural scales, each with its own substrate:

- **Cellular**: BCR on cell membrane; intracellular cytokine signaling;
  phagocyte engulfing — single-cell recognition and response
- **Tissue**: tissue-resident memory cells (Trm); mucosal immunity localized
  to gut/lung; tissue-specific inflammatory milieu
- **Organ**: spleen filters blood-borne antigens; bone marrow houses plasma
  cell niches; liver Kupffer cells clear bacterial debris; GALT (gut-
  associated lymphoid tissue) handles intestinal antigens
- **Systemic**: serum antibody concentrations; circulating memory cell pools;
  complement system as soluble surveillance; vaccine-induced population-level
  immunity

Mapping to aristotle's taxonomy:

| Aristotle scope | Biological scope analog | Substrate at this scope |
|---|---|---|
| site (function/struct) | cellular | single cell's BCR + recognition machinery |
| file | sub-tissue (cell aggregate, e.g., hepatic lobule) | small coordinated cell population |
| module/package | tissue / organ | functional unit with coordinated machinery |
| crate | organ system | bone marrow / spleen / GALT — coordinated organ-systems |
| workspace | systemic | serum-circulating + population-level immunity |

The mapping is approximate at the middle scales (biology has continuous
hierarchy, not 5 discrete levels), but the extremes (site ↔ cellular,
workspace ↔ systemic) map cleanly.

---

## The structural prediction aristotle didn't reach

Aristotle's analysis treated scope as a property of WHERE the substrate
lives. The biology adds a sharper prediction: scope is ALSO a property of
WHAT KIND OF CLAIM IT MAKES.

### Secondary lymphoid organs as coordination structures

Lymph nodes, spleen, Peyer's patches, MALT (mucosa-associated lymphoid
tissue) are tissue/organ-level structures that exist SPECIFICALLY to
coordinate cellular-level immunity. They are not just "containers for many
cells" — they are *architectural substrate* that shapes how cellular
immunity operates:

- Lymph nodes drain peripheral regions via afferent lymph vessels
- They bring naive lymphocytes into structured contact with antigen-presenting
  cells in segregated T-cell zones and B-cell follicles
- Germinal centers form within lymph nodes — substructures where B-cells
  undergo affinity maturation
- High endothelial venules, follicular dendritic cells, T-zone vs B-zone
  segregation — the tissue ARCHITECTURE itself is immunity machinery

This means: coarser-scope immunity is NOT just aggregation of finer-scope
immunity. It has structural roles that finer-scope immunity can't have.

### What this predicts for antigen

**Prediction 1: coarser-scope antigens can have a different KIND of claim.**

A site-scope antigen: "this function satisfies discipline X."
A workspace-scope antigen could be: "all functions in this workspace that
present X must have signers including someone from role Y."

That's not an aggregation — it's a *quantified constraint over the population
of site-scope claims*. Biology has this: T-regulatory cells suppress
self-reactive B-cells across the whole organism (systemic-scope constraint
operating on cellular-scope cells); central tolerance in thymic education
shapes which T-cells the organism can ever have (organism-scope filter
operating on cellular-scope generation).

**Prediction 2: coarser-scope substrate may include the architecture of
finer-scope substrate.**

A workspace sidecar might attest not just to a workspace-level fact but to
facts ABOUT the per-site sidecars: "all per-site sidecars in this workspace
are within freshness window of N days" or "all per-site claims are signed by
someone in the math-researcher role."

Biology has this: a lymph node's structural integrity (T-zone, B-zone,
follicular dendritic cells properly positioned) is itself substrate that
affects whether germinal centers can form. The architecture IS substrate.

**Prediction 3: coarser-scope sidecars should be at coordination-points,
not arbitrary higher-level locations.**

Aristotle suggested: workspace-scope → `.attest/` at workspace root. The
biology suggests something more specific: workspace-scope substrate should
live at COORDINATION SITES — `.github/CODEOWNERS` if it's about role-routing;
`Cargo.lock` if it's about dependency review; workspace `Cargo.toml`'s
`[workspace.metadata.antigen]` if it's about workspace-wide invariants.

These aren't arbitrary — they're the existing substrate that ALREADY
COORDINATES across sites. Biology's lymph nodes aren't at random tissue
locations; they're at confluences where afferent lymph vessels meet. Software
coordination points (CODEOWNERS, Cargo.lock, workspace metadata) are the
analog — they're where workspace-wide concerns already aggregate naturally.

---

## What aristotle's analysis named and what it missed

**Named correctly**:
- Scope is a first-class dimension (cellular ≠ systemic in biology too)
- Substrate-currency follows scope (cellular substrate = cell; systemic
  substrate = serum + circulating pools)
- Fingerprint dimension scales with scope (item-level → file-level →
  workspace-level)

**Missed (this capture surfaces)**:
- Scope is also a property of CLAIM KIND, not just SUBSTRATE LOCATION
- Coarser scopes can encode coordination structures across finer scopes
- Coarser-scope sidecar locations should follow existing coordination
  substrate (CODEOWNERS, Cargo.toml workspace metadata), not arbitrary
  higher-level files

---

## Implication for v0.1 vs v0.2+

**For v0.1**: site-scope and file-scope only. Biology supports this — start
with cellular-level recognition and add coordination scopes as the system
matures. Aristotle's deferral to v0.2 for coarser-than-file is biology-
aligned.

**For v0.2+**: when workspace-scope lands, it should NOT be just "site-scope
but at higher level." It should encode the claim-kind difference. Concrete
examples:

```rust
// v0.2+ candidate syntax (illustrative; pathmaker owns syntax)
#[workspace_immune(AllUnsafeHasSafetyDocs, requires = quantified_over_sites(
    sites_presenting(UnsafeWithoutSafetyDoc),
    each_satisfies = signers(required = ["security-team"])
))]
```

Sidecar at workspace coordination-point: `.github/workspace.attest.json` or
extension of CODEOWNERS, NOT a dummy `src/workspace_invariants.rs` (which
would be a workaround for the v0.1 site-only assumption).

Predicate language extends to include site-quantification primitives:
`sites_presenting(X)`, `for_all(sites, predicate)`, `at_least_one(sites,
predicate)`. These are workspace-scope leaves; v0.1 doesn't need them.

---

## Open question for the team

Does the existing `cargo-antigen scan` model support multi-scope queries, or
does it flatten everything to per-site? If scan is flat per-site,
workspace-scope antigens would need scan to learn a new query shape. That's
substrate-cost worth surfacing before v0.2 commits to workspace-scope.

I haven't substrate-grep'd this; deferring to scout / pathmaker who know the
scan internals. Flagging it here as the load-bearing v0.2 dependency.

---

## What this is NOT

- Not a v0.1 design recommendation. v0.1 ships site + file scope only.
- Not a critique of aristotle's F3 analysis. F3 elevated scope correctly;
  this capture extends it with claim-kind distinction and coordination-
  substrate location guidance.
- Not architecture-prescription (per framing-call correction). Biology
  predicts the STRUCTURE (scope is multi-dimensional, includes coordination)
  but not the IMPLEMENTATION CHOICES that carry it.

---

## Posture

This is pure biology-validation work — does biology validate aristotle's
scope-as-first-class-dimension? Yes, and with a sharper prediction (scope is
also claim-kind, not just substrate-location). The prediction is structural,
not architectural. Pathmaker / aristotle decide implementation.

Per `feedback_clean_without_snag_is_argument_mode.md`: snag-feel fired on
the question "does biology really have scope?" — substrate-checked against
known immunology (cellular/tissue/organ/systemic is textbook). Answer is
clean and substrate-grounded; biology speaks in prediction-mode, with the
extension beyond aristotle's frame being the genuinely novel contribution.

Sources:
- Standard immunology references (lymphoid organ structure;
  cellular/tissue/organ/systemic scales of immunity; Treg systemic
  suppression). Did not require new web research — this is
  textbook-immunology validation of an aristotle finding.
