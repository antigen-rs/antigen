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
//!   pending tambear Phase 4 adoption feedback).
//!
//! Two additional subcommands (`new`, `vaccinate`) exist as design-phase
//! stubs but are hidden from `--help` until they ship beyond stub state
//! (per A3.5 onboarding sweep).
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

use clap::{Parser, Subcommand};

use antigen::{audit, scan};

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
    /// Scaffold a new antigen declaration (design phase).
    ///
    /// Hidden from `--help` output until the command ships beyond its
    /// design-phase stub. Stub message remains for users who discover the
    /// name via docs or history. Per the onboarding sweep (Phase 1) +
    /// Tekgy's directive: show new users the surface that works, not the
    /// surface that doesn't.
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
    /// in JSON output. Per A3 scope-lock: no cross-crate `addresses()` matching
    /// in v0.1 — each crate's report stays its own bag of antigens. Default OFF
    /// for backward compatibility.
    #[arg(long)]
    include_deps: bool,
    /// Filter to antigen declarations of a single category (ADR-028 §CLI
    /// integration). Accepts `substrate-alignment` or `functional-correctness`.
    /// A hybrid antigen (both categories) matches either filter.
    #[arg(long)]
    category: Option<String>,
}

#[derive(Debug, Parser)]
struct AuditArgs {
    /// Workspace root (default: current directory)
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Output format: human or json
    #[arg(long, default_value = "human")]
    format: OutputFormat,
    /// Exit with non-zero status if any immunity witness is unresolved
    /// (`Missing`, `NotFound`, or `Ambiguous`). v0.1: gates on `Reachability`
    /// tier minimum; `Execution`-tier gating arrives in A3 with `cargo test`
    /// integration.
    #[arg(long)]
    strict: bool,
    /// Filter the category audit to a single category (ADR-028 §CLI
    /// integration). Accepts `substrate-alignment` or `functional-correctness`.
    /// A hybrid antigen (both categories) matches either filter.
    #[arg(long)]
    category: Option<String>,
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
        }
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
        }
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
        }
        MaintainerState::Changed { added, removed } => {
            println!("result: MAINTAINER-CHANGE-WITHOUT-REATTESTATION");
            println!("  added owners: {added:?}");
            println!("  removed owners: {removed:?}");
            ExitCode::from(1)
        }
        MaintainerState::SnapshotMissing => {
            println!("result: snapshot missing");
            println!(
                "  expected: .attest/supply-chain/maintainer/{crate_name}.json",
                crate_name = args.crate_name
            );
            println!("  (v0.2: snapshot is operator-managed; v0.3+ adds live crates.io query)");
            ExitCode::from(1)
        }
        MaintainerState::CratesIoQueryUnavailable => {
            println!("result: crates.io query unavailable (v0.2 limitation per ADR-025)");
            ExitCode::from(2)
        }
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
        }
    };

    let signed_by = match resolve_steward_name(args.signed_by.as_deref()) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
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
        }
    };
    if let Err(e) = std::fs::write(&path, &json) {
        eprintln!("error: write {}: {e}", path.display());
        return ExitCode::from(2);
    }
    println!("recorded dep-attestation at {}", path.display());
    ExitCode::SUCCESS
}

fn run_verify_dep_pin(args: VerifyDepPinArgs) -> ExitCode {
    use antigen::supply_chain::evaluate::evaluate_dep_pinned;
    use antigen::supply_chain::witness::DepPinnedState;
    let state = evaluate_dep_pinned(&args.root, None);
    match state {
        DepPinnedState::AllPinned => {
            println!("verify dep-pin: all Cargo.toml deps already exact-pinned. No edits needed.");
            ExitCode::SUCCESS
        }
        DepPinnedState::Unpinned { unpinned_deps } => {
            println!(
                "verify dep-pin: {n} unpinned dep(s) detected. Suggested edits:",
                n = unpinned_deps.len()
            );
            for name in &unpinned_deps {
                println!("  - in Cargo.toml [dependencies]: pin `{name}` to `=<RESOLVED_VERSION>`");
            }
            println!();
            println!(
                "v0.2: this subcommand prints suggested edits. In-place \
                 rewriting is deferred to v0.3+ (needs a real toml parser to \
                 preserve formatting + comments)."
            );
            ExitCode::SUCCESS
        }
        DepPinnedState::NotInManifest { crate_name } => {
            eprintln!("error: crate `{crate_name}` not found in manifest");
            ExitCode::from(2)
        }
    }
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
        }
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
        }
        Err(e) => {
            eprintln!("error: write record: {e}");
            ExitCode::from(2)
        }
    }
}

fn run_verify_content_hash_check(args: VerifyContentHashCheckArgs) -> ExitCode {
    use antigen::supply_chain::evaluate::evaluate_content_hash_matches;
    use antigen::supply_chain::witness::ContentHashState;

    let Some((crate_name, version)) = parse_crate_at_version(&args.crate_at_version) else {
        eprintln!("error: argument must be `<crate>@<version>`");
        return ExitCode::from(2);
    };

    let state = evaluate_content_hash_matches(&args.root, &crate_name, &version);
    match state {
        ContentHashState::Matches => {
            println!("content-hash: MATCH for {crate_name}@{version}");
            ExitCode::SUCCESS
        }
        ContentHashState::Mismatch { recorded, current } => {
            println!("content-hash: MISMATCH for {crate_name}@{version}");
            println!("  recorded: {recorded}");
            println!("  current:  {current}");
            println!();
            println!("This is the chalk/debug/eslint-config attack signal. Investigate before");
            println!("re-recording — if the change is legitimate, re-attest with a fresh");
            println!("signer + review artifact. Per ADR-025 §ContentHashMismatch.");
            ExitCode::from(1)
        }
        ContentHashState::NoAttestation => {
            println!("content-hash: no first-attestation for {crate_name}@{version}");
            println!("  run: cargo antigen verify content-hash record {crate_name}@{version}");
            ExitCode::from(1)
        }
        ContentHashState::CrateNotInLockfile { crate_name: cn } => {
            eprintln!("error: crate `{cn}` not found in Cargo.lock (no checksum to compare)");
            ExitCode::from(2)
        }
        ContentHashState::SidecarMalformed { error } => {
            eprintln!(
                "error: content-hash sidecar exists but did NOT deserialize cleanly. \
                 Per ATK-SC-2-A this is distinct from no-attestation — corrupting the \
                 sidecar to silently downgrade a Mismatch into a NoAttestation is exactly \
                 the attack the audit blocks. Inspect the file and re-record. \
                 Parse error: {error}"
            );
            ExitCode::from(1)
        }
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

fn run_vcs(cli: VcsCli) -> ExitCode {
    match cli.command {
        VcsSubcommand::CheckCommit(args) => run_vcs_check_commit(args),
        VcsSubcommand::Scan(args) => run_vcs_scan(args),
        VcsSubcommand::BranchArchive(args) => run_vcs_branch_archive(args),
        VcsSubcommand::RollbackPrepare(args) => run_vcs_rollback_prepare(args),
        VcsSubcommand::Attest => run_vcs_attest_stub(),
    }
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
        }
        OutputFormat::Human => match &state {
            RollbackTriageState::ChainPresent { decision } => {
                println!(
                    "commit {}: rollback-triage chain present (Triage-Decision: {})",
                    args.commit,
                    decision.as_str()
                );
            }
            RollbackTriageState::ChainMalformed { value } => {
                println!(
                    "commit {}: Triage-Decision trailer present but value {value:?} is not a \
                     valid triage decision (black|red|yellow|green|white)",
                    args.commit
                );
            }
            RollbackTriageState::ChainAbsent => {
                println!(
                    "commit {}: no Triage-Decision trailer — backs \
                     vcs-rollback-without-triage-commit if this is a rollback",
                    args.commit
                );
            }
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
        }
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
        }
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
        }
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
        }
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
        }
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
        }
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
        }
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
                }
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
        }
    };
    let mut found = 0usize;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "json") {
            let Ok(content) = std::fs::read_to_string(&path) else {
                continue;
            };
            if let Ok(oracle) =
                serde_json::from_str::<antigen_attestation::schema::Oracle>(&content)
            {
                match args.format {
                    OutputFormat::Human => {
                        println!(
                            "{} [{:?}] — {} steward(s)",
                            oracle.id,
                            oracle.state,
                            oracle.stewards.len()
                        );
                    }
                    OutputFormat::Json => {
                        let obj = serde_json::json!({ "id": oracle.id, "state": format!("{:?}", oracle.state) });
                        println!("{obj}");
                    }
                }
                found += 1;
            }
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
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(1)
        }
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
        }
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
            }
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
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
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
        }
    };
    let mut oracle = match load_oracle(root, id) {
        Ok(o) => o,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
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
        }
        Err(e) => {
            eprintln!("{e}");
            ExitCode::from(2)
        }
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
        }
    };
    oracle.version = OracleVersion {
        pinned: args.version.clone(),
        pinned_at: Local::now().date_naive(),
    };
    match save_oracle(&args.root, &oracle) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
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
        }
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
        }
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

fn run_scan(args: ScanArgs) -> ExitCode {
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

    // ADR-028 §CLI integration: --category filters to a single category.
    let Ok(category_filter) = parse_category_filter(args.category.as_deref()) else {
        return ExitCode::from(2);
    };

    eprintln!("Scanning workspace: {}", args.root.display());

    let mut report = match scan::scan_workspace(&args.root, None) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: scan failed: {e}");
            return ExitCode::from(2);
        }
    };

    if let Some(cat) = category_filter {
        filter_report_by_category(&mut report, cat);
    }

    let unaddressed = report.unaddressed_presentations();

    // A3 D3: optional cross-crate dep enumeration + per-crate scan. Per
    // navigator's 2026-05-09 ruling, deps are scanned independently — no
    // cross-crate addresses() matching in v0.1. Each dep report stands on
    // its own; the union appears under `dep_reports` in JSON output.
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
                        }
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
            }
            Err(e) => {
                eprintln!("error: cargo metadata failed: {e}");
                return ExitCode::from(2);
            }
        }
    } else {
        None
    };

    match args.format {
        OutputFormat::Human => {
            print_human_report(&report, &unaddressed);
            if let Some(deps) = dep_reports.as_ref() {
                print_human_dep_summary(deps);
            }
        }
        OutputFormat::Json => match serde_json::to_string_pretty(&JsonReport {
            report: &report,
            unaddressed: &unaddressed,
            dep_reports: dep_reports.as_deref(),
        }) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("error: failed to serialize report: {e}");
                return ExitCode::from(2);
            }
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
    if args.strict && (unaddressed_explicit_count > 0 || !report.orphaned_tolerances().is_empty()) {
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
        println!("  (P5 finding 2026-05-09: zero `#[antigen(...)]` instances in the wild.)");
    } else {
        for d in deps_with_antigens {
            println!(
                "  {} v{}: {} antigen(s), {} presentation(s), {} immunity claim(s)",
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
    println!("  - {} immunity claims", report.immunities.len());
    if !report.parse_failures.is_empty() {
        println!(
            "  - {} parse failures (see --format json for details)",
            report.parse_failures.len()
        );
    }
    println!();

    print_fingerprint_matches(report);
    print_orphaned_tolerances(report);
    print_unaddressed(unaddressed);
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
    use antigen::scan::MatchKind;
    use std::collections::BTreeMap;

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
    println!("    #[presents(<antigen>)] to mark explicitly,");
    println!("    #[immune(<antigen>, witness = ...)] if defended,");
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
    println!("  #[immune(<antigen>, witness = ...)] on the same item, OR #[antigen_tolerance(<antigen>, rationale = \"...\")]");
}

#[derive(serde::Serialize)]
struct JsonReport<'a> {
    report: &'a scan::ScanReport,
    unaddressed: &'a [scan::UnaddressedPresentation],
    /// A3 D3: when `--include-deps` is set, one entry per scanned dep
    /// crate. `None` (skipped in JSON output via `skip_serializing_if`)
    /// when the flag wasn't passed — preserves byte-identical output for
    /// existing consumers.
    #[serde(skip_serializing_if = "Option::is_none")]
    dep_reports: Option<&'a [DepScanResult]>,
}

/// Per-dependency scan result returned by the `--include-deps` mode of
/// `cargo antigen scan`. Each dep is scanned independently — per navigator's
/// 2026-05-09 ruling on cross-crate scope, no cross-crate `addresses()`
/// matching happens here (ATK-A3-005 / module-path-qualified `ItemTarget`
/// is an ADR-class decision deferred to aristotle Phase 1-8).
#[derive(serde::Serialize)]
struct DepScanResult {
    package_name: String,
    version: String,
    origin: scan::CrateOrigin,
    report: scan::ScanReport,
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
        }
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
            }
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
        }
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

    eprintln!("Auditing workspace: {}", args.root.display());

    let mut scan_report = match scan::scan_workspace(&args.root, None) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: scan failed: {e}");
            return ExitCode::from(2);
        }
    };

    if let Some(cat) = category_filter {
        filter_report_by_category(&mut scan_report, cat);
    }

    let audit_report = audit::audit(&scan_report, &args.root);

    // ADR-028 category audit (audit-time):
    //   G1 (scan-time-only enforcement): surface the
    //   antigen-category-defaulted-implicit-functional migration hint for every
    //   antigen declaration with an absent category — the load-bearing signal
    //   (per adversarial's G1 ratification) that makes absent-category visible
    //   rather than a silent false-green.
    //   G2 (category↔witness-type cross-check): surface
    //   antigen-category-claim-inconsistent-with-predicate-type for any
    //   explicit-category declaration whose immunities are the wrong witness
    //   type for the declared category (per Amendment 2 + aristotle F1).
    let category_report = audit::audit_category(&scan_report);

    match args.format {
        OutputFormat::Human => {
            print_audit_human(&scan_report, &audit_report);
            print_category_audit_human(&category_report);
        }
        OutputFormat::Json => match serde_json::to_string_pretty(&JsonAuditReport {
            scan: &scan_report,
            audit: &audit_report,
            category: &category_report,
        }) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("error: failed to serialize report: {e}");
                return ExitCode::from(2);
            }
        },
    }

    // ADR-018 §"Audit diagnostic text" + §"7-state interaction matrix":
    // `--strict` promotes state 7 (inherited + unaddressed) from warn to
    // error. Without `--strict`, the audit reports state 7 as a warning
    // but still exits 0.
    let strict_state7_fails = args.strict && !audit_report.inherited_unaddressed.is_empty();
    let strict_witness_fails =
        args.strict && !audit_report.all_meet_tier(audit::WitnessTier::Reachability);
    if strict_state7_fails || strict_witness_fails {
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
        }
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
        }
    }
}

fn run_tolerate(cli: TolerateCli) -> ExitCode {
    match cli.command {
        TolerateSubcommand::Scaffold(args) => {
            run_attest_scaffold(args, antigen_attestation::RatificationKind::Tolerance)
        }
        TolerateSubcommand::Sign(args) => run_attest_sign(args),
        TolerateSubcommand::Check(args) => run_attest_check(args),
        TolerateSubcommand::List(mut args) => {
            args.tolerance_only = true;
            run_attest_list(args)
        }
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
    use antigen_attestation::{
        AntigenIdentifier, ItemRatification, Ratification, RatificationKind, SchemaVersion,
    };
    use std::collections::BTreeMap;

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
            }
            Ok(None) => {}
            Err(reason) => {
                eprintln!("error: {reason}");
                return ExitCode::from(1);
            }
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
        }
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
        }
    };
    let mut ratification: Ratification = match serde_json::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: sidecar is not valid JSON (Ratification schema): {e}");
            return ExitCode::from(2);
        }
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
        }
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
        }
    };
    let mut ratification: Ratification = match serde_json::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: sidecar is not valid Ratification JSON: {e}");
            return ExitCode::from(2);
        }
    };

    // Resolve signer name from arg or git config.
    let signer_name = match resolve_steward_name(args.signer.as_deref()) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(1);
        }
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
        }
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
            }
        };
        let rat: Ratification = match serde_json::from_str(&content) {
            Ok(r) => r,
            Err(e) => {
                eprintln!(
                    "warning: `{}` is not valid Ratification JSON: {e}",
                    path.display()
                );
                continue;
            }
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
            }
            OutputFormat::Json => {
                // One JSON object per line (newline-delimited JSON).
                let obj = serde_json::json!({
                    "path": path.display().to_string(),
                    "kind": format!("{:?}", rat.kind),
                    "antigen": rat.antigen.name,
                    "item_count": rat.items.len(),
                });
                println!("{obj}");
            }
        }
        printed += 1;
    }

    if args.orphan_scan {
        eprintln!("\n-- Orphan scan (--orphan-scan): comparing sidecar item_paths against source macros --");
        eprintln!("(Note: full bidirectional scan requires `cargo antigen scan` integration; v0.2 adds gc bidirectional traversal)");
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
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        if let Ok(rat) = serde_json::from_str::<Ratification>(&content) {
            // An orphan heuristic: if source_file doesn't exist relative to workspace root.
            let source = args.root.join(&rat.source_file);
            if !source.exists() {
                orphans.push(path.clone());
            }
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
                }
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
    use antigen_attestation::{evaluate::evaluate_predicate_with_kind, Predicate, Ratification};

    // Load sidecar.
    let content = match std::fs::read_to_string(&args.sidecar) {
        Ok(c) => c,
        Err(e) => {
            eprintln!(
                "error: failed to read sidecar {}: {e}",
                args.sidecar.display()
            );
            return ExitCode::from(2);
        }
    };
    let ratification: Ratification = match serde_json::from_str(&content) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: sidecar schema invalid: {e}");
            return ExitCode::from(2);
        }
    };

    // Deserialize the predicate.
    let predicate: Predicate = match serde_json::from_str(&args.predicate) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("error: predicate JSON invalid: {e}");
            return ExitCode::from(2);
        }
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
        }
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
        println!("✓ All immunity claims meet the Execution tier or higher.");
        println!(
            "  Note: semantic verification (does the witness actually test this failure class?)"
        );
        println!("  requires fingerprint-aware audit, planned for Sweep A4-A5.");
        if scan_report.immunities.is_empty() {
            println!("  (No immunity declarations found in the workspace.)");
        }
    } else {
        println!(
            "⚠ {} immunity claim(s) below Execution tier:",
            problematic.len()
        );
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
                }
                audit::WitnessStatus::Missing => {
                    println!(
                        "    → missing: declaration has no witness identifier; \
                         a marker without proof is not a claim (per ADR-005)"
                    );
                }
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
                }
                audit::WitnessStatus::External { tool_hint } => {
                    println!(
                        "    → external ({tool_hint}): tool prefix recognized but not invoked. \
                         A3+ will run the tool to promote this witness to Execution tier."
                    );
                }
                audit::WitnessStatus::Resolved { .. } => {
                    // Resolved witnesses below Execution tier (Reachability):
                    // empty function bodies, ignored tests, or unrun tests.
                    // The hint already says which case applies.
                }
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
            "Resolve below-Execution claims by either:\n  \
             a) Adding test invocation that exercises the witness path (A4-A5 feature)\n  \
             b) Pointing the witness at a runnable test (#[test] without #[ignore])\n  \
             c) Renaming colliding functions or qualifying ambiguous witness paths\n  \
             d) Adding the witness function to the workspace if it's missing\n  \
             e) Tolerating the gap with `#[antigen_tolerance(...)]` if intentional"
        );
    }

    print_state7_diagnostics(audit_report);
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
}

/// Audit summary with per-tier sub-counts per ATK-A3-019 (A3.5 onboarding sweep).
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

    println!("Audited {} immunity claim(s):", audit_report.audits.len());
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

/// Confirmed-immunity-claims block per ATK-A3-019 (A3.5 onboarding sweep).
///
/// Parallel to the warnings block but for the positive case: lists immunity
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
        "✓ {} immunity claim(s) at Execution tier or higher:",
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
            "  Add `#[immune({}, witness = ...)]` or \
             `#[antigen_tolerance({}, rationale = \"...\")]` on the \
             descendant.",
            p.antigen_type, p.antigen_type
        );
        println!("    --> {}:{}", p.file.display(), p.line);
        println!();
    }
    println!(
        "  Note: behavioral re-validation (does the ancestor's witness \
         apply to the descendant?) is A4-A5 work; reachability-tier \
         audit cannot perform this check."
    );
    println!(
        "  Use `cargo antigen audit --strict` to promote state-7 \
         warnings to errors for CI gating."
    );
}
