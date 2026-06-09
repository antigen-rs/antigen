//! Scan a crate that declares zero antigens — the bundled catalog auto-injects
//! a default repertoire so the scan still finds real matches.
//!
//! A fresh crate has no `#[antigen]` declarations of its own. Without a
//! repertoire, a scan has nothing to match against and reports an empty result —
//! indistinguishable from "this code is clean." The bundled stdlib catalog closes
//! that gap: antigen ships a default set of flagship failure-class fingerprints,
//! and a scan with the catalog injected checks a zero-declaration crate against
//! them. The crate's author writes no antigens and still gets findings.
//!
//! This example writes a small zero-declaration crate to a temp directory, scans
//! it with the bundled catalog, and prints each match with its claim-scope.
//!
//! On the command line the same scan is:
//!
//! ```sh
//! cargo antigen scan --root <crate> --bundled-catalog
//! ```
//!
//! An explicit `--bundled-catalog` always injects the catalog — it *augments*
//! whatever the crate already declares. A plain scan with no flag auto-detects
//! instead: it injects the catalog only when the crate has no in-tree antigens of
//! its own, closing the zero-hits cliff for a newcomer.
//!
//! Run this example:
//!
//! ```sh
//! cargo run --example bundled_catalog_scan --package antigen
//! ```
//!
//! ## What a fingerprint match claims, and what it doesn't
//!
//! Each finding is a **fingerprint match**: the site's structure matches a known
//! failure-class fingerprint, at the tier the catalog authored. It is a syntactic
//! fact to inspect — not an audited verdict that the site is broken, and not a
//! claim that any defense was checked. A match says "this shape is worth a look,"
//! and points to the markers (`#[presents]` / `#[defended_by]` /
//! `#[antigen_tolerance]`) that record what you decide. Expect candidates, not a
//! failure list; the witness layer is what refines a match into a defended,
//! tolerated, or genuinely-undefended site.

use std::path::Path;

use antigen::finding::FindingBody;
use antigen::scan::{bundled_catalog_findings, scan_workspace_bundled_catalog};

/// A zero-declaration toy crate: it has no `#[antigen]` markers of its own, but it
/// reaches two known failure-class shapes — an unchecked slice index and a panic
/// in a `Drop` impl. The bundled catalog is what lets the scan see them.
const ZERO_DECL_CRATE: &str = r"
    pub fn first(slice: &[u32]) -> u32 {
        // get-unchecked-without-proof: an unchecked index with no safety argument.
        unsafe { *slice.get_unchecked(0) }
    }

    pub struct Guard;
    impl Drop for Guard {
        fn drop(&mut self) {
            // panic-in-drop: an unwrap in teardown can panic while unwinding.
            let _ = cleanup().unwrap();
        }
    }

    fn cleanup() -> Result<(), ()> {
        Ok(())
    }
";

fn main() {
    // Write the zero-declaration crate to a temp directory the scan can walk.
    let dir = tempfile::tempdir().expect("create a temp dir for the toy crate");
    let src = dir.path().join("src");
    std::fs::create_dir_all(&src).expect("create src/");
    std::fs::write(src.join("lib.rs"), ZERO_DECL_CRATE).expect("write the toy crate");

    // Scan with the bundled catalog, auto-detecting (inject only because this
    // crate declares zero antigens of its own).
    let report = scan_workspace_bundled_catalog(dir.path(), None, /* auto_detect */ true)
        .expect("the scan walks the temp crate");

    println!("== bundled-catalog scan of a zero-declaration crate ==\n");
    println!(
        "in-tree antigen declarations: {}   (the crate authored none)",
        report.antigens.len()
    );

    // Project the catalog matches into claim-scoped findings.
    let findings = bundled_catalog_findings(&report);
    println!(
        "fingerprint matches from the bundled catalog: {}\n",
        findings.len()
    );

    for f in &findings {
        if let FindingBody::FingerprintMatch { class, tier } = &f.body {
            let file = Path::new(&f.file)
                .file_name()
                .map_or_else(|| f.file.clone(), |n| n.to_string_lossy().into_owned());
            println!("  {class}  ({tier:?})");
            println!("    at {file}:{}", f.line);
            println!("    provenance: {:?}", f.class_provenance);
            println!("    a fingerprint match to inspect, not an audited verdict.\n");
        }
    }

    println!("next: mark a site you accept with one of");
    println!("  #[presents(<class>)] + #[defended_by(<test>)]   (record a defense)");
    println!("  #[antigen_tolerance(<class>, rationale = \"...\")]   (accept it)");
}
