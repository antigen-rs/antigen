//! Cargo subcommand for antigen.
//!
//! Provides the `cargo antigen <subcommand>` CLI for working with antigen
//! declarations in a Rust workspace. After `cargo install cargo-antigen`,
//! invoke as `cargo antigen scan` / `cargo antigen audit` from any directory
//! containing a `Cargo.toml`.
//!
//! ## Subcommands (v0.1.0-rc.1)
//!
//! - `cargo antigen scan` — walk the workspace, extract antigen-related
//!   attributes, report unaddressed presentations + tolerated sites + parse
//!   failures. Supports `--include-deps` for cross-crate enumeration
//!   (ADR-017) and `--format json` for machine-readable output.
//!   See `cargo antigen scan --help` for the full surface.
//! - `cargo antigen audit` — validate each immunity declaration's witness
//!   identifier against the workspace function index. Classifies witnesses
//!   by `WitnessTier` (`Reachability` / `Execution` / `FormalProof` / `None`) and
//!   emits state-7 diagnostics for inherited presentations lacking
//!   re-attestation (ADR-018). `--strict` gates CI on tier minimums +
//!   state-7 absence.
//! - `cargo antigen attest` — manage `.attest/<Antigen>.json` substrate-witness
//!   sidecars (ADR-019). Subcommands: `scaffold` (create sidecar), `sign`
//!   (add signer entry), `check` (evaluate predicate against sidecar).
//!   Further subcommands (`delta`, `oracle`, `list`, `move`, `migrate`, `gc`)
//!   are design-phase stubs hidden from `--help`.
//! - `cargo antigen tolerate` — manage tolerance-ratification sidecars (ADR-019
//!   §tolerance tier). Subcommands: `scaffold`, `sign`, `check`, `list` (stubs
//!   pending adopter feedback).
//!
//! Two additional subcommands (`new`, `vaccinate`) exist as design-phase
//! stubs but are hidden from `--help` until they ship beyond stub state.
//!
//! ## See also
//!
//! - [`antigen`](https://docs.rs/antigen) — the library crate with the
//!   attribute macros and the `scan` + `audit` modules this binary drives.
//! - The project's
//!   [`docs/tutorial.md`](https://github.com/antigen-rs/antigen/blob/main/docs/tutorial.md)
//!   for the narrative walkthrough.

use std::path::{Path, PathBuf};
use std::process::ExitCode;

use antigen::{audit, presents, scan};
use clap::{Parser, Subcommand};

/// Cargo subcommand for antigen.
#[derive(Debug, Parser)]
#[command(name = "cargo-antigen", bin_name = "cargo", version)]
struct CargoCli {
    #[command(subcommand)]
    command: CargoSubcommand,
}

#[derive(Debug, Subcommand)]
enum CargoSubcommand {
    /// The "antigen" subcommand of cargo.
    Antigen(AntigenCli),
}

#[derive(Debug, Parser)]
#[command(version)]
struct AntigenCli {
    #[command(subcommand)]
    command: AntigenSubcommand,
}

#[derive(Debug, Subcommand)]
enum AntigenSubcommand {
    /// Scan the workspace for antigen presentations and report unaddressed ones.
    Scan(ScanArgs),
    /// Propose a candidate failure-class fingerprint from a cluster of marked
    /// sites (the keystone goes live — ADR-045/047/048, the learning core).
    ///
    /// Re-acquires the `#[dread]`/`#[aura]`-marked cluster under `--cluster-root`,
    /// anti-unifies it into a draft, and routes it through the B-gate (GATE-G)
    /// against the OPERATOR-supplied `--clean-root` corpus. Renders a **ratifiable
    /// suggestion** (observe-don't-declare, ADR-044) — never an auto-`#[presents]`
    /// or an auto-named class. A `propose` run leaves the source tree byte-unchanged;
    /// the machine supplies the syntactic half, a human ratifies the semantic half.
    Propose(ProposeArgs),
    /// Scaffold a new antigen declaration (design phase).
    ///
    /// Hidden from `--help` output until the command ships beyond its
    /// design-phase stub. Stub message remains for users who discover the
    /// name via docs or history. The guiding principle: show new users the
    /// surface that works, not the surface that doesn't.
    #[command(hide = true)]
    New {
        /// kebab-case name for the new antigen
        name: String,
    },
    /// Apply known immunity pattern across a structural family (design phase).
    ///
    /// Hidden from `--help` output until the command ships beyond its
    /// design-phase stub (see `New` above for the same discipline).
    #[command(hide = true)]
    Vaccinate {
        /// Antigen type to apply
        antigen: String,
        /// Pattern (glob or path) describing target sites
        pattern: String,
    },
    /// Comprehensive immunity coverage report — witness resolution and tier validation.
    Audit(AuditArgs),
    /// Manage `.attest/<Antigen>.json` substrate-witness sidecars (ADR-019).
    Attest(AttestCli),
    /// Manage tolerance-ratification sidecars (ADR-019 §tolerance tier).
    Tolerate(TolerateCli),
    /// Manage Oracle artifact-class records (ADR-021 §D3).
    ///
    /// Oracles are structurally-distinguished discipline artifacts with lifecycle
    /// state (Draft → Complete → Deprecated/Retired/Revoked), dedicated stewards,
    /// and provenance tracking. `cargo antigen oracle` manages the Oracle JSON
    /// records and their state-machine transitions.
    Oracle(OracleCli),
    /// Drive Supply-Chain Defense Family verifications (ADR-025).
    ///
    /// Eight subcommands cover dep-pinning, dep-attestation, content-hash
    /// recording/verification, maintainer snapshots, and (v0.4+ stubs)
    /// sandbox/behavioral checks. The `cargo antigen verify` family backs
    /// the supply-chain stdlib antigens declared in
    /// `antigen::stdlib::supply_chain` — `cargo antigen audit` walks
    /// `requires = ...` substrate-witness predicates and routes them
    /// through these handlers.
    Verify(VerifyCli),
    /// Drive VCS-Information-Loss Family observations (ADR-026).
    ///
    /// Observation subcommands (v0.2): `scan` (surface VCS-info-loss risk
    /// across the repo), `check-commit` (evaluate one commit's trailers
    /// against the rollback-triage chain), `attest` (record a branch-archive
    /// or rollback-triage attestation sidecar), `rollback-prepare` (scaffold
    /// a triage-commit before a rollback), `branch-archive` (attest a branch
    /// deletion). These OBSERVE the git substrate via the
    /// `antigen::vcs_witness` evaluators; they do not install hooks.
    /// `install-hooks` / `install-server-hooks` (the enforcement layer that
    /// executes the detection decision tree) defer to v0.2.x post-ADR-026
    /// Amendment 4 ratification per the witness-layer-independence split.
    Vcs(VcsCli),
    /// Map mucosal trust boundaries across the workspace (ADR-027 + Amd 1).
    ///
    /// Walks the scan report's mucosal declarations and runs the
    /// `audit_mucosal` pipeline (incl. the Change-5 three-tier delegate
    /// kind-matching diagnosis). `--undefended` lists boundaries with no
    /// `#[mucosal]` / `#[mucosal_tolerant]` declaration; `--tolerant` lists
    /// the active-tolerance boundaries for periodic reviewer audit;
    /// `--kind <kind>` filters to one `MucosalKind`.
    MucosalMap(MucosalMapArgs),
    /// Print the structural fingerprint of a scanned item.
    ///
    /// The same digest `attest scaffold`/`sign` need and `scan --format json`
    /// surfaces, exposed as a first-class verb so an operator can obtain a
    /// fingerprint WITHOUT scaffolding first — for the `sign` step, for
    /// hand-editing a sidecar, or for scripting. Scans the `--root` subtree and
    /// prints the `structural_fingerprint` of every immune/presents site,
    /// optionally narrowed by `--antigen` and/or `--item-path`.
    Fingerprint(FingerprintArgs),
}

// A CLI args struct: each bool is an independent `--flag` (strict / include_deps
// / workspace / bundled_catalog). They are orthogonal user toggles, not a state
// enum to collapse — the idiomatic clap shape — so the excessive-bools lint
// (aimed at domain structs) does not apply here.
#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Parser)]
struct ScanArgs {
    /// Workspace root (default: current directory)
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Output format: human or json
    #[arg(long, default_value = "human")]
    format: OutputFormat,
    /// Exit with non-zero status if unaddressed presentations are found
    #[arg(long)]
    strict: bool,
    /// Also scan dependency crates (registry/git) resolved by `cargo metadata`.
    /// Each dep is scanned independently; results appear under `dep_reports`
    /// in JSON output. No cross-crate `addresses()` matching yet — each crate's
    /// report stays its own bag of antigens. Default OFF for backward
    /// compatibility.
    #[arg(long)]
    include_deps: bool,
    /// Member-aware multi-crate scan (v0.3). Instead of walking `--root` as one
    /// flat tree (every record sharing the same identity), enumerate the
    /// workspace's member crates via `cargo metadata`, scan each independently,
    /// and stamp each member's declarations with its own `<name>@<version>`
    /// canonical path. Cross-member `#[descended_from]` lineage resolves across
    /// members. This is the substrate for cross-crate identity (ADR-001 C7).
    /// Default OFF: the flat single-bag scan stays the default for
    /// backward-compatible output.
    #[arg(long)]
    workspace: bool,
    /// Filter to antigen declarations of a single category (ADR-028 §CLI
    /// integration). Accepts `substrate-alignment` or `functional-correctness`.
    /// A hybrid antigen (both categories) matches either filter.
    #[arg(long)]
    category: Option<String>,
    /// Scan against antigen's **bundled stdlib catalog** (v0.4 E0). Supplies
    /// antigen's flagship failure-class fingerprints so a crate with ZERO antigen
    /// declarations of its own still gets real fingerprint-match findings —
    /// closing the zero-hits-cliff (an empty repertoire is otherwise a false
    /// all-clear). An EXPLICIT `--bundled-catalog` ALWAYS injects (augments local
    /// antigens); without the flag, the catalog auto-injects only when no in-tree
    /// antigens are found (ADR-043 Amendment 2). Bundled matches are SCAN-FACTS
    /// ("structure matches a known class"), never audited defense verdicts
    /// (claim-scope, ADR-043 Amendment 1 / ADR-044).
    #[arg(long)]
    bundled_catalog: bool,
    /// Emit findings in the **cargo/rustc `--message-format=json` shape** (v0.4
    /// render B) so an editor's flycheck consumes antigen findings as compiler
    /// diagnostics — point rust-analyzer's `check.overrideCommand` at
    /// `cargo antigen scan --message-format json`, NO custom LSP server. This is
    /// the rustc line-protocol (newline-delimited `compiler-message` objects),
    /// distinct from `--format json` (antigen's own report envelope). Findings
    /// emit at `warning` level only — antigen never fails the build, and a
    /// fingerprint match is a candidate to inspect, not an audited verdict.
    #[arg(long, value_name = "FMT")]
    message_format: Option<MessageFormat>,
    /// Write the full JSON report to this file (a *render of this run*, never
    /// stored state antigen reads back — see the report-as-live-projection
    /// floor). Implies `--format json` for the file content regardless of the
    /// console `--format`. Console still prints the human/json summary so a
    /// piped invocation and a saved render coexist (e.g. CI prints the summary
    /// AND saves the detail). The file is overwritten each run because the
    /// report is recomputed each run — it cannot drift from the code.
    #[arg(long, value_name = "FILE")]
    output: Option<PathBuf>,
}

/// Args for `cargo antigen propose` (Island 3 — the keystone goes live).
///
/// Two trust-distinct source roots: `--cluster-root` (where the marked DEFECT
/// sites live) and `--clean-root` (the OPERATOR-asserted clean corpus). The split
/// is load-bearing: antigen NEVER auto-labels unmarked code as clean — the
/// operator supplies and labels the clean corpus, and the gate verifies only
/// against what they supply (ADR-044/047; auto-labeling "unmarked = clean" is the
/// ATK-047-4 mislabeled-clean residual the gate must not trust).
#[derive(Debug, Parser)]
struct ProposeArgs {
    /// Root to scan for the marked DEFECT cluster (the `#[dread]`/`#[aura]` sites
    /// to anti-unify). Default: current directory.
    #[arg(long, default_value = ".")]
    cluster_root: PathBuf,
    /// The OPERATOR-supplied, OPERATOR-labeled clean corpus root (scanned; its
    /// function/impl items are the asserted-clean siblings the gate spares against).
    /// **Required for the gate** — there is no auto-derived "the rest of the tree is
    /// clean" default. Antigen never labels unmarked code clean (ADR-044/047,
    /// ATK-047-4): the gate is only as strong as the corpus you supply + label.
    /// (Optional only with `--list-clusters`, a dry-run preview that never runs the
    /// gate and so never consults the clean corpus.)
    #[arg(long, value_name = "PATH")]
    clean_root: Option<PathBuf>,
    /// Which marker-class is the defect cluster (`dread` / `aura` / `red-flag`).
    /// A cluster mixes one marker-class only — different felt-classes anti-unify to
    /// a worse generalization. Default: `dread`.
    #[arg(long, default_value = "dread")]
    marker: String,
    /// Output format: human or json.
    #[arg(long, default_value = "human")]
    format: OutputFormat,
    /// **Preview the cluster landscape and STOP** — group the marked sites by
    /// structural shape and print every candidate cluster (its shape digest, its
    /// source-distinct site count, and which one propose would anti-unify), WITHOUT
    /// running the gate. A pure read of the `by_shape` grouping the CLI already
    /// computes — the diagnostic that answers "why did propose say *no cluster*?"
    /// (usually: every shape is a singleton). Reads `--cluster-root` only;
    /// `--clean-root` is not consulted. Exit `0`.
    #[arg(long)]
    list_clusters: bool,
    /// **Opt CI into a distinct exit code per outcome** (mirrors `audit --strict`).
    /// Without this flag propose always exits `0` (the human-facing default —
    /// route-to-human is first-class, never a "failure"). WITH it, the outcome is
    /// categorized for a CI gate, NOT graded pass/fail:
    /// `0` promoted · `10` route-to-human · `11` refused-autoimmune ·
    /// `12` degenerate · `13` no-cluster. (IO/usage errors stay `2` regardless —
    /// the `10+` range never collides with them.) A non-zero here is a *category*,
    /// not a verdict: `10` (route-to-human) is the gate being honest, not failing.
    #[arg(long)]
    exit_code: bool,
    /// **Show the GATE-G reasoning behind the verdict** — turn the gate from oracle
    /// into teacher. Every propose render tells you the VERDICT but hides the PATH to
    /// a YES; `--explain` surfaces the reasoning the render discards. On
    /// route-to-human: that NO clean sibling is one constraint from binding the draft
    /// (so you know to add a near-miss sibling to your corpus). On autoimmune refusal:
    /// WHICH clean-corpus item the draft wrongly bound (the twin it would have
    /// flagged — the autoimmunity made concrete). On promote: which clean sibling
    /// WITNESSED the generalization (the near-miss that proved B made a real in-family
    /// discrimination). Pure additive output — it NEVER changes the verdict or the
    /// exit code (the gate holds; GATE-G is untouched). The source tree is byte-unchanged.
    #[arg(long)]
    explain: bool,
}

#[derive(Debug, Parser)]
struct AuditArgs {
    /// Workspace root (default: current directory)
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Output format: human or json
    #[arg(long, default_value = "human")]
    format: OutputFormat,
    /// Exit with non-zero status if any defense witness is unresolved
    /// (`Missing`, `NotFound`, or `Ambiguous`). Gates on `Reachability`
    /// tier minimum; `Execution`-tier gating is not yet wired (awaits
    /// `cargo test` integration).
    #[arg(long)]
    strict: bool,
    /// Filter the category audit to a single category (ADR-028 §CLI
    /// integration). Accepts `substrate-alignment` or `functional-correctness`.
    /// A hybrid antigen (both categories) matches either filter.
    #[arg(long)]
    category: Option<String>,
    /// Member-aware multi-crate audit (mirrors `scan --workspace`). Enumerate the
    /// workspace's member crates via `cargo metadata`, scan each independently,
    /// and stamp each member's declarations with its own `<name>@<version>`. This
    /// is what populates the scan-coverage record, so the **coverage /
    /// reachability audit** (the ignorance frontier: enumerated-but-unscanned
    /// members) can surface. A flat audit (default) has no member concept and so
    /// cannot report unreached members — tier-honest, not a completeness claim.
    #[arg(long)]
    workspace: bool,
    /// Write the full JSON audit report to this file (a *render of this run*,
    /// never stored state antigen reads back — the report-as-live-projection
    /// floor). The file is overwritten each run; the report is recomputed each
    /// run so it cannot drift. Console output is unchanged. Running this at a
    /// tagged commit produces that tag's defense-posture SBOM as a reproducible
    /// render — antigen never reads it back as authoritative.
    #[arg(long, value_name = "FILE")]
    output: Option<PathBuf>,
}

#[derive(Debug, Parser)]
struct FingerprintArgs {
    /// Workspace (or crate) root to scan (default: current directory).
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Only report sites whose antigen type matches (last path segment, e.g.
    /// `SignedZeroDiscipline`). Omit to report every scanned immune/presents site.
    #[arg(long)]
    antigen: Option<String>,
    /// Only report the site at this item path (matched against the item's
    /// rendered label, e.g. `sinh`, `Type::method`). Requires the named site to
    /// exist in the scanned root.
    #[arg(long)]
    item_path: Option<String>,
    /// Output format: human or json.
    #[arg(long, default_value = "human")]
    format: OutputFormat,
}

// ============================================================================
// cargo antigen attest subcommand family (ADR-019 substrate-witness sidecars)
// ============================================================================

#[derive(Debug, Parser)]
struct AttestCli {
    #[command(subcommand)]
    command: AttestSubcommand,
}

// AttestSubcommand carries the `Oracle` variant (hidden stub) which is a
// declared CLI surface whose implementation is gated on ADR-021 plumbing not
// yet written — a DeclaredCapabilityWithNoProductionPath instance at the
// type-system level (the variant exists in the enum and surfaces in clap, but
// invoking it returns the design-phase error rather than executing the
// promised behavior). Type-level placement on the enclosing enum because Rust
// forbids proc-macro attribute macros on variants directly (variant-position
// placement does not compile — see compile-fail fixture in antigen-macros
// tests/ui/presents_on_enum_variant.rs).
#[presents(antigen::stdlib::dogfood::DeclaredCapabilityWithNoProductionPath)]
#[derive(Debug, Subcommand)]
enum AttestSubcommand {
    /// Create a new `.attest/<Antigen>.json` sidecar file adjacent to the source file.
    Scaffold(AttestScaffoldArgs),
    /// Add a fresh signer entry to an existing `.attest/<Antigen>.json` sidecar.
    Sign(AttestSignArgs),
    /// Evaluate a substrate-witness predicate against a sidecar and report the result.
    Check(AttestCheckArgs),
    /// Add a delta-attestation entry to an existing sidecar.
    Delta(AttestDeltaArgs),
    /// Per-attestation oracle marker (design phase; v0.1-rc stub).
    ///
    /// Renamed from `oracle complete` per F28-R2 cross-ADR collision-avoidance:
    /// `cargo antigen oracle complete` (top-level) is the steward-authorized
    /// oracle-state transition Draft→Complete; this verb (`attest oracle mark`)
    /// is the per-attestation marker recording that a signer reviewed an
    /// oracle reference at this attestation. Distinct semantics, distinct
    /// CLI families. Implementation gated on ADR-021 `OracleRef` schema
    /// (now ratified — Task 1 shipped; this verb's handler is design-phase
    /// pending the `OracleCompletionMarker` schema field plumbing).
    ///
    /// **ATK-021-19 implementation requirement** (per navigator routing
    /// 2026-05-20): the CLI MUST write
    /// `OracleCompletionMarker { oracle_state_at_attestation: <current
    /// oracle state at sign time>, .. }` into the sidecar at every
    /// invocation. Without the sign-time-state anchor, sign-time-validity
    /// (ADR-021 §D4) cannot be structurally enforced — the audit cannot
    /// distinguish "signer attested when oracle was Complete, oracle later
    /// Deprecated" (hint-only, tier preserved) from "signer attested when
    /// oracle was already Deprecated" (tier reject). The audit's fallback
    /// for legacy sidecars without this field is the
    /// `oracle-sign-time-state-unknown` hint at Execution+advisory tier;
    /// new sidecars written by this verb MUST always populate it.
    #[command(hide = true, name = "oracle")]
    Oracle,
    /// List all `.attest/` sidecars in the workspace.
    List(AttestListArgs),
    /// Garbage-collect orphaned sidecar entries (report-only in v0.1).
    Gc(AttestGcArgs),
}

#[derive(Debug, Parser)]
struct AttestDeltaArgs {
    /// Path to the `.attest/<Antigen>.json` sidecar to update.
    #[arg(long)]
    sidecar: PathBuf,
    /// Item path within the sidecar to add a delta entry for.
    #[arg(long)]
    item_path: String,
    /// Signer name (defaults to `git config user.name`).
    #[arg(long)]
    signer: Option<String>,
    /// Role tag for this signer (optional).
    #[arg(long)]
    role: Option<String>,
    /// Current structural fingerprint of the item. The delta is anchored
    /// against this fingerprint. Signer attests the change from
    /// `prior_fingerprint` to this fingerprint is invariant-preserving.
    #[arg(long)]
    fingerprint: String,
    /// Fingerprint this delta is rooted against (the signer's last signature
    /// at this item).
    #[arg(long)]
    prior_fingerprint: String,
    /// Why the change is invariant-preserving. Required non-empty.
    #[arg(long)]
    rationale: String,
    /// Identity-binding strength.
    #[arg(long, default_value = "git-trust")]
    strength: SignatureStrengthArg,
}

#[derive(Debug, Parser)]
struct AttestListArgs {
    /// Workspace root to walk (defaults to current directory).
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Only list tolerance sidecars (`RatificationKind::Tolerance`).
    #[arg(long)]
    tolerance_only: bool,
    /// Walk `.attest/` directories independent of scan-side macro discovery
    /// and report orphaned sidecars (sidecars whose `item_path` doesn't appear
    /// in any scan-side Immunity declaration at that path).
    #[arg(long)]
    orphan_scan: bool,
    /// Output format.
    #[arg(long, default_value = "human")]
    format: OutputFormat,
}

#[derive(Debug, Parser)]
struct AttestGcArgs {
    /// Workspace root to walk (defaults to current directory).
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Actually remove orphaned sidecars (default: report only).
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Parser)]
struct AttestScaffoldArgs {
    /// The antigen type name (e.g., `SignedZeroCancellation`).
    #[arg(long)]
    antigen: String,
    /// The source file the `#[immune]` declaration lives in. The sidecar is
    /// created at `<source-file-dir>/.attest/<antigen>.json`.
    #[arg(long)]
    source_file: PathBuf,
    /// The item path within the source file (e.g., `sinh`, `cosh`, `MyStruct::method`).
    /// Used as the `item_path` field in the sidecar's `items` array.
    #[arg(long, default_value = "")]
    item_path: String,
    /// The current structural fingerprint of the item. Use the fingerprint from
    /// `cargo antigen scan --format json` or compute via `antigen-fingerprint`.
    /// If omitted the sidecar uses an empty placeholder — update before signing.
    #[arg(long, default_value = "")]
    fingerprint: String,
    /// Ratification kind: `immunity` (default) or `tolerance`.
    #[arg(long, default_value = "immunity")]
    kind: RatificationKindArg,
    /// Overwrite an existing sidecar without prompting.
    #[arg(long)]
    force: bool,
}

#[derive(Debug, Parser)]
struct AttestSignArgs {
    /// Path to the `.attest/<Antigen>.json` sidecar to update.
    #[arg(long)]
    sidecar: PathBuf,
    /// The item path within the sidecar's `items` array to add the signer to.
    /// Must match an existing `item_path` in the sidecar.
    #[arg(long)]
    item_path: String,
    /// Signer's display name (e.g., `alice`).
    #[arg(long)]
    signer: String,
    /// The current structural fingerprint of the item (must match the sidecar's
    /// `current_fingerprint` for the item). This is the value that `signed_against_fingerprint`
    /// records — `cargo antigen audit` compares it against the item's real fingerprint
    /// to detect staleness.
    #[arg(long)]
    fingerprint: String,
    /// Optional role for this signer (e.g., `math-reviewer`, `security-reviewer`).
    #[arg(long)]
    role: Option<String>,
    /// Optional free-text reasoning for this signature. Records WHY the signer
    /// attested to discipline compliance at this moment.
    #[arg(long)]
    reasoning: Option<String>,
    /// Identity-binding strength of this signature.
    ///
    /// `text-stamp` — name + timestamp only; no external identity verification;
    /// suitable for LLM agents or reviewers without git config.
    /// `git-trust` — identity bound to `git config user.name/email` (default).
    /// `crypto-signed` — cryptographic identity binding (v0.4+ activation path).
    #[arg(long, default_value = "git-trust")]
    strength: SignatureStrengthArg,
}

#[derive(Debug, Parser)]
struct AttestCheckArgs {
    /// Path to the `.attest/<Antigen>.json` sidecar to evaluate against.
    #[arg(long)]
    sidecar: PathBuf,
    /// The predicate JSON to evaluate. Must match the `antigen_attestation::Predicate`
    /// serde format. Example: `{"kind":"leaf","leaf":{"name":"signers","required":["alice"]}}`
    #[arg(long)]
    predicate: String,
    /// The item path within the sidecar's `items` array to evaluate.
    /// If omitted, evaluates the first item.
    #[arg(long)]
    item_path: Option<String>,
    /// The current structural fingerprint of the item, for stale-signer detection.
    /// If omitted, uses the sidecar's stored `current_fingerprint`.
    #[arg(long)]
    fingerprint: Option<String>,
}

/// CLI representation of `antigen_attestation::RatificationKind`.
#[derive(Debug, Clone, clap::ValueEnum)]
enum RatificationKindArg {
    Immunity,
    Tolerance,
}

impl From<RatificationKindArg> for antigen_attestation::RatificationKind {
    fn from(k: RatificationKindArg) -> Self {
        match k {
            RatificationKindArg::Immunity => Self::Immunity,
            RatificationKindArg::Tolerance => Self::Tolerance,
        }
    }
}

/// CLI representation of `antigen_attestation::SignatureStrength`.
#[derive(Debug, Clone, clap::ValueEnum)]
enum SignatureStrengthArg {
    TextStamp,
    GitTrust,
    CryptoSigned,
}

impl From<SignatureStrengthArg> for antigen_attestation::SignatureStrength {
    fn from(s: SignatureStrengthArg) -> Self {
        match s {
            SignatureStrengthArg::TextStamp => Self::TextStamp,
            SignatureStrengthArg::GitTrust => Self::GitTrust,
            SignatureStrengthArg::CryptoSigned => Self::CryptoSigned,
        }
    }
}

// ============================================================================
// cargo antigen tolerate subcommand family (ADR-019 tolerance sidecars)
// ============================================================================

#[derive(Debug, Parser)]
struct TolerateCli {
    #[command(subcommand)]
    command: TolerateSubcommand,
}

#[derive(Debug, Subcommand)]
enum TolerateSubcommand {
    /// Create a new tolerance sidecar at `.attest/<Antigen>.json` adjacent to source.
    ///
    /// Equivalent to `attest scaffold --kind tolerance`. Tolerance sidecars share
    /// the same `Ratification` schema — the `kind` discriminator distinguishes them.
    Scaffold(AttestScaffoldArgs),
    /// Add a fresh signer entry to an existing tolerance sidecar.
    Sign(AttestSignArgs),
    /// Evaluate a substrate-witness predicate against a tolerance sidecar.
    Check(AttestCheckArgs),
    /// List all tolerance sidecars in the workspace.
    List(AttestListArgs),
}

// ============================================================================
// cargo antigen oracle subcommand family (ADR-021 oracle-as-artifact-class)
// ============================================================================

#[derive(Debug, Parser)]
struct OracleCli {
    #[command(subcommand)]
    command: OracleSubcommand,
}

#[derive(Debug, Subcommand)]
enum OracleSubcommand {
    /// List all Oracle artifact records in the workspace.
    List(OracleListArgs),
    /// Show current state, stewards, transitions, and attestations for one oracle.
    Status(OracleStatusArgs),
    /// Create a new Oracle in DRAFT state.
    Declare(OracleDeclareArgs),
    /// Transition an oracle from DRAFT to COMPLETE (steward-authorized).
    ///
    /// Signers may attest against Complete oracles; Draft oracles block
    /// the `oracles_complete(...)` predicate per ADR-021 §D3.
    Complete(OracleCompleteArgs),
    /// Transition an oracle from COMPLETE to DEPRECATED (steward-authorized).
    ///
    /// Existing attestations honored at Execution tier (sign-time-validity D4).
    /// For INCORRECT oracles use `oracle revoke` instead.
    Deprecate(OracleDeprecateArgs),
    /// Permanently retire an oracle (steward-authorized).
    ///
    /// All prior attestations honored. Use only when oracle is gone for reasons
    /// other than incorrectness. For incorrect oracles use `oracle revoke`.
    Retire(OracleRetireArgs),
    /// Revoke an oracle for incorrectness or fraud (steward-authorized).
    ///
    /// `--invalidates-prior true` retroactively demotes prior attestations to
    /// Reachability. `--invalidates-prior false` preserves prior attestation tiers.
    Revoke(OracleRevokeArgs),
}

#[derive(Debug, Parser)]
struct OracleListArgs {
    /// Workspace root (default: current directory).
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Output format.
    #[arg(long, default_value = "human")]
    format: OutputFormat,
}

#[derive(Debug, Parser)]
struct OracleStatusArgs {
    /// Oracle ID.
    #[arg(long)]
    id: String,
    /// Workspace root (default: current directory).
    #[arg(long, default_value = ".")]
    root: PathBuf,
}

#[derive(Debug, Parser)]
struct OracleDeclareArgs {
    /// Stable oracle identifier.
    #[arg(long)]
    id: String,
    /// Reference kind: `local-file`, `url`, `doi`, `arxiv`, `github-issue`, `other`.
    #[arg(long)]
    kind: OracleRefKindArg,
    /// Reference value (file path, URL, DOI, arXiv ID, `owner/repo#N`, or free-form).
    #[arg(long)]
    reference: String,
    /// Steward name (defaults to git config user.name). Pass twice for 2 stewards.
    #[arg(long, action = clap::ArgAction::Append)]
    steward: Vec<String>,
    /// WHY this oracle is being declared (required non-empty per Amendment 2).
    #[arg(long)]
    rationale: String,
    /// Version pin at declaration time.
    #[arg(long)]
    version: Option<String>,
    /// Workspace root (default: current directory).
    #[arg(long, default_value = ".")]
    root: PathBuf,
}

#[derive(Debug, Clone, clap::ValueEnum)]
#[clap(rename_all = "kebab-case")]
enum OracleRefKindArg {
    LocalFile,
    Url,
    Doi,
    Arxiv,
    GithubIssue,
    Other,
}

#[derive(Debug, Parser)]
struct OracleCompleteArgs {
    /// Oracle ID to transition Draft→Complete.
    #[arg(long)]
    id: String,
    /// Authorizing steward (defaults to git config user.name).
    #[arg(long)]
    steward: Option<String>,
    /// Version pin at completion time.
    #[arg(long)]
    version: String,
    /// WHY completing (required non-empty per Amendment 2).
    #[arg(long)]
    rationale: String,
    /// Workspace root (default: current directory).
    #[arg(long, default_value = ".")]
    root: PathBuf,
}

#[derive(Debug, Parser)]
struct OracleDeprecateArgs {
    /// Oracle ID to deprecate.
    #[arg(long)]
    id: String,
    /// Authorizing steward (defaults to git config user.name).
    #[arg(long)]
    steward: Option<String>,
    /// Optional successor oracle ID.
    #[arg(long)]
    superseded_by: Option<String>,
    /// WHY deprecating (required non-empty per Amendment 2).
    #[arg(long)]
    rationale: String,
    /// Workspace root (default: current directory).
    #[arg(long, default_value = ".")]
    root: PathBuf,
}

#[derive(Debug, Parser)]
struct OracleRetireArgs {
    /// Oracle ID to retire.
    #[arg(long)]
    id: String,
    /// Authorizing steward (defaults to git config user.name).
    #[arg(long)]
    steward: Option<String>,
    /// WHY retiring (required non-empty per Amendment 2).
    #[arg(long)]
    rationale: String,
    /// Workspace root (default: current directory).
    #[arg(long, default_value = ".")]
    root: PathBuf,
}

#[derive(Debug, Parser)]
struct OracleRevokeArgs {
    /// Oracle ID to revoke.
    #[arg(long)]
    id: String,
    /// Authorizing steward (defaults to git config user.name).
    #[arg(long)]
    steward: Option<String>,
    /// WHY revoking (required non-empty per Amendment 2).
    #[arg(long)]
    rationale: String,
    /// Whether to retroactively demote prior attestations to Reachability.
    #[arg(long)]
    invalidates_prior: bool,
    /// Workspace root (default: current directory).
    #[arg(long, default_value = ".")]
    root: PathBuf,
}

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Human,
    Json,
}

/// The editor-flycheck message format (v0.4 render B). Currently the single
/// `json` value (the cargo/rustc `--message-format=json` line-protocol); an enum
/// rather than a bool so a future `json-diagnostic-short` / `json-render-...`
/// variant is additive.
#[derive(Debug, Clone, clap::ValueEnum)]
enum MessageFormat {
    /// The cargo/rustc JSON line-protocol (newline-delimited `compiler-message`).
    Json,
}

// ============================================================================
// cargo antigen verify — Supply-Chain Defense Family CLI (ADR-025)
// ============================================================================

#[derive(Debug, Parser)]
struct VerifyCli {
    #[command(subcommand)]
    command: VerifySubcommand,
}

#[derive(Debug, Subcommand)]
enum VerifySubcommand {
    /// Check `Cargo.toml` for unpinned + unattested dependencies.
    Deps(VerifyDepsArgs),
    /// Detect crate-ownership changes since the last attested snapshot.
    ///
    /// **CI sequencing constraint (load-bearing)**: this subcommand MUST
    /// run BEFORE `cargo update`. After `cargo update` the new
    /// maintainer's code has already landed in `Cargo.lock`; the gate
    /// has effectively already passed. Document the sequencing in
    /// CI scripts. Per ADR-025 §Decision.
    MaintainerChanges(VerifyMaintainerChangesArgs),
    /// Record a dep-attestation sidecar.
    ///
    /// `--reviewable-artifact <PATH>` is REQUIRED — empty/missing
    /// values produce a rubber-stamp sidecar that the audit will
    /// flag with `dep-attest-without-reviewable-artifact`. Per
    /// ADR-025 §Schema-additions.
    DepAttest(VerifyDepAttestArgs),
    /// Bulk-pin every unpinned `Cargo.toml` dep with the current
    /// resolved version (advisory v0.2: prints suggested edits;
    /// in-place rewrite is v0.3+).
    DepPin(VerifyDepPinArgs),
    /// Record or check a content-hash for `<crate@version>`.
    ///
    /// **THE NON-NEGOTIABLE CHALK/DEBUG DEFENSE**. First-attestation:
    /// `cargo antigen verify content-hash record <crate@version>`.
    /// Verification: `cargo antigen verify content-hash <crate@version>`.
    /// Per ADR-025 §Decision + B1-R.
    ContentHash(VerifyContentHashArgs),
    /// v0.4+: sandbox-execute a proc-macro dep and report observations.
    /// Currently a stub that surfaces the "tooling not yet available"
    /// awareness signal per ADR-005 Amendment 2 honest-tier-naming.
    #[command(name = "proc-macro-sandbox")]
    ProcMacroSandbox,
    /// v0.4+: sandbox-execute a `build.rs` and report observations.
    /// Currently a stub (see `proc-macro-sandbox`).
    Sandbox,
    /// v0.5+: compare behavioral fingerprints across dep versions.
    /// Currently a stub.
    BehavioralDiff,
}

#[derive(Debug, Parser)]
struct VerifyDepsArgs {
    /// Workspace root (default: current directory).
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Output format: human or json.
    #[arg(long, default_value = "human")]
    format: OutputFormat,
    /// Optional crate name to scope the check (default: every dep).
    #[arg(long)]
    crate_name: Option<String>,
    /// Exit non-zero when any unpinned/unattested dep is found.
    #[arg(long)]
    strict: bool,
}

#[derive(Debug, Parser)]
struct VerifyMaintainerChangesArgs {
    /// Workspace root.
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Crate to check.
    #[arg(long)]
    crate_name: String,
    /// Anchor version. The snapshot at
    /// `.attest/supply-chain/maintainer/<crate>.json` must match.
    #[arg(long)]
    since_version: String,
}

// `review_scope: String` (below) is the UnvalidatedSealedEnumAcceptance site:
// the arg accepts an arbitrary String at clap parse-time and is matched in
// run_verify_dep_attest against 5 valid ReviewScope variants + an `other`
// fallthrough — the sealed-enum vocabulary lives in the type system
// (`antigen::supply_chain::schema::ReviewScope`) but the CLI never enforces
// membership at the boundary. A typo or invented value parses fine and falls
// into the `other` arm with no signal. Type-level placement on the enclosing
// struct (Rust forbids proc-macro attrs on struct fields directly — same
// compile barrier as variants; see compile-fail fixture).
#[presents(antigen::stdlib::dogfood::UnvalidatedSealedEnumAcceptance)]
#[derive(Debug, Parser)]
struct VerifyDepAttestArgs {
    /// Workspace root.
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// `<crate>@<version>` positional argument.
    crate_at_version: String,
    /// **REQUIRED** non-empty path to the human-reviewable artifact
    /// that documents the review (Markdown, ADR, PR comment export,
    /// etc.). Empty values are rejected outright.
    #[arg(long, required = true)]
    reviewable_artifact: PathBuf,
    /// Review scope: `full` | `diff` | `build-script-only` |
    /// `proc-macro-only` | `metadata-only`.
    #[arg(long, default_value = "metadata-only")]
    review_scope: String,
    /// Whether the attestation applies only to this exact version
    /// (default `true`).
    #[arg(long, default_value_t = true)]
    exact_version: bool,
    /// Signer name (defaults to `git config user.name`).
    #[arg(long)]
    signed_by: Option<String>,
    /// Optional human rationale.
    #[arg(long)]
    rationale: Option<String>,
}

#[derive(Debug, Parser)]
struct VerifyDepPinArgs {
    /// Workspace root.
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Rewrite the adopter's `Cargo.toml` IN PLACE, pinning each unpinned dep to
    /// its resolved `=<version>` from `Cargo.lock` (format-preserving — comments
    /// and layout are kept). OPT-IN by design: rewriting the adopter's manifest
    /// is an outward-facing, hard-to-reverse mutation, so it is NEVER the default
    /// (ADR-017-Amd1 posture: gate mutation-safety). Without `--write` the
    /// subcommand only PRINTS the suggested edits. Deps with no resolved version
    /// in `Cargo.lock` are left unchanged (never guessed).
    #[arg(long)]
    write: bool,
}

#[derive(Debug, Parser)]
struct VerifyContentHashArgs {
    #[command(subcommand)]
    command: VerifyContentHashSubcommand,
}

#[derive(Debug, Subcommand)]
enum VerifyContentHashSubcommand {
    /// Record the current `Cargo.lock` checksum as the first-attestation
    /// hash for `<crate@version>`.
    Record(VerifyContentHashRecordArgs),
    /// Compare the current `Cargo.lock` checksum for `<crate@version>`
    /// against the recorded first-attestation hash.
    Check(VerifyContentHashCheckArgs),
}

#[derive(Debug, Parser)]
struct VerifyContentHashRecordArgs {
    /// Workspace root.
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// `<crate>@<version>` positional argument.
    crate_at_version: String,
    /// Signer name (defaults to `git config user.name`).
    #[arg(long)]
    signed_by: Option<String>,
}

#[derive(Debug, Parser)]
struct VerifyContentHashCheckArgs {
    /// Workspace root.
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// `<crate>@<version>` positional argument.
    crate_at_version: String,
    /// Additionally verify against the hash crates.io ACTUALLY SERVES (the live
    /// depth claim): fetch the version's checksum from the crates.io sparse
    /// index and compare. Degrades gracefully — if the registry is unreachable
    /// (offline / network error / version absent), the live check is reported
    /// UNVERIFIABLE and SKIPPED, never blocking the local check. Off by default.
    #[arg(long)]
    live: bool,
    /// With `--live`: exit non-zero on a live MISMATCH (the substitution signal).
    /// Without `--strict`, a live mismatch is reported loudly but does not change
    /// the exit code (the local check governs). An UNVERIFIABLE live check never
    /// affects the exit code regardless of `--strict` (offline ≠ failure).
    #[arg(long)]
    strict: bool,
}

fn run_verify(cli: VerifyCli) -> ExitCode {
    match cli.command {
        VerifySubcommand::Deps(args) => run_verify_deps(args),
        VerifySubcommand::MaintainerChanges(args) => run_verify_maintainer_changes(args),
        VerifySubcommand::DepAttest(args) => run_verify_dep_attest(args),
        VerifySubcommand::DepPin(args) => run_verify_dep_pin(args),
        VerifySubcommand::ContentHash(args) => match args.command {
            VerifyContentHashSubcommand::Record(a) => run_verify_content_hash_record(a),
            VerifyContentHashSubcommand::Check(a) => run_verify_content_hash_check(a),
        },
        VerifySubcommand::ProcMacroSandbox => run_verify_stub(
            "proc-macro-sandbox",
            "v0.4+",
            "Active proc-macro sandboxing (ADR-025 tooling-phase 3).",
        ),
        VerifySubcommand::Sandbox => run_verify_stub(
            "sandbox",
            "v0.4+",
            "Active build.rs sandboxing (ADR-025 tooling-phase 3).",
        ),
        VerifySubcommand::BehavioralDiff => run_verify_stub(
            "behavioral-diff",
            "v0.5+",
            "Behavioral-fingerprint comparison across versions (ADR-025 tooling-phase 4).",
        ),
    }
}

fn run_verify_deps(args: VerifyDepsArgs) -> ExitCode {
    use antigen::supply_chain::evaluate::evaluate_dep_pinned;
    use antigen::supply_chain::witness::DepPinnedState;
    let state = evaluate_dep_pinned(&args.root, args.crate_name.as_deref());
    let unpinned_names = match &state {
        DepPinnedState::AllPinned => Vec::new(),
        DepPinnedState::Unpinned { unpinned_deps } => unpinned_deps.clone(),
        DepPinnedState::NotInManifest { crate_name } => vec![crate_name.clone()],
    };
    match args.format {
        OutputFormat::Human => {
            if unpinned_names.is_empty() {
                println!("verify deps: all checked dependencies are exact-pinned.");
            } else {
                println!(
                    "verify deps: {n} dep(s) without exact-pin `=` specifiers:",
                    n = unpinned_names.len()
                );
                for n in &unpinned_names {
                    println!("  - {n}");
                }
                println!();
                println!("To fix:");
                println!("  edit Cargo.toml; change each entry to `<name> = \"=X.Y.Z\"`.");
                println!("Per ADR-025 §UnpinnedDependency — exact-pin is the discipline.");
            }
        },
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string(&serde_json::json!({
                    "subcommand": "verify-deps",
                    "unpinned_deps": unpinned_names,
                    "passed": unpinned_names.is_empty(),
                }))
                .unwrap_or_else(|_| "{}".to_string())
            );
        },
    }
    if args.strict && !unpinned_names.is_empty() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn run_verify_maintainer_changes(args: VerifyMaintainerChangesArgs) -> ExitCode {
    use antigen::supply_chain::evaluate::{evaluate_maintainer_unchanged, is_valid_crate_name};
    use antigen::supply_chain::witness::MaintainerState;
    if !is_valid_crate_name(&args.crate_name) {
        eprintln!(
            "error: --crate-name must contain only ASCII alphanumeric characters, `_`, or `-` \
             (got `{}`). Path traversal sequences are not permitted.",
            args.crate_name
        );
        return ExitCode::from(2);
    }
    println!(
        "verify maintainer-changes: checking {crate_name} @ {since_version}",
        crate_name = args.crate_name,
        since_version = args.since_version,
    );
    println!();
    println!("WARNING: this subcommand MUST run BEFORE `cargo update`. After");
    println!("`cargo update`, the new maintainer's code is already in Cargo.lock");
    println!("and the gate has effectively already passed. Per ADR-025.");
    println!();
    let state = evaluate_maintainer_unchanged(&args.root, &args.crate_name, &args.since_version);
    match state {
        MaintainerState::Unchanged => {
            println!("result: maintainer-unchanged (snapshot matches)");
            ExitCode::SUCCESS
        },
        MaintainerState::Changed { added, removed } => {
            println!("result: MAINTAINER-CHANGE-WITHOUT-REATTESTATION");
            println!("  added owners: {added:?}");
            println!("  removed owners: {removed:?}");
            ExitCode::from(1)
        },
        MaintainerState::SnapshotMissing => {
            println!("result: snapshot missing");
            println!(
                "  expected: .attest/supply-chain/maintainer/{crate_name}.json",
                crate_name = args.crate_name
            );
            println!("  (v0.2: snapshot is operator-managed; v0.3+ adds live crates.io query)");
            ExitCode::from(1)
        },
        MaintainerState::CratesIoQueryUnavailable => {
            println!("result: crates.io query unavailable (v0.2 limitation per ADR-025)");
            ExitCode::from(2)
        },
    }
}

fn run_verify_dep_attest(args: VerifyDepAttestArgs) -> ExitCode {
    use antigen::supply_chain::evaluate::dep_attest_path;
    use antigen::supply_chain::schema::{DepAttestation, ReviewScope};

    if args.reviewable_artifact.as_os_str().is_empty()
        || args.reviewable_artifact.to_string_lossy().trim().is_empty()
    {
        eprintln!(
            "error: --reviewable-artifact must be a non-empty, non-whitespace path. \
             Empty values create a rubber-stamp sidecar that the audit will flag with \
             dep-attest-without-reviewable-artifact. Per ADR-025 + ATK-SC-1-A."
        );
        return ExitCode::from(2);
    }

    let Some((crate_name, version)) = parse_crate_at_version(&args.crate_at_version) else {
        eprintln!("error: argument must be `<crate>@<version>` (e.g., `serde@1.0.197`)");
        return ExitCode::from(2);
    };

    let scope = match args.review_scope.as_str() {
        "full" => ReviewScope::Full,
        "diff" => ReviewScope::Diff,
        "build-script-only" | "build_script_only" => ReviewScope::BuildScriptOnly,
        "proc-macro-only" | "proc_macro_only" => ReviewScope::ProcMacroOnly,
        "metadata-only" | "metadata_only" => ReviewScope::MetadataOnly,
        other => {
            eprintln!(
                "error: --review-scope must be one of: full, diff, build-script-only, \
                 proc-macro-only, metadata-only (got `{other}`)"
            );
            return ExitCode::from(2);
        },
    };

    let signed_by = match resolve_steward_name(args.signed_by.as_deref()) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        },
    };
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let attest = DepAttestation {
        crate_name: crate_name.clone(),
        version: version.clone(),
        exact_version: args.exact_version,
        reviewable_artifact: args.reviewable_artifact,
        review_scope: scope,
        signed_by,
        date,
        rationale: args.rationale,
    };

    let path = dep_attest_path(&args.root, &crate_name, &version);
    if let Some(parent) = path.parent() {
        if let Err(e) = std::fs::create_dir_all(parent) {
            eprintln!("error: could not create {}: {e}", parent.display());
            return ExitCode::from(2);
        }
    }
    let json = match serde_json::to_string_pretty(&attest) {
        Ok(j) => j,
        Err(e) => {
            eprintln!("error: serialize attestation: {e}");
            return ExitCode::from(2);
        },
    };
    if let Err(e) = std::fs::write(&path, &json) {
        eprintln!("error: write {}: {e}", path.display());
        return ExitCode::from(2);
    }
    println!("recorded dep-attestation at {}", path.display());
    ExitCode::SUCCESS
}

/// The dep tables `verify dep-pin --write` rewrites — the same tables the
/// evaluator's `read_manifest_deps` scans (minus target-conditional forms, which
/// are nested sub-tables a v0.3 in-place rewrite leaves to the suggestion path).
const PIN_REWRITE_TABLES: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

/// In-place pin a single dep `name` to `=<version>` in a `toml_edit` `table`,
/// preserving surrounding formatting. Handles both value shapes:
///   `name = "1.2"`            → `name = "=1.2.3"`
///   `name = { version = .. }` → the inline table's `version` becomes `=1.2.3`
/// Returns true iff the dep was found in this table and rewritten. A path/git/
/// workspace dep (no `version` key in its inline table) is left untouched — it
/// was never in the unpinned set.
fn pin_dep_in_table(table: &mut toml_edit::Table, name: &str, version: &str) -> bool {
    let pinned = format!("={version}");
    let Some(item) = table.get_mut(name) else {
        return false;
    };
    match item {
        // `name = "1.2"` — a bare string requirement.
        toml_edit::Item::Value(toml_edit::Value::String(_)) => {
            *item = toml_edit::value(pinned);
            true
        },
        // `name = { version = "1.2", features = [..] }` — pin only the version
        // key, keeping every other key (features, default-features, ...) intact.
        // A path/git/workspace inline table (no `version` key) is left untouched.
        toml_edit::Item::Value(toml_edit::Value::InlineTable(t)) if t.contains_key("version") => {
            t.insert("version", toml_edit::Value::from(pinned));
            true
        },
        // `[dependencies.name]` dotted sub-table — pin the version key if present.
        toml_edit::Item::Table(t) if t.contains_key("version") => {
            t.insert("version", toml_edit::value(pinned));
            true
        },
        _ => false,
    }
}

fn run_verify_dep_pin(args: VerifyDepPinArgs) -> ExitCode {
    use antigen::supply_chain::evaluate::{evaluate_dep_pinned, resolved_version_from_lockfile};
    use antigen::supply_chain::witness::DepPinnedState;
    let state = evaluate_dep_pinned(&args.root, None);
    match state {
        DepPinnedState::AllPinned => {
            println!("verify dep-pin: all Cargo.toml deps already exact-pinned. No edits needed.");
            ExitCode::SUCCESS
        },
        DepPinnedState::Unpinned { unpinned_deps } => {
            let lockfile = args.root.join("Cargo.lock");
            // Resolve each unpinned dep's version from Cargo.lock. A dep with no
            // lockfile entry is NEVER guessed — it stays a suggestion/placeholder
            // and is excluded from any --write rewrite.
            let resolved: Vec<(String, String)> = unpinned_deps
                .iter()
                .filter_map(|name| {
                    resolved_version_from_lockfile(&lockfile, name).map(|v| (name.clone(), v))
                })
                .collect();
            let unresolved: Vec<&String> = unpinned_deps
                .iter()
                .filter(|n| !resolved.iter().any(|(rn, _)| rn == *n))
                .collect();

            if args.write {
                return write_dep_pins(&args.root, &resolved, &unresolved);
            }

            println!(
                "verify dep-pin: {n} unpinned dep(s) detected. Suggested edits:",
                n = unpinned_deps.len()
            );
            for (name, version) in &resolved {
                println!("  - in Cargo.toml [dependencies]: pin `{name}` to `={version}`");
            }
            for name in &unresolved {
                println!(
                    "  - in Cargo.toml [dependencies]: pin `{name}` to `=<RESOLVED_VERSION>` \
                     (no Cargo.lock entry — run `cargo generate-lockfile` first)"
                );
            }
            println!();
            if !unresolved.is_empty() {
                println!(
                    "note: {} dep(s) had no resolved version in Cargo.lock.",
                    unresolved.len()
                );
            }
            println!(
                "Re-run with `--write` to apply these pins IN PLACE (format-preserving; \
                 deps with no resolved version are left untouched). Rewriting the adopter's \
                 manifest is opt-in by design — never the default."
            );
            ExitCode::SUCCESS
        },
        DepPinnedState::NotInManifest { crate_name } => {
            eprintln!("error: crate `{crate_name}` not found in manifest");
            ExitCode::from(2)
        },
    }
}

/// Apply the resolved pins to `Cargo.toml` IN PLACE via `toml_edit` (the `--write`
/// path). Format-preserving: comments and layout are kept; only the `version`
/// of each resolved unpinned dep changes to `=<version>`. Unresolved deps are
/// reported and left untouched (never guessed). Returns exit 0 on a clean write,
/// 2 on an IO/parse error.
fn write_dep_pins(
    root: &std::path::Path,
    resolved: &[(String, String)],
    unresolved: &[&String],
) -> ExitCode {
    let manifest_path = root.join("Cargo.toml");
    let content = match std::fs::read_to_string(&manifest_path) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: read {}: {e}", manifest_path.display());
            return ExitCode::from(2);
        },
    };
    let mut doc = match content.parse::<toml_edit::DocumentMut>() {
        Ok(d) => d,
        Err(e) => {
            eprintln!("error: {} is not valid TOML: {e}", manifest_path.display());
            return ExitCode::from(2);
        },
    };

    let mut applied: Vec<&str> = Vec::new();
    let mut not_found: Vec<&str> = Vec::new();
    for (name, version) in resolved {
        // Try each dep table; a dep lives in exactly one (the evaluator would
        // have reported it once per table, but the name is what we pin).
        let mut hit = false;
        for table_name in PIN_REWRITE_TABLES {
            if let Some(table) = doc
                .get_mut(table_name)
                .and_then(toml_edit::Item::as_table_mut)
            {
                if pin_dep_in_table(table, name, version) {
                    hit = true;
                }
            }
        }
        if hit {
            applied.push(name.as_str());
        } else {
            // Resolved a version but couldn't locate a rewritable entry — e.g.
            // the dep is only under a target-conditional table (v0.3 leaves
            // those to the suggestion path). Report, never silently drop.
            not_found.push(name.as_str());
        }
    }

    if applied.is_empty() {
        println!("verify dep-pin --write: no rewritable unpinned deps with a resolved version.");
    } else if let Err(e) = std::fs::write(&manifest_path, doc.to_string()) {
        eprintln!("error: write {}: {e}", manifest_path.display());
        return ExitCode::from(2);
    } else {
        println!(
            "verify dep-pin --write: pinned {} dep(s) in place in {}:",
            applied.len(),
            manifest_path.display()
        );
        for (name, version) in resolved {
            if applied.contains(&name.as_str()) {
                println!("  - `{name}` → `={version}`");
            }
        }
    }
    if !not_found.is_empty() {
        println!();
        println!(
            "note: {} resolved dep(s) were not in a rewritable [dependencies]/[dev-dependencies]/\
             [build-dependencies] table (e.g. target-conditional) — pin them manually: {}",
            not_found.len(),
            not_found.join(", ")
        );
    }
    if !unresolved.is_empty() {
        println!();
        println!(
            "note: {} unpinned dep(s) had no resolved version in Cargo.lock and were left \
             untouched (run `cargo generate-lockfile`, then re-run): {}",
            unresolved.len(),
            unresolved
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
    }
    ExitCode::SUCCESS
}

fn run_verify_content_hash_record(args: VerifyContentHashRecordArgs) -> ExitCode {
    use antigen::supply_chain::evaluate::{current_hash_from_lockfile, save_content_hash_record};
    use antigen::supply_chain::schema::ContentHashRecord;

    let Some((crate_name, version)) = parse_crate_at_version(&args.crate_at_version) else {
        eprintln!("error: argument must be `<crate>@<version>`");
        return ExitCode::from(2);
    };

    let lockfile = args.root.join("Cargo.lock");
    let Some(checksum) = current_hash_from_lockfile(&lockfile, &crate_name, &version) else {
        eprintln!(
            "error: no `[[package]] name = \"{crate_name}\" version = \"{version}\" checksum = ...` \
             entry in {}. The crate must be in the lockfile with a checksum.",
            lockfile.display()
        );
        return ExitCode::from(2);
    };

    let signed_by = match resolve_steward_name(args.signed_by.as_deref()) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        },
    };
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();

    let record = ContentHashRecord {
        crate_name,
        version,
        content_hash: checksum,
        hash_source: "cargo-lock-checksum".to_string(),
        signed_by,
        date,
    };

    match save_content_hash_record(&args.root, &record) {
        Ok(p) => {
            println!("recorded content-hash at {}", p.display());
            println!("  hash-source: cargo-lock-checksum (v0.2; tarball SHA-256 is v0.3+)");
            ExitCode::SUCCESS
        },
        Err(e) => {
            eprintln!("error: write record: {e}");
            ExitCode::from(2)
        },
    }
}

/// The crates.io sparse-index path prefix for `name`, per cargo's convention:
/// 1-char → `1/<name>`; 2-char → `2/<name>`; 3-char → `3/<c1>/<name>`;
/// 4+-char → `<c1c2>/<c3c4>/<name>`. Names are lowercased (the index is
/// case-insensitive, stored lowercase). Returns `None` for an empty name.
fn cratesio_index_path(name: &str) -> Option<String> {
    let lower = name.to_lowercase();
    let n = lower.chars().count();
    match n {
        0 => None,
        1 => Some(format!("1/{lower}")),
        2 => Some(format!("2/{lower}")),
        3 => Some(format!("3/{}/{lower}", &lower[..1])),
        _ => Some(format!("{}/{}/{lower}", &lower[..2], &lower[2..4])),
    }
}

/// Network shell for the live verification: fetch the SHA-256 `cksum` crates.io
/// serves for `name@version` from the sparse index. Returns `None` on ANY
/// network failure / parse failure / version-absent — the caller treats `None`
/// as ⊥ (Unverifiable), so the live check degrades gracefully (never blocks).
/// This is the ONLY networked code; the verdict is the pure
/// `compare_live_cksum`. A 5s timeout bounds the offline-degradation latency.
fn fetch_cratesio_cksum(name: &str, version: &str) -> Option<String> {
    let index_path = cratesio_index_path(name)?;
    let url = format!("https://index.crates.io/{index_path}");
    let agent = ureq::AgentBuilder::new()
        .timeout(std::time::Duration::from_secs(5))
        .build();
    let body = agent.get(&url).call().ok()?.into_string().ok()?;
    // The sparse index is NDJSON: one JSON object per published version. Find the
    // line whose `vers` matches and return its `cksum` (the tarball SHA-256).
    for line in body.lines() {
        let Ok(entry) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if entry.get("vers").and_then(|v| v.as_str()) == Some(version) {
            return entry
                .get("cksum")
                .and_then(|c| c.as_str())
                .map(str::to_owned);
        }
    }
    None // version not present in the index
}

fn run_verify_content_hash_check(args: VerifyContentHashCheckArgs) -> ExitCode {
    use antigen::supply_chain::evaluate::evaluate_content_hash_matches;
    use antigen::supply_chain::witness::ContentHashState;

    let Some((crate_name, version)) = parse_crate_at_version(&args.crate_at_version) else {
        eprintln!("error: argument must be `<crate>@<version>`");
        return ExitCode::from(2);
    };

    // The live verification (--live) runs ALONGSIDE the local check and never
    // overrides it except (under --strict) to escalate a live mismatch. An
    // UNVERIFIABLE live check (offline) is reported and skipped — exit code
    // unaffected. The local check below is always authoritative for exit 0/1/2.
    let live_escalates = if args.live {
        run_live_cksum_check(&args.root, &crate_name, &version, args.strict)
    } else {
        false
    };

    let state = evaluate_content_hash_matches(&args.root, &crate_name, &version);
    let local = match state {
        ContentHashState::Matches => {
            println!("content-hash: MATCH for {crate_name}@{version}");
            ExitCode::SUCCESS
        },
        ContentHashState::Mismatch { recorded, current } => {
            println!("content-hash: MISMATCH for {crate_name}@{version}");
            println!("  recorded: {recorded}");
            println!("  current:  {current}");
            println!();
            println!("This is the chalk/debug/eslint-config attack signal. Investigate before");
            println!("re-recording — if the change is legitimate, re-attest with a fresh");
            println!("signer + review artifact. Per ADR-025 §ContentHashMismatch.");
            ExitCode::from(1)
        },
        ContentHashState::NoAttestation => {
            println!("content-hash: no first-attestation for {crate_name}@{version}");
            println!("  run: cargo antigen verify content-hash record {crate_name}@{version}");
            ExitCode::from(1)
        },
        ContentHashState::CrateNotInLockfile { crate_name: cn } => {
            eprintln!("error: crate `{cn}` not found in Cargo.lock (no checksum to compare)");
            ExitCode::from(2)
        },
        ContentHashState::SidecarMalformed { error } => {
            eprintln!(
                "error: content-hash sidecar exists but did NOT deserialize cleanly. \
                 Per ATK-SC-2-A this is distinct from no-attestation — corrupting the \
                 sidecar to silently downgrade a Mismatch into a NoAttestation is exactly \
                 the attack the audit blocks. Inspect the file and re-record. \
                 Parse error: {error}"
            );
            ExitCode::from(1)
        },
    };

    // A live MISMATCH under --strict escalates a local pass to a failure (the
    // registry served a different hash than what we have locally — a real
    // supply-chain signal). A local failure already governs; an UNVERIFIABLE
    // live check never escalates (offline ≠ failure).
    if live_escalates {
        return ExitCode::from(1);
    }
    local
}

/// Run the live crates.io content-hash verification (the `--live` path). Fetches
/// the served cksum, runs the pure 3-valued
/// [`antigen::supply_chain::evaluate::compare_live_cksum`], prints the
/// outcome, and returns `true` iff the result should ESCALATE the exit code (a
/// `Mismatch` under `--strict`). `Verified` and `Unverifiable` never escalate;
/// `Unverifiable` (offline) is reported and skipped so the audit is not blocked.
fn run_live_cksum_check(
    root: &std::path::Path,
    crate_name: &str,
    version: &str,
    strict: bool,
) -> bool {
    use antigen::supply_chain::evaluate::{compare_live_cksum, current_hash_from_lockfile};
    use antigen::supply_chain::witness::LiveCksumState;

    // The expected hash is the one cargo recorded locally (Cargo.lock checksum).
    // If we don't even have a local hash to compare, the live check has no
    // expectation to verify against — report and skip (no escalation).
    let lockfile = root.join("Cargo.lock");
    let Some(expected) = current_hash_from_lockfile(&lockfile, crate_name, version) else {
        println!(
            "content-hash --live: SKIPPED — no local Cargo.lock checksum for \
             {crate_name}@{version} to compare against the registry."
        );
        return false;
    };

    let served = fetch_cratesio_cksum(crate_name, version);
    match compare_live_cksum(served.as_deref(), &expected) {
        LiveCksumState::Verified { hash } => {
            println!(
                "content-hash --live: VERIFIED for {crate_name}@{version} — the crates.io \
                 sparse-index cksum matches the local lockfile checksum ({hash})."
            );
            false
        },
        LiveCksumState::Mismatch { expected, served } => {
            println!("content-hash --live: MISMATCH for {crate_name}@{version}");
            println!("  local (lockfile): {expected}");
            println!("  crates.io served: {served}");
            println!();
            println!(
                "The registry serves a DIFFERENT hash than your lockfile records — a \
                 supply-chain substitution / yank-and-republish signal. Investigate before \
                 trusting this dependency."
            );
            if strict {
                println!("  (--strict: escalating to a non-zero exit)");
            }
            strict
        },
        LiveCksumState::Unverifiable { reason } => {
            // ⊥: offline / network error / version absent. Report and SKIP —
            // never block the audit, never escalate, regardless of --strict.
            println!("content-hash --live: UNVERIFIABLE for {crate_name}@{version} — skipped.");
            println!("  reason: {reason}");
            false
        },
    }
}

fn run_verify_stub(name: &str, version_target: &str, description: &str) -> ExitCode {
    println!("cargo antigen verify {name}: {version_target} stub");
    println!();
    println!("  {description}");
    println!();
    println!("Per ADR-005 Amendment 2 honest-tier-naming: this subcommand is");
    println!("a deliberate stub. It does NOT silently pass — the supply-chain");
    println!("audit hints surface the un-evaluated witness as the appropriate");
    println!("unsandboxed-* hint, NOT as a passing evaluation.");
    ExitCode::from(2)
}

// ============================================================================
// cargo antigen vcs subcommand family (VCS-Information-Loss Family, ADR-026)
//
// Observation layer (v0.2): reads git substrate (commit trailers, branch
// state) and routes through the `antigen::vcs_witness` evaluators. Git is
// invoked via fixed-arg subprocess per the ADR-019 §Decision §4 bright-line
// rule (git is named, has its own release process, does not execute user
// code, args are fixed). The enforcement layer (install-hooks /
// install-server-hooks that EXECUTE the detection decision tree) defers to
// v0.2.x post-ADR-026 Amendment 4 per witness-layer-independence.
// ============================================================================

#[derive(Debug, Parser)]
struct VcsCli {
    #[command(subcommand)]
    command: VcsSubcommand,
}

#[derive(Debug, Subcommand)]
enum VcsSubcommand {
    /// Evaluate a single commit's `Triage-Decision:` trailer against the
    /// rollback-triage chain (ADR-026 Amendment 4 commit-trailer signal).
    CheckCommit(VcsCheckCommitArgs),
    /// Surface VCS-info-loss risk across recent history (v0.2: reports the
    /// rollback-triage state of revert/reset commits in the recent log).
    Scan(VcsScanArgs),
    /// Record a branch-deletion attestation sidecar before deleting a branch
    /// (`.attest/vcs/branch-archive/<branch>.json`). v0.2: prints the sidecar
    /// JSON the adopter should commit; in-place write is v0.2.x.
    BranchArchive(VcsBranchArchiveArgs),
    /// Scaffold the `#[triage_commit]` + `Triage-Decision:` trailer guidance
    /// for a rollback (v0.2 advisory: prints the trailer line to add).
    RollbackPrepare(VcsRollbackPrepareArgs),
    /// v0.2.x: record a generic VCS attestation sidecar. Currently a stub
    /// surfacing the tooling-not-yet-available awareness signal per
    /// ADR-005 Amendment 2 honest-tier-naming.
    Attest,
    /// Mine git history for the recurrent-emergence stdlib failure-classes and
    /// surface how many times each pattern fired — the passive→active loop for
    /// the recurrent family (`infra/recurrence-automation`). DETECTION only:
    /// reports observed counts; the *verdict* (is this recurrence worth a
    /// `#[recurrence_anchor]`?) stays the adopter's call (the structural seam —
    /// mining detects the fact, the adopter/ADR owns the recognition).
    Recurrence(VcsRecurrenceArgs),
}

#[derive(Debug, Parser)]
struct VcsCheckCommitArgs {
    /// Commit-ish to check (default: HEAD).
    #[arg(long, default_value = "HEAD")]
    commit: String,
    /// Output format: human or json.
    #[arg(long, default_value = "human")]
    format: OutputFormat,
    /// Exit non-zero if the rollback-triage chain is absent/malformed.
    #[arg(long)]
    strict: bool,
}

#[derive(Debug, Parser)]
struct VcsScanArgs {
    /// How many recent commits to inspect.
    #[arg(long, default_value = "50")]
    depth: usize,
    /// Output format: human or json.
    #[arg(long, default_value = "human")]
    format: OutputFormat,
}

#[derive(Debug, Parser)]
struct VcsBranchArchiveArgs {
    /// Branch name being archived (attested before deletion).
    branch: String,
    /// Role or name attesting the deletion.
    #[arg(long)]
    by: String,
    /// Rationale for the deletion (non-empty).
    #[arg(long)]
    rationale: String,
}

#[derive(Debug, Parser)]
struct VcsRollbackPrepareArgs {
    /// Triage decision (black|red|yellow|green|white).
    #[arg(long)]
    decision: String,
    /// Commit sha to roll back to.
    #[arg(long)]
    target: String,
}

#[derive(Debug, Parser)]
struct VcsRecurrenceArgs {
    /// How many recent commits to mine (the recurrence window).
    #[arg(long, default_value = "200")]
    depth: usize,
    /// Output format: human or json.
    #[arg(long, default_value = "human")]
    format: OutputFormat,
}

fn run_vcs(cli: VcsCli) -> ExitCode {
    match cli.command {
        VcsSubcommand::CheckCommit(args) => run_vcs_check_commit(args),
        VcsSubcommand::Scan(args) => run_vcs_scan(args),
        VcsSubcommand::BranchArchive(args) => run_vcs_branch_archive(args),
        VcsSubcommand::RollbackPrepare(args) => run_vcs_rollback_prepare(args),
        VcsSubcommand::Attest => run_vcs_attest_stub(),
        VcsSubcommand::Recurrence(args) => run_vcs_recurrence(args),
    }
}

/// One mined recurrent-emergence pattern: its stdlib antigen name, the
/// observable git substrate it reads, and how many commits in the window
/// touched that substrate (the recurrence count). DETECTION only — the count
/// is the structural fact; whether it merits a `#[recurrence_anchor]` is the
/// adopter's recognition call (the seam).
struct RecurrenceObservation {
    /// The stdlib antigen this observation surfaces evidence for.
    antigen: &'static str,
    /// What git substrate the count reads (human-readable).
    substrate: &'static str,
    /// How many commits in the window touched that substrate.
    count: usize,
}

/// Count commits in the last `depth` commits that touched any path matching
/// `pathspec` (a `git log` pathspec, e.g. `*Cargo.toml`). Returns `None` when
/// git is unavailable (not a repo / git missing) so the caller can degrade
/// honestly — an unavailable mine is NOT "zero recurrences", it is "could not
/// observe". Fixed-arg subprocess per the ADR-019 §4 bright-line (git named,
/// fixed args, no user-code exec).
fn count_commits_touching(depth: usize, pathspec: &str) -> Option<usize> {
    let out = std::process::Command::new("git")
        .args(["log", &format!("-{depth}"), "--format=%H", "--", pathspec])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).lines().count())
}

/// Like [`count_commits_touching`] but additionally requires the diff to
/// add/remove a line matching `regex` (`git log -G<regex>`) — for MSRV creep,
/// "touched Cargo.toml" over-counts; "changed a `rust-version` line" is the
/// real signal. `None` on git failure (honest degradation).
fn count_commits_changing_line(depth: usize, regex: &str, pathspec: &str) -> Option<usize> {
    let out = std::process::Command::new("git")
        .args([
            "log",
            &format!("-{depth}"),
            "--format=%H",
            &format!("-G{regex}"),
            "--",
            pathspec,
        ])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).lines().count())
}

/// `vcs recurrence`: mine git history for the three recurrent-emergence stdlib
/// failure-classes and surface their recurrence counts — the passive→active
/// loop (`infra/recurrence-automation`).
///
/// **The structural seam (pathmaker convergence, this expedition).** Mining
/// DETECTS the structural fact (this pattern fired N times across the window);
/// the VERDICT (is N a recurrence worth a `#[recurrence_anchor]`?) stays the
/// adopter's recognition call. So this command never auto-anchors and never
/// fails the build — it reports observations and points at the macro the
/// adopter would use. Parallel to camp's recurrence (which mines the team
/// ACTIVITY LOG for recurring work); antigen mines GIT HISTORY for recurring
/// failure-classes — different substrates, different objects, not competing.
///
/// **Honest degradation (the offline/no-repo axis).** When git is unavailable
/// each observation reads "unobservable", never "zero" — the absence of a mine
/// is not evidence of absence (tier-honest, the same discipline the coverage
/// audit uses for a flat scan).
fn run_vcs_recurrence(args: VcsRecurrenceArgs) -> ExitCode {
    // Each detector grounded in the recurrent stdlib antigen it feeds
    // (antigen/src/stdlib/recurrent.rs):
    //   MsrvCreepAfterMajorVersionBump   ← commits changing a rust-version line
    //   GitignorePatternDriftOverReleases ← commits touching .gitignore
    //   LockfileChurnFromUnpinnedTooling  ← commits touching Cargo.lock
    let msrv = count_commits_changing_line(args.depth, "rust-version", "*Cargo.toml");
    let gitignore = count_commits_touching(args.depth, ".gitignore");
    let lockfile = count_commits_touching(args.depth, "Cargo.lock");

    // A `None` from any detector means git itself is unavailable — degrade the
    // WHOLE command honestly rather than report a misleading partial.
    let (Some(msrv), Some(gitignore), Some(lockfile)) = (msrv, gitignore, lockfile) else {
        match args.format {
            OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::json!({ "observable": false, "reason": "git unavailable (not a repo, or git missing)" })
                );
            },
            OutputFormat::Human => {
                eprintln!(
                    "cargo antigen vcs recurrence: git unavailable (not a repo, or git \
                     missing) — recurrence is UNOBSERVABLE here, not zero."
                );
            },
        }
        // Honest-degradation is not an error verdict: exit 0 (the audit must not
        // be blocked by an unobservable mine).
        return ExitCode::SUCCESS;
    };

    let observations = [
        RecurrenceObservation {
            antigen: "MsrvCreepAfterMajorVersionBump",
            substrate: "commits changing a `rust-version` line in any Cargo.toml",
            count: msrv,
        },
        RecurrenceObservation {
            antigen: "GitignorePatternDriftOverReleases",
            substrate: "commits touching .gitignore",
            count: gitignore,
        },
        RecurrenceObservation {
            antigen: "LockfileChurnFromUnpinnedTooling",
            substrate: "commits touching Cargo.lock",
            count: lockfile,
        },
    ];

    match args.format {
        OutputFormat::Json => {
            let arr: Vec<_> = observations
                .iter()
                .map(|o| {
                    serde_json::json!({
                        "antigen": o.antigen,
                        "substrate": o.substrate,
                        "recurrence_count": o.count,
                        "window_commits": args.depth,
                    })
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "observable": true,
                    "observations": arr,
                }))
                .unwrap_or_else(|_| "{}".to_string())
            );
        },
        OutputFormat::Human => {
            println!(
                "Recurrent-emergence mine over the last {} commits (recurrent-emergence \
                 family, ADR-024):",
                args.depth
            );
            println!();
            for o in &observations {
                println!("  {:>4}×  {}", o.count, o.antigen);
                println!("        substrate: {}", o.substrate);
            }
            println!();
            println!(
                "  These are OBSERVATIONS, not verdicts. A high count is evidence the \
                 failure-class recurs in this repo — anchor it with \
                 #[recurrence_anchor(<Antigen>)] + #[itch(<Antigen>)] so the next \
                 occurrence is recognized, not re-discovered. Whether a count is \
                 'high enough' to anchor is your call (the recognition seam)."
            );
        },
    }
    ExitCode::SUCCESS
}

/// Read a single commit's message and parse its trailers via
/// `git show --format=%B -s <commit>` piped through
/// `git interpret-trailers --parse`. Fixed-arg subprocess per the ADR-019
/// bright-line rule. Returns `(name, value)` pairs; empty on any git failure
/// (tier-honest: no trailers → chain reads as absent).
fn read_commit_trailers(commit: &str) -> Vec<(String, String)> {
    let body = std::process::Command::new("git")
        .args(["show", "--no-patch", "--format=%B", commit])
        .output();
    let body_bytes = match body {
        Ok(o) if o.status.success() => o.stdout,
        _ => return Vec::new(),
    };
    let parsed = std::process::Command::new("git")
        .args(["interpret-trailers", "--parse"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(stdin) = child.stdin.take() {
                let mut stdin = stdin;
                let _ = stdin.write_all(&body_bytes);
            }
            child.wait_with_output()
        });
    match parsed {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
            .lines()
            .filter_map(|l| {
                let (k, v) = l.split_once(':')?;
                Some((k.trim().to_owned(), v.trim().to_owned()))
            })
            .collect(),
        _ => Vec::new(),
    }
}

fn run_vcs_check_commit(args: VcsCheckCommitArgs) -> ExitCode {
    use antigen::vcs_witness::RollbackTriageState;
    let trailers = read_commit_trailers(&args.commit);
    let state = RollbackTriageState::evaluate(&trailers);
    match args.format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::to_string_pretty(&state).unwrap_or_else(|_| "{}".to_string())
            );
        },
        OutputFormat::Human => match &state {
            RollbackTriageState::ChainPresent { decision } => {
                println!(
                    "commit {}: rollback-triage chain present (Triage-Decision: {})",
                    args.commit,
                    decision.as_str()
                );
            },
            RollbackTriageState::ChainMalformed { value } => {
                println!(
                    "commit {}: Triage-Decision trailer present but value {value:?} is not a \
                     valid triage decision (black|red|yellow|green|white)",
                    args.commit
                );
            },
            RollbackTriageState::ChainAbsent => {
                println!(
                    "commit {}: no Triage-Decision trailer — backs \
                     vcs-rollback-without-triage-commit if this is a rollback",
                    args.commit
                );
            },
        },
    }
    if args.strict && !state.is_pass() {
        return ExitCode::from(1);
    }
    ExitCode::SUCCESS
}

fn run_vcs_scan(args: VcsScanArgs) -> ExitCode {
    use antigen::vcs_witness::RollbackTriageState;
    // List recent commit shas + subjects; flag revert/reset-shaped commits
    // whose rollback-triage chain is absent.
    let log = std::process::Command::new("git")
        .args(["log", &format!("-{}", args.depth), "--format=%H%x1f%s"])
        .output();
    let log_text = match log {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).into_owned(),
        _ => {
            eprintln!("cargo antigen vcs scan: git log unavailable (not a repo, or git missing)");
            return ExitCode::from(2);
        },
    };
    let mut flagged = 0usize;
    let mut entries: Vec<(String, String, RollbackTriageState)> = Vec::new();
    for line in log_text.lines() {
        let Some((sha, subject)) = line.split_once('\u{1f}') else {
            continue;
        };
        // A revert-shaped subject is the v0.2 heuristic for "candidate
        // rollback" — the commit-trailer chain is then the real signal.
        let looks_like_rollback = subject.to_lowercase().contains("revert")
            || subject.to_lowercase().contains("rollback");
        if !looks_like_rollback {
            continue;
        }
        let state = RollbackTriageState::evaluate(&read_commit_trailers(sha));
        if !state.is_pass() {
            flagged += 1;
        }
        entries.push((sha.to_owned(), subject.to_owned(), state));
    }
    match args.format {
        OutputFormat::Json => {
            // Minimal JSON: array of {sha, subject, chain_pass}.
            let arr: Vec<_> = entries
                .iter()
                .map(|(sha, subject, state)| {
                    serde_json::json!({
                        "sha": sha,
                        "subject": subject,
                        "chain_pass": state.is_pass(),
                    })
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&arr).unwrap_or_else(|_| "[]".to_string())
            );
        },
        OutputFormat::Human => {
            if entries.is_empty() {
                println!(
                    "cargo antigen vcs scan: no revert/rollback-shaped commits in the last {} \
                     commits",
                    args.depth
                );
            } else {
                for (sha, subject, state) in &entries {
                    let short = &sha[..sha.len().min(8)];
                    let mark = if state.is_pass() { "ok " } else { "!! " };
                    println!("{mark}{short} {subject}");
                }
                println!();
                println!(
                    "{flagged} rollback-shaped commit(s) without a valid Triage-Decision chain"
                );
            }
        },
    }
    ExitCode::SUCCESS
}

fn run_vcs_branch_archive(args: VcsBranchArchiveArgs) -> ExitCode {
    if args.rationale.trim().is_empty() {
        eprintln!("cargo antigen vcs branch-archive: --rationale cannot be empty");
        return ExitCode::from(1);
    }
    // v0.2 advisory: print the sidecar JSON the adopter commits to
    // `.attest/vcs/branch-archive/<branch>.json` before deleting the branch.
    let sidecar = serde_json::json!({
        "branch": args.branch,
        "by_role": args.by,
        "rationale": args.rationale,
        "attested_at": chrono::Utc::now().to_rfc3339(),
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&sidecar).unwrap_or_else(|_| "{}".to_string())
    );
    eprintln!();
    eprintln!(
        "Write this to .attest/vcs/branch-archive/{}.json and commit it BEFORE \
         deleting the branch (v0.2 advisory; in-place write is v0.2.x).",
        args.branch
    );
    ExitCode::SUCCESS
}

fn run_vcs_rollback_prepare(args: VcsRollbackPrepareArgs) -> ExitCode {
    use antigen::vcs::TriageDecision;
    if TriageDecision::parse_decision(&args.decision).is_none() {
        eprintln!(
            "cargo antigen vcs rollback-prepare: --decision {:?} is not a valid triage \
             decision (black|red|yellow|green|white)",
            args.decision
        );
        return ExitCode::from(1);
    }
    println!("Add this trailer to your rollback commit message (ADR-026 Amendment 4):");
    println!();
    println!("    Triage-Decision: {}", args.decision.to_lowercase());
    println!();
    println!(
        "Rolling back to {}. The Triage-Decision trailer is the commit-intent signal \
         the rollback-triage chain reads — NOT a source-code scan.",
        args.target
    );
    ExitCode::SUCCESS
}

fn run_vcs_attest_stub() -> ExitCode {
    println!("cargo antigen vcs attest: v0.2.x stub");
    println!();
    println!("Generic VCS attestation-sidecar recording lands in v0.2.x. For now,");
    println!("use `branch-archive` (branch-deletion attestation) or `rollback-prepare`");
    println!("(triage-commit trailer scaffolding). Per ADR-005 Amendment 2 honest-tier-");
    println!("naming: this stub does NOT silently pass.");
    ExitCode::from(2)
}

// ============================================================================
// cargo antigen mucosal-map (Mucosal Boundary Family, ADR-027 + Amendment 1)
// ============================================================================

#[derive(Debug, Parser)]
struct MucosalMapArgs {
    /// Workspace root (default: current directory).
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Output format: human or json.
    #[arg(long, default_value = "human")]
    format: OutputFormat,
    /// List only declarations whose audit surfaced a concern.
    #[arg(long)]
    undefended: bool,
    /// List only `#[mucosal_tolerant]` (active-tolerance) declarations for
    /// periodic reviewer audit.
    #[arg(long)]
    tolerant: bool,
    /// Filter to a single `MucosalKind` (kebab-case, e.g. `user-input`).
    #[arg(long)]
    kind: Option<String>,
}

fn run_mucosal_map(args: MucosalMapArgs) -> ExitCode {
    use antigen::mucosal::MucosalKind;
    use antigen::scan::MucosalKindTag;

    if !args.root.is_dir() {
        eprintln!("error: expected a directory: {}", args.root.display());
        return ExitCode::from(2);
    }
    let report = match scan::scan_workspace(&args.root, None) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: scan failed: {e}");
            return ExitCode::from(2);
        },
    };
    let mucosal_audit = audit::audit_mucosal(&report);

    // Optional kind filter (validated against the sealed set).
    let kind_filter: Option<MucosalKind> = match args.kind.as_deref() {
        None => None,
        Some(k) => {
            let Some(mk) = MucosalKind::parse_kind(k) else {
                eprintln!(
                    "error: unknown MucosalKind {k:?}; expected a kebab-case variant \
                     (e.g. user-input, database-query, shell-argument)"
                );
                return ExitCode::from(2);
            };
            Some(mk)
        },
    };

    let entries: Vec<&antigen::audit::MucosalAudit> = mucosal_audit
        .audits
        .iter()
        .filter(|a| {
            if args.tolerant && a.declaration.tag != MucosalKindTag::MucosalTolerant {
                return false;
            }
            if args.undefended && a.hints.is_empty() {
                return false;
            }
            if let Some(kf) = kind_filter {
                // The scan stores the PascalCase final segment ("UserInput");
                // parse it back to a MucosalKind and compare enum values so
                // the --kind filter accepts any input form.
                let decl_kind = a
                    .declaration
                    .boundary_kind
                    .as_deref()
                    .and_then(MucosalKind::parse_kind);
                if decl_kind != Some(kf) {
                    return false;
                }
            }
            true
        })
        .collect();

    match args.format {
        OutputFormat::Json => {
            let arr: Vec<_> = entries
                .iter()
                .map(|a| {
                    serde_json::json!({
                        "tag": a.declaration.tag,
                        "boundary_kind": a.declaration.boundary_kind,
                        "file": a.declaration.file,
                        "line": a.declaration.line,
                        "hints": a.hints,
                    })
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&arr).unwrap_or_else(|_| "[]".to_string())
            );
        },
        OutputFormat::Human => {
            if entries.is_empty() {
                println!("cargo antigen mucosal-map: no matching mucosal declarations");
            } else {
                for a in &entries {
                    let kind = a.declaration.boundary_kind.as_deref().unwrap_or("?");
                    let mark = if a.hints.is_empty() { "ok " } else { "!! " };
                    let file = a.declaration.file.display();
                    println!(
                        "{mark}{:<18} {kind:<20} {file}:{}",
                        format!("{:?}", a.declaration.tag),
                        a.declaration.line
                    );
                    for hint in &a.hints {
                        println!("       - {hint:?}");
                    }
                }
                println!();
                println!(
                    "{} declaration(s); {} clean, {} with concerns",
                    entries.len(),
                    entries.iter().filter(|a| a.hints.is_empty()).count(),
                    entries.iter().filter(|a| !a.hints.is_empty()).count(),
                );
            }
        },
    }
    ExitCode::SUCCESS
}

/// Parse `<crate>@<version>` into `(crate, version)`. Returns `None` on
/// any of: no `@`, empty crate, empty version, multiple `@`s, or characters
/// that could escape the `.attest/supply-chain/` directory (path traversal
/// guard: crate names must be `[a-zA-Z0-9_-]`, versions `[a-zA-Z0-9._+-]`).
fn parse_crate_at_version(s: &str) -> Option<(String, String)> {
    let (c, v) = s.split_once('@')?;
    if c.is_empty() || v.is_empty() || v.contains('@') {
        return None;
    }
    let crate_ok = c
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '-');
    let version_ok = v
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '+'));
    if !crate_ok || !version_ok {
        return None;
    }
    Some((c.to_string(), v.to_string()))
}

fn main() -> ExitCode {
    let cli = CargoCli::parse();
    let CargoSubcommand::Antigen(antigen_cli) = cli.command;

    match antigen_cli.command {
        AntigenSubcommand::Scan(args) => run_scan(args),
        AntigenSubcommand::Propose(args) => run_propose(args),
        AntigenSubcommand::New { name } => run_new(name),
        AntigenSubcommand::Vaccinate { antigen, pattern } => run_vaccinate(antigen, pattern),
        AntigenSubcommand::Audit(args) => run_audit(args),
        AntigenSubcommand::Attest(cli) => run_attest(cli),
        AntigenSubcommand::Tolerate(cli) => run_tolerate(cli),
        AntigenSubcommand::Oracle(cli) => run_oracle(cli),
        AntigenSubcommand::Verify(cli) => run_verify(cli),
        AntigenSubcommand::Vcs(cli) => run_vcs(cli),
        AntigenSubcommand::MucosalMap(args) => run_mucosal_map(args),
        AntigenSubcommand::Fingerprint(args) => run_fingerprint(args),
    }
}

// ============================================================================
// cargo antigen oracle handlers (ADR-021 oracle-as-artifact-class)
// ============================================================================

fn run_oracle(cli: OracleCli) -> ExitCode {
    match cli.command {
        OracleSubcommand::List(args) => run_oracle_list(args),
        OracleSubcommand::Status(args) => run_oracle_status(args),
        OracleSubcommand::Declare(args) => run_oracle_declare(args),
        OracleSubcommand::Complete(args) => run_oracle_complete(args),
        OracleSubcommand::Deprecate(args) => run_oracle_deprecate(args),
        OracleSubcommand::Retire(args) => run_oracle_retire(args),
        OracleSubcommand::Revoke(args) => run_oracle_revoke(args),
    }
}

fn oracle_json_path(root: &std::path::Path, id: &str) -> std::path::PathBuf {
    root.join(".antigen")
        .join("oracles")
        .join(format!("{id}.oracle.json"))
}

fn load_oracle(
    root: &std::path::Path,
    id: &str,
) -> Result<antigen_attestation::schema::Oracle, String> {
    let path = oracle_json_path(root, id);
    let content = std::fs::read_to_string(&path).map_err(|e| {
        format!(
            "error: oracle `{id}` not found at `{}`: {e}",
            path.display()
        )
    })?;
    serde_json::from_str(&content)
        .map_err(|e| format!("error: oracle `{id}` is not valid Oracle JSON: {e}"))
}

fn save_oracle(
    root: &std::path::Path,
    oracle: &antigen_attestation::schema::Oracle,
) -> Result<(), String> {
    // Tier-honesty: validate schema invariants BEFORE persisting to disk
    // (ADR-021 §D3 + Oracle::validate). Persisting an invalid oracle would
    // mean the audit later refuses to parse what the CLI wrote — a silent
    // tier-inversion. Catch at write time so the operator sees the
    // validation error immediately instead of at next audit.
    oracle
        .validate()
        .map_err(|e| format!("error: oracle validation failed: {e}"))?;
    let dir = root.join(".antigen").join("oracles");
    std::fs::create_dir_all(&dir).map_err(|e| {
        format!(
            "error: could not create oracle directory `{}`: {e}",
            dir.display()
        )
    })?;
    let path = oracle_json_path(root, &oracle.id);
    let json = serde_json::to_string_pretty(oracle)
        .map_err(|e| format!("error: failed to serialize oracle: {e}"))?;
    std::fs::write(&path, &json)
        .map_err(|e| format!("error: failed to write oracle `{}`: {e}", path.display()))?;
    Ok(())
}

fn resolve_steward_name(explicit: Option<&str>) -> Result<String, String> {
    explicit.map_or_else(
        || {
            let out = std::process::Command::new("git")
                .args(["config", "user.name"])
                .output();
            match out {
                Ok(o) if o.status.success() => {
                    Ok(String::from_utf8_lossy(&o.stdout).trim().to_owned())
                },
                _ => Err(
                    "error: --steward not provided and `git config user.name` failed".to_owned(),
                ),
            }
        },
        |s| Ok(s.to_owned()),
    )
}

fn run_oracle_list(args: OracleListArgs) -> ExitCode {
    let oracle_dir = args.root.join(".antigen").join("oracles");
    if !oracle_dir.exists() {
        eprintln!("No oracle records found under `{}`.", args.root.display());
        return ExitCode::SUCCESS;
    }
    let entries = match std::fs::read_dir(&oracle_dir) {
        Ok(e) => e,
        Err(e) => {
            eprintln!("error: could not read oracle directory: {e}");
            return ExitCode::from(2);
        },
    };
    let mut found = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            // Warn-and-continue on unreadable / corrupt oracle records rather
            // than silently skipping them. A silent skip lets a corrupt `.json`
            // collapse into the same "No oracle records found" + exit 0 as a
            // genuinely-empty directory — the adopter gets zero signal their
            // oracle data is broken (ATK-oracle-corrupt-json). Mirrors the
            // `attest list` warn-and-continue model.
            let content = match std::fs::read_to_string(&path) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("warning: could not read `{}`: {e}", path.display());
                    continue;
                },
            };
            let oracle = match serde_json::from_str::<antigen_attestation::schema::Oracle>(&content)
            {
                Ok(o) => o,
                Err(e) => {
                    eprintln!(
                        "warning: `{}` is not valid Oracle JSON: {e}",
                        path.display()
                    );
                    continue;
                },
            };
            match args.format {
                OutputFormat::Human => {
                    println!(
                        "{} [{:?}] — {} steward(s)",
                        oracle.id,
                        oracle.state,
                        oracle.stewards.len()
                    );
                },
                OutputFormat::Json => {
                    let obj = serde_json::json!({ "id": oracle.id, "state": format!("{:?}", oracle.state) });
                    println!("{obj}");
                },
            }
            found += 1;
        }
    }
    if found == 0 {
        eprintln!("No oracle records found under `{}`.", args.root.display());
    } else {
        eprintln!("{found} oracle(s) listed.");
    }
    ExitCode::SUCCESS
}

fn run_oracle_status(args: OracleStatusArgs) -> ExitCode {
    match load_oracle(&args.root, &args.id) {
        Ok(oracle) => {
            println!("Oracle: {}", oracle.id);
            println!("State:  {:?}", oracle.state);
            println!("Stewards ({}):", oracle.stewards.len());
            for s in &oracle.stewards {
                println!("  {} ({})", s.name, s.authorization_basis);
            }
            println!("Transitions ({}):", oracle.transitions.len());
            for t in &oracle.transitions {
                println!(
                    "  {} → {} by {} on {} — {}",
                    t.from, t.to, t.authorized_by, t.at, t.rationale
                );
            }
            ExitCode::SUCCESS
        },
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(1)
        },
    }
}

/// Count the existing delta-chain depth for this signer since their last
/// Fresh-basis entry. CLI-SF-4 fix: a naive `take_while` on the reversed list
/// stops at the first non-(name+delta) entry, undercounting when other
/// signers have interleaved entries. Correct computation: find the last
/// Fresh entry for this signer, then count all subsequent delta entries
/// for this signer regardless of interleaving. Extracted from
/// `run_attest_delta` to keep that function under clippy's `too_many_lines`
/// threshold.
fn compute_delta_chain_depth(signers: &[antigen_attestation::Signer], signer_name: &str) -> u32 {
    let last_fresh_index = signers
        .iter()
        .enumerate()
        .filter(|(_, s)| s.name == signer_name && s.basis.is_fresh())
        .map(|(i, _)| i)
        .next_back();
    // No-Fresh case: caller's guard above already rejected this path; the `1`
    // here is defensive and unreachable in practice.
    last_fresh_index.map_or(1, |fresh_idx| {
        let count = signers[fresh_idx + 1..]
            .iter()
            .filter(|s| s.name == signer_name && s.basis.is_delta())
            .count();
        u32::try_from(count).unwrap_or(u32::MAX).saturating_add(1)
    })
}

/// Anti-laundering T2R-B/T2R-C: enforce minimum rationale character count at
/// CLI layer for delta attestations. The schema `validate()` also enforces
/// this at audit time, but catching here prevents writing a sidecar that
/// immediately fails validation (CLI-SF-3). Extracted from `run_attest_delta`
/// to keep that function under clippy's `too_many_lines` threshold.
fn validate_delta_rationale(rationale: &str) -> Result<(), ExitCode> {
    use antigen_attestation::schema::DEFAULT_DELTA_RATIONALE_MIN_CHARS;
    let trimmed = rationale.trim();
    if trimmed.is_empty() {
        eprintln!("error: --rationale must be non-empty (anti-laundering safeguard T2R-C)");
        return Err(ExitCode::from(1));
    }
    if trimmed.chars().count() < DEFAULT_DELTA_RATIONALE_MIN_CHARS {
        eprintln!(
            "error: --rationale is too short ({} chars); minimum is {} chars \
             (anti-laundering safeguard T2R-B). Rubber-stamp rationales are rejected.",
            trimmed.chars().count(),
            DEFAULT_DELTA_RATIONALE_MIN_CHARS,
        );
        return Err(ExitCode::from(1));
    }
    Ok(())
}

/// Build an [`antigen_attestation::schema::OracleRef`] from CLI args.
/// Extracted from `run_oracle_declare`
/// to keep that function under clippy's `too_many_lines` threshold. Returns
/// `Err(ExitCode)` on user-facing parse errors so the caller can propagate.
fn build_oracle_ref(
    kind: OracleRefKindArg,
    reference: &str,
) -> Result<antigen_attestation::schema::OracleRef, ExitCode> {
    use antigen_attestation::schema::OracleRef;
    match kind {
        OracleRefKindArg::LocalFile => Ok(OracleRef::LocalFile {
            path: std::path::PathBuf::from(reference),
            status_field: None,
            expected_status: None,
        }),
        OracleRefKindArg::Url => Ok(OracleRef::Url {
            url: reference.to_owned(),
            label: None,
        }),
        OracleRefKindArg::Doi => Ok(OracleRef::Doi {
            doi: reference.to_owned(),
            section: None,
        }),
        OracleRefKindArg::Arxiv => Ok(OracleRef::Arxiv {
            arxiv_id: reference.to_owned(),
            section: None,
        }),
        OracleRefKindArg::GithubIssue => {
            let parts: Vec<&str> = reference.splitn(2, '#').collect();
            if parts.len() != 2 {
                eprintln!("error: github-issue reference must be `owner/repo#N`");
                return Err(ExitCode::from(1));
            }
            let Ok(issue) = parts[1].parse::<u32>() else {
                eprintln!("error: issue number must be a positive integer");
                return Err(ExitCode::from(1));
            };
            Ok(OracleRef::GitHubIssue {
                repo: parts[0].to_owned(),
                issue,
            })
        },
        OracleRefKindArg::Other => Ok(OracleRef::Other {
            subkind: "other".to_owned(),
            reference: reference.to_owned(),
            label: None,
        }),
    }
}

fn run_oracle_declare(args: OracleDeclareArgs) -> ExitCode {
    use antigen_attestation::schema::{Oracle, OracleState, OracleVersion, Provenance, Steward};
    use chrono::Local;

    if args.rationale.trim().is_empty() {
        eprintln!("error: --rationale must be non-empty (Amendment 2 discipline)");
        return ExitCode::from(1);
    }

    // Resolve stewards: either from --steward flags or git config.
    let mut steward_names = args.steward.clone();
    if steward_names.is_empty() {
        match resolve_steward_name(None) {
            Ok(name) => steward_names.push(name),
            Err(e) => {
                eprintln!("{e}");
                return ExitCode::from(1);
            },
        }
    }
    if steward_names.len() < 2 {
        eprintln!(
            "warning: only {} steward(s) declared; minimum 2 required for oracle to be transitioned \
             to Complete (ATK-021-13 succession mitigation). Add a second steward via --steward.",
            steward_names.len()
        );
    }

    let stewards: Vec<Steward> = steward_names
        .iter()
        .map(|name| Steward {
            name: name.clone(),
            role: None,
            authorization_basis: args.rationale.clone(),
        })
        .collect();

    let reference = match build_oracle_ref(args.kind, &args.reference) {
        Ok(r) => r,
        Err(code) => return code,
    };

    let today = Local::now().date_naive();
    // Declare does NOT write a creation-as-transition. The creation event is
    // recorded by `created: Provenance` (recorded_by + at). The transitions log
    // captures STATE-MACHINE TRANSITIONS only — Draft is the constructor state,
    // so the first transition entry is the eventual Draft→Complete written by
    // `oracle complete`. The schema validator (Oracle::validate) enforces that
    // the first transition's `from` matches the implicit initial state
    // `"draft"`; a synthetic `none → draft` creation-transition would fail
    // validation. ATK-021 schema-CLI integration test:
    // atk_a3_oracle_cli_declare_initial_state_is_draft + ..._full_lifecycle_round_trip.
    let oracle = Oracle {
        id: args.id.clone(),
        reference,
        state: OracleState::Draft,
        stewards,
        created: Provenance {
            recorded_by: steward_names[0].clone(),
            at: today,
        },
        version: OracleVersion {
            pinned: args.version.unwrap_or_else(|| format!("declared-{today}")),
            pinned_at: today,
        },
        transitions: vec![],
        extensions: std::collections::BTreeMap::default(),
    };

    match save_oracle(&args.root, &oracle) {
        Ok(()) => {
            eprintln!("Oracle `{}` created in DRAFT state.", args.id);
            eprintln!("Path: {}", oracle_json_path(&args.root, &args.id).display());
            ExitCode::SUCCESS
        },
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        },
    }
}

const fn oracle_state_discriminant(
    state: &antigen_attestation::schema::OracleState,
) -> &'static str {
    use antigen_attestation::schema::OracleState;
    match state {
        OracleState::Draft => "draft",
        OracleState::Complete => "complete",
        OracleState::Deprecated { .. } => "deprecated",
        OracleState::Retired { .. } => "retired",
        OracleState::Revoked { .. } => "revoked",
    }
}

fn run_oracle_transition(
    root: &std::path::Path,
    id: &str,
    steward: Option<String>,
    rationale: &str,
    new_state: antigen_attestation::schema::OracleState,
) -> ExitCode {
    use antigen_attestation::schema::StateTransition;
    use chrono::Local;

    if rationale.trim().is_empty() {
        eprintln!("error: --rationale must be non-empty (Amendment 2 discipline)");
        return ExitCode::from(1);
    }
    let steward_name = match resolve_steward_name(steward.as_deref()) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        },
    };
    let mut oracle = match load_oracle(root, id) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        },
    };
    // Validate steward is in the steward list (ATK-021-15).
    if !oracle.stewards.iter().any(|s| s.name == steward_name) {
        eprintln!(
            "error: `{steward_name}` is not a declared steward of oracle `{id}`.\n\
             Declared stewards: {}",
            oracle
                .stewards
                .iter()
                .map(|s| s.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        return ExitCode::from(1);
    }
    // Derive the `from` label from the oracle's CURRENT state (not a hardcoded
    // string per call-site). This is what the schema validator expects per
    // ADR-021 §D3 — the `from` of transition N must match the `to` of
    // transition N-1 (or the implicit "draft" initial state when N=1).
    let from_label = oracle_state_discriminant(&oracle.state).to_owned();
    let to_label = oracle_state_discriminant(&new_state).to_owned();
    let today = Local::now().date_naive();
    oracle.transitions.push(StateTransition {
        from: from_label.clone(),
        to: to_label.clone(),
        authorized_by: steward_name.clone(),
        at: today,
        rationale: rationale.to_owned(),
    });
    oracle.state = new_state;
    match save_oracle(root, &oracle) {
        Ok(()) => {
            eprintln!("Oracle `{id}` transitioned {from_label}→{to_label} by {steward_name}.");
            ExitCode::SUCCESS
        },
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        },
    }
}

fn run_oracle_complete(args: OracleCompleteArgs) -> ExitCode {
    use antigen_attestation::schema::{OracleState, OracleVersion};
    use chrono::Local;

    // Load oracle to update version pin at completion time.
    let mut oracle = match load_oracle(&args.root, &args.id) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        },
    };
    oracle.version = OracleVersion {
        pinned: args.version.clone(),
        pinned_at: Local::now().date_naive(),
    };
    match save_oracle(&args.root, &oracle) {
        Ok(()) => {},
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        },
    }
    run_oracle_transition(
        &args.root,
        &args.id,
        args.steward,
        &args.rationale,
        OracleState::Complete,
    )
}

fn run_oracle_deprecate(args: OracleDeprecateArgs) -> ExitCode {
    use antigen_attestation::schema::OracleState;
    let new_state = OracleState::Deprecated {
        superseded_by: args.superseded_by,
        reason: args.rationale.clone(),
    };
    run_oracle_transition(
        &args.root,
        &args.id,
        args.steward,
        &args.rationale,
        new_state,
    )
}

fn run_oracle_retire(args: OracleRetireArgs) -> ExitCode {
    use antigen_attestation::schema::OracleState;
    let steward_name = match resolve_steward_name(args.steward.as_deref()) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        },
    };
    let new_state = OracleState::Retired {
        reason: args.rationale.clone(),
        retired_by: steward_name.clone(),
    };
    run_oracle_transition(
        &args.root,
        &args.id,
        Some(steward_name),
        &args.rationale,
        new_state,
    )
}

fn run_oracle_revoke(args: OracleRevokeArgs) -> ExitCode {
    use antigen_attestation::schema::OracleState;
    let steward_name = match resolve_steward_name(args.steward.as_deref()) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        },
    };
    let new_state = OracleState::Revoked {
        reason: args.rationale.clone(),
        revoked_by: steward_name.clone(),
        invalidates_prior_attestations: args.invalidates_prior,
    };
    run_oracle_transition(
        &args.root,
        &args.id,
        Some(steward_name),
        &args.rationale,
        new_state,
    )
}

/// Parse the `--category` CLI argument (ADR-028 §CLI integration). Returns
/// `Ok(None)` when no filter was supplied, `Ok(Some(cat))` for a valid
/// category string, and `Err(())` (after printing a diagnostic) for an
/// unrecognized value.
fn parse_category_filter(
    raw: Option<&str>,
) -> Result<Option<antigen::category::AntigenCategory>, ()> {
    let Some(s) = raw else {
        return Ok(None);
    };
    antigen::category::AntigenCategory::parse_category(s)
        .map(Some)
        .ok_or_else(|| {
            eprintln!(
                "error: unrecognized --category `{s}`; \
                 expected `substrate-alignment` or `functional-correctness`"
            );
        })
}

/// Filter a scan report in place to antigen declarations of `cat` (ADR-028
/// §CLI integration). A hybrid antigen (both categories) matches either
/// filter. Antigens with an absent category default to `FunctionalCorrectness`
/// per ADR-028 §v0.2-backward-compat, so they match the functional-correctness
/// filter. Presentations, immunities, and tolerations addressing a dropped
/// antigen are pruned so the filtered view stays internally consistent.
fn filter_report_by_category(
    report: &mut scan::ScanReport,
    cat: antigen::category::AntigenCategory,
) {
    use antigen::category::AntigenCategory;

    let matches = |decl_category: &[AntigenCategory]| -> bool {
        if decl_category.is_empty() {
            // Absent category defaults to FunctionalCorrectness.
            cat == AntigenCategory::FunctionalCorrectness
        } else {
            decl_category.contains(&cat)
        }
    };

    report.antigens.retain(|a| matches(&a.category));
    let kept: std::collections::HashSet<String> = report
        .antigens
        .iter()
        .map(|a| a.type_name.clone())
        .collect();

    report
        .presentations
        .retain(|p| kept.contains(&p.antigen_type));
    report.immunities.retain(|i| kept.contains(&i.antigen_type));
    report.tolerances.retain(|t| kept.contains(&t.antigen_type));
}

/// Validate `--root` and run the appropriate scan (flat or member-aware).
///
/// Returns the [`scan::ScanReport`] on success, or the [`ExitCode`] the caller
/// should propagate on a validation/scan failure. Extracted from [`run_scan`]
/// so the body of `run_scan` stays under the clippy line cap and the
/// flat-vs-member-aware dispatch lives in one place.
fn acquire_scan_report(args: &ScanArgs) -> Result<scan::ScanReport, ExitCode> {
    if !args.root.exists() {
        eprintln!("error: path does not exist: {}", args.root.display());
        return Err(ExitCode::from(2));
    }
    if !args.root.is_dir() {
        eprintln!(
            "error: expected a directory, got a file: {}",
            args.root.display()
        );
        return Err(ExitCode::from(2));
    }

    if args.workspace {
        if args.bundled_catalog {
            eprintln!(
                "warning: --bundled-catalog has no effect with --workspace (member-aware scan); \
                 use a flat scan to inject the bundled catalog"
            );
        }
        eprintln!("Scanning workspace (member-aware): {}", args.root.display());
        scan::scan_workspace_multi_crate(&args.root)
    } else if args.bundled_catalog {
        eprintln!(
            "Scanning workspace (bundled stdlib catalog): {}",
            args.root.display()
        );
        // Captain's ruling / ADR-043 Amendment 2: an EXPLICIT --bundled-catalog
        // flag ALWAYS injects (augments the crate's own antigens with the bundled
        // catalog), regardless of how many local antigens the crate declares. The
        // real adopters (tambear, camp) are PARTIAL adopters, not blank crates —
        // a partial adopter who explicitly asks for the catalog and gets it
        // suppressed is the exact silent-miss E0 exists to kill. (auto_detect =
        // false → Always.) The auto-detect-when-empty convenience is the *no-flag*
        // default, handled by the plain scan path; it is not reachable here.
        scan::scan_workspace_bundled_catalog(&args.root, None, false)
    } else {
        eprintln!("Scanning workspace: {}", args.root.display());
        // No flag: AUTO-DETECT (captain's ruling / ADR-043 Amd-2 case 2). A crate
        // with ZERO local antigens auto-injects the bundled catalog — that closes
        // the zero-hits-cliff for a total newcomer who didn't know to pass the
        // flag. A crate WITH local antigens stays local-only (it has its own
        // declarations). `auto_detect = true` encodes exactly this: inject iff
        // report.antigens.is_empty().
        scan::scan_workspace_bundled_catalog(&args.root, None, true)
    }
    .map_err(|e| {
        eprintln!("error: scan failed: {e}");
        ExitCode::from(2)
    })
}

/// Render B (editor-flycheck): emit the scan's fingerprint matches as the
/// cargo/rustc JSON line-protocol on stdout, then return success.
///
/// The class→provenance map is built from BOTH the in-tree antigen declarations
/// (their authored provenance) and the bundled stdlib catalog, so a flycheck run
/// surfaces matches against either repertoire with the honest provenance label.
/// The match findings are projected by the E1 catalog-match spine (a scan-fact
/// `FingerprintMatch`, never an audited verdict — claim-scope, ADR-044).
fn emit_flycheck_json(report: &scan::ScanReport) -> ExitCode {
    use std::collections::HashMap;

    let mut provenance_by_class: HashMap<String, antigen::finding::Provenance> = report
        .antigens
        .iter()
        .map(|a| (a.type_name.clone(), a.resolved_provenance()))
        .collect();
    // The bundled catalog augments the in-tree map; in-tree authored provenance
    // wins on a name collision (entry() keeps the existing value).
    for entry in antigen::stdlib::catalog::stdlib_catalog_entries() {
        provenance_by_class
            .entry(entry.name)
            .or_insert(entry.provenance);
    }

    let findings = scan::catalog_match_findings(report, &provenance_by_class);
    match antigen::render::flycheck::findings_to_cargo_jsonl(&findings) {
        Ok(jsonl) => {
            print!("{jsonl}");
            ExitCode::SUCCESS
        },
        Err(e) => {
            eprintln!("error: failed to serialize flycheck JSON: {e}");
            ExitCode::from(2)
        },
    }
}

fn run_scan(args: ScanArgs) -> ExitCode {
    // ADR-028 §CLI integration: --category filters to a single category.
    let Ok(category_filter) = parse_category_filter(args.category.as_deref()) else {
        return ExitCode::from(2);
    };

    let mut report = match acquire_scan_report(&args) {
        Ok(r) => r,
        Err(code) => return code,
    };

    if let Some(cat) = category_filter {
        filter_report_by_category(&mut report, cat);
    }

    // v0.4 render B — editor-flycheck. When `--message-format json` is set, emit
    // the cargo/rustc JSON line-protocol (one `compiler-message` per
    // fingerprint match) and return; this is the rust-analyzer `check.overrideCommand`
    // surface, distinct from `--format json` (antigen's own envelope).
    if matches!(args.message_format, Some(MessageFormat::Json)) {
        return emit_flycheck_json(&report);
    }

    let unaddressed = report.unaddressed_presentations();

    // Optional cross-crate dep enumeration + per-crate scan. Deps are scanned
    // independently — no cross-crate addresses() matching yet. Each dep report
    // stands on its own; the union appears under `dep_reports` in JSON output.
    let dep_reports = if args.include_deps {
        match scan::enumerate_dep_crate_roots(&args.root, false) {
            Ok(roots) => {
                eprintln!("Scanning {} dependency crate(s)...", roots.len());
                let mut out: Vec<DepScanResult> = Vec::with_capacity(roots.len());
                for dep in roots {
                    let mut r = match scan::scan_workspace(&dep.crate_root, None) {
                        Ok(r) => r,
                        Err(e) => {
                            eprintln!(
                                "  warning: scan failed for `{}` v{}: {e}",
                                dep.package_name, dep.version
                            );
                            continue;
                        },
                    };
                    // ADR-017 Option A: stamp canonical_path post-scan in
                    // the `"<crate-name>@<version>"` format. The
                    // scan_workspace function is a pure directory scanner
                    // and does not know which crate it just scanned; the
                    // driver carries the crate-graph context and stamps
                    // here.
                    let crate_id = format!("{}@{}", dep.package_name, dep.version);
                    r.stamp_canonical_path(&crate_id);
                    out.push(DepScanResult {
                        package_name: dep.package_name,
                        version: dep.version,
                        origin: dep.origin,
                        report: r,
                    });
                }
                Some(out)
            },
            Err(e) => {
                eprintln!("error: cargo metadata failed: {e}");
                return ExitCode::from(2);
            },
        }
    } else {
        None
    };

    // The report is a live projection — recomputed above, never read back from
    // a store. Wrap the JSON payload in the provenance envelope so any render
    // (console-json or a `--output` file) is self-describing. The envelope's
    // keys are additive siblings of the flattened payload, so existing JSON
    // consumers that navigate by key (`report.report.presentations`, …) are
    // byte-compatible.
    let enveloped = ReportEnvelope::new(
        &args.root,
        JsonReport {
            report: &report,
            unaddressed: &unaddressed,
            orphaned_lineage_edges: report.orphaned_lineage_edges(),
            dangling_child_lineage_edges: report.dangling_child_lineage_edges(),
            dep_reports: dep_reports.as_deref(),
        },
    );

    // `--output <file>` writes the full JSON render regardless of console
    // `--format`, so CI can print a human summary AND save the machine detail.
    if let Some(path) = args.output.as_ref() {
        if let Err(e) = write_report_render(path, &enveloped) {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    }

    match args.format {
        OutputFormat::Human => {
            print_human_report(&report, &unaddressed);
            if let Some(deps) = dep_reports.as_ref() {
                print_human_dep_summary(deps);
            }
        },
        OutputFormat::Json => match serde_json::to_string_pretty(&enveloped) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("error: failed to serialize report: {e}");
                return ExitCode::from(2);
            },
        },
    }

    // --strict gates only on unaddressed EXPLICIT presentations (match_kind == ExplicitMarker)
    // and orphaned tolerances. Fingerprint matches are informational — they're potential
    // vulnerabilities that require human triage, not CI-enforced gates.
    // Gating on ALL unaddressed (including FingerprintMatch) produces a silent mismatch:
    // the output says "All explicit presentations are addressed" but exits 1 because of
    // fingerprint matches, causing CI to fail with a confusing human-readable message.
    let unaddressed_explicit_count = unaddressed
        .iter()
        .filter(|u| u.presentation.match_kind == antigen::scan::MatchKind::ExplicitMarker)
        .count();
    // Orphaned/dangling lineage edges are structural defects (a #[descended_from]
    // pointing at a non-existent parent, or on a non-antigen child) — gate on them
    // under --strict alongside unaddressed explicit presentations and orphaned
    // tolerances.
    let lineage_broken = !report.orphaned_lineage_edges().is_empty()
        || !report.dangling_child_lineage_edges().is_empty();
    if args.strict
        && (unaddressed_explicit_count > 0
            || !report.orphaned_tolerances().is_empty()
            || lineage_broken)
    {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

/// Render a brief summary of dep scan results in human format. Per
/// navigator's ruling, dep reports are independent — we only summarize
/// counts here (full per-crate detail goes through --format json).
fn print_human_dep_summary(deps: &[DepScanResult]) {
    println!();
    println!("Cross-crate dep scan ({} crates):", deps.len());
    let mut deps_with_antigens: Vec<&DepScanResult> = deps
        .iter()
        .filter(|d| !d.report.antigens.is_empty())
        .collect();
    deps_with_antigens.sort_by_key(|d| d.package_name.clone());
    if deps_with_antigens.is_empty() {
        println!("  No antigen declarations found in any dependency.");
        println!("  (Few published crates declare antigens.)");
    } else {
        for d in deps_with_antigens {
            println!(
                "  {} v{}: {} antigen(s), {} presentation(s), {} defense(s)",
                d.package_name,
                d.version,
                d.report.antigens.len(),
                d.report.presentations.len(),
                d.report.immunities.len()
            );
        }
    }
}

fn print_human_report(report: &scan::ScanReport, unaddressed: &[scan::UnaddressedPresentation]) {
    use antigen::scan::MatchKind;

    let explicit_count = report
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::ExplicitMarker)
        .count();
    let fingerprint_count = report
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::FingerprintMatch)
        .count();

    println!();
    println!(
        "Scanned {} files, found {} antigen-related declarations:",
        report.files_scanned,
        report.total_declarations()
    );
    println!("  - {} antigen declarations", report.antigens.len());
    println!("  - {} explicit #[presents] markers", explicit_count);
    if fingerprint_count > 0 {
        println!("  - {fingerprint_count} fingerprint matches (candidate sites — see below)");
    }
    if !report.tolerances.is_empty() {
        println!(
            "  - {} tolerated sites (#[antigen_tolerance])",
            report.tolerances.len()
        );
    }
    if !report.defenses.is_empty() {
        println!("  - {} #[defended_by] declarations", report.defenses.len());
    }
    if !report.immunities.is_empty() {
        println!(
            "  - {} #[immune] declarations (deprecated — migrate to #[defended_by]/#[presents])",
            report.immunities.len()
        );
    }
    if !report.parse_failures.is_empty() {
        println!(
            "  - {} parse failures (see --format json for details)",
            report.parse_failures.len()
        );
    }
    println!();

    print_fingerprint_matches(report);
    print_orphaned_tolerances(report);
    print_lineage_integrity(report);
    print_unaddressed(unaddressed);
}

/// Render `#[descended_from]` lineage-integrity defects: edges whose parent
/// antigen is not declared in the workspace (orphaned) and edges whose child is
/// not an `#[antigen]` declaration (dangling). Both are computed by the scan
/// layer (`orphaned_lineage_edges` / `dangling_child_lineage_edges`) but were
/// never rendered — the `AuditVerdictComputedButNotDelivered` delivery-arm
/// severance (same shape as dogfood antigen #25). A `#[descended_from(P)]` that
/// names a parent the workspace doesn't declare is a structurally-broken lineage
/// claim; surfacing it is the whole point of the check.
fn print_lineage_integrity(report: &scan::ScanReport) {
    let orphaned = report.orphaned_lineage_edges();
    let dangling = report.dangling_child_lineage_edges();
    if orphaned.is_empty() && dangling.is_empty() {
        return;
    }
    if !orphaned.is_empty() {
        println!(
            "{} orphaned lineage edge(s) — #[descended_from] names a parent antigen \
             not declared in the workspace:",
            orphaned.len()
        );
        println!();
        for e in &orphaned {
            println!(
                "  {}:{}  {} ⟶ {} [parent antigen `{}` not found]",
                e.file.display(),
                e.line,
                e.child,
                e.parent,
                e.parent
            );
        }
        println!();
    }
    if !dangling.is_empty() {
        println!(
            "{} dangling lineage edge(s) — the child of a #[descended_from] is not \
             itself an #[antigen] declaration:",
            dangling.len()
        );
        println!();
        for e in &dangling {
            println!(
                "  {}:{}  {} ⟶ {} [child `{}` has no #[antigen] declaration]",
                e.file.display(),
                e.line,
                e.child,
                e.parent,
                e.child
            );
        }
        println!();
    }
    println!(
        "  A lineage claim must resolve to real declarations on both ends. Either \
         declare the missing antigen, or remove/correct the #[descended_from] edge."
    );
    println!();
}

/// Render deferred-defense declarations (`#[anergy]` / `#[immunosuppress]` /
/// `#[poxparty]` / `#[orient]`) LOUDLY (ADR-023 + forward/suppression-loud-must-
/// be-removed). These are intentional dev permissions to proceed with a known
/// defense gap — they do NOT block the build and do NOT auto-expire, but the
/// audit must ALWAYS announce active ones prominently so they cannot become
/// silent accumulated debt. `audit_deferred_defenses` computes the state; this
/// is its delivery arm (it was computed in the library but never reached the
/// CLI — the `AuditVerdictComputedButNotDelivered` severance). Each must be
/// EXPLICITLY REMOVED to resolve; the audit keeps surfacing it until then.
fn print_deferred_defenses_loud(report: &audit::DeferredDefenseAuditReport) {
    use antigen::audit::AuditHint;
    use antigen::scan::DeferredDefenseKind;

    if report.audits.is_empty() {
        return;
    }

    println!(
        "⚠ {} deferred-defense declaration(s) — intentional, accepted defense gaps \
         (NOT blocking; must be EXPLICITLY removed to resolve):",
        report.audits.len()
    );
    println!(
        "  {} active · {} past-deadline · {} stale",
        report.active_count, report.expired_count, report.stale_count
    );
    println!();

    for a in &report.audits {
        let d = &a.declaration;
        let kind = match d.kind {
            DeferredDefenseKind::Anergy => "anergy",
            DeferredDefenseKind::Immunosuppress => "immunosuppress",
            DeferredDefenseKind::Poxparty => "poxparty",
            DeferredDefenseKind::Orient => "orient",
        };
        let antigen = d.antigen_type.as_deref().unwrap_or("(unlinked)");
        // State marker from the audit hint — past-deadline/stale states are
        // louder so an accumulated old suppression stands out from a fresh one.
        let state = match a.hint {
            AuditHint::AnergyStale => "STALE — long past review",
            AuditHint::AnergyCostimulationNotArrived
            | AuditHint::ImmunosuppressExpired
            | AuditHint::PoxpartyOutcomePending
            | AuditHint::OrientPendingActionRequired => "PAST DEADLINE — action required",
            // All active hints (AnergyActive / ImmunosuppressActive /
            // PoxpartyActive / OrientActive) — and any future deferred-defense
            // hint not yet classified expired/stale — read as active.
            _ => "active",
        };
        println!(
            "  {}:{}  #[{kind}] on `{antigen}` — {state}",
            d.file.display(),
            d.line
        );
        if !d.text.trim().is_empty() {
            println!("      reason: {}", d.text.trim());
        }
        if let Some(until) = d.until.as_deref() {
            if !until.is_empty() {
                println!("      until: {until}");
            }
        }
    }
    println!();
    println!(
        "  These suppress the BLOCK, never the GAP — the failure-class is still \
         present and undefended at these sites. Remove the declaration when the \
         defense lands (or the gap is genuinely accepted forever)."
    );
    println!();
}

/// Render concern hints for ADR-024 convergent-evidence declarations
/// (`#[diagnostic]` / `#[clonal]` / `#[igg]` / `#[crossreactive]` / etc.). The
/// library computes these (modality-insufficient, class-collapsed, clonal
/// fixed-seed, igg identity-collapse, …) but the CLI never delivered them — the
/// `AuditVerdictComputedButNotDelivered` severance. Render only declarations with
/// concerns; a clean convergent declaration is background, not a TODO.
fn print_convergent_evidence_concerns(report: &audit::ConvergentEvidenceAuditReport) {
    if report.concern_count == 0 {
        return;
    }
    println!(
        "⚠ {} convergent-evidence declaration(s) with concerns (ADR-024):",
        report.concern_count
    );
    println!();
    for a in &report.audits {
        if a.hints.is_empty() {
            continue;
        }
        let d = &a.declaration;
        let hints = a
            .hints
            .iter()
            .map(audit_hint_kebab)
            .collect::<Vec<_>>()
            .join(", ");
        println!(
            "  {}:{}  #[{}] — {hints}",
            d.file.display(),
            d.line,
            convergent_kind_str(&d.kind)
        );
    }
    println!();
}

/// Render concern hints for ADR-024 recurrent-emergence declarations
/// (`#[itch]` / `#[recurrence_anchor]` / `#[crystallize]` / `#[chronic]` /
/// `#[saturate]` / `#[strand]`). Same severance fix as the convergent surface:
/// the library computes the hints (recurrence-anchor-no-itch-precondition,
/// chronic-signal-past-review-date, …); deliver them. Concerns only.
fn print_recurrent_concerns(report: &audit::RecurrentAuditReport) {
    if report.concern_count == 0 {
        return;
    }
    println!(
        "⚠ {} recurrent-emergence declaration(s) with concerns (ADR-024):",
        report.concern_count
    );
    println!();
    for a in &report.audits {
        if a.hints.is_empty() {
            continue;
        }
        let d = &a.declaration;
        let hints = a
            .hints
            .iter()
            .map(audit_hint_kebab)
            .collect::<Vec<_>>()
            .join(", ");
        let label = d
            .antigen_type
            .as_deref()
            .or(d.name.as_deref())
            .unwrap_or("(unlinked)");
        println!(
            "  {}:{}  #[{}] `{label}` — {hints}",
            d.file.display(),
            d.line,
            recurrent_kind_str(d.kind)
        );
    }
    println!();
}

/// Render the `#[descended_from]` lineage-fidelity advisory (scientist
/// 2026-05-27: ADVISORY for v0.3). Flags edges where the child antigen's
/// fingerprint is detectably NOT a refinement of the parent's. Non-blocking —
/// it does NOT affect the exit code; it surfaces a lineage claim that doesn't
/// survive the structural check (the MHC-restriction / negative-selection
/// cognate). Quiet when no divergences (advisory = signal, not noise).
fn print_lineage_fidelity_advisory(report: &audit::LineageFidelityAuditReport) {
    if report.divergences.is_empty() {
        return;
    }
    println!(
        "ℹ {} #[descended_from] lineage(s) whose child fingerprint does not refine \
         the parent (ADVISORY — lineage-fidelity, ADR-029-adjacent):",
        report.divergences.len()
    );
    println!();
    for d in &report.divergences {
        println!(
            "  {}:{}  {} ⟶ {} — {}",
            d.file.display(),
            d.line,
            d.child,
            d.parent,
            d.detail
        );
    }
    println!();
    println!(
        "  A `#[descended_from(Parent)]` claims the child is a more-specific case of \
         the parent's failure-class; if the child's fingerprint matches items the \
         parent's does not, the lineage is structurally unsound. Advisory only in \
         v0.3 (hard-fail deferred to a future ADR — biology: negative selection is \
         strict). Tighten the child's fingerprint, or correct the lineage edge."
    );
    println!();
}

/// Render the coverage / reachability frontier (the ignorance frontier): sites
/// the scanner should have evaluated but did not, grouped by cause. Silent when
/// the frontier is empty — which under a flat audit means "no member concept,"
/// not "complete" (the absence is tier-honest, not a completeness claim).
fn print_coverage_frontier(report: &audit::CoverageAuditReport) {
    if report.is_complete() {
        return;
    }
    println!(
        "⚠ {} site(s) the scanner did not reach (the ignorance frontier — the 4th \
         peripheral-tolerance mechanism: tolerance-by-non-encounter):",
        report.unreached_sites.len()
    );
    println!();
    for site in &report.unreached_sites {
        // cause is rendered as a short tag; the remedy carries the actionable text.
        let cause = match site.cause {
            audit::UnreachedCause::Barrier => "barrier (never enumerated)",
            audit::UnreachedCause::SubThreshold => "sub-threshold (scanned, not recognized)",
            audit::UnreachedCause::Cryptic => "cryptic (present, unparsed form)",
        };
        println!("  {}  [{}]", site.region, cause);
        println!("    → {}", site.remedy);
    }
    println!();
    println!(
        "  An unreached site is not defended and not tolerated — it is UNSEEN. \
         Ignorance is observed, never declared (there is no #[ignorance] marker: \
         to write one you'd have reached the site). Address each by its remedy above."
    );
    println!();
}

/// Macro-keyword string for a prescriptive work-need kind (display only).
const fn prescriptive_kind_str(kind: antigen::scan::PrescriptiveKind) -> &'static str {
    use antigen::scan::PrescriptiveKind as K;
    match kind {
        K::Panel => "panel",
        K::Rx => "rx",
        K::Refer => "refer",
        K::Biopsy => "biopsy",
        K::Ddx => "ddx",
        K::Triage => "triage",
        K::Culture => "culture",
        K::Quarantine => "quarantine",
    }
}

/// Short tag for a four-valued [`audit::WorkVerdict`] (board display).
const fn work_verdict_tag(verdict: audit::WorkVerdict) -> &'static str {
    use audit::WorkVerdict as V;
    match verdict {
        V::Overdue => "OVERDUE",
        V::OutOfFrame => "out-of-frame",
        V::Pending => "pending",
        V::Fulfilled => "fulfilled",
    }
}

/// Render the prescriptive work-orchestration BOARD (ADR-033 §Decision 4):
/// every `#[panel]`/`#[rx]`/.../`#[quarantine]` work-need as a board row, with
/// `OVERDUE` rows sorted LOUD to the top. "Code IS the Asana board" — this is a
/// live projection of the current code (ADR-034), recomputed every audit run and
/// never stored, so it cannot drift from reality.
///
/// A row names the macro, the work-need text, the verdict, and what blocks it
/// (the un-attested who-step / elapsed frame / unresolvable ref). The
/// load-bearing distinction the board preserves: `OVERDUE` (late, evaluated)
/// vs `out-of-frame` (un-evaluable — needs investigation, NOT an alarm); the
/// two are never collapsed (the ADR-029/ADR-033 three-valued-logic gem).
fn print_prescriptive_board(report: &audit::PrescriptiveAuditReport) {
    if report.verdicts.is_empty() {
        return;
    }
    let overdue = report.overdue_count();
    let out_of_frame = report.count_by_verdict(audit::WorkVerdict::OutOfFrame);
    let pending = report.count_by_verdict(audit::WorkVerdict::Pending);
    let fulfilled = report.count_by_verdict(audit::WorkVerdict::Fulfilled);

    println!();
    println!(
        "── Work board (ADR-033): {} work-need(s) — {overdue} overdue, \
         {out_of_frame} out-of-frame, {pending} pending, {fulfilled} fulfilled",
        report.verdicts.len()
    );
    if report.is_clean() {
        println!("   (no overdue work — the board is quiet)");
    }
    println!(
        "   code IS the board — this is a live projection of the current code, \
         recomputed every run (never stored, never drifts)."
    );
    println!();

    for v in report.board_ordered() {
        let decl = &v.declaration;
        let macro_name = prescriptive_kind_str(decl.kind);
        let tag = work_verdict_tag(v.verdict);
        // The work-need's headline text: prefer the free-text need, else the
        // first list item (needs/rule_out/priority_order), else the item label.
        let need = decl
            .need_text
            .as_deref()
            .or_else(|| decl.items.first().map(String::as_str))
            .unwrap_or("(no need text)");
        let loud = if v.verdict.is_loud() { "‼ " } else { "  " };
        println!(
            "{loud}[{tag}] #[{macro_name}] {need}  ({}:{})",
            decl.file.display(),
            decl.line
        );
        // Frame line: remaining-or-elapsed, when a frame is declared.
        if let Some(frame) = decl.frame.as_deref() {
            println!("      frame: {frame}");
        }
        // What blocks fulfillment (None when fulfilled).
        if let Some(blocking) = v.blocking.as_deref() {
            println!("      → {blocking}");
        }
        // Typed OutOfFrame sub-cause + its per-cause remedy (the
        // SubCauseCollapseInTheUnit fix): an un-evaluable need routes a DIFFERENT
        // remedy per cause (scaffold-sidecar vs declare-who-step vs fix-date vs
        // fix-dangling-ref), instead of fusing them into one opaque "out of frame".
        if let Some(cause) = v.out_of_frame_cause {
            let cause_tag = match cause {
                audit::OutOfFrameCause::UnknownWhoRef => "unknown-who-ref",
                audit::OutOfFrameCause::MissingWorkStep => "missing-work-step",
                audit::OutOfFrameCause::UnparseableFrame => "unparseable-frame",
                audit::OutOfFrameCause::UnresolvableRef => "unresolvable-ref",
            };
            println!("      cause: {cause_tag} — remedy: {}", cause.remedy());
        }
        // Per-step detail: which who-step is open (the board's actionable core).
        for step in &v.steps {
            let mark = match step.state {
                audit::StepState::Attested => "✓",
                audit::StepState::Unattested => "·",
                audit::StepState::Unevaluable => "?",
            };
            println!("      {mark} {}: {}", step.role, step.reference);
        }
    }
    println!();
}

/// Macro-keyword string for a convergent-evidence kind (display only).
const fn convergent_kind_str(kind: &antigen::scan::ConvergentEvidenceKind) -> &'static str {
    use antigen::scan::ConvergentEvidenceKind as K;
    match kind {
        K::Diagnostic => "diagnostic",
        K::Clonal => "clonal",
        K::Igg => "igg",
        K::Crossreactive => "crossreactive",
        K::Polyclonal => "polyclonal",
        K::Monoclonal => "monoclonal",
        K::Adcc => "adcc",
    }
}

/// Macro-keyword string for a recurrent-emergence kind (display only).
const fn recurrent_kind_str(kind: antigen::scan::RecurrentKind) -> &'static str {
    use antigen::scan::RecurrentKind as K;
    match kind {
        K::Itch => "itch",
        K::RecurrenceAnchor => "recurrence_anchor",
        K::Crystallize => "crystallize",
        K::Chronic => "chronic",
        K::Saturate => "saturate",
        K::Strand => "strand",
    }
}

/// Render an `AuditHint` as its serde kebab-case key for human concern lines.
/// (The hint's `Serialize` impl is kebab-case; reuse it so the human string
/// matches the JSON + the ADR vocabulary exactly.)
fn audit_hint_kebab(hint: &audit::AuditHint) -> String {
    serde_json::to_value(hint)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| format!("{hint:?}"))
}

/// Human scan output shows at most this many fingerprint-match detail lines per
/// antigen type, then a "+N more" summary pointing at `--format json`.
///
/// Fingerprint matches are the FILTER half of antigen's filter/proof split
/// (glossary): expected, noisy *candidate* sites the witness layer refines —
/// not a TODO list. On antigen's own tree these number ~18K; an unbounded
/// per-site wall teaches a newcomer the tool is noise (the opposite of
/// onboarding). The cap mirrors rustc's "...and N more" and clippy's per-lint
/// grouping. `--format json` stays exhaustive for CI gates.
const MAX_FINGERPRINT_MATCHES_PER_ANTIGEN: usize = 10;

fn print_fingerprint_matches(report: &scan::ScanReport) {
    use std::collections::BTreeMap;

    use antigen::scan::MatchKind;

    let fp_matches: Vec<_> = report
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::FingerprintMatch)
        .collect();
    if fp_matches.is_empty() {
        return;
    }

    // Group by antigen type so the cap is per-antigen (one noisy antigen can't
    // crowd out the others), and so the summary reads per the design role.
    let mut by_antigen: BTreeMap<&str, Vec<&&scan::Presentation>> = BTreeMap::new();
    for p in &fp_matches {
        by_antigen
            .entry(p.antigen_type.as_str())
            .or_default()
            .push(p);
    }

    println!(
        "{} fingerprint match(es) across {} antigen type(s) — candidate sites \
         (expected noise; the witness layer refines them, per the filter/proof split). \
         Not a TODO list.",
        fp_matches.len(),
        by_antigen.len()
    );
    println!();

    for (antigen_type, sites) in &by_antigen {
        for p in sites.iter().take(MAX_FINGERPRINT_MATCHES_PER_ANTIGEN) {
            println!(
                "  {}:{}  {} on {} [fingerprint match]",
                p.file.display(),
                p.line,
                p.antigen_type,
                p.item_kind
            );
        }
        if let Some(extra) = sites
            .len()
            .checked_sub(MAX_FINGERPRINT_MATCHES_PER_ANTIGEN)
            .filter(|n| *n > 0)
        {
            println!(
                "  … +{extra} more `{antigen_type}` candidate(s) — `cargo antigen scan \
                 --format json` for the full list."
            );
        }
    }
    println!();
    println!(
        "  These are CANDIDATES, not failures. If a site genuinely presents the \
         failure-class, acknowledge it:"
    );
    println!("    #[presents(<antigen>)] to mark the site explicitly,");
    println!("      then defend it: #[defended_by(<antigen>)] on a test (code-tier), or");
    println!("      #[presents(<antigen>, requires = ...)] for substrate-witness evidence,");
    println!("    #[antigen_tolerance(<antigen>, rationale = \"...\")] to document intent.");
    println!();
}

fn print_orphaned_tolerances(report: &scan::ScanReport) {
    let orphans = report.orphaned_tolerances();
    if orphans.is_empty() {
        return;
    }
    println!(
        "{} orphaned tolerance(s) — antigen no longer declared in workspace:",
        orphans.len()
    );
    println!();
    for t in &orphans {
        println!(
            "  {}:{}  {} [tolerance for unknown antigen]",
            t.file.display(),
            t.line,
            t.antigen_type
        );
    }
    println!();
    println!("  Remove or update these tolerances — the antigen they suppress is gone.");
    println!();
}

fn print_unaddressed(unaddressed: &[scan::UnaddressedPresentation]) {
    use antigen::scan::MatchKind;
    let explicit_unaddressed: Vec<_> = unaddressed
        .iter()
        .filter(|u| u.presentation.match_kind == MatchKind::ExplicitMarker)
        .collect();
    if explicit_unaddressed.is_empty() {
        println!("All explicit presentations are addressed.");
        return;
    }
    println!(
        "{} unaddressed explicit presentation(s):",
        explicit_unaddressed.len()
    );
    println!();
    for u in &explicit_unaddressed {
        let p = &u.presentation;
        println!(
            "  {}:{}  {} on {}",
            p.file.display(),
            p.line,
            p.antigen_type,
            p.item_kind
        );
        if !u.antigen_known {
            println!(
                "    note: antigen `{}` was not declared in the scanned workspace",
                p.antigen_type
            );
        }
    }
    println!();
    println!("To address each site, use the antigen type shown above:");
    println!("  #[defended_by(<antigen>)] on a test that exercises the defense (code-tier),");
    println!(
        "  OR #[presents(<antigen>, requires = ...)] on the site for substrate-witness evidence,"
    );
    println!("  OR #[antigen_tolerance(<antigen>, rationale = \"...\")] to document intent.");
}

// ============================================================================
// Report envelope — the report-as-live-projection floor
// ============================================================================
//
// The report is NEVER a stored / parallel state tracker. A stored,
// release-anchored report is itself a `ParallelStateTrackersDiverge` instance
// (antigen's own failure-class) — the moment it is committed it can drift from
// the code it claims to describe. So antigen does not persist reports it reads
// back as authoritative; `scan` / `audit` recompute the report from the current
// code on every run, exactly the way clippy reflects current source every
// invocation. The code is the source of truth; the report is a live view.
//
// The envelope makes each render self-describing so a saved render
// (`--output <file>`, a clippy-SARIF-style dump) carries the provenance needed
// to interpret it later WITHOUT antigen ever reading it back: which antigen
// version produced it, which git commit the workspace was at, and when. That is
// the difference between a *reproducible render of a tagged state* (regenerate
// it any time by re-running antigen at that tag) and a *stored truth* (which
// would rot). Re-running `cargo antigen audit` at the `v0.3.0` tag *is* the
// v0.3.0 defense-posture SBOM — the file is just a convenience copy of a
// recomputation, never the authority.
//
// `ReportEnvelope<T>` EXTENDS the stabilized scan-json / audit-json rather than
// forking it: the payload is `#[serde(flatten)]`-ed, so the existing top-level
// keys (`report`, `unaddressed`, …; `scan`, `audit`, …) stay exactly where
// consumers already read them, and the four envelope keys appear as additive
// siblings. Older consumers that navigate by key are unaffected; newer
// consumers gain provenance.

/// Provenance + freshness metadata stamped onto every machine-readable report.
/// These are the four envelope keys serialized as siblings of the report
/// payload. All are derived live from the current run — none is read back from
/// any stored file.
#[derive(serde::Serialize)]
struct ReportProvenance {
    /// The `cargo-antigen` version that produced this render
    /// (`CARGO_PKG_VERSION` at build time). Lets a consumer reason about which
    /// analysis vintage a saved render reflects.
    antigen_version: &'static str,
    /// The git commit the scanned workspace was at when the report was
    /// recomputed (`git rev-parse HEAD` in `--root`). `None` when the root is
    /// not a git repository or git is unavailable — tier-honest, matching the
    /// graceful-absence convention of `read_commit_trailers`. This is what makes
    /// a saved render a *reproducible render of a tagged state*: re-run antigen
    /// at this SHA to regenerate it.
    #[serde(skip_serializing_if = "Option::is_none")]
    git_sha: Option<String>,
    /// When this render was produced, RFC3339 UTC. The render's own timestamp,
    /// not a stored "last computed" that could go stale — every run restamps.
    generated_at: String,
    /// Version of the *report envelope schema* itself (distinct from the
    /// attestation-sidecar `schema_version` and from `ScanReport`'s internal
    /// versioning). Bumped only when the envelope's own shape changes, so a
    /// consumer can branch on envelope structure. Starts at 1.
    report_schema_version: u32,
}

/// The current report-envelope schema version. Bump when the envelope's own
/// key set changes shape (not when the underlying scan/audit payload evolves —
/// that carries its own field-level back-compat via serde defaults).
const REPORT_SCHEMA_VERSION: u32 = 1;

impl ReportProvenance {
    /// Gather provenance for a report recomputed against `root`. Pure
    /// derivation from the current run + the workspace's git state — nothing is
    /// read back from a stored report.
    fn gather(root: &Path) -> Self {
        Self {
            antigen_version: env!("CARGO_PKG_VERSION"),
            git_sha: git_head_sha(root),
            generated_at: chrono::Utc::now().to_rfc3339(),
            report_schema_version: REPORT_SCHEMA_VERSION,
        }
    }
}

/// `git rev-parse HEAD` in `dir`, returning the commit SHA. `None` on any git
/// failure (not a repo, detached/empty, git not installed) — tier-honest, the
/// same graceful-absence shape as [`read_commit_trailers`]. Fixed-arg
/// subprocess per the ADR-019 bright-line rule.
fn git_head_sha(dir: &Path) -> Option<String> {
    let out = std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let sha = String::from_utf8_lossy(&out.stdout).trim().to_owned();
    if sha.is_empty() { None } else { Some(sha) }
}

/// Wrap a serializable report payload in the provenance envelope. The payload's
/// own keys are flattened to the top level (extend, not fork); the four
/// provenance keys are added as siblings.
#[derive(serde::Serialize)]
struct ReportEnvelope<T> {
    #[serde(flatten)]
    provenance: ReportProvenance,
    #[serde(flatten)]
    payload: T,
}

impl<T: serde::Serialize> ReportEnvelope<T> {
    /// Envelope a payload with provenance gathered against `root`.
    fn new(root: &Path, payload: T) -> Self {
        Self {
            provenance: ReportProvenance::gather(root),
            payload,
        }
    }
}

/// Render a serializable report to pretty JSON, write it to `path`, and report
/// success on stderr (so it doesn't pollute a piped stdout). Used by
/// `--output <file>`: the file is a *render of this run*, overwritten every
/// time, never read back as authoritative. Returns an error string on failure.
fn write_report_render<T: serde::Serialize>(path: &Path, report: &T) -> Result<(), String> {
    let json =
        serde_json::to_string_pretty(report).map_err(|e| format!("failed to serialize: {e}"))?;
    std::fs::write(path, json).map_err(|e| format!("failed to write {}: {e}", path.display()))?;
    eprintln!("Wrote report render to {}", path.display());
    Ok(())
}

#[derive(serde::Serialize)]
struct JsonReport<'a> {
    report: &'a scan::ScanReport,
    unaddressed: &'a [scan::UnaddressedPresentation],
    /// `#[descended_from]` edges whose parent antigen is not declared in the
    /// workspace (orphaned) — a structurally-broken lineage claim. Computed via
    /// `ScanReport::orphaned_lineage_edges()` and attached here (sibling to
    /// `unaddressed`, same computed-then-attached pattern) so the JSON surface
    /// DELIVERS the verdict the scan layer computes. Empty vec when sound.
    orphaned_lineage_edges: Vec<&'a scan::LineageEdge>,
    /// `#[descended_from]` edges whose child is not itself an `#[antigen]`
    /// declaration (dangling). Computed via `dangling_child_lineage_edges()`.
    dangling_child_lineage_edges: Vec<&'a scan::LineageEdge>,
    /// When `--include-deps` is set, one entry per scanned dep
    /// crate. `None` (skipped in JSON output via `skip_serializing_if`)
    /// when the flag wasn't passed — preserves byte-identical output for
    /// existing consumers.
    #[serde(skip_serializing_if = "Option::is_none")]
    dep_reports: Option<&'a [DepScanResult]>,
}

/// Per-dependency scan result returned by the `--include-deps` mode of
/// `cargo antigen scan`. Each dep is scanned independently — no cross-crate
/// `addresses()` matching happens here (ATK-A3-005 / module-path-qualified
/// `ItemTarget` is an ADR-class decision, not yet decided).
#[derive(serde::Serialize)]
struct DepScanResult {
    package_name: String,
    version: String,
    origin: scan::CrateOrigin,
    report: scan::ScanReport,
}

/// `cargo antigen propose` — the keystone goes live (Island 3, ADR-045/047/048).
///
/// Re-acquires the marked DEFECT cluster under `--cluster-root`, collects the
/// OPERATOR-supplied clean corpus under `--clean-root`, routes them through the
/// learning core's `propose()` (anti-unify → GATE-G), and RENDERS the outcome as a
/// **ratifiable suggestion** (observe-don't-declare, ADR-044). It writes NOTHING to
/// the source tree — the machine drafts the syntactic half; a human ratifies the
/// semantic half. Every non-promotion reason is legible (ADR-048); route-to-human
/// (`NotCorpusWitnessable`) is a FIRST-CLASS, expected outcome — the gate refusing
/// to fake a verdict it cannot make — not a failure.
fn run_propose(args: ProposeArgs) -> ExitCode {
    // The cluster-root must exist + be a directory (needed by every path, including
    // the --list-clusters dry-run).
    if let Err(code) = validate_dir(&args.cluster_root, "--cluster-root") {
        return code;
    }

    // --list-clusters: preview the cluster landscape and STOP (a pure read of the
    //     by_shape grouping — no gate, no clean corpus). Runs BEFORE the clean-root
    //     requirement: a dry-run preview never consults the corpus. The diagnostic
    //     for "why no cluster?" — usually because every marked shape is a singleton.
    if args.list_clusters {
        return run_list_clusters(&args);
    }

    // The clean-root is REQUIRED for the gate (no auto-derived "rest of tree is clean"
    // default — ADR-044/047, ATK-047-4). It is Option only so --list-clusters can omit
    // it; on the gate path an absent corpus is a usage error.
    let Some(clean_root) = args.clean_root.as_deref() else {
        eprintln!(
            "error: --clean-root is required to gate a draft (the OPERATOR-supplied, \
             OPERATOR-labeled clean corpus the gate spares against). Antigen never \
             auto-labels unmarked code clean. (It is optional only with --list-clusters, \
             a dry-run preview that never runs the gate.)"
        );
        return ExitCode::from(2);
    };
    if let Err(code) = validate_dir(clean_root, "--clean-root") {
        return code;
    }

    // (1) Re-acquire the marked DEFECT cluster: scan --cluster-root, keep the marks
    //     of the chosen marker-class, group by shape digest, re-parse each member's
    //     enclosing item. v0.5 clusters by EXACT shape_digest (the scan's PROPOSE
    //     key); abstract-recall clustering past exact-shape is the v0.6 charter.
    let cluster = match assemble_marked_cluster(&args.cluster_root, &args.marker) {
        Ok(c) => c,
        Err(code) => return code,
    };
    if cluster.len() < 2 {
        // No source-distinct ≥2 cluster: there is nothing to anti-unify. On
        // antigen's OWN tree this is the expected state (its `#[dread]` marks are
        // singletons in shape-space — the v0.6 abstract-recall frontier). Honest,
        // not an error.
        println!(
            "no `{}` cluster found under {} — propose needs ≥2 marked sites sharing a \
             structural shape to anti-unify (found {}). Antigen's own marks are \
             singletons in shape-space today; auto-clustering heterogeneous marks is \
             the v0.6 abstract-recall frontier.",
            args.marker,
            args.cluster_root.display(),
            cluster.len()
        );
        // A no-≥2-cluster is the NoCluster outcome category (same as an empty/
        // heterogeneous cluster downstream) — honor --exit-code consistently.
        return ProposeExit::NoCluster.code(args.exit_code);
    }

    // (2) Collect the OPERATOR-supplied clean corpus: every fn/impl item under
    //     --clean-root. The operator SUPPLIES + LABELS it; the gate spares against
    //     exactly this (corpus-bounded claim-scope). antigen adds NOTHING from
    //     --cluster-root or elsewhere — no auto-clean path exists.
    let clean_corpus = match collect_clean_corpus(clean_root) {
        Ok(c) => c,
        Err(code) => return code,
    };
    if clean_corpus.is_empty() {
        eprintln!(
            "error: --clean-root {} yielded no function/impl items to spare against. \
             Supply a clean corpus of known-good sibling code; antigen cannot certify \
             safety against an empty corpus (it never auto-labels unmarked code clean).",
            clean_root.display()
        );
        return ExitCode::from(2);
    }

    // (3) THE KEYSTONE GOES LIVE: route the cluster through propose() (anti-unify →
    //     GATE-G against the operator corpus). The ONLY path to a promotable draft;
    //     the emit surface below is typed on the Result, so no bare Fingerprint is
    //     ever rendered as if it were gated.
    let outcome = antigen::learn::propose::propose(&cluster, &clean_corpus);

    // --explain: compute the GATE-G reasoning the render would otherwise hide (the
    //     near-miss that witnessed the draft, or the clean twin it wrongly bound). A
    //     pure READ of the gate's primitives over the SAME cluster + corpus — it never
    //     re-routes the draft or changes the verdict. None when --explain is off.
    let explanation = args
        .explain
        .then(|| compute_explanation(&cluster, &clean_corpus, &outcome));

    render_propose_outcome(&outcome, &args, explanation.as_ref())
}

/// The GATE-G reasoning behind a propose verdict (the `--explain` payload). A pure
/// projection of the gate's primitives over the cluster + corpus — computed only when
/// `--explain` is set, NEVER consulted by the gate itself (it cannot change a verdict).
///
/// Each field is the reasoning for the outcome it belongs to; the render prints only
/// the one matching the fired verdict.
struct ProposeExplanation {
    /// The clean corpus item that WITNESSED the draft (the near-miss — one constraint
    /// from binding), described as `<kind> <name>`. `None` when the corpus holds no
    /// near-miss (the route-to-human reason) or there is no draft (degenerate/no-cluster).
    near_miss: Option<String>,
    /// The clean corpus item the draft WRONGLY bound (the autoimmune twin), described
    /// as `<kind> <name>`. Only populated on the `BindsCleanItem { Some(i) }` path.
    bound_twin: Option<String>,
}

/// Compute the `--explain` reasoning by RE-DERIVING the draft (a pure
/// `anti_unify` over the cluster — the same draft `propose` built) and reading the
/// gate's primitives (`near_miss_index`, `evaluate`) over the corpus. Read-only; it
/// never mints a token or changes the verdict.
fn compute_explanation(
    cluster: &[syn::Item],
    clean_corpus: &[syn::Item],
    outcome: &Result<
        antigen::learn::self_tolerance::PromotedDraft,
        antigen::learn::propose::ProposeOutcome,
    >,
) -> ProposeExplanation {
    use antigen::learn::propose::{ProposeOutcome, anti_unify};
    use antigen::learn::self_tolerance::{ToleranceVerdict, near_miss_index};

    // Re-derive the draft `propose` built (same pure anti_unify over the cluster). On
    // the degenerate / no-skeleton / empty paths there is no usable draft.
    let draft = anti_unify(cluster);

    // The near-miss that witnessed (or would witness) the generalization — the spared
    // clean sibling one constraint from binding. Present on promote + (its ABSENCE is)
    // the route-to-human reason.
    let near_miss = draft
        .as_ref()
        .and_then(|d| near_miss_index(d, clean_corpus).map(|i| describe_item(&clean_corpus[i])));

    // The clean twin the draft WRONGLY bound — only on the autoimmune path, where the
    // verdict carries the bound index.
    let bound_twin = match outcome {
        Err(ProposeOutcome::Rejected(ToleranceVerdict::BindsCleanItem {
            clean_index: Some(i),
        })) => clean_corpus.get(*i).map(describe_item),
        _ => None,
    };

    ProposeExplanation {
        near_miss,
        bound_twin,
    }
}

/// A one-line identity for a `syn::Item` — `<kind> <name>` (e.g. `impl Drop for
/// CleanGuard`, `fn flush`). Zero-dependency (reads the AST directly; no
/// source-printer needed): for `--explain` the user wants to know WHICH sibling, not
/// re-read its whole body.
fn describe_item(item: &syn::Item) -> String {
    match item {
        syn::Item::Fn(f) => format!("fn {}", f.sig.ident),
        syn::Item::Struct(s) => format!("struct {}", s.ident),
        syn::Item::Enum(e) => format!("enum {}", e.ident),
        syn::Item::Trait(t) => format!("trait {}", t.ident),
        syn::Item::Impl(imp) => {
            let self_ty = type_last_segment(&imp.self_ty);
            match &imp.trait_ {
                Some((_, path, _)) => {
                    let tr = path
                        .segments
                        .last()
                        .map_or_else(|| "?".to_string(), |s| s.ident.to_string());
                    format!("impl {tr} for {self_ty}")
                },
                None => format!("impl {self_ty}"),
            }
        },
        _ => "item".to_string(),
    }
}

/// The last path-segment of a type (`Foo` from `crate::a::Foo`), for `describe_item`.
fn type_last_segment(ty: &syn::Type) -> String {
    match ty {
        syn::Type::Path(p) => p
            .path
            .segments
            .last()
            .map_or_else(|| "?".to_string(), |s| s.ident.to_string()),
        _ => "?".to_string(),
    }
}

/// Validate that `path` exists and is a directory, with a `flag`-named diagnostic.
fn validate_dir(path: &Path, flag: &str) -> Result<(), ExitCode> {
    if !path.exists() {
        eprintln!("error: {flag} path does not exist: {}", path.display());
        return Err(ExitCode::from(2));
    }
    if !path.is_dir() {
        eprintln!(
            "error: {flag} expected a directory, got a file: {}",
            path.display()
        );
        return Err(ExitCode::from(2));
    }
    Ok(())
}

/// One candidate cluster in the `--list-clusters` landscape: a marked structural
/// shape, how many source-distinct sites carry it, and a representative site. The
/// `chosen` flag marks the group propose would actually anti-unify (the largest
/// ≥2-site group). A pure projection of the `by_shape` grouping the CLI already
/// computes — the reasoning the gate-path throws away after picking the winner.
struct ClusterCandidate {
    shape_digest: String,
    site_count: usize,
    sample_file: PathBuf,
    sample_line: usize,
    chosen: bool,
}

/// `--list-clusters`: preview the cluster landscape and STOP (the dry-run diagnostic).
///
/// Scans `--cluster-root` only (the `--clean-root` is irrelevant to a grouping
/// preview — the gate never runs), groups the chosen marker's marks by structural
/// shape, and renders EVERY candidate cluster: its shape, source-distinct site
/// count, a sample site, and which one propose would pick. This surfaces the
/// `by_shape` map propose computes then discards — the direct answer to "why did
/// propose say *no cluster found*?" (almost always: every shape is a singleton, so
/// no ≥2 group exists to anti-unify). Read-only; the source tree is byte-unchanged.
fn run_list_clusters(args: &ProposeArgs) -> ExitCode {
    let report = match scan::scan_workspace(&args.cluster_root, None) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: scan of --cluster-root failed: {e}");
            return ExitCode::from(2);
        },
    };

    // The SAME grouping assemble_marked_cluster does — kept structurally identical so
    // the preview reflects exactly what the gate path would see (no divergent view).
    let mut by_shape: std::collections::BTreeMap<String, Vec<&scan::MarkedUnknown>> =
        std::collections::BTreeMap::new();
    for m in &report.marked_unknowns {
        if m.marker == args.marker && !m.shape_digest.is_empty() {
            by_shape.entry(m.shape_digest.clone()).or_default().push(m);
        }
    }

    // The chosen shape = the largest group with ≥2 source-distinct sites (the same
    // max_by_key assemble_marked_cluster uses). None if every group is a singleton.
    let chosen_shape: Option<String> = by_shape
        .iter()
        .filter(|(_, marks)| source_distinct_count(marks) >= 2)
        .max_by_key(|(_, marks)| source_distinct_count(marks))
        .map(|(shape, _)| shape.clone());

    let mut candidates: Vec<ClusterCandidate> = by_shape
        .iter()
        .map(|(shape, marks)| {
            // A stable representative site (the lexicographically-first (file, line)).
            let sample = marks
                .iter()
                .map(|m| (m.file.clone(), m.line))
                .min()
                .unwrap_or_else(|| (PathBuf::new(), 0));
            ClusterCandidate {
                shape_digest: shape.clone(),
                site_count: source_distinct_count(marks),
                sample_file: sample.0,
                sample_line: sample.1,
                chosen: chosen_shape.as_deref() == Some(shape.as_str()),
            }
        })
        .collect();
    // Largest groups first (then by shape for determinism) — the most cluster-like
    // shapes lead, the singletons trail.
    candidates.sort_by(|a, b| {
        b.site_count
            .cmp(&a.site_count)
            .then_with(|| a.shape_digest.cmp(&b.shape_digest))
    });

    if matches!(args.format, OutputFormat::Json) {
        return render_list_clusters_json(args, &candidates, chosen_shape.is_some());
    }
    render_list_clusters_human(args, &candidates, chosen_shape.is_some())
}

/// Human render of the `--list-clusters` landscape.
fn render_list_clusters_human(
    args: &ProposeArgs,
    candidates: &[ClusterCandidate],
    has_cluster: bool,
) -> ExitCode {
    println!(
        "== `{}` cluster landscape under {} (dry-run; gate NOT run) ==\n",
        args.marker,
        args.cluster_root.display()
    );
    if candidates.is_empty() {
        println!(
            "No `{}` marks found. propose anti-unifies a cluster of ≥2 marked sites that \
             share a structural shape; there are none here to group.",
            args.marker
        );
        // No marks at all is a NoCluster outcome for --exit-code purposes.
        return ProposeExit::NoCluster.code(args.exit_code);
    }
    println!("  sites  chosen   shape digest (sample site)");
    for c in candidates {
        let marker = if c.chosen { "  <==" } else { "" };
        // Short shape prefix keeps the line readable; the full digest is the cluster
        // key, not a user-facing name (a fingerprint is the named thing, not a shape).
        let short = c.shape_digest.chars().take(16).collect::<String>();
        println!(
            "  {:>5}  {:<7}  {}… ({}:{}){}",
            c.site_count,
            if c.chosen { "yes" } else { "" },
            short,
            c.sample_file.display(),
            c.sample_line,
            marker
        );
    }
    println!();
    if has_cluster {
        println!(
            "propose would anti-unify the `<==` group (the largest with ≥2 \
             source-distinct sites). Run without --list-clusters to gate it."
        );
    } else {
        println!(
            "No group has ≥2 source-distinct sites — every `{}` shape is a singleton, \
             so propose finds NO cluster to anti-unify (the expected state on antigen's \
             own marks today; abstract-recall clustering past exact-shape is the v0.6 \
             frontier).",
            args.marker
        );
    }
    // A preview is informational; categorize by whether a cluster exists so --exit-code
    // lets CI distinguish "there is something to propose" from "nothing to anti-unify".
    if has_cluster {
        ProposeExit::Promoted.code(args.exit_code)
    } else {
        ProposeExit::NoCluster.code(args.exit_code)
    }
}

/// JSON render of the `--list-clusters` landscape (machine-readable preview).
fn render_list_clusters_json(
    args: &ProposeArgs,
    candidates: &[ClusterCandidate],
    has_cluster: bool,
) -> ExitCode {
    let clusters: Vec<serde_json::Value> = candidates
        .iter()
        .map(|c| {
            serde_json::json!({
                "shape_digest": c.shape_digest,
                "site_count": c.site_count,
                "sample_site": format!("{}:{}", c.sample_file.display(), c.sample_line),
                "chosen": c.chosen,
            })
        })
        .collect();
    let value = serde_json::json!({
        "outcome": "cluster-landscape",
        "marker": args.marker,
        "cluster_root": args.cluster_root.display().to_string(),
        "has_cluster": has_cluster,
        "clusters": clusters,
        "note": "dry-run preview of the by_shape grouping; the gate was NOT run (--clean-root not consulted)",
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
    );
    if has_cluster {
        ProposeExit::Promoted.code(args.exit_code)
    } else {
        ProposeExit::NoCluster.code(args.exit_code)
    }
}

/// Re-acquire the marked DEFECT cluster: scan `root`, keep the `marker`-class
/// marked-unknowns, group them by shape digest, and re-acquire the enclosing item's
/// AST for the LARGEST source-distinct group (the cluster to anti-unify).
///
/// "Source-distinct" = the members come from ≥2 distinct (file, line) sites (a
/// single site listed twice is not a cluster). Items are re-acquired by matching the
/// re-parsed top-level item's name-insensitive shape digest against the mark's
/// recorded `shape_digest` — name-independent re-acquisition (the two-digest
/// discipline, ADR-045 Amd-1/2).
fn assemble_marked_cluster(root: &Path, marker: &str) -> Result<Vec<syn::Item>, ExitCode> {
    let report = scan::scan_workspace(root, None).map_err(|e| {
        eprintln!("error: scan of --cluster-root failed: {e}");
        ExitCode::from(2)
    })?;

    // Group the chosen marker's marks by shape digest; keep source-distinct sites.
    let mut by_shape: std::collections::BTreeMap<String, Vec<&scan::MarkedUnknown>> =
        std::collections::BTreeMap::new();
    for m in &report.marked_unknowns {
        if m.marker == marker && !m.shape_digest.is_empty() {
            by_shape.entry(m.shape_digest.clone()).or_default().push(m);
        }
    }

    // The cluster = the largest group whose members span ≥2 distinct (file, line).
    let Some((shape, marks)) = by_shape
        .iter()
        .filter(|(_, marks)| source_distinct_count(marks) >= 2)
        .max_by_key(|(_, marks)| source_distinct_count(marks))
    else {
        // No ≥2 source-distinct group — return empty; the caller renders the honest
        // "no cluster" message (not an error).
        return Ok(Vec::new());
    };

    // Re-acquire each member's enclosing item by re-parsing its file and matching the
    // recorded shape digest (name-independent). De-dup by (file, line) source site.
    let mut seen_sites: std::collections::BTreeSet<(PathBuf, usize)> =
        std::collections::BTreeSet::new();
    let mut items: Vec<syn::Item> = Vec::new();
    for m in marks {
        if !seen_sites.insert((m.file.clone(), m.line)) {
            continue; // same source site already acquired
        }
        if let Some(item) = reacquire_item_by_shape(&m.file, shape) {
            items.push(item);
        }
    }
    Ok(items)
}

/// Count the distinct `(file, line)` source sites among a group of marks.
fn source_distinct_count(marks: &[&scan::MarkedUnknown]) -> usize {
    marks
        .iter()
        .map(|m| (m.file.clone(), m.line))
        .collect::<std::collections::BTreeSet<_>>()
        .len()
}

/// Re-parse `file` and return the first top-level item whose name-insensitive shape
/// digest equals `shape` — name-independent AST re-acquisition (ADR-045 Amd-1/2).
fn reacquire_item_by_shape(file: &Path, shape: &str) -> Option<syn::Item> {
    let src = std::fs::read_to_string(file).ok()?;
    let parsed = syn::parse_file(&src).ok()?;
    parsed
        .items
        .into_iter()
        .find(|item| item_shape_digest(item).as_deref() == Some(shape))
}

/// The name-insensitive shape digest of a top-level item (the scan's PROPOSE
/// clustering key), or `None` for an item-class with no shape digest.
fn item_shape_digest(item: &syn::Item) -> Option<String> {
    match item {
        syn::Item::Fn(f) => Some(antigen_fingerprint::structural_shape_digest(f)),
        syn::Item::Impl(i) => Some(antigen_fingerprint::structural_shape_digest(i)),
        syn::Item::Struct(s) => Some(antigen_fingerprint::structural_shape_digest(s)),
        syn::Item::Enum(e) => Some(antigen_fingerprint::structural_shape_digest(e)),
        syn::Item::Trait(t) => Some(antigen_fingerprint::structural_shape_digest(t)),
        _ => None,
    }
}

/// Collect the OPERATOR-supplied clean corpus: every `fn` / `impl` item in every
/// `.rs` file under `clean_root`. The operator supplies + labels this; antigen never
/// adds an auto-labeled item (ADR-044/047). Returns the parsed `syn::Item`s.
///
/// **HARD-ERRORS on any unreadable/unparseable `.rs` file — never silently skips**
/// (captain's ruling). A silently-incomplete clean corpus is a *weaker gate*: a
/// dropped clean item is one the spare-clean/near-miss checks never run against, so a
/// draft that would have bound it can promote — autoimmunity admitted at the gate
/// INPUT. That silent read-or-continue is exactly antigen's own `scan_workspace_inner`
/// `#[dread]`; the propose CLI must not commit the very failure-class it dreads. A
/// file the operator pointed `--clean-root` at that cannot be read/parsed is a usage
/// error they must resolve (named) — not a quiet narrowing of their labeled corpus.
fn collect_clean_corpus(clean_root: &Path) -> Result<Vec<syn::Item>, ExitCode> {
    let mut corpus: Vec<syn::Item> = Vec::new();
    if let Err(msg) = collect_rs_items(clean_root, &mut corpus) {
        eprintln!(
            "error: reading --clean-root failed: {msg}\n\
             A clean corpus that silently drops files is a WEAKER gate (a dropped clean \
             sibling is never checked, so a draft that would bind it could promote — \
             autoimmunity at the gate input). Resolve the file above, or point \
             --clean-root at a readable, parseable source tree."
        );
        return Err(ExitCode::from(2));
    }
    // Keep only the item-classes the gate's matcher reasons over a body for (fn /
    // impl) — the clean siblings a defect-draft must spare. (Structs/enums carry no
    // body locus for the body-call leaves a propose draft is built from.)
    corpus.retain(|it| matches!(it, syn::Item::Fn(_) | syn::Item::Impl(_)));
    Ok(corpus)
}

/// Recursively walk `dir` for `.rs` files, parse each, and push its top-level items
/// into `out`. Skips `target` + hidden dirs (a clean corpus is source, not build
/// output). **Returns `Err(message)` naming the first file it cannot read OR parse —
/// it does NOT silently skip** (the captain's ruling; see [`collect_clean_corpus`]):
/// a silently-dropped clean file narrows the labeled corpus and weakens the gate,
/// which is antigen's own silent-skip `#[dread]` committed at the gate input.
fn collect_rs_items(dir: &Path, out: &mut Vec<syn::Item>) -> Result<(), String> {
    let entries = std::fs::read_dir(dir)
        .map_err(|e| format!("cannot read directory {}: {e}", dir.display()))?;
    for entry in entries {
        let entry =
            entry.map_err(|e| format!("cannot read a dir entry under {}: {e}", dir.display()))?;
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if name == "target" || name.starts_with('.') {
                continue;
            }
            collect_rs_items(&path, out)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let src = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            let file = syn::parse_file(&src)
                .map_err(|e| format!("cannot parse {} as Rust: {e}", path.display()))?;
            out.extend(file.items);
        }
    }
    Ok(())
}

/// The categorical OUTCOME of a propose run — the CI-routable distinction the
/// `--exit-code` flag surfaces. Computed once, after rendering, so the print-side
/// (the human suggestion) and the code-side (the CI signal) share one source of
/// truth and never drift (antigen's own `ParallelStateTrackersDiverge`, kept out of
/// its own CLI).
///
/// **A category, never a pass/fail grade.** Route-to-human is the gate being honest,
/// not a failure — so its code (`10`) reads as "needs a human", distinct from
/// promoted (`0`) but NOT an error. The IO/usage error path stays `2` (returned
/// upstream, before an outcome category exists); the `10+` range never collides.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProposeExit {
    /// A ratifiable suggestion was minted. The actionable-positive outcome.
    Promoted,
    /// The gate routed the candidate to a human (no near-miss to witness it).
    /// First-class, NOT a failure.
    RouteToHuman,
    /// The draft bound a clean-corpus item (or was bare-structural over-general) —
    /// the gate refused it as autoimmune.
    RefusedAutoimmune,
    /// The cluster shares only structural shape (no discriminating signal).
    Degenerate,
    /// Nothing to anti-unify — empty/heterogeneous cluster, or no ≥2 cluster found.
    NoCluster,
}

impl ProposeExit {
    /// Map the outcome category to an [`ExitCode`]. With `exit_code == false` (the
    /// human default) EVERY category is `0` — propose never grades a first-class
    /// outcome a "failure". With `exit_code == true` (opt-in, mirrors `audit
    /// --strict`) each category gets a distinct CI-routable code in the `10+` range
    /// (never colliding with the `2` IO/usage-error convention).
    fn code(self, exit_code: bool) -> ExitCode {
        if !exit_code {
            return ExitCode::SUCCESS;
        }
        match self {
            Self::Promoted => ExitCode::SUCCESS,
            Self::RouteToHuman => ExitCode::from(10),
            Self::RefusedAutoimmune => ExitCode::from(11),
            Self::Degenerate => ExitCode::from(12),
            Self::NoCluster => ExitCode::from(13),
        }
    }
}

/// Render the propose outcome as a ratifiable SUGGESTION (observe-don't-declare,
/// ADR-044). Every outcome is legible (ADR-048); route-to-human is first-class, NOT
/// a failure. Writes only to stdout/stderr — the source tree is byte-unchanged.
///
/// The exit code is computed from the outcome CATEGORY ([`ProposeExit`]) honoring
/// `--exit-code`: `0` for every category by default (human-facing), or a distinct
/// CI-routable code per category when opted in.
///
/// `explain` is `Some` iff `--explain` was set — extra GATE-G reasoning lines are
/// appended to the matching arm. It is PURE OUTPUT: it never changes which arm fires
/// (the verdict) nor the exit code (the gate holds).
// One arm per ToleranceVerdict/ProposeOutcome, each with its own diagnostic prose +
// the optional --explain block — the length is the legibility (every non-promotion is
// spelled out, ADR-048). Splitting the arms into helpers would scatter the
// observe-don't-declare contract across functions for no real gain.
#[allow(clippy::too_many_lines)]
fn render_propose_outcome(
    outcome: &Result<
        antigen::learn::self_tolerance::PromotedDraft,
        antigen::learn::propose::ProposeOutcome,
    >,
    args: &ProposeArgs,
    explain: Option<&ProposeExplanation>,
) -> ExitCode {
    use antigen::learn::propose::ProposeOutcome;
    use antigen::learn::self_tolerance::ToleranceVerdict;

    if matches!(args.format, OutputFormat::Json) {
        return render_propose_json(outcome, args, explain);
    }

    let category = match outcome {
        Ok(token) => {
            // A capability token — the only assertable generalization. Surface it as
            // a SUGGESTION the human inspects + ratifies, never an audited verdict.
            println!("== candidate failure-class fingerprint (ratifiable suggestion) ==\n");
            println!("  fingerprint: {}", render_fingerprint(token.fingerprint()));
            println!("  score tier:  {:?}", token.tier());
            println!(
                "\nThis is a SUGGESTION drafted from your `{}` marks and gated against your\n\
                 clean corpus — inspect it and ratify by hand. It is NOT an audited verdict,\n\
                 NOT an auto-`#[presents]`, and NOT a named failure-class. The machine drafted\n\
                 the syntactic half; you ratify the semantic half (observe-don't-declare).",
                args.marker
            );
            if let Some(ex) = explain {
                match &ex.near_miss {
                    Some(item) => println!(
                        "\n--explain (GATE-G reasoning):\n  \
                         The generalization is WITNESSED by clean sibling `{item}` — it matches\n  \
                         all-but-one of the draft's conjuncts and is SPARED by failing exactly\n  \
                         one. That near-miss is the proof B made a REAL in-family discrimination\n  \
                         (it spared a sibling it plausibly could have flagged), not a\n  \
                         bare-structural over-bind."
                    ),
                    None => println!(
                        "\n--explain (GATE-G reasoning):\n  \
                         (no single near-miss sibling identified — the draft is\n  \
                         near-miss-witnessed by the corpus as a whole.)"
                    ),
                }
            }
            ProposeExit::Promoted
        },
        Err(ProposeOutcome::Rejected(ToleranceVerdict::NotCorpusWitnessable)) => {
            // The settled-thesis outcome on antigen's own marks: the gate refuses to
            // fake a generalization-verdict it cannot make. FIRST-CLASS, expected.
            println!("== drafted a candidate — routed to a human ratifier ==\n");
            println!(
                "Antigen anti-unified a draft from your `{}` marks, but the B-gate cannot\n\
                 certify it GENERALIZES against your clean corpus (no near-miss: no clean\n\
                 sibling is one discriminating constraint from binding the draft). So it\n\
                 routes the candidate to a HUMAN ratifier rather than promote it.\n\n\
                 This is the gate being honest — refusing to certify a generalization it\n\
                 cannot witness is the trust-floor, not a failure. (A promote fires when the\n\
                 cluster has discriminating diversity AND your corpus holds a near-miss\n\
                 sibling.)",
                args.marker
            );
            if explain.is_some() {
                // On route-to-human, the reasoning IS the absence: no corpus item is
                // one discriminating constraint from binding the draft. (near_miss is
                // None here by construction — the gate routed BECAUSE there is none.)
                println!(
                    "\n--explain (GATE-G reasoning):\n  \
                     The gate scanned your clean corpus for a NEAR-MISS — a sibling that\n  \
                     matches all-but-one of the draft's conjuncts and is spared by failing\n  \
                     exactly one. It found NONE: every clean item either fully matches the\n  \
                     draft (would be flagged) or is MORE than one constraint away (unrelated).\n  \
                     To let the gate certify a promote, add a clean sibling that is ONE\n  \
                     discriminating constraint from the defect (the in-family near-miss that\n  \
                     proves the discrimination is real)."
                );
            }
            ProposeExit::RouteToHuman
        },
        Err(ProposeOutcome::Rejected(ToleranceVerdict::BindsCleanItem { clean_index })) => {
            println!("== refused: the draft binds your clean corpus (autoimmune) ==\n");
            match clean_index {
                Some(i) => println!(
                    "The anti-unified draft MATCHES clean-corpus item #{i} — promoting it\n\
                     would flag known-good code (antigen's own autoimmunity). Refused. Your\n\
                     clean corpus contains a site the draft would have flagged; refine the\n\
                     cluster or the corpus."
                ),
                None => println!(
                    "The anti-unified draft is BARE-STRUCTURAL (no discriminating signal) — it\n\
                     would over-bind its whole structural family. Refused at the (A)-binary\n\
                     safety check; refine the cluster (these sites share only their shape)."
                ),
            }
            if let Some(ex) = explain {
                if let Some(twin) = &ex.bound_twin {
                    println!(
                        "\n--explain (GATE-G reasoning):\n  \
                         The clean item the draft would flag is `{twin}` — promoting the draft\n  \
                         would fire on THIS known-good code (the autoimmunity made concrete).\n  \
                         Either tighten the cluster so the draft no longer matches `{twin}`, or\n  \
                         remove `{twin}` from the clean corpus if it is NOT actually clean."
                    );
                }
            }
            ProposeExit::RefusedAutoimmune
        },
        Err(ProposeOutcome::Rejected(ToleranceVerdict::Spared)) => {
            // The gate never returns Err(Spared) — Spared is the success predicate.
            // Render defensively rather than panic (totality). Categorize as
            // no-cluster (no promotable draft was minted).
            println!("== no promotable draft (spared, but not minted) ==");
            ProposeExit::NoCluster
        },
        Err(ProposeOutcome::Degenerate) => {
            println!("== no candidate: the cluster is not a real failure-family ==\n");
            println!(
                "The `{}` sites share only their structural shape (item-kind / trait) with\n\
                 no discriminating behavioral signal — anti-unifying them yields a\n\
                 bare-structural over-binder, which the generator refuses. Refine the\n\
                 cluster to sites that share a real defect signal.",
                args.marker
            );
            ProposeExit::Degenerate
        },
        Err(ProposeOutcome::EmptyCluster | ProposeOutcome::NoSharedSkeleton) => {
            println!("== no candidate: the cluster could not be anti-unified ==\n");
            println!(
                "The `{}` cluster is empty or its members share no common item-kind skeleton\n\
                 (a heterogeneous group is not a real family). Nothing to generalize.",
                args.marker
            );
            ProposeExit::NoCluster
        },
    };

    category.code(args.exit_code)
}

/// Render the `outcome` as a compact JSON object (machine-readable; the same
/// observe-don't-declare content as the human render, never an asserted class).
///
/// The exit code honors `--exit-code` via the SAME [`ProposeExit`] category map the
/// human render uses — one source of truth, so the human and JSON surfaces never
/// disagree on the CI code for an outcome.
fn render_propose_json(
    outcome: &Result<
        antigen::learn::self_tolerance::PromotedDraft,
        antigen::learn::propose::ProposeOutcome,
    >,
    args: &ProposeArgs,
    explain: Option<&ProposeExplanation>,
) -> ExitCode {
    use antigen::learn::propose::ProposeOutcome;
    use antigen::learn::self_tolerance::ToleranceVerdict;

    let (mut value, category) = match outcome {
        Ok(token) => (
            serde_json::json!({
                "outcome": "candidate-suggestion",
                // ALWAYS false in v0.5: a PromotedDraft token is a ratifiable SUGGESTION,
                // never an asserted/named class (observe-don't-declare). Pinning it false
                // keeps a future reader from mistaking the suggestion for an auto-promotion.
                "promoted": false,
                "fingerprint": render_fingerprint(token.fingerprint()),
                "tier": format!("{:?}", token.tier()),
                "note": "ratifiable suggestion (observe-don't-declare); inspect + ratify by hand",
            }),
            ProposeExit::Promoted,
        ),
        Err(ProposeOutcome::Rejected(ToleranceVerdict::NotCorpusWitnessable)) => (
            serde_json::json!({
                "outcome": "route-to-human",
                "note": "B cannot certify the draft generalizes against the supplied corpus (no near-miss); routed to a human ratifier",
            }),
            ProposeExit::RouteToHuman,
        ),
        Err(ProposeOutcome::Rejected(ToleranceVerdict::BindsCleanItem { clean_index })) => (
            serde_json::json!({
                "outcome": "refused-autoimmune",
                "clean_index": clean_index,
                "note": "the draft binds a clean-corpus item (or is bare-structural over-general); refused",
            }),
            ProposeExit::RefusedAutoimmune,
        ),
        Err(ProposeOutcome::Rejected(ToleranceVerdict::Spared)) => (
            serde_json::json!({ "outcome": "no-promotable-draft" }),
            ProposeExit::NoCluster,
        ),
        Err(ProposeOutcome::Degenerate) => (
            serde_json::json!({
                "outcome": "degenerate",
                "note": "the cluster shares only structural shape (no discriminating signal); not a real failure-family",
            }),
            ProposeExit::Degenerate,
        ),
        Err(ProposeOutcome::EmptyCluster) => (
            serde_json::json!({ "outcome": "empty-cluster" }),
            ProposeExit::NoCluster,
        ),
        Err(ProposeOutcome::NoSharedSkeleton) => (
            serde_json::json!({ "outcome": "no-shared-skeleton" }),
            ProposeExit::NoCluster,
        ),
    };
    // --explain: attach the GATE-G reasoning as an `explain` object (the same
    // near-miss / bound-twin identities the human render shows). Pure additive — it
    // never changes the outcome or the exit code.
    if let Some(ex) = explain {
        if let serde_json::Value::Object(map) = &mut value {
            map.insert(
                "explain".to_string(),
                serde_json::json!({
                    "near_miss": ex.near_miss,
                    "bound_twin": ex.bound_twin,
                }),
            );
        }
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&value).unwrap_or_else(|_| "{}".to_string())
    );
    category.code(args.exit_code)
}

/// Render a [`antigen_fingerprint::Fingerprint`] for the suggestion (a human reads +
/// ratifies it). Uses the `Debug` shape — a canonical human-facing pretty-printer is
/// a render charter; for v0.5 the Debug form is honest + inspectable.
fn render_fingerprint(fp: &antigen_fingerprint::Fingerprint) -> String {
    format!("{fp:?}")
}

fn run_new(name: String) -> ExitCode {
    eprintln!(
        "cargo-antigen new {} — design phase\n\
         \n\
         Antigen scaffolding (interactive scaffolding for a new declaration) is\n\
         under design. The eventual command will:\n\
           - Prompt for family (one of the 8 first-principles classes or custom)\n\
           - Prompt for fingerprint (assist with structural pattern composition)\n\
           - Prompt for witness type (test, proptest, lint, formal-verification)\n\
           - Generate a starter declaration file in your project's antigen module\n\
         \n\
         For now, please write antigen declarations by hand. See the project's\n\
         docs/expedition/conventions.md for naming guidance.\n",
        name
    );
    ExitCode::FAILURE
}

fn run_vaccinate(antigen: String, pattern: String) -> ExitCode {
    eprintln!(
        "cargo-antigen vaccinate {} {} — design phase\n\
         \n\
         Vaccination (apply known immunity pattern across a structural family) is\n\
         under design. The eventual command will:\n\
           - Search the workspace for items matching `pattern`\n\
           - For each match without existing immunity, scaffold a witness stub\n\
           - Add #[presents] and #[immune] markers atomically with confirmation\n\
         \n\
         For now, apply immunity manually per site.\n",
        antigen, pattern
    );
    ExitCode::FAILURE
}

/// One scanned site's fingerprint, for `cargo antigen fingerprint` output.
#[derive(serde::Serialize)]
struct FingerprintMatch {
    antigen_type: String,
    item_path: String,
    /// `"immune"` or `"presents"` — which kind of declaration this site is.
    site_kind: &'static str,
    structural_fingerprint: String,
    file: String,
    line: usize,
}

fn run_fingerprint(args: FingerprintArgs) -> ExitCode {
    if !args.root.exists() {
        eprintln!("error: path does not exist: {}", args.root.display());
        return ExitCode::from(2);
    }

    let report = match scan::scan_workspace(&args.root, None) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: scan failed: {e}");
            return ExitCode::from(2);
        },
    };

    let antigen_filter = args.antigen.as_deref();
    let item_filter = args.item_path.as_deref();
    let keep = |antigen_type: &str, target: &scan::ItemTarget| {
        antigen_filter.is_none_or(|a| antigen_type == a)
            && item_filter.is_none_or(|p| target.label() == p)
    };

    // Immunities always carry a fingerprint; presentations carry one only for
    // fingerprint-matches (explicit markers leave it empty). Report immune
    // sites first, then presentations whose fingerprint is non-empty.
    let mut matches: Vec<FingerprintMatch> = report
        .immunities
        .iter()
        .filter(|i| keep(&i.antigen_type, &i.item_target))
        .map(|i| FingerprintMatch {
            antigen_type: i.antigen_type.clone(),
            item_path: i.item_target.label(),
            site_kind: "immune",
            structural_fingerprint: i.structural_fingerprint.clone(),
            file: i.file.display().to_string(),
            line: i.line,
        })
        .collect();
    matches.extend(
        report
            .presentations
            .iter()
            .filter(|p| keep(&p.antigen_type, &p.item_target))
            .filter(|p| !p.structural_fingerprint.is_empty())
            .map(|p| FingerprintMatch {
                antigen_type: p.antigen_type.clone(),
                item_path: p.item_target.label(),
                site_kind: "presents",
                structural_fingerprint: p.structural_fingerprint.clone(),
                file: p.file.display().to_string(),
                line: p.line,
            }),
    );

    match args.format {
        OutputFormat::Json => match serde_json::to_string_pretty(&matches) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("error: failed to serialize fingerprints: {e}");
                return ExitCode::from(2);
            },
        },
        OutputFormat::Human => {
            if matches.is_empty() {
                let scope = match (antigen_filter, item_filter) {
                    (Some(a), Some(p)) => format!(" for antigen `{a}` at item `{p}`"),
                    (Some(a), None) => format!(" for antigen `{a}`"),
                    (None, Some(p)) => format!(" at item `{p}`"),
                    (None, None) => String::new(),
                };
                eprintln!(
                    "no immune/presents site with an obtainable fingerprint found{scope} \
                     under {}.",
                    args.root.display()
                );
            } else {
                for m in &matches {
                    println!(
                        "{}  {} [{}] on {}  {}:{}",
                        m.structural_fingerprint,
                        m.antigen_type,
                        m.site_kind,
                        m.item_path,
                        m.file,
                        m.line
                    );
                }
            }
        },
    }

    // A requested-but-not-found site is a user-visible failure (exit 1) so
    // scripts (e.g. `FP=$(cargo antigen fingerprint --antigen X --item-path y)`)
    // don't silently capture an empty value. An unfiltered empty workspace is
    // not a failure.
    if matches.is_empty() && (antigen_filter.is_some() || item_filter.is_some()) {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn run_audit(args: AuditArgs) -> ExitCode {
    if !args.root.exists() {
        eprintln!("error: path does not exist: {}", args.root.display());
        return ExitCode::from(2);
    }
    if !args.root.is_dir() {
        eprintln!(
            "error: expected a directory, got a file: {}",
            args.root.display()
        );
        return ExitCode::from(2);
    }

    // ADR-028 §CLI integration: --category filters the audit to a single category.
    let Ok(category_filter) = parse_category_filter(args.category.as_deref()) else {
        return ExitCode::from(2);
    };

    // Member-aware (`--workspace`) audit populates the scan-coverage record so
    // the coverage/reachability audit can surface unreached members; the flat
    // audit (default) keeps byte-compatible behavior for existing consumers.
    let scan_result = if args.workspace {
        eprintln!("Auditing workspace (member-aware): {}", args.root.display());
        scan::scan_workspace_multi_crate(&args.root)
    } else {
        eprintln!("Auditing workspace: {}", args.root.display());
        scan::scan_workspace(&args.root, None)
    };
    let mut scan_report = match scan_result {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: scan failed: {e}");
            return ExitCode::from(2);
        },
    };

    if let Some(cat) = category_filter {
        filter_report_by_category(&mut scan_report, cat);
    }

    // ADR-036: the audit detector fan-out is now owned by the thin audit-side
    // sequencer `audit::orchestrate::run`, which drives the detector sequence in
    // the established order and bundles each report. This is the recognition of
    // the fan-out that used to live inline here (immunity audit + the ADR-028
    // category cross-check + the ADR-023 deferred-defense state + the ADR-024
    // convergent/recurrent audits + the lineage-fidelity advisory + the coverage
    // ignorance-frontier + the ADR-033 prescriptive work-board) — same calls,
    // same order, same arguments; the sequencer gives them a home and a name (and
    // is the layer a future cascade-governor's SCRAM sits above). Each report is
    // delivered to the renderer below (the delivery-arm discipline that the
    // AuditVerdictComputedButNotDelivered dogfood antigen guards).
    let audit::orchestrate::AuditBundle {
        audit: audit_report,
        category: category_report,
        deferred: deferred_report,
        convergent: convergent_report,
        recurrent: recurrent_report,
        lineage_fidelity: lineage_fidelity_report,
        coverage: coverage_report,
        prescriptive: prescriptive_report,
    } = audit::orchestrate::run(&scan_report, &args.root);

    // Live projection: the audit recomputed all of the above from the current
    // code. Envelope it for provenance; running this at a tagged commit yields
    // that tag's reproducible defense-posture SBOM (regenerable, never read
    // back as authoritative — so it cannot drift).
    let enveloped = ReportEnvelope::new(
        &args.root,
        JsonAuditReport {
            scan: &scan_report,
            audit: &audit_report,
            category: &category_report,
            deferred_defense_audit: &deferred_report,
            convergent_evidence_audit: &convergent_report,
            recurrent_audit: &recurrent_report,
            lineage_fidelity_audit: &lineage_fidelity_report,
            coverage_audit: &coverage_report,
            prescriptive_audit: &prescriptive_report,
        },
    );

    if let Some(path) = args.output.as_ref() {
        if let Err(e) = write_report_render(path, &enveloped) {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
    }

    match args.format {
        OutputFormat::Human => {
            print_audit_human(&scan_report, &audit_report);
            print_deferred_defenses_loud(&deferred_report);
            print_convergent_evidence_concerns(&convergent_report);
            print_recurrent_concerns(&recurrent_report);
            print_lineage_fidelity_advisory(&lineage_fidelity_report);
            print_category_audit_human(&category_report);
            print_coverage_frontier(&coverage_report);
            print_prescriptive_board(&prescriptive_report);
        },
        OutputFormat::Json => match serde_json::to_string_pretty(&enveloped) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("error: failed to serialize report: {e}");
                return ExitCode::from(2);
            },
        },
    }

    // ADR-018 §"Audit diagnostic text" + §"7-state interaction matrix":
    // `--strict` promotes state 7 (inherited + unaddressed) from warn to
    // error. Without `--strict`, the audit reports state 7 as a warning
    // but still exits 0.
    let strict_state7_fails = args.strict && !audit_report.inherited_unaddressed.is_empty();
    let strict_witness_fails =
        args.strict && !audit_report.all_meet_tier(audit::WitnessTier::Reachability);
    // ADR-029 §Enforcement: under --strict, an undefended presents-site fails the
    // gate. Immunity is observed: a presents-site with no #[defended_by] witness
    // and no passing requires= predicate is an open defense circuit. (substrate-gap
    // is NOT gated here — the intent exists; it warrants a warning, not a hard
    // fail, until per-antigen severity lands in a later slice.)
    let strict_undefended_fails = args.strict && !audit_report.undefended_verdicts().is_empty();
    // ATK-STRICT-5 / findings/scan-audit-strict-divergence: audit --strict is the
    // CI integration point and MUST be a superset of scan --strict. The three
    // structural-defect gates that `scan --strict` enforces (orphaned tolerances,
    // orphaned lineage edges, dangling child lineage edges) belong in the audit
    // strict gate too — otherwise an adopter who runs `cargo antigen audit --strict`
    // for CI silently misses them. scan_report is already in scope here (line
    // ~3040 above), so this is a substrate-witness lift, not a recompute.
    let strict_orphaned_tolerances = args.strict && !scan_report.orphaned_tolerances().is_empty();
    let strict_lineage_broken = args.strict
        && (!scan_report.orphaned_lineage_edges().is_empty()
            || !scan_report.dangling_child_lineage_edges().is_empty());
    if strict_state7_fails
        || strict_witness_fails
        || strict_undefended_fails
        || strict_orphaned_tolerances
        || strict_lineage_broken
    {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

// ============================================================================
// cargo antigen attest handlers
// ============================================================================

fn run_attest(cli: AttestCli) -> ExitCode {
    match cli.command {
        AttestSubcommand::Scaffold(args) => {
            run_attest_scaffold(args, antigen_attestation::RatificationKind::Immunity)
        },
        AttestSubcommand::Sign(args) => run_attest_sign(args),
        AttestSubcommand::Check(args) => run_attest_check(args),
        AttestSubcommand::Delta(args) => run_attest_delta(args),
        AttestSubcommand::List(args) => run_attest_list(args),
        AttestSubcommand::Gc(args) => run_attest_gc(args),
        AttestSubcommand::Oracle => {
            eprintln!(
                "`attest oracle mark` is not yet implemented in v0.1-rc.\n\
                 Renamed from `attest oracle complete` per ADR-021 F28-R2 to \
                 disambiguate from the top-level `cargo antigen oracle complete` \
                 state-machine verb. Implementation pending: must write \
                 `OracleCompletionMarker {{ oracle_state_at_attestation, .. }}` \
                 at sign time per ADR-021 §D4 + ATK-021-19 (without the \
                 sign-time-state anchor, sign-time-validity cannot be \
                 structurally enforced — the audit cannot distinguish post-\
                 sign-time deprecation from pre-sign-time deprecation).\n\
                 Operator scripts MUST NOT rely on this exit code as success."
            );
            ExitCode::FAILURE
        },
    }
}

fn run_tolerate(cli: TolerateCli) -> ExitCode {
    match cli.command {
        TolerateSubcommand::Scaffold(args) => {
            run_attest_scaffold(args, antigen_attestation::RatificationKind::Tolerance)
        },
        TolerateSubcommand::Sign(args) => run_attest_sign(args),
        TolerateSubcommand::Check(args) => run_attest_check(args),
        TolerateSubcommand::List(mut args) => {
            args.tolerance_only = true;
            run_attest_list(args)
        },
    }
}

/// `attest scaffold` / `tolerate scaffold`: create a new `.attest/<Antigen>.json` sidecar.
/// Attempt to auto-fill the `current_fingerprint` for a scaffold whose
/// `--fingerprint` was omitted, by scanning the source file's crate subtree and
/// matching the immunity/presentation the operator named.
///
/// Composes with the existing scan machinery (ADR-002 compose-don't-compete)
/// rather than re-deriving the item-walk: scan already finds every immune/
/// presents site and computes its `structural_fingerprint`. We scan the source
/// file's directory subtree (a tight, predictable radius — the operator told us
/// which file the declaration lives in), then match on the antigen's last path
/// segment plus, if the operator passed one, the item path (against
/// [`scan::ItemTarget::label`]).
///
/// Returns:
/// - `Ok(Some(fp))` — exactly one match with a non-empty fingerprint.
/// - `Ok(None)` — no usable match (zero matches, or only matches with empty
///   fingerprints, e.g. explicit-marker presentations). Caller keeps the
///   empty-placeholder behavior; auto-fill is a convenience, never a hard fail.
/// - `Err(reason)` — ambiguous: multiple distinct fingerprints matched and the
///   operator didn't disambiguate with `--item-path`. Caller surfaces the
///   reason so the operator can narrow it.
fn autofill_fingerprint(
    source_file: &Path,
    antigen_stem: &str,
    item_path: &str,
) -> Result<Option<String>, String> {
    let scan_root = source_file.parent().unwrap_or_else(|| Path::new("."));
    let Ok(report) = scan::scan_workspace(scan_root, None) else {
        // A scan failure is not the operator's problem to solve here — fall
        // back to the placeholder path silently.
        return Ok(None);
    };

    let item_filter = (!item_path.is_empty()).then_some(item_path);
    let matches = |antigen_type: &str, target: &scan::ItemTarget| {
        antigen_type == antigen_stem && item_filter.is_none_or(|p| target.label() == p)
    };

    // Immunities always carry a fingerprint; presentations only for
    // fingerprint-matches (explicit markers leave it empty). Prefer immunities,
    // then fall back to presentations with a non-empty fingerprint.
    let mut fingerprints: Vec<String> = report
        .immunities
        .iter()
        .filter(|i| matches(&i.antigen_type, &i.item_target))
        .map(|i| i.structural_fingerprint.clone())
        .filter(|fp| !fp.is_empty())
        .collect();
    if fingerprints.is_empty() {
        fingerprints = report
            .presentations
            .iter()
            .filter(|p| matches(&p.antigen_type, &p.item_target))
            .map(|p| p.structural_fingerprint.clone())
            .filter(|fp| !fp.is_empty())
            .collect();
    }

    fingerprints.sort();
    fingerprints.dedup();
    match fingerprints.as_slice() {
        [] => Ok(None),
        [single] => Ok(Some(single.clone())),
        many => Err(format!(
            "auto-fill found {} distinct fingerprints for antigen `{antigen_stem}`\
             {} — pass `--item-path` to disambiguate, or `--fingerprint` explicitly.",
            many.len(),
            if item_filter.is_some() {
                format!(" at item `{item_path}`")
            } else {
                String::new()
            }
        )),
    }
}

fn run_attest_scaffold(
    args: AttestScaffoldArgs,
    kind_override: antigen_attestation::RatificationKind,
) -> ExitCode {
    use std::collections::BTreeMap;

    use antigen_attestation::{
        AntigenIdentifier, ItemRatification, Ratification, RatificationKind, SchemaVersion,
    };

    // The effective kind: --kind arg on the scaffold command, but `run_tolerate`
    // forces Tolerance regardless of the --kind arg (tolerate scaffold implies it).
    let kind = if matches!(kind_override, RatificationKind::Immunity) {
        args.kind.into()
    } else {
        kind_override
    };

    // Derive the antigen's last segment as the sidecar filename stem.
    let stem = args.antigen.rsplit("::").next().unwrap_or(&args.antigen);
    let Some(source_dir_ref) = args.source_file.parent() else {
        eprintln!("error: source-file has no parent directory");
        return ExitCode::from(2);
    };
    let source_dir = source_dir_ref.to_path_buf();
    let attest_dir = source_dir.join(".attest");
    let sidecar_path = attest_dir.join(format!("{stem}.json"));

    if sidecar_path.exists() && !args.force {
        eprintln!(
            "error: sidecar already exists: {}\n\
             Use --force to overwrite.",
            sidecar_path.display()
        );
        return ExitCode::from(1);
    }

    if let Err(e) = std::fs::create_dir_all(&attest_dir) {
        eprintln!("error: failed to create .attest/ directory: {e}");
        return ExitCode::from(2);
    }

    // When `--fingerprint` is omitted, try to auto-fill it by scanning the
    // source file's crate subtree for the immunity/presentation the operator
    // named. This removes the manual `scan --format json | jq` + hand-edit
    // dance the placeholder note otherwise prescribes.
    let mut fingerprint = args.fingerprint.clone();
    let mut autofilled = false;
    if fingerprint.is_empty() {
        match autofill_fingerprint(&args.source_file, stem, &args.item_path) {
            Ok(Some(fp)) => {
                fingerprint = fp;
                autofilled = true;
            },
            Ok(None) => {},
            Err(reason) => {
                eprintln!("error: {reason}");
                return ExitCode::from(1);
            },
        }
    }

    // An explicitly-passed --fingerprint may be malformed (auto-filled values
    // come from scan and are always valid, so this only fires for operator
    // input). A malformed digest scaffolds a sidecar whose current_fingerprint
    // can never match the item's real digest at audit — warn loudly rather than
    // bake a dead value silently. (FingerprintDigestWithoutFormatValidation —
    // the same cross-site guard run at the sign site.)
    if !autofilled {
        warn_if_empty_fingerprint(&fingerprint);
    }

    let ratification = Ratification {
        schema_version: SchemaVersion::V1,
        kind,
        antigen: AntigenIdentifier {
            name: stem.to_string(),
            defined_in: None,
        },
        source_file: args.source_file.clone(),
        items: vec![ItemRatification {
            item_path: args.item_path.clone(),
            current_fingerprint: fingerprint.clone(),
            doc_ref: None,
            signers: vec![],
            oracles: vec![],
            fresh_through: None,
            extensions: BTreeMap::new(),
        }],
    };

    let json = match serde_json::to_string_pretty(&ratification) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: failed to serialize sidecar: {e}");
            return ExitCode::from(2);
        },
    };

    if let Err(e) = std::fs::write(&sidecar_path, &json) {
        eprintln!("error: failed to write sidecar: {e}");
        return ExitCode::from(2);
    }

    eprintln!("Created sidecar: {}", sidecar_path.display());
    if autofilled {
        eprintln!("  fingerprint auto-filled from scan: {fingerprint}");
    } else if fingerprint.is_empty() {
        eprintln!(
            "  note: fingerprint is empty — auto-fill found no matching scanned site, so \
             update `current_fingerprint` before signing.\n\
             \n\
             Get the item fingerprint from:\n\
             \n  cargo antigen scan --format json | jq '.report.immunities[] | select(.antigen_type==\"{}\") | .structural_fingerprint'\n",
            stem
        );
    }
    let next_fp = if fingerprint.is_empty() {
        "<fp>".to_string()
    } else {
        fingerprint
    };
    eprintln!(
        "\nNext: `cargo antigen attest sign --sidecar {} --item-path \"{}\" --signer <name> --fingerprint {next_fp}`",
        sidecar_path.display(),
        args.item_path
    );
    ExitCode::SUCCESS
}

/// DX finding 8 guard: warn when signing against empty fingerprint.
/// Extracted to keep `run_attest_sign` under clippy's `too_many_lines` threshold.
/// Whether a string has the shape of a structural digest: the self-describing
/// `fnv1a64:` version prefix followed by 16 lowercase-hex chars (see
/// `antigen_fingerprint::digest`). A digest that doesn't match this shape can
/// never equal an item's real digest, so a substrate-witness predicate bound to
/// it silently fails at audit forever.
fn looks_like_structural_digest(fingerprint: &str) -> bool {
    let Some(hex) = fingerprint.strip_prefix("fnv1a64:") else {
        return false;
    };
    hex.len() == 16
        && hex
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

/// DX finding 8 + the digest-format gap: warn when signing against a fingerprint
/// that can't be a real structural digest — empty, OR present-but-malformed
/// (e.g. a DSL grammar string, an unprefixed hash, or a typo). Both cases record
/// a `signed_against_fingerprint` that never matches the item's real digest, so
/// the signature is dead-on-arrival at audit. A warning (not a hard error) keeps
/// the empty-case posture and tolerates a future digest scheme, but makes the
/// dead-signature loud instead of silent.
fn warn_if_empty_fingerprint(fingerprint: &str) {
    if fingerprint.is_empty() {
        eprintln!(
            "warning: signing against an empty fingerprint. A substrate-witness \
             predicate bound to `against = \"current\"` (or `fresh_within_days`) \
             will fail at audit time because the signed-against fingerprint `` \
             cannot match the item's real structural digest. Obtain the item's \
             fingerprint from `cargo antigen scan --format json` (the \
             `structural_fingerprint` field on the immunity/presentation entry) \
             and pass it via `--fingerprint`."
        );
    } else if !looks_like_structural_digest(fingerprint) {
        eprintln!(
            "warning: `--fingerprint {fingerprint}` does not look like a structural \
             digest (expected `fnv1a64:` + 16 lowercase-hex chars). It will be \
             recorded as the signed-against fingerprint, but a substrate-witness \
             predicate bound to `against = \"current\"` will never match the item's \
             real digest at audit time — the signature is dead-on-arrival. Obtain \
             the digest from `cargo antigen fingerprint` (or `scan --format json`) \
             and pass that value."
        );
    }
}

/// `attest sign` / `tolerate sign`: add a signer entry to an existing sidecar.
fn run_attest_sign(args: AttestSignArgs) -> ExitCode {
    use antigen_attestation::{Ratification, Signer, SignerBasis};

    // Load the existing sidecar.
    let content = match std::fs::read_to_string(&args.sidecar) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "error: failed to read sidecar {}: {e}",
                args.sidecar.display()
            );
            return ExitCode::from(2);
        },
    };
    let mut ratification: Ratification = match serde_json::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: sidecar is not valid JSON (Ratification schema): {e}");
            return ExitCode::from(2);
        },
    };

    // Find the target item.
    let item = ratification
        .items
        .iter_mut()
        .find(|i| i.item_path == args.item_path);
    let Some(item) = item else {
        eprintln!(
            "error: no item with path `{}` in sidecar.\n\
             Available item paths: {}",
            args.item_path,
            ratification
                .items
                .iter()
                .map(|i| i.item_path.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        );
        return ExitCode::from(1);
    };

    // Check for duplicate signer + fingerprint combo.
    let already_signed = item
        .signers
        .iter()
        .any(|s| s.name == args.signer && s.signed_against_fingerprint == args.fingerprint);
    if already_signed {
        eprintln!(
            "warning: signer `{}` has already signed this item against fingerprint `{}`.\n\
             No entry added.",
            args.signer, args.fingerprint
        );
        return ExitCode::SUCCESS;
    }

    warn_if_empty_fingerprint(&args.fingerprint);

    // Warn if signing against a fingerprint that doesn't match the sidecar's stored
    // current_fingerprint — the resulting entry will be immediately stale at audit time.
    if !item.current_fingerprint.is_empty() && args.fingerprint != item.current_fingerprint {
        eprintln!(
            "warning: --fingerprint `{}` does not match sidecar's stored \
             current_fingerprint `{}`. The new signer entry will be immediately \
             stale at audit time. Update the sidecar's current_fingerprint first, \
             or re-run `cargo antigen scan --format json` to get the current fingerprint.",
            args.fingerprint, item.current_fingerprint
        );
    }

    let today = chrono::Local::now().date_naive();
    item.signers.push(Signer {
        name: args.signer.clone(),
        role: args.role.clone(),
        date: today,
        signed_against_fingerprint: args.fingerprint.clone(),
        basis: SignerBasis::Fresh {
            reasoning: args.reasoning.clone(),
        },
        strength: antigen_attestation::SignatureStrength::from(args.strength),
        signature: None,
    });

    let json = match serde_json::to_string_pretty(&ratification) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: failed to serialize updated sidecar: {e}");
            return ExitCode::from(2);
        },
    };
    if let Err(e) = std::fs::write(&args.sidecar, &json) {
        eprintln!("error: failed to write updated sidecar: {e}");
        return ExitCode::from(2);
    }

    eprintln!(
        "Signed: {} added to `{}` item `{}` against fingerprint `{}`",
        args.signer,
        args.sidecar.display(),
        args.item_path,
        args.fingerprint
    );
    if let Some(role) = &args.role {
        eprintln!("  role: {role}");
    }
    ExitCode::SUCCESS
}

/// `attest delta`: append a carry-forward delta entry to an existing sidecar.
///
/// Reads the current sidecar, finds the last fresh-basis signature for the
/// named signer at the named item, computes `chain_depth`, enforces the
/// anti-laundering safeguards (ADR-019 §Decision §E3), and writes the new
/// `DeltaFrom` entry back.
fn run_attest_delta(args: AttestDeltaArgs) -> ExitCode {
    use antigen_attestation::schema::HARD_DELTA_CHAIN_CAP_MAX;
    use antigen_attestation::{Ratification, Signer, SignerBasis};

    // Anti-laundering safeguard: enforce chain-depth cap (default 3; hard max
    // = HARD_DELTA_CHAIN_CAP_MAX). Project TOML config may tighten; tighter
    // caps also enforced at audit time by evaluator.
    const DEFAULT_DELTA_CAP: u32 = 3;

    let content = match std::fs::read_to_string(&args.sidecar) {
        Ok(s) => s,
        Err(e) => {
            eprintln!(
                "error: could not read sidecar `{}`: {e}",
                args.sidecar.display()
            );
            return ExitCode::from(2);
        },
    };
    let mut ratification: Ratification = match serde_json::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: sidecar is not valid Ratification JSON: {e}");
            return ExitCode::from(2);
        },
    };

    // Resolve signer name from arg or git config.
    let signer_name = match resolve_steward_name(args.signer.as_deref()) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        },
    };

    if let Err(code) = validate_delta_rationale(&args.rationale) {
        return code;
    }

    let item = ratification
        .items
        .iter_mut()
        .find(|i| i.item_path == args.item_path);
    let Some(item) = item else {
        eprintln!(
            "error: no item with path `{}` in sidecar `{}`",
            args.item_path,
            args.sidecar.display()
        );
        return ExitCode::from(1);
    };

    // Find the last Fresh basis for this signer to determine cumulative root.
    let cumulative_root = item
        .signers
        .iter()
        .rfind(|s| s.name == signer_name && s.basis.is_fresh())
        .map(|s| s.signed_against_fingerprint.clone());
    let Some(cumulative_root_fingerprint) = cumulative_root else {
        eprintln!(
            "error: no prior Fresh-basis signature found for signer `{signer_name}` \
             at item `{}`. Run `attest sign` first to establish a fresh attestation \
             before using `attest delta`.",
            args.item_path
        );
        return ExitCode::from(1);
    };

    let chain_depth = compute_delta_chain_depth(&item.signers, &signer_name);

    if chain_depth > DEFAULT_DELTA_CAP {
        eprintln!(
            "error: delta chain depth {chain_depth} exceeds default cap \
             {DEFAULT_DELTA_CAP} (hard max = {HARD_DELTA_CHAIN_CAP_MAX}). \
             Run `attest sign` to re-anchor with a Fresh basis."
        );
        return ExitCode::from(1);
    }

    // Both delta fingerprints are digests that must match real item digests at
    // audit (the new signed-against and the prior it carries forward from). A
    // malformed value silently breaks the chain — warn at the boundary
    // (FingerprintDigestWithoutFormatValidation, same guard as sign/scaffold).
    warn_if_empty_fingerprint(&args.fingerprint);
    warn_if_empty_fingerprint(&args.prior_fingerprint);

    let today = chrono::Local::now().date_naive();
    item.signers.push(Signer {
        name: signer_name.clone(),
        role: args.role,
        date: today,
        signed_against_fingerprint: args.fingerprint.clone(),
        basis: SignerBasis::DeltaFrom {
            prior_fingerprint: args.prior_fingerprint.clone(),
            cumulative_root_fingerprint,
            chain_depth,
            rationale: args.rationale.clone(),
        },
        strength: antigen_attestation::SignatureStrength::from(args.strength),
        signature: None,
    });

    let json = match serde_json::to_string_pretty(&ratification) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: failed to serialize updated sidecar: {e}");
            return ExitCode::from(2);
        },
    };
    if let Err(e) = std::fs::write(&args.sidecar, &json) {
        eprintln!("error: failed to write sidecar: {e}");
        return ExitCode::from(2);
    }

    eprintln!(
        "Delta: `{}` signed `{}` item `{}` at depth {chain_depth}",
        signer_name,
        args.sidecar.display(),
        args.item_path,
    );
    ExitCode::SUCCESS
}

/// `attest list` / `tolerate list`: walk the workspace and enumerate all `.attest/` sidecars.
fn run_attest_list(args: AttestListArgs) -> ExitCode {
    use antigen_attestation::Ratification;

    let sidecars = collect_sidecars(&args.root);
    if sidecars.is_empty() {
        eprintln!(
            "No .attest/ sidecars found under `{}`.",
            args.root.display()
        );
        return ExitCode::SUCCESS;
    }

    let mut printed = 0usize;
    for path in &sidecars {
        let content = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("warning: could not read `{}`: {e}", path.display());
                continue;
            },
        };
        let rat: Ratification = match serde_json::from_str(&content) {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "warning: `{}` is not valid Ratification JSON: {e}",
                    path.display()
                );
                continue;
            },
        };
        if args.tolerance_only && rat.kind != antigen_attestation::RatificationKind::Tolerance {
            continue;
        }

        match args.format {
            OutputFormat::Human => {
                println!(
                    "{} [{:?}] — {} item(s)",
                    path.display(),
                    rat.kind,
                    rat.items.len()
                );
                for item in &rat.items {
                    println!("  {} ({} signer(s))", item.item_path, item.signers.len());
                }
            },
            OutputFormat::Json => {
                // One JSON object per line (newline-delimited JSON).
                let obj = serde_json::json!({
                    "path": path.display().to_string(),
                    "kind": format!("{:?}", rat.kind),
                    "antigen": rat.antigen.name,
                    "item_count": rat.items.len(),
                });
                println!("{obj}");
            },
        }
        printed += 1;
    }

    if args.orphan_scan {
        eprintln!(
            "\n-- Orphan scan (--orphan-scan): comparing sidecar item_paths against source macros --"
        );
        eprintln!(
            "(Note: full bidirectional scan requires `cargo antigen scan` integration; v0.2 adds gc bidirectional traversal)"
        );
        for path in &sidecars {
            let Ok(content) = std::fs::read_to_string(path) else {
                continue;
            };
            if let Ok(rat) = serde_json::from_str::<Ratification>(&content) {
                for item in &rat.items {
                    if item.signers.is_empty() {
                        println!(
                            "ORPHAN-CANDIDATE: {} item `{}` has no signers",
                            path.display(),
                            item.item_path
                        );
                    }
                }
            }
        }
    }

    eprintln!("{printed} sidecar(s) listed.");
    ExitCode::SUCCESS
}

/// `attest gc`: report orphaned sidecars (report-only in v0.1; --force deletes).
fn run_attest_gc(args: AttestGcArgs) -> ExitCode {
    use antigen_attestation::Ratification;

    let sidecars = collect_sidecars(&args.root);
    let mut orphans: Vec<std::path::PathBuf> = Vec::new();

    for path in &sidecars {
        // Warn-and-continue on unreadable / corrupt sidecars (same fix as
        // `oracle list` / the `attest list` model). A silently-skipped corrupt
        // sidecar can never be gc'd AND gives the operator no signal it's broken
        // — it just vanishes from the orphan scan (ATK-oracle-corrupt-json,
        // gc-side). A corrupt sidecar can't have its source_file checked, so it
        // is neither confirmed-orphan nor confirmed-live; surface it loudly.
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("warning: could not read `{}`: {e}", path.display());
                continue;
            },
        };
        let rat: Ratification = match serde_json::from_str(&content) {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "warning: `{}` is not valid Ratification JSON: {e} \
                     (corrupt sidecar — cannot evaluate for gc; surfaced rather than \
                     silently skipped)",
                    path.display()
                );
                continue;
            },
        };
        // An orphan heuristic: if source_file doesn't exist relative to workspace root.
        let source = args.root.join(&rat.source_file);
        if !source.exists() {
            orphans.push(path.clone());
        }
    }

    if orphans.is_empty() {
        eprintln!(
            "No orphaned sidecars found under `{}`.",
            args.root.display()
        );
        return ExitCode::SUCCESS;
    }

    eprintln!("{} orphaned sidecar(s) found:", orphans.len());
    for path in &orphans {
        println!("{}", path.display());
    }

    if args.force {
        let mut removed = 0usize;
        for path in &orphans {
            match std::fs::remove_file(path) {
                Ok(()) => {
                    eprintln!("Removed: {}", path.display());
                    removed += 1;
                },
                Err(e) => eprintln!("error removing `{}`: {e}", path.display()),
            }
        }
        eprintln!("{removed} sidecar(s) removed.");
    } else {
        eprintln!("(Run with --force to delete. Report-only in v0.1.)");
    }

    ExitCode::SUCCESS
}

/// Walk `root` recursively and return all `.attest/*.json` files found.
fn collect_sidecars(root: &std::path::Path) -> Vec<std::path::PathBuf> {
    let mut result = Vec::new();
    let Ok(entries) = std::fs::read_dir(root) else {
        return result;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path
                .file_name()
                .is_some_and(|n| n.to_string_lossy().ends_with(".attest"))
            {
                // Collect JSON files inside .attest/ directories.
                if let Ok(inner) = std::fs::read_dir(&path) {
                    for inner_entry in inner.flatten() {
                        let inner_path = inner_entry.path();
                        if inner_path.extension().is_some_and(|e| e == "json") {
                            result.push(inner_path);
                        }
                    }
                }
            } else if path.file_name().is_none_or(|n| {
                let s = n.to_string_lossy();
                !s.starts_with('.') && s != "target"
            }) {
                // Recurse into non-hidden, non-target directories.
                result.extend(collect_sidecars(&path));
            }
        }
    }
    result
}

/// Filesystem-backed evaluation context for the CLI check commands.
struct CheckContext;

impl antigen_attestation::EvaluationContext for CheckContext {
    fn today(&self) -> chrono::NaiveDate {
        chrono::Local::now().date_naive()
    }

    fn read_doc(&self, path: &std::path::Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    fn read_oracle(&self, path: &std::path::Path) -> Option<String> {
        std::fs::read_to_string(path).ok()
    }

    fn read_git_trailers(
        &self,
        item_source_file: &std::path::Path,
        _item_path: &str,
    ) -> Vec<String> {
        // Invoke `git log --format=%B -- <file>` to collect all commit messages
        // touching the item's source file, then pipe through `git interpret-trailers
        // --parse` to extract structured trailers. This satisfies the signed_trailer
        // leaf contract (ADR-019 §Decision §4 bright-line rule: git is named, has own
        // release process, does not execute user code, args are fixed).
        //
        // If git is unavailable or the file has no commits, returns Vec::new()
        // (tier-honest: no trailers found → signed_trailer predicate fails → None tier).
        let log_output = std::process::Command::new("git")
            .args([
                "log",
                "--format=%B",
                "--",
                item_source_file.to_str().unwrap_or(""),
            ])
            .output();
        let log_bytes = match log_output {
            Ok(o) if o.status.success() => o.stdout,
            _ => return Vec::new(),
        };

        let trailer_output = std::process::Command::new("git")
            .args(["interpret-trailers", "--parse"])
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .spawn()
            .and_then(|mut child| {
                use std::io::Write;
                if let Some(stdin) = child.stdin.take() {
                    let mut stdin = stdin;
                    let _ = stdin.write_all(&log_bytes);
                }
                child.wait_with_output()
            });

        match trailer_output {
            Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout)
                .lines()
                .map(|l| l.trim().to_owned())
                .filter(|l| !l.is_empty())
                .collect(),
            _ => Vec::new(),
        }
    }
}

/// `attest check` / `tolerate check`: evaluate a predicate against a sidecar.
fn run_attest_check(args: AttestCheckArgs) -> ExitCode {
    use antigen_attestation::{Predicate, Ratification, evaluate::evaluate_predicate_with_kind};

    // Load sidecar.
    let content = match std::fs::read_to_string(&args.sidecar) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "error: failed to read sidecar {}: {e}",
                args.sidecar.display()
            );
            return ExitCode::from(2);
        },
    };
    let ratification: Ratification = match serde_json::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: sidecar schema invalid: {e}");
            return ExitCode::from(2);
        },
    };

    // Deserialize the predicate.
    let predicate: Predicate = match serde_json::from_str(&args.predicate) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: predicate JSON invalid: {e}");
            return ExitCode::from(2);
        },
    };

    // Find the target item (first, or by item_path).
    let item = if let Some(ref path) = args.item_path {
        if let Some(i) = ratification.items.iter().find(|i| i.item_path == *path) {
            i
        } else {
            eprintln!("error: no item with path `{path}` in sidecar.");
            return ExitCode::from(1);
        }
    } else if let Some(i) = ratification.items.first() {
        i
    } else {
        eprintln!("error: sidecar has no items.");
        return ExitCode::from(1);
    };

    // CLI-SF-1: when --fingerprint is omitted, fall back to the sidecar's stored
    // current_fingerprint. This is self-referential and cannot detect stale signers —
    // a signer who signed against fp-old looks current if the sidecar's stored fp is
    // also fp-old, even when the real item has changed to fp-new. Always supply
    // --fingerprint from `cargo antigen scan --format json` for accurate stale detection.
    if args.fingerprint.is_none() {
        eprintln!(
            "warning: --fingerprint not supplied; using sidecar's stored \
             current_fingerprint for stale-signer detection.\n\
             If the item's code has changed since the sidecar was written, stale \
             signers will appear current. Supply --fingerprint from \
             `cargo antigen scan --format json` for accurate stale detection.\n\
             (CLI-SF-1: self-referential fingerprint cannot detect real staleness)"
        );
    } else if let Some(fp) = args.fingerprint.as_deref() {
        // A supplied --fingerprint is compared against signers' signed-against
        // digests; a malformed one mis-reports staleness either way. Warn at the
        // boundary (FingerprintDigestWithoutFormatValidation, same guard as
        // sign/scaffold/delta). Empty is impossible here (it's Some), so only the
        // format branch can fire.
        warn_if_empty_fingerprint(fp);
    }
    let current_fingerprint = args
        .fingerprint
        .as_deref()
        .unwrap_or(&item.current_fingerprint);

    let result = match evaluate_predicate_with_kind(
        &predicate,
        item,
        current_fingerprint,
        &args.sidecar,
        ratification.kind,
        &CheckContext,
    ) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: evaluation error: {e}");
            return ExitCode::from(2);
        },
    };

    eprintln!("Sidecar:   {}", args.sidecar.display());
    eprintln!("Item path: {}", item.item_path);
    eprintln!("Kind:      {:?}", ratification.kind);
    eprintln!();
    eprintln!("Result:");
    eprintln!("  witness_tier:      {:?}", result.witness_tier);
    eprintln!("  audit_hint:        {:?}", result.audit_hint);
    eprintln!("  evidence_kind:     {:?}", result.evidence_kind);
    eprintln!("  signature_strength:{:?}", result.signature_strength);

    // Finding 7: per-leaf pass/fail with expected-vs-found, so a failed compound
    // predicate is debuggable from the CLI output alone (no evaluator source dive).
    if !result.leaf_outcomes.is_empty() {
        eprintln!();
        eprintln!("Per-leaf:");
        for leaf in &result.leaf_outcomes {
            let mark = if leaf.passed { "PASS" } else { "FAIL" };
            eprintln!("  {}: {} — {}", leaf.label, mark, leaf.reason);
        }
    }

    // Exit 1 if predicate failed (tier = None means failed or missing-sidecar).
    if result.witness_tier == antigen_attestation::WitnessTier::None {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

#[derive(serde::Serialize)]
struct JsonAuditReport<'a> {
    scan: &'a scan::ScanReport,
    audit: &'a audit::AuditReport,
    /// ADR-028 G1 antigen-category coverage (defaulted-implicit-functional).
    category: &'a audit::CategoryAuditReport,
    /// ADR-023 deferred-defense state (anergy / immunosuppress / poxparty /
    /// orient): active/expired/stale counts + per-declaration hints. Always
    /// present so consumers can detect active suppressions — the LOUD invariant
    /// (forward/suppression-loud-must-be-removed): intentional defense gaps must
    /// never be silently invisible.
    deferred_defense_audit: &'a audit::DeferredDefenseAuditReport,
    /// ADR-024 convergent-evidence audit (`#[diagnostic]`/`#[clonal]`/`#[igg]`/...):
    /// per-declaration concern hints + clean/concern counts. Delivered here so
    /// the computed verdict reaches consumers (was a severed delivery arm).
    convergent_evidence_audit: &'a audit::ConvergentEvidenceAuditReport,
    /// ADR-024 recurrent-emergence audit (`#[itch]`/`#[recurrence_anchor]`/...):
    /// per-declaration concern hints + clean/concern counts. Delivered here so
    /// the computed verdict reaches consumers (was a severed delivery arm).
    recurrent_audit: &'a audit::RecurrentAuditReport,
    /// `#[descended_from]` lineage-fidelity advisory (scientist 2026-05-27):
    /// edges where the child antigen's fingerprint is detectably NOT a
    /// refinement of the parent's. ADVISORY (non-blocking) for v0.3.
    lineage_fidelity_audit: &'a audit::LineageFidelityAuditReport,
    /// Coverage / reachability audit (the ignorance frontier): per-site
    /// `UnreachedSite` verdicts with a `cause` (Barrier / `SubThreshold` /
    /// Cryptic). Empty under a flat audit (no member concept); Barrier verdicts
    /// surface under `--workspace`. Delivered here so the computed verdict
    /// reaches the adopter (the delivery-arm discipline).
    coverage_audit: &'a audit::CoverageAuditReport,
    /// ADR-033 prescriptive work-orchestration audit: each work-need
    /// (`#[panel]`/`#[rx]`/`#[refer]`/`#[biopsy]`/`#[ddx]`/`#[triage]`/`#[culture]`/
    /// `#[quarantine]`) projected to a four-valued `WorkVerdict`. This is the
    /// machine-readable form of the audit board ("code IS the board") — a live
    /// projection per ADR-034, recomputed every run, never stored.
    prescriptive_audit: &'a audit::PrescriptiveAuditReport,
}

fn print_audit_human(scan_report: &scan::ScanReport, audit_report: &audit::AuditReport) {
    println!();
    print_audit_summary(audit_report);
    println!();

    // ATK-A3-019 fix: confirmed-claims block, parallel to the warnings
    // block below. Above-Execution-tier claims get an explicit positive
    // confirmation; without it, a FormalProof witness is invisible in the
    // human-readable output (the warnings block only lists claims BELOW
    // Execution). The JSON output has always carried this; the human path
    // was display-incomplete.
    print_confirmed_immunity_claims(audit_report);

    let problematic = audit_report.problematic_audits();

    if problematic.is_empty() {
        println!("✓ All defenses meet the Execution tier or higher.");
        println!("  Note: this audit checks witness tier, not semantic adequacy");
        println!("  (whether the witness actually tests this failure class).");
        if scan_report.immunities.is_empty() {
            println!("  (No defense declarations found in the workspace.)");
        }
    } else {
        println!("⚠ {} defense(s) below Execution tier:", problematic.len());
        println!();
        for a in &problematic {
            let i = &a.immunity;
            println!(
                "  {}:{}  {} (witness = `{}`)",
                i.file.display(),
                i.line,
                i.antigen_type,
                i.witness
            );
            println!("    tier = {:?}, hint = {:?}", a.witness_tier, a.audit_hint);
            // Finding 7: per-leaf expected-vs-found, so a failed substrate-witness
            // predicate is legible without reading evaluator source.
            for leaf in &a.leaf_outcomes {
                let mark = if !leaf.evaluated {
                    "NOT-EVALUATED"
                } else if leaf.passed {
                    "PASS"
                } else {
                    "FAIL"
                };
                println!("      {}: {} — {}", leaf.label, mark, leaf.reason);
            }
            match &a.witness_status {
                audit::WitnessStatus::NotFound { reason } => {
                    println!("    → broken: {reason}");
                },
                audit::WitnessStatus::Missing => {
                    println!(
                        "    → missing: declaration has no witness identifier; \
                         a marker without proof is not a claim (per ADR-005)"
                    );
                },
                audit::WitnessStatus::Ambiguous { candidates } => {
                    println!(
                        "    → ambiguous: witness name matches {} workspace functions",
                        candidates.len(),
                    );
                    for c in candidates {
                        println!("        - {}", c.display());
                    }
                    println!(
                        "      Fix: rename one of the colliding functions, or \
                         qualify the witness path"
                    );
                },
                audit::WitnessStatus::External { tool_hint } => {
                    println!(
                        "    → external ({tool_hint}): tool prefix recognized but not invoked. \
                         antigen does not run external tools, so this witness stays at Reachability tier."
                    );
                },
                audit::WitnessStatus::Resolved { .. } => {
                    // Resolved witnesses below Execution tier (Reachability):
                    // empty function bodies, ignored tests, or unrun tests.
                    // The hint already says which case applies.
                },
            }
            // DX finding 3: a code-witness (`witness = ...`) site that also has
            // a `.attest/` sidecar on disk. The sidecar is silently uncredited
            // — substrate-witness sidecars are evaluated only on the
            // `requires = ...` path — so warn rather than let the adopter
            // believe a signed sidecar attests this site.
            if a.code_witness_sidecar_ignored {
                println!(
                    "    → sidecar ignored: a `.attest/` substrate-witness sidecar exists \
                     for this antigen, but this site uses `witness = ...`, not \
                     `requires = ...`. Substrate-witness sidecars are credited only for \
                     `requires =` immunities, so the sidecar can never be counted here. \
                     Either switch this site to `requires = <predicate>` to use the \
                     sidecar, or remove the orphan sidecar."
                );
            }
        }
        println!();
        println!(
            "Resolve below-Execution defenses by either:\n  \
             a) Pointing the witness at a runnable test (#[test] without #[ignore])\n  \
             b) Renaming colliding functions or qualifying ambiguous witness paths\n  \
             c) Adding the witness function to the workspace if it's missing\n  \
             d) Tolerating the gap with `#[antigen_tolerance(...)]` if intentional"
        );
    }

    print_state7_diagnostics(audit_report);
    print_immune_state_verdicts(audit_report);
}

/// Render the per-presents-site immune-state verdicts (ADR-029: Immunity Is
/// Observed, Not Declared). This is the audit's authoritative voice on immune
/// state — it reports `defended` / `undefended` / `substrate-gap` per site and
/// NEVER says "immune to X." The verdict describes the state of the defense
/// circuit, not whether the failure mode can fire.
fn print_immune_state_verdicts(audit_report: &audit::AuditReport) {
    use audit::ImmuneVerdict;

    if audit_report.presentation_verdicts.is_empty() {
        return;
    }

    println!();
    println!("Immune-state verdicts (ADR-029 — observed, not declared):");

    let undefended = audit_report.undefended_verdicts();
    let defended_count = audit_report
        .presentation_verdicts
        .iter()
        .filter(|v| matches!(v.verdict, ImmuneVerdict::Defended { .. }))
        .count();
    let gap_count = audit_report
        .presentation_verdicts
        .iter()
        .filter(|v| matches!(v.verdict, ImmuneVerdict::SubstrateGap))
        .count();

    println!(
        "  {} defended, {} undefended, {} substrate-gap \
         (across {} presents-site(s))",
        defended_count,
        undefended.len(),
        gap_count,
        audit_report.presentation_verdicts.len()
    );

    for v in &audit_report.presentation_verdicts {
        let site = format!("{}:{}", v.presentation.file.display(), v.presentation.line);
        match &v.verdict {
            ImmuneVerdict::Defended { tier } => {
                let witnesses = if v.defended_by.is_empty() {
                    String::new()
                } else {
                    format!(" by {}", v.defended_by.join(", "))
                };
                println!(
                    "  ✓ {site}  {} — defended at {tier:?}{witnesses}",
                    v.antigen_type
                );
            },
            ImmuneVerdict::Undefended => {
                println!(
                    "  ✗ {site}  {} — undefended (no #[defended_by] witness, \
                     no passing requires= predicate)",
                    v.antigen_type
                );
            },
            ImmuneVerdict::SubstrateGap => {
                println!(
                    "  ⚠ {site}  {} — substrate-gap (defense intent present; \
                     current substrate does not satisfy the requires= predicate)",
                    v.antigen_type
                );
            },
        }
    }
}

fn print_category_audit_human(category_report: &audit::CategoryAuditReport) {
    if !category_report.all_explicit() {
        println!();
        println!(
            "antigen-category: {} declaration(s) with absent category \
             (defaulted to FunctionalCorrectness):",
            category_report.defaulted_count
        );
        for ca in &category_report.audits {
            if ca
                .hints
                .contains(&audit::AuditHint::AntigenCategoryDefaultedImplicitFunctional)
            {
                println!(
                    "  - {} ({}:{}) — antigen-category-defaulted-implicit-functional",
                    ca.antigen_type,
                    ca.file.display(),
                    ca.line
                );
            }
        }
        println!(
            "  Add `category = AntigenCategory::...` per ADR-028. (v0.2: \
             scan-time hint; v0.2.x: parse-time hard-error for new decls.)"
        );
    }
    if !category_report.no_category_witness_mismatch() {
        println!();
        println!(
            "antigen-category: {} declaration(s) whose category is not \
             backed by a matching witness type:",
            category_report.mismatch_count
        );
        for ca in &category_report.audits {
            if ca
                .hints
                .contains(&audit::AuditHint::AntigenCategoryClaimInconsistentWithPredicateType)
            {
                println!(
                    "  - {} ({}:{}) — antigen-category-claim-inconsistent-with-predicate-type",
                    ca.antigen_type,
                    ca.file.display(),
                    ca.line
                );
            } else if ca
                .hints
                .contains(&audit::AuditHint::AntigenCategoryHybridIncompleteEvidence)
            {
                println!(
                    "  - {} ({}:{}) — antigen-category-hybrid-incomplete-evidence",
                    ca.antigen_type,
                    ca.file.display(),
                    ca.line
                );
            }
        }
        println!(
            "  SubstrateAlignment needs a `requires = ...` immunity; \
             FunctionalCorrectness needs a `witness = ...` immunity; \
             hybrid needs both (one axis present → \
             hybrid-incomplete-evidence; ADR-028 §Schema)."
        );
    }
    print_silence_witness_advisory(category_report);
}

/// Render the silence-witness shape-mismatch advisories (scientist design +
/// aristotle gate, forward/silence-witness-shape-mismatch-{hint,impl}). A
/// `SubstrateAlignment` antigen fails by silence-by-absence: a representation
/// drifts and nothing fires, because the antigen is about the ABSENCE of a
/// closure-mechanism. The two advisories flag witness shapes that cannot detect
/// that silence — no witness at all (no-witness) or a code-tier-only witness
/// that detects behavioral, not substrate-alignment, failures (wrong-tier).
/// Split out of [`print_category_audit_human`] to keep each section within the
/// per-function line budget.
fn print_silence_witness_advisory(category_report: &audit::CategoryAuditReport) {
    if category_report.no_silence_witness_mismatch() {
        return;
    }
    println!();
    println!(
        "antigen-category: SubstrateAlignment declaration(s) whose witness \
         shape cannot detect silence (advisory):"
    );
    for ca in &category_report.audits {
        if ca
            .hints
            .contains(&audit::AuditHint::AntigenWitnessShapeMismatchForSilenceNoWitness)
        {
            println!(
                "  - {} ({}:{}) — antigen-witness-shape-mismatch-for-silence-no-witness",
                ca.antigen_type,
                ca.file.display(),
                ca.line
            );
        }
        if ca
            .hints
            .contains(&audit::AuditHint::AntigenWitnessShapeMismatchForSilenceWrongTier)
        {
            println!(
                "  - {} ({}:{}) — antigen-witness-shape-mismatch-for-silence-wrong-tier",
                ca.antigen_type,
                ca.file.display(),
                ca.line
            );
        }
    }
    println!(
        "  A SubstrateAlignment failure is silence-by-absence: it surfaces \
         only when a mechanism asserts the closure exists. no-witness → wire \
         a parity/bijection witness (assert the mechanism exists, not just \
         that the two representations agree now). wrong-tier → a code-tier \
         test detects behavioral failures; reach for a `requires = ...` \
         substrate predicate or a bijection-parity test (exception: the \
         wrong-weighting generator legitimately uses a code-tier \
         confidence test — confirm the intended generator first)."
    );
}

/// Audit summary with per-tier sub-counts per ATK-A3-019.
///
/// Per ADR-005 Amendment 3: tier counts report the work the audit ACTUALLY
/// PERFORMED, never potential maximum evidence. A `#[test]` whose run was
/// not invoked sits at Reachability, not Execution.
///
/// `resolved_count` is split into per-tier sub-counts so a `FormalProof`
/// claim (phantom-type witness, type-system-encoded proof) does NOT get
/// labeled "not yet semantically verified" — that label is true for
/// `Reachability`-tier resolutions only. `FormalProof` is semantically
/// verified at compile time; `Execution` is verified by an executed test run.
fn print_audit_summary(audit_report: &audit::AuditReport) {
    let formal_proof_count = audit_report
        .audits
        .iter()
        .filter(|a| a.witness_tier == audit::WitnessTier::FormalProof)
        .count();
    let execution_count = audit_report
        .audits
        .iter()
        .filter(|a| a.witness_tier == audit::WitnessTier::Execution)
        .count();
    // `Resolved` status entries that aren't FormalProof or Execution sit at
    // Reachability — the original "declared but not semantically verified"
    // case. resolved_count is the total of Resolved entries from the
    // AuditReport bookkeeping; the remainder after subtracting the higher
    // tiers is what stays at Reachability.
    let reachability_resolved_count = audit_report
        .resolved_count
        .saturating_sub(formal_proof_count + execution_count);

    println!("Audited {} defense(s):", audit_report.audits.len());
    if formal_proof_count > 0 {
        println!(
            "  - {formal_proof_count} formal-proof (phantom-type or formal-verification \
             tool — compile-time evidence)"
        );
    }
    if execution_count > 0 {
        println!("  - {execution_count} execution (test/proptest run confirmed by audit)");
    }
    println!(
        "  - {reachability_resolved_count} declared (witness identifier found in \
         workspace — not yet semantically verified)"
    );
    println!(
        "  - {} external (delegated to clippy/kani/prusti/etc. — not yet executed by antigen)",
        audit_report.external_count
    );
    println!(
        "  - {} ambiguous (witness name resolves to multiple workspace functions)",
        audit_report.ambiguous_count
    );
    println!(
        "  - {} broken (witness identifier not found)",
        audit_report.broken_count
    );
    println!(
        "  - {} missing (no witness identifier)",
        audit_report.missing_count
    );
}

/// Confirmed-defense block per ATK-A3-019.
///
/// Parallel to the warnings block but for the positive case: lists defense
/// claims whose witness reached `Execution` or `FormalProof` tier — the
/// audit tiers that represent confirmed evidence rather than mere
/// reachability. Without this block, a `FormalProof` claim (phantom-type
/// witness) was invisible in human-readable output; the warnings block only
/// surfaces below-Execution claims.
fn print_confirmed_immunity_claims(audit_report: &audit::AuditReport) {
    let confirmed: Vec<&audit::ImmunityAudit> = audit_report
        .audits
        .iter()
        .filter(|a| a.witness_tier >= audit::WitnessTier::Execution)
        .collect();
    if confirmed.is_empty() {
        return;
    }
    println!(
        "✓ {} defense(s) at Execution tier or higher:",
        confirmed.len()
    );
    println!();
    for a in &confirmed {
        let i = &a.immunity;
        println!(
            "  {}:{}  {} (witness = `{}`)",
            i.file.display(),
            i.line,
            i.antigen_type,
            i.witness
        );
        println!("    tier = {:?}, hint = {:?}", a.witness_tier, a.audit_hint);
    }
    println!();
}

fn print_state7_diagnostics(audit_report: &audit::AuditReport) {
    // ADR-018 §"Audit diagnostic text": state-7 warnings (inherited
    // Presentations lacking re-attestation on the descendant site).
    if audit_report.inherited_unaddressed.is_empty() {
        return;
    }
    println!();
    println!(
        "⚠ {} inherited presentation(s) not re-attested on the descendant \
         (state 7 of the 7-state interaction matrix):",
        audit_report.inherited_unaddressed.len()
    );
    println!();
    for iu in &audit_report.inherited_unaddressed {
        let p = &iu.presentation;
        let ancestors: Vec<String> = p
            .inherited_from
            .as_ref()
            .map(|chain| chain.iter().map(|pe| pe.antigen_type.clone()).collect())
            .unwrap_or_default();
        println!(
            "  warning: inherited presentation: `{}` flowed from {:?} \
             to `{}` via `#[descended_from]`;",
            p.antigen_type, ancestors, p.item_kind
        );
        println!(
            "  the witness inherited from the ancestor has not been \
             re-attested on the descendant."
        );
        println!(
            "  Add `#[defended_by({})]` on a test (code-tier), or \
             `#[presents({}, requires = ...)]` for substrate-witness evidence, or \
             `#[antigen_tolerance({}, rationale = \"...\")]` on the descendant.",
            p.antigen_type, p.antigen_type, p.antigen_type
        );
        println!("    --> {}:{}", p.file.display(), p.line);
        println!();
    }
    println!(
        "  Note: behavioral re-validation (does the ancestor's witness \
         apply to the descendant?) is not performed; reachability-tier \
         audit cannot perform this check."
    );
    println!(
        "  Use `cargo antigen audit --strict` to promote state-7 \
         warnings to errors for CI gating."
    );
}

#[cfg(test)]
mod tests {
    use super::cratesio_index_path;

    // The crates.io sparse-index path convention is the deterministic core of
    // the live-verification network shell — unit-tested here. The 3-valued
    // verdict (compare_live_cksum) is tested in the antigen lib; the network
    // fetch itself (real HTTP to crates.io) is verified manually, since it
    // cannot be made hermetic.

    #[test]
    fn cratesio_index_path_follows_cargo_length_convention() {
        // 1/2/3-char names get the short prefixes; 4+ get the <c1c2>/<c3c4> form.
        assert_eq!(cratesio_index_path("a").as_deref(), Some("1/a"));
        assert_eq!(cratesio_index_path("ab").as_deref(), Some("2/ab"));
        assert_eq!(cratesio_index_path("abc").as_deref(), Some("3/a/abc"));
        assert_eq!(cratesio_index_path("serde").as_deref(), Some("se/rd/serde"));
        assert_eq!(cratesio_index_path("ureq").as_deref(), Some("ur/eq/ureq"));
    }

    #[test]
    fn cratesio_index_path_lowercases_the_name() {
        // The index is stored lowercase; a mixed-case name must normalize.
        assert_eq!(cratesio_index_path("Serde").as_deref(), Some("se/rd/serde"));
        assert_eq!(cratesio_index_path("AB").as_deref(), Some("2/ab"));
    }

    #[test]
    fn cratesio_index_path_empty_name_is_none() {
        assert_eq!(cratesio_index_path(""), None);
    }
}
