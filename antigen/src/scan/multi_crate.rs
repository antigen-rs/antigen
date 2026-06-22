//! Cross-crate enumeration + the member-aware multi-crate scan.
//!
//! Extracted from the former monolithic `scan.rs` per ADR-036 (the scan/audit
//! orchestration decomposition). The member-aware path: `enumerate_dep_crate_roots`
//! / `enumerate_workspace_member_roots` (driven by `cargo metadata`) +
//! `resolve_cross_member_lineage_parents` + `scan_workspace_multi_crate` (which
//! unions per-member scans, dedups/checks lineage, and runs the SHARED finalize
//! pass so its synthesis/propagation semantics stay identical to the single-crate
//! path). It composes the lineage + finalize passes + the `scan_workspace` walk;
//! it holds no stop-authority (single-conductor invariant, ADR-036).
//!
//! API-invisible: the public types + fns (`CrateOrigin` / `DepCrateRoot` /
//! `WorkspaceMemberRoot` / `enumerate_*` / `scan_workspace_multi_crate`) are
//! re-exported from the scan root via `pub use` exactly as before;
//! `resolve_cross_member_lineage_parents` is a private helper the scan test
//! module exercises, re-exported `#[cfg(test)]` only.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::{
    MAX_LINEAGE_DEPTH, ParseFailure, ScanCoverage, ScanReport, dedupe_lineage_edges,
    detect_lineage_failures, scan_workspace, synthesize_inherited_presentations,
};

// ============================================================================
// Cross-crate enumeration
//
// Cross-crate scope is enumeration + per-crate scanning, NOT merged cross-crate
// matching. The `addresses()` relation stays file-scoped; module-path-qualified
// `ItemTarget` is an ADR-class decision (ATK-A3-005), not yet decided.
//
// Empirical substrate findings:
//   - `cargo metadata --format-version 1` returns `manifest_path` already
//     resolved per-package — no need to construct paths from cargo home +
//     index hash + crate-version suffix. Path-deps, workspace-internal,
//     and registry deps share the same shape.
//   - `~/.cargo/registry/src/index.crates.io-<hash>/<crate>-<version>/`
//     hosts multiple co-existing versions of the same crate. The
//     `cargo metadata`-driven approach avoids the multi-version problem
//     entirely because cargo dedupes by version per package.
//   - zero `#[antigen(...)]` instances in the wild across the registry
//     (sampled across hundreds of registry deps). The collision question is
//     hypothetical until antigen-stdlib lands.
//
// Sub-clause F (ADR-005): cross-crate antigen declarations are trusted
// inputs; the trust anchor is cargo's own checksum verification chain.
// The trust-model ADR sentence is still in flight.
// ============================================================================

/// How a [`DepCrateRoot`] was sourced — the `cargo metadata` `source` field
/// classified into the buckets the scan tooling cares about.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CrateOrigin {
    /// `source: null` — workspace-internal package or path-dep (cross- or
    /// in-workspace). Source already lives at `manifest_path`'s parent.
    PathOrWorkspace,
    /// `source: "registry+..."` — a crates.io or alt-registry dependency
    /// downloaded into `~/.cargo/registry/src/<index>/<crate>-<version>/`.
    Registry,
    /// `source: "git+..."` — a git dependency cloned into
    /// `~/.cargo/git/checkouts/`. Captures `manifest_path` directly without
    /// path-construction.
    Git,
    /// Anything else cargo returns we don't classify yet (sparse registries,
    /// alternative registry indices, future cargo source kinds). The raw
    /// source string is preserved so consumers can decide to scan it or not.
    Other(String),
}

impl CrateOrigin {
    fn from_source(source: Option<&str>) -> Self {
        match source {
            None => Self::PathOrWorkspace,
            Some(s) if s.starts_with("registry+") => Self::Registry,
            Some(s) if s.starts_with("git+") => Self::Git,
            Some(s) => Self::Other(s.to_string()),
        }
    }
}

/// A single dependency crate's enumerated source root.
///
/// Returned by [`enumerate_dep_crate_roots`]. The `crate_root` directory is
/// the parent of the package's `Cargo.toml`; passing it to [`scan_workspace`]
/// scans the crate's full source tree. The `package_name` and `version`
/// pair uniquely identifies the dep across the workspace's resolved graph.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DepCrateRoot {
    /// Cargo package name (e.g., `"serde"`, `"antigen-fingerprint"`).
    pub package_name: String,
    /// Cargo package version (e.g., `"1.0.219"`).
    pub version: String,
    /// Directory containing the package's `Cargo.toml` — i.e., the crate
    /// root suitable for [`scan_workspace`].
    pub crate_root: PathBuf,
    /// Where this crate came from. See [`CrateOrigin`].
    pub origin: CrateOrigin,
}

/// Enumerate dependency crates resolved by cargo for the workspace at
/// `workspace_root`.
///
/// Runs `cargo metadata --format-version 1 --manifest-path <workspace>/Cargo.toml`
/// in a subprocess, parses the JSON, and returns one [`DepCrateRoot`] per
/// non-workspace-member package. Workspace-internal members are excluded:
/// when [`scan_workspace`] is called on the workspace root, it already
/// covers them.
///
/// `include_path_workspace` controls whether `CrateOrigin::PathOrWorkspace`
/// dependencies (cross-workspace path-deps) are returned. The default for
/// CLI consumers is `false` — these path-deps usually live alongside the
/// workspace and are scanned independently. Set `true` to opt in.
///
/// # Errors
///
/// Returns an `io::Error` if:
/// - the `cargo` binary cannot be invoked (`PATH` or executable issue),
/// - `cargo metadata` exits non-zero (manifest parse error, lock-file out
///   of date, network failure on first resolve, etc.),
/// - the JSON output cannot be parsed.
///
/// In all error cases, the error message preserves the underlying cause
/// (cargo's stderr or the JSON parse error) for diagnostic surfacing.
///
/// # Sub-clause F note (ADR-005 / ADR-017 trust delegation)
///
/// Cross-crate antigen declarations are trusted inputs — the trust anchor
/// is cargo's own checksum verification of registry sources + git revision
/// pinning. The ADR-017 (draft) trust delegation model requires two
/// preconditions before extending trust to a registry path:
///
/// 1. The path is reachable from `cargo metadata`'s resolution graph as
///    a transitive dependency of the consumer workspace.
/// 2. The path's parent directory matches the registry's expected layout
///    (`<index>/<crate>-<version>/`).
///
/// **Both preconditions are satisfied by construction here**: this function
/// is the only public mechanism for enumerating cross-crate scan targets,
/// and every path it returns is sourced from `cargo metadata`'s output.
/// Cargo verifies registry layout itself before populating that output;
/// we inherit cargo's verification rather than re-implementing it.
///
/// **Discipline for future contributors**: do NOT add a non-cargo-metadata
/// path discovery mechanism (e.g., recursive walking of
/// `~/.cargo/registry/src/`) without explicitly adding the layout-matching
/// and reachability checks. Such a path would extend trust outside cargo's
/// resolution chain. Adversarial ATK-A3-007 (in
/// `antigen/tests/atk_a3_fractal_preview.rs`) is the green-test for that
/// scenario.
pub fn enumerate_dep_crate_roots(
    workspace_root: &Path,
    include_path_workspace: bool,
) -> std::io::Result<Vec<DepCrateRoot>> {
    use std::process::Command;

    let manifest_path = workspace_root.join("Cargo.toml");
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--format-version")
        .arg("1")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!(
                    "failed to invoke `cargo metadata` at `{}`: {e} \
                     (is cargo on PATH?)",
                    manifest_path.display()
                ),
            )
        })?;

    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "`cargo metadata` exited with status {} for manifest `{}`: {}",
            output.status,
            manifest_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|e| {
        std::io::Error::other(format!("failed to parse `cargo metadata` JSON output: {e}"))
    })?;

    // Identify workspace-member package IDs so we can exclude them — running
    // scan_workspace on the workspace root already covers these.
    let workspace_members: std::collections::HashSet<String> = metadata
        .get("workspace_members")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let packages = metadata
        .get("packages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            std::io::Error::other(
                "`cargo metadata` output missing `packages` array — unexpected schema",
            )
        })?;

    let mut roots: Vec<DepCrateRoot> = Vec::new();
    for pkg in packages {
        let id = pkg.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        if workspace_members.contains(id) {
            continue;
        }

        let package_name = pkg
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let version = pkg
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let source = pkg.get("source").and_then(|v| v.as_str());
        let manifest_str = pkg.get("manifest_path").and_then(|v| v.as_str());

        let Some(manifest_str) = manifest_str else {
            // No manifest_path — defensive guard. Skip rather than panic;
            // future cargo schemas may surface unexpected shapes.
            continue;
        };
        let manifest = PathBuf::from(manifest_str);
        let Some(crate_root) = manifest.parent().map(Path::to_path_buf) else {
            continue;
        };

        let origin = CrateOrigin::from_source(source);

        // Path-or-workspace deps: `source` is null. Some are workspace
        // members (already excluded above by id); the rest are path-deps to
        // sibling workspaces (e.g., a consuming crate's path-dep to a
        // separately-maintained antigen workspace checkout). Skip by default
        // — those workspaces are normally scanned on their own — but allow
        // opt-in for full transitive coverage.
        if matches!(origin, CrateOrigin::PathOrWorkspace) && !include_path_workspace {
            continue;
        }

        roots.push(DepCrateRoot {
            package_name,
            version,
            crate_root,
            origin,
        });
    }

    Ok(roots)
}

/// A single workspace **member** crate's enumerated source root.
///
/// Returned by [`enumerate_workspace_member_roots`]. The dual of
/// [`DepCrateRoot`]: where `enumerate_dep_crate_roots` deliberately *excludes*
/// workspace members (running [`scan_workspace`] on the root already covers
/// them as one flat tree), this carries the per-member identity that
/// member-aware multi-crate scanning needs.
///
/// `crate_root` is the parent of the member's `Cargo.toml`. `package_name` +
/// `version` form the ADR-017 canonical path `"<name>@<version>"` that each
/// member's declarations are stamped with, making cross-member
/// `#[descended_from]` lineage edges and (ADR-001 C7) cross-crate matching
/// first-class.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WorkspaceMemberRoot {
    /// Cargo package name (e.g., `"antigen"`, `"cargo-antigen"`).
    pub package_name: String,
    /// Cargo package version (e.g., `"0.3.0-beta.1"`).
    pub version: String,
    /// Directory containing the member's `Cargo.toml` — the crate root
    /// suitable for [`scan_workspace`].
    pub crate_root: PathBuf,
}

impl WorkspaceMemberRoot {
    /// The ADR-017 canonical path for this member: `"<name>@<version>"`.
    #[must_use]
    pub fn canonical_path(&self) -> String {
        format!("{}@{}", self.package_name, self.version)
    }
}

/// Enumerate the **member** crates of the Cargo workspace rooted at
/// `workspace_root` — the dual of [`enumerate_dep_crate_roots`].
///
/// Runs `cargo metadata --no-deps --format-version 1` (members only — no
/// dependency resolution, so it is fast and works offline) and returns one
/// [`WorkspaceMemberRoot`] per `workspace_members` entry, each carrying the
/// member's name, version, and crate root.
///
/// A single-crate package (no `[workspace]` table) reports itself as its sole
/// member, so this returns a one-element vec for ordinary crates. That makes
/// member-aware scanning a strict generalization: it degrades to "scan the one
/// crate" rather than special-casing the non-workspace shape.
///
/// # Errors
///
/// Returns an `io::Error` if the `cargo` binary cannot be invoked, if
/// `cargo metadata` exits non-zero (manifest parse error, etc.), or if the
/// JSON cannot be parsed. The underlying cause (cargo stderr or the JSON parse
/// error) is preserved for diagnostic surfacing.
///
/// # Sub-clause F note (ADR-005)
///
/// Members are first-party code (the adopter's own workspace), not a trust
/// boundary — unlike the registry/git deps [`enumerate_dep_crate_roots`]
/// handles. The only trust assumption is that `cargo metadata`'s
/// `workspace_members` list and `manifest_path`s are accurate, which is
/// cargo's own invariant.
pub fn enumerate_workspace_member_roots(
    workspace_root: &Path,
) -> std::io::Result<Vec<WorkspaceMemberRoot>> {
    use std::process::Command;

    let manifest_path = workspace_root.join("Cargo.toml");
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--no-deps")
        .arg("--format-version")
        .arg("1")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .output()
        .map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!(
                    "failed to invoke `cargo metadata` at `{}`: {e} (is cargo on PATH?)",
                    manifest_path.display()
                ),
            )
        })?;

    if !output.status.success() {
        return Err(std::io::Error::other(format!(
            "`cargo metadata --no-deps` exited with status {} for manifest `{}`: {}",
            output.status,
            manifest_path.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        )));
    }

    let metadata: serde_json::Value = serde_json::from_slice(&output.stdout).map_err(|e| {
        std::io::Error::other(format!("failed to parse `cargo metadata` JSON output: {e}"))
    })?;

    // `workspace_members` is the authoritative set of member package IDs. With
    // `--no-deps`, `packages` already contains *only* the members, but we still
    // intersect against `workspace_members` for defensiveness against future
    // cargo schemas that might include more.
    let member_ids: std::collections::HashSet<String> = metadata
        .get("workspace_members")
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();

    let packages = metadata
        .get("packages")
        .and_then(|v| v.as_array())
        .ok_or_else(|| {
            std::io::Error::other(
                "`cargo metadata` output missing `packages` array — unexpected schema",
            )
        })?;

    let mut roots: Vec<WorkspaceMemberRoot> = Vec::new();
    for pkg in packages {
        let id = pkg.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        // Keep only declared members. If `workspace_members` is somehow empty
        // (older cargo, odd manifest), fall back to "every package `--no-deps`
        // returned is a member" — which is the documented `--no-deps` contract.
        if !member_ids.is_empty() && !member_ids.contains(id) {
            continue;
        }

        let package_name = pkg
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let version = pkg
            .get("version")
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_string();
        let manifest_str = pkg.get("manifest_path").and_then(|v| v.as_str());

        let Some(manifest_str) = manifest_str else {
            continue;
        };
        let Some(crate_root) = PathBuf::from(manifest_str).parent().map(Path::to_path_buf) else {
            continue;
        };

        roots.push(WorkspaceMemberRoot {
            package_name,
            version,
            crate_root,
        });
    }

    // Deterministic order (cargo's package order is stable but unspecified;
    // sort by name so merged reports + diagnostics are reproducible).
    roots.sort_by(|a, b| a.package_name.cmp(&b.package_name));
    Ok(roots)
}

/// Merge `other`'s records into `self`, appending every declaration/site
/// vector and summing the file-scan counts.
///
/// Used by [`scan_workspace_multi_crate`] to union per-member
/// [`ScanReport`]s into one. Each source report must already have its
/// `canonical_path`s stamped (member-aware) **before** merging, so that
/// identity is preserved across the union — `merge` does not stamp.
///
/// `merge` is a pure concatenation: it does **not** re-run lineage
/// propagation or fingerprint synthesis (those run once on the merged whole
/// so cross-member edges and fingerprints are resolved across the union, not
/// per-member).
impl ScanReport {
    pub(crate) fn merge(&mut self, mut other: Self) {
        self.antigens.append(&mut other.antigens);
        self.presentations.append(&mut other.presentations);
        self.immunities.append(&mut other.immunities);
        self.tolerances.append(&mut other.tolerances);
        self.lineage_edges.append(&mut other.lineage_edges);
        self.deferred_defenses.append(&mut other.deferred_defenses);
        self.convergent_evidences
            .append(&mut other.convergent_evidences);
        self.recurrent_declarations
            .append(&mut other.recurrent_declarations);
        self.mucosal_declarations
            .append(&mut other.mucosal_declarations);
        self.prescriptive_declarations
            .append(&mut other.prescriptive_declarations);
        self.defenses.append(&mut other.defenses);
        self.generates_declarations
            .append(&mut other.generates_declarations);
        self.files_scanned += other.files_scanned;
        self.parse_failures.append(&mut other.parse_failures);
    }
}

/// Re-resolve each lineage edge's `parent_canonical_path` to the member crate
/// that actually *declares* the parent antigen.
///
/// **Why this is the heart of cross-crate `#[descended_from]`.** Per-member
/// stamping ([`ScanReport::stamp_canonical_path`]) stamps *both* endpoints of
/// every edge to the member the edge was found in — correct for the child
/// (the `#[descended_from]` site lives there) but wrong for a parent declared
/// in a *different* member. Left unfixed, the propagation walk keys the parent
/// by `(parent_name, wrong_member_path)`, fails the antigen-index lookup, and
/// treats the edge as orphaned — so a cross-member ancestor's presentations
/// never propagate to the descendant. This pass fixes the parent endpoint by
/// looking up where `parent_name` is genuinely declared among the merged
/// antigens, making cross-member lineage first-class.
///
/// This is **pure structural identity resolution** — "where is this type
/// declared" — not a semantic `addresses()` verdict. The semantic cross-crate
/// matching question (does an `X` declared in one member satisfy a
/// `#[presents(X)]` in another) is ADR-class scope tracked separately
/// (ADR-001 C7 activation / ATK-A3-005).
///
/// Resolution rule, by parent-name declaration multiplicity among members:
/// - **Exactly one member declares `parent_name`** → re-stamp the edge's
///   `parent_canonical_path` to that member's canonical path. Unambiguous.
/// - **Zero members declare it** → leave the edge unchanged; it surfaces as
///   an orphaned edge ([`ScanReport::orphaned_lineage_edges`]) as before.
/// - **Two or more members declare the same bare name** → ambiguous; leave
///   the edge's parent endpoint as the child's own member (the conservative
///   intra-member assumption) and record one [`ParseFailure`] naming the
///   collision, so the ambiguity is explicit (ADR-004) rather than silently
///   resolved to an arbitrary member.
pub fn resolve_cross_member_lineage_parents(report: &mut ScanReport) {
    use std::collections::HashMap;

    // bare parent name -> set of member canonical paths declaring it.
    let mut decl_members: HashMap<String, std::collections::BTreeSet<String>> = HashMap::new();
    for a in &report.antigens {
        if let Some(cp) = a.canonical_path.as_deref() {
            decl_members
                .entry(a.type_name.clone())
                .or_default()
                .insert(cp.to_string());
        }
    }

    let mut ambiguity_failures: Vec<ParseFailure> = Vec::new();
    for e in &mut report.lineage_edges {
        let Some(members) = decl_members.get(&e.parent) else {
            // Parent declared in no member — leave as-is; orphaned-edge
            // detection downstream surfaces it.
            continue;
        };
        match members.len() {
            0 => {},
            1 => {
                // Unambiguous: re-stamp parent endpoint to the declaring member.
                let target = members.iter().next().expect("len==1");
                if e.parent_canonical_path.as_deref() != Some(target.as_str()) {
                    e.parent_canonical_path = Some(target.clone());
                }
            },
            _ => {
                // Ambiguous cross-member name collision. Keep the conservative
                // intra-member parent endpoint (whatever stamping set) and make
                // the collision explicit.
                ambiguity_failures.push(ParseFailure {
                    file: e.file.clone(),
                    error: format!(
                        "#[descended_from({parent})] on `{child}` is ambiguous across the \
                         workspace: `{parent}` is declared in {n} members ({members}); \
                         cross-member lineage parent left unresolved (qualify the parent path \
                         to disambiguate)",
                        parent = e.parent,
                        child = e.child,
                        n = members.len(),
                        members = members.iter().cloned().collect::<Vec<_>>().join(", "),
                    ),
                });
            },
        }
    }
    report.parse_failures.extend(ambiguity_failures);
}

/// Re-resolve each reference record's `canonical_path` to the member that
/// actually *declares* the antigen it addresses — the Layer-2 cross-crate
/// `addresses()` resolution (ADR-017 Amendment 1, ATK-A3-005), the verdict-side
/// sibling of [`resolve_cross_member_lineage_parents`].
///
/// **Why this closes `DelegateCrossCrateResolutionGap`.** Per-member stamping
/// ([`ScanReport::stamp_canonical_path`]) stamps every reference record
/// (`#[presents]` / `#[defended_by]` / `#[immune]` / `#[antigen_tolerance]`)
/// with the canonical path of the member it was *found in*. But each record's
/// `canonical_path` is contractually the declaration site of the *antigen it
/// addresses* (see [`Presentation::canonical_path`](crate::scan::Presentation::canonical_path) et al.), not its own
/// location. For an intra-member reference the two coincide; for a genuine
/// cross-member reference — a `#[presents(crate_a::X)]` living in crate B — the
/// stamp puts `B@v` on a record whose semantic key should be `A@v`. Left
/// unfixed, [`defense_addresses`](crate::scan::defense_addresses) / [`canonical_paths_match`](crate::scan::canonical_paths_match) compare
/// `Some("B@v")` against the antigen's `Some("A@v")` and FAIL to match a
/// legitimate cross-crate defense (and a cross-crate presents-site reads as
/// `antigen_known = false`). This pass re-stamps the reference endpoint to the
/// declaring member, making cross-member `addresses()` first-class.
///
/// This is **pure structural identity resolution** — "where is this antigen
/// declared" — not a verdict. The verdict layer reads the resolved
/// `canonical_path`: a reference that resolves matches its antigen; one that
/// resolves to no member is the out-of-frame third value (ADR-017 Amendment 1
/// clause 1 — an unresolvable cross-crate reference is a loud GAP, never a
/// silent pass). This pass does the resolution; the audit reads the result.
///
/// Resolution rule, by antigen-name declaration multiplicity among members
/// (identical to the lineage-parent rule, so the two passes cannot drift):
/// - **Exactly one member declares the antigen** → re-stamp the record's
///   `canonical_path` to that member (a no-op for an intra-member reference;
///   the real work for a cross-member one). Unambiguous.
/// - **Zero members declare it** → leave the record unchanged. It stays keyed
///   to its own member; the antigen is unknown in the workspace, so the
///   `addresses()` match fails and the audit surfaces it (out-of-frame for an
///   explicit reference; the ADR-017-Amd1 resolution gate). Canonical-path-keyed
///   trust means this never silently cross-satisfies.
/// - **Two or more members declare the same bare antigen name** → ambiguous;
///   leave the record on its own member (conservative intra-member assumption)
///   and record one [`ParseFailure`] naming the collision, so a same-name
///   cross-crate collision is explicit (ADR-004) rather than silently resolved
///   to an arbitrary member (ADR-017 Amendment 1 clause 2).
pub fn resolve_cross_member_addresses(report: &mut ScanReport) {
    use std::collections::{BTreeMap, BTreeSet};

    // bare antigen type name -> set of member canonical paths declaring it.
    // BTreeSet keeps the collision diagnostic deterministic. (Same index the
    // lineage-parent pass builds — kept local so each pass is self-contained
    // and the two cannot read a stale shared map.)
    let mut decl_members: BTreeMap<&str, BTreeSet<&str>> = BTreeMap::new();
    for a in &report.antigens {
        if let Some(cp) = a.canonical_path.as_deref() {
            decl_members
                .entry(a.type_name.as_str())
                .or_default()
                .insert(cp);
        }
    }

    // Collisions collected once per antigen name so a same-name cross-member
    // ambiguity touched by N references emits ONE diagnostic, not N. Built up
    // by the per-family re-stamp loop below, drained into `parse_failures` after.
    let mut collisions: BTreeMap<&str, String> = BTreeMap::new();

    // Re-stamp every reference record (`presents` / `defended_by` / `immune` /
    // `tolerance`) whose addressed antigen resolves to exactly one declaring
    // member; record a collision for an ambiguous (≥2-member) name; leave a
    // zero-declarer reference unchanged (it stays out-of-frame at the verdict).
    // The four families share the identical rule — `restamp` keeps them in
    // lockstep so they cannot drift. `&decl_members` is reborrowed each call so
    // the disjoint mutable borrow of each `report` field is sound.
    macro_rules! restamp_family {
        ($field:ident) => {
            for rec in &mut report.$field {
                let Some(members) = decl_members.get(rec.antigen_type.as_str()) else {
                    continue; // antigen declared in no member — leave keyed to its own.
                };
                let mut it = members.iter();
                match (it.next(), it.next()) {
                    // Exactly one declaring member → re-stamp to it (no-op when
                    // the record already carries that member's path).
                    (Some(&target), None) => {
                        if rec.canonical_path.as_deref() != Some(target) {
                            rec.canonical_path = Some(target.to_owned());
                        }
                    }
                    // Two or more → ambiguous; leave the record on its own member
                    // and record the collision once.
                    (Some(_), Some(_)) => {
                        let name = rec.antigen_type.as_str();
                        collisions.entry(name).or_insert_with(|| {
                            format!(
                                "cross-crate addresses() for `{name}` is ambiguous across the \
                                 workspace: `{name}` is declared in {n} members ({members}); the \
                                 reference is left keyed to its own member and reads as \
                                 out-of-frame (qualify the antigen path to disambiguate)",
                                n = members.len(),
                                members = members.iter().copied().collect::<Vec<_>>().join(", "),
                            )
                        });
                    }
                    // Empty set is impossible (entries are only created on insert)
                    // — treat as leave-unchanged for total coverage.
                    (None, _) => {}
                }
            }
        };
    }
    restamp_family!(presentations);
    restamp_family!(defenses);
    restamp_family!(immunities);
    restamp_family!(tolerances);

    // Emit one ParseFailure per colliding antigen name (file = workspace-root
    // marker; the collision is a workspace-level fact, not a single-file one).
    for error in collisions.into_values() {
        report.parse_failures.push(ParseFailure {
            file: PathBuf::from("<workspace>"),
            error,
        });
    }
}

/// Member-aware multi-crate workspace scan — the v0.3 cornerstone.
///
/// Where [`scan_workspace`] walks `root` as one **flat** tree (every record
/// shares the same — usually `None` — `canonical_path`, so member-crate
/// boundaries are lost), this:
///
/// 1. enumerates the workspace's member crates via
///    [`enumerate_workspace_member_roots`];
/// 2. runs [`scan_workspace`] on each member's crate root independently;
/// 3. stamps each member's records with that member's ADR-017 canonical path
///    (`"<name>@<version>"`) so identity is per-member;
/// 4. unions the per-member reports;
/// 5. re-resolves cross-member `#[descended_from]` parent endpoints
///    (`resolve_cross_member_lineage_parents`); and
/// 6. runs the synthesis + lineage-propagation finalize **once over the
///    merged whole**, so cross-member lineage propagation and fingerprint
///    synthesis see all members at once.
///
/// The result is a single [`ScanReport`] in which a `#[presents]` /
/// `#[antigen]` / `#[descended_from]` carries the identity of the member it
/// lives in, and a `#[descended_from(Parent)]` in member A resolves to a
/// `Parent` declared in member B. This is the substrate that closes the
/// cross-crate-resolution gaps documented for v0.2 (e.g.
/// [`crate::stdlib::agentic_coordination::DelegateCrossCrateResolutionGap`]).
///
/// Per-member stamping is **non-overwriting**: a record that a nested scan
/// already stamped keeps its stamp (see [`ScanReport::stamp_canonical_path`]).
///
/// The merged report carries a [`ScanCoverage`] (`scan_coverage`) recording
/// every enumerated member and the subset actually scanned — the substrate for
/// ignorance detection (an enumerated-but-unscanned member is a region where
/// `#[presents]` sites go unseen).
///
/// # Errors
///
/// Returns the `io::Error` from [`enumerate_workspace_member_roots`] if member
/// enumeration fails (cargo not on PATH, manifest parse error, etc.) — that is
/// the only fatal case. A per-member scan that fails records the failure in
/// [`ScanReport::parse_failures`] and leaves the member out of
/// `scan_coverage.scanned_members` (an ignorance frontier), rather than
/// aborting the whole scan.
pub fn scan_workspace_multi_crate(workspace_root: &Path) -> std::io::Result<ScanReport> {
    let members = enumerate_workspace_member_roots(workspace_root)?;

    // Coverage record: every enumerated member, and the subset actually scanned.
    // The complement is the ignorance frontier (members whose `#[presents]`
    // sites were never seen). In the happy path the two sets are equal.
    let mut enumerated_members: Vec<String> = members
        .iter()
        .map(WorkspaceMemberRoot::canonical_path)
        .collect();
    let mut scanned_members: Vec<String> = Vec::with_capacity(members.len());

    let mut merged = ScanReport::default();
    for member in &members {
        // A per-member scan that *errors* must not abort the whole multi-crate
        // scan — that would convert one unscannable member into a total
        // failure. Instead, record the error in `parse_failures` and leave the
        // member OUT of `scanned_members`, so it surfaces as an unscanned
        // (ignored) member in the coverage record. (`scan_workspace` currently
        // never returns Err, but the coverage semantics must be honest if that
        // changes — an unscannable member is an ignorance frontier, not a crash.)
        let mut member_report = match scan_workspace(&member.crate_root, None) {
            Ok(r) => r,
            Err(e) => {
                merged.parse_failures.push(ParseFailure {
                    file: member.crate_root.clone(),
                    error: format!(
                        "member `{}` could not be scanned ({e}); its sites are UNSEEN \
                         (ignorance frontier), not defended",
                        member.canonical_path()
                    ),
                });
                continue;
            },
        };
        // Stamp this member's records with its own canonical path BEFORE
        // merging, so cross-member identity survives the union.
        member_report.stamp_canonical_path(&member.canonical_path());
        merged.merge(member_report);
        scanned_members.push(member.canonical_path());
    }

    enumerated_members.sort();
    scanned_members.sort();
    merged.scan_coverage = Some(ScanCoverage {
        enumerated_members,
        scanned_members,
    });

    // Cross-member parent re-resolution must run on the merged whole — only
    // there are all members' antigen declarations visible to resolve a parent
    // that lives in a different member than its `#[descended_from]` child.
    resolve_cross_member_lineage_parents(&mut merged);

    // Layer-2 cross-crate addresses() resolution (ADR-017 Amendment 1): re-stamp
    // every reference record (presents / defended_by / immune / tolerance) whose
    // addressed antigen is declared in a *different* member than the record was
    // found in, so cross-member `addresses()` matches. Like the lineage pass,
    // this needs the merged whole (all members' antigen declarations visible)
    // and must run BEFORE propagation/audit read the canonical_paths. Closes
    // DelegateCrossCrateResolutionGap.
    resolve_cross_member_addresses(&mut merged);

    // Re-dedup edges across the union: an edge collected once per member could
    // now collapse only if its four-tuple key matches, but cross-member
    // re-resolution may have made two members' edges (same child+parent bare
    // names) point at the same parent canonical path. Dedup keeps the ADR-018
    // edge-identity invariant on the merged graph + emits collapse diagnostics.
    let (deduped_edges, dedup_failures) = dedupe_lineage_edges(&merged.lineage_edges);
    merged.lineage_edges = deduped_edges;
    merged.parse_failures.extend(dedup_failures);
    let lineage_failures = detect_lineage_failures(&merged.lineage_edges, MAX_LINEAGE_DEPTH);
    merged.parse_failures.extend(lineage_failures);

    // ---- Merged-whole lineage propagation ONLY ----
    //
    // Each member's `scan_workspace` already ran its own intra-member
    // fingerprint-synthesis pass, so the merged report's presentations already
    // include every member's fingerprint matches. We must NOT re-run synthesis
    // over the union here — doing so double-counts every intra-member match
    // (and would additionally produce *cross-member* fingerprint matches, which
    // are ADR-001 C7 / Layer-2 scope, not member-aware identity scope).
    //
    // What DOES need the merged whole is lineage propagation: a cross-member
    // `#[descended_from(Parent)]` edge only resolves after the union makes both
    // endpoints' antigen declarations visible (and after
    // `resolve_cross_member_lineage_parents` re-stamped the parent endpoint).
    // So we run only that pass — the same `synthesize_inherited_presentations`
    // the single-crate `finalize_report` runs, but without the synthesis pass
    // that precedes it.
    synthesize_inherited_presentations(&mut merged);

    Ok(merged)
}
