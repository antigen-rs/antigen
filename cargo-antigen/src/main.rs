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

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use antigen::{audit, scan};

/// Cargo subcommand for antigen.
#[derive(Debug, Parser)]
#[command(name = "cargo-antigen", bin_name = "cargo")]
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
    /// Register an oracle completion marker (design phase).
    #[command(hide = true)]
    Oracle,
    /// List all `.attest/` sidecars in the workspace.
    List(AttestListArgs),
    /// Migrate a sidecar to a new schema version (design phase).
    #[command(hide = true)]
    Migrate,
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
    /// Only list tolerance sidecars (RatificationKind::Tolerance).
    #[arg(long)]
    tolerance_only: bool,
    /// Walk `.attest/` directories independent of scan-side macro discovery
    /// and report orphaned sidecars (sidecars whose item_path doesn't appear
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

#[derive(Debug, Clone, clap::ValueEnum)]
enum OutputFormat {
    Human,
    Json,
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
    }
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
    eprintln!("Scanning workspace: {}", args.root.display());

    let report = match scan::scan_workspace(&args.root, None) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: scan failed: {e}");
            return ExitCode::from(2);
        }
    };

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

    if args.strict && (!unaddressed.is_empty() || !report.orphaned_tolerances().is_empty()) {
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
        println!("  - {fingerprint_count} fingerprint matches (unmarked sites)");
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

fn print_fingerprint_matches(report: &scan::ScanReport) {
    use antigen::scan::MatchKind;
    let fp_matches: Vec<_> = report
        .presentations
        .iter()
        .filter(|p| p.match_kind == MatchKind::FingerprintMatch)
        .collect();
    if fp_matches.is_empty() {
        return;
    }
    println!(
        "{} fingerprint match(es) — structurally similar to a declared antigen:",
        fp_matches.len()
    );
    println!();
    for p in &fp_matches {
        println!(
            "  {}:{}  {} on {} [fingerprint match]",
            p.file.display(),
            p.line,
            p.antigen_type,
            p.item_kind
        );
    }
    println!();
    println!("  To acknowledge each site, use the antigen type shown above:");
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
    eprintln!("Auditing workspace: {}", args.root.display());

    let scan_report = match scan::scan_workspace(&args.root, None) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: scan failed: {e}");
            return ExitCode::from(2);
        }
    };

    let audit_report = audit::audit(&scan_report, &args.root);

    match args.format {
        OutputFormat::Human => {
            print_audit_human(&scan_report, &audit_report);
        }
        OutputFormat::Json => match serde_json::to_string_pretty(&JsonAuditReport {
            scan: &scan_report,
            audit: &audit_report,
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
        AttestSubcommand::Oracle | AttestSubcommand::Migrate => {
            eprintln!(
                "This attest subcommand is not implemented in v0.1-rc.\n\
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
            current_fingerprint: args.fingerprint.clone(),
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
    if args.fingerprint.is_empty() {
        eprintln!(
            "  note: fingerprint is empty — update `current_fingerprint` before signing.\n\
             \n\
             Get the item fingerprint from:\n\
             \n  cargo antigen scan --format json | jq '.immunities[] | select(.antigen_type==\"{}\") | .requires_predicate'\n",
            stem
        );
    }
    eprintln!(
        "\nNext: `cargo antigen attest sign --sidecar {} --item-path \"{}\" --signer <name> --fingerprint <fp>`",
        sidecar_path.display(),
        args.item_path
    );
    ExitCode::SUCCESS
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
/// named signer at the named item, computes chain_depth, enforces the
/// anti-laundering safeguards (ADR-019 §Decision §E3), and writes the new
/// DeltaFrom entry back.
fn run_attest_delta(args: AttestDeltaArgs) -> ExitCode {
    use antigen_attestation::{Ratification, Signer, SignerBasis};
    use antigen_attestation::schema::HARD_DELTA_CHAIN_CAP_MAX;

    let content = match std::fs::read_to_string(&args.sidecar) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: could not read sidecar `{}`: {e}", args.sidecar.display());
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
    let signer_name = match args.signer {
        Some(ref s) => s.clone(),
        None => {
            let out = std::process::Command::new("git")
                .args(["config", "user.name"])
                .output();
            match out {
                Ok(o) if o.status.success() => {
                    String::from_utf8_lossy(&o.stdout).trim().to_owned()
                }
                _ => {
                    eprintln!("error: --signer not provided and `git config user.name` failed");
                    return ExitCode::from(1);
                }
            }
        }
    };

    // Anti-laundering T2R-B: enforce minimum rationale character count at CLI layer.
    // The schema validate() also enforces this at audit time, but catching it here
    // prevents writing a sidecar that immediately fails validation (CLI-SF-3).
    {
        use antigen_attestation::schema::DEFAULT_DELTA_RATIONALE_MIN_CHARS;
        let trimmed = args.rationale.trim();
        if trimmed.is_empty() {
            eprintln!("error: --rationale must be non-empty (anti-laundering safeguard T2R-C)");
            return ExitCode::from(1);
        }
        if trimmed.chars().count() < DEFAULT_DELTA_RATIONALE_MIN_CHARS {
            eprintln!(
                "error: --rationale is too short ({} chars); minimum is {} chars \
                 (anti-laundering safeguard T2R-B). Rubber-stamp rationales are rejected.",
                trimmed.chars().count(),
                DEFAULT_DELTA_RATIONALE_MIN_CHARS,
            );
            return ExitCode::from(1);
        }
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
        .filter(|s| s.name == signer_name && s.basis.is_fresh())
        .last()
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

    // Count existing delta chain depth for this signer since last Fresh.
    let chain_depth = item
        .signers
        .iter()
        .rev()
        .take_while(|s| s.name == signer_name && s.basis.is_delta())
        .count() as u32
        + 1;

    // Anti-laundering safeguard: enforce chain-depth cap (default 3; hard max = HARD_DELTA_CHAIN_CAP_MAX).
    // Project TOML config may tighten; tighter caps also enforced at audit time by evaluator.
    const DEFAULT_DELTA_CAP: u32 = 3;
    if chain_depth > DEFAULT_DELTA_CAP {
        eprintln!(
            "error: delta chain depth {chain_depth} exceeds default cap \
             {DEFAULT_DELTA_CAP} (hard max = {HARD_DELTA_CHAIN_CAP_MAX}). \
             Run `attest sign` to re-anchor with a Fresh basis."
        );
        return ExitCode::from(1);
    }

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
        eprintln!("No .attest/ sidecars found under `{}`.", args.root.display());
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
                eprintln!("warning: `{}` is not valid Ratification JSON: {e}", path.display());
                continue;
            }
        };
        if args.tolerance_only
            && rat.kind != antigen_attestation::RatificationKind::Tolerance
        {
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
            let content = match std::fs::read_to_string(path) {
                Ok(s) => s,
                Err(_) => continue,
            };
            if let Ok(rat) = serde_json::from_str::<Ratification>(&content) {
                for item in &rat.items {
                    if item.signers.is_empty() {
                        println!("ORPHAN-CANDIDATE: {} item `{}` has no signers", path.display(), item.item_path);
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
        let content = match std::fs::read_to_string(path) {
            Ok(s) => s,
            Err(_) => continue,
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
        eprintln!("No orphaned sidecars found under `{}`.", args.root.display());
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
    let Ok(entries) = std::fs::read_dir(root) else { return result; };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            if path.file_name().map_or(false, |n| n.to_string_lossy().ends_with(".attest")) {
                // Collect JSON files inside .attest/ directories.
                if let Ok(inner) = std::fs::read_dir(&path) {
                    for inner_entry in inner.flatten() {
                        let inner_path = inner_entry.path();
                        if inner_path.extension().map_or(false, |e| e == "json") {
                            result.push(inner_path);
                        }
                    }
                }
            } else if path.file_name().map_or(true, |n| {
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
        _item_source_file: &std::path::Path,
        _item_path: &str,
    ) -> Vec<String> {
        // CLI-SF-2: git trailer resolution is not yet wired in the check command.
        // Predicates using `Leaf::SignedTrailer` will always fail here — not a
        // predicate error, just "no trailers found." The audit command has the same
        // gap. v0.2 wires real `git interpret-trailers` invocation.
        Vec::new()
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
            "note: --fingerprint not supplied; using sidecar's stored current_fingerprint.\n\
             Stale-signer detection requires the real current fingerprint.\n\
             Run `cargo antigen scan --format json` to get the item fingerprint."
        );
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
            println!("    tier = {:?}, hint = {:?}", a.witness_tier, a.audit_hint,);
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
