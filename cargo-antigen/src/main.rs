//! Cargo subcommand for antigen.
//!
//! Provides `cargo antigen scan` (and future `new`, `vaccinate`, `audit`)
//! subcommands for working with antigen declarations in a Rust workspace.
//!
//! ## Status (v0.0.1)
//!
//! Initial functional release of `scan`. Other subcommands are stubs printing
//! design-phase notices.

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

use antigen::scan;

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
    New {
        /// kebab-case name for the new antigen
        name: String,
    },
    /// Apply known immunity pattern across a structural family (design phase).
    Vaccinate {
        /// Antigen type to apply
        antigen: String,
        /// Pattern (glob or path) describing target sites
        pattern: String,
    },
    /// Comprehensive immunity coverage report (design phase).
    Audit(AuditArgs),
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
}

#[derive(Debug, Parser)]
struct AuditArgs {
    /// Workspace root (default: current directory)
    #[arg(long, default_value = ".")]
    root: PathBuf,
    /// Output format: human or json
    #[arg(long, default_value = "human")]
    format: OutputFormat,
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
    }
}

fn run_scan(args: ScanArgs) -> ExitCode {
    eprintln!("Scanning workspace: {}", args.root.display());

    let report = match scan::scan_workspace(&args.root, None) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("error: scan failed: {e}");
            return ExitCode::from(2);
        }
    };

    let unaddressed = report.unaddressed_presentations();

    match args.format {
        OutputFormat::Human => {
            print_human_report(&report, &unaddressed);
        }
        OutputFormat::Json => match serde_json::to_string_pretty(&JsonReport {
            report: &report,
            unaddressed: &unaddressed,
        }) {
            Ok(s) => println!("{s}"),
            Err(e) => {
                eprintln!("error: failed to serialize report: {e}");
                return ExitCode::from(2);
            }
        },
    }

    if args.strict && !unaddressed.is_empty() {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn print_human_report(report: &scan::ScanReport, unaddressed: &[scan::UnaddressedPresentation]) {
    println!();
    println!(
        "Scanned {} files, found {} antigen-related declarations:",
        report.files_scanned,
        report.total_declarations()
    );
    println!("  - {} antigen declarations", report.antigens.len());
    println!("  - {} presentations", report.presentations.len());
    println!("  - {} immunity claims", report.immunities.len());
    if !report.parse_failures.is_empty() {
        println!(
            "  - {} files failed to parse (see --format json for details)",
            report.parse_failures.len()
        );
    }
    println!();

    if unaddressed.is_empty() {
        println!("✓ No unaddressed presentations.");
        if !report.presentations.is_empty() {
            println!(
                "  All {} presentations have nearby immunity declarations.",
                report.presentations.len()
            );
        }
    } else {
        println!("⚠ {} unaddressed presentation(s):", unaddressed.len());
        println!();
        for u in unaddressed {
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
                    "    note: antigen `{}` was not declared in the scanned workspace; \
                     may be imported from an external crate or undeclared",
                    p.antigen_type
                );
            }
        }
        println!();
        println!("To address: add #[immune({}, witness = ...)] on the same item, OR mark with #[antigen_tolerance(...)]", unaddressed[0].presentation.antigen_type);
    }
}

#[derive(serde::Serialize)]
struct JsonReport<'a> {
    report: &'a scan::ScanReport,
    unaddressed: &'a [scan::UnaddressedPresentation],
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
    ExitCode::SUCCESS
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
    ExitCode::SUCCESS
}

fn run_audit(args: AuditArgs) -> ExitCode {
    eprintln!(
        "cargo-antigen audit (design phase) — using `scan` for now.\n\
         Comprehensive audit (witness validation, cross-crate inheritance walks,\n\
         coverage trend reporting) is under design. The eventual command will\n\
         supplement scan with:\n\
           - Witness function existence + freshness checks\n\
           - #[descended_from] propagation walks\n\
           - antigen-stdlib version drift detection\n\
           - Cross-crate antigen consumption reporting\n\
         \n\
         Falling through to `scan` semantics for the current release.\n"
    );
    run_scan(ScanArgs {
        root: args.root,
        format: args.format,
        strict: false,
    })
}
