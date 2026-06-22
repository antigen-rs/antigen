// Fixture: #[diagnostic] with class-collapsed modalities.
//
// Purpose: verify the CLASS-COLLAPSE correctness invariant.
//
// The most critical correctness test for convergent-evidence:
//   min_independent = distinct WitnessClass CATEGORIES, not count of witnesses.
//
// Two witnesses of WitnessClass::StaticAnalysis = 1 distinct class, not 2.
// If min_independent = 2 and only 1 distinct class is present, audit MUST emit
// `diagnostic-modalities-class-collapsed` — the independence claim is false.
//
// This is the failure mode identified in ADR-024 (C1). An
// implementation that counts witnesses (not classes) would incorrectly pass
// `[StaticAnalysis, StaticAnalysis]` with min_independent = 2.
//
// Failing-as-passing intent: audit MUST detect class collapse here.
// If this fixture does NOT trigger class-collapse, the audit has a false-negative.
//
// ADR-024 §WitnessClass — min_independent counts distinct CLASSES.

#[antigen(
    name = "OverfitModel",
    fingerprint = "item: fn"
)]
pub struct OverfitModel;

/// A function with two static-analysis witnesses but claiming 2-class independence.
/// clippy and rustc are BOTH in WitnessClass::StaticAnalysis — they are the
/// same class. Claiming min_independent = 2 here is false; the audit must say so.
#[presents(OverfitModel)]
#[diagnostic(
    modalities = [WitnessClass::StaticAnalysis, WitnessClass::StaticAnalysis],
    min_independent = 2
)]
pub fn train_model(data: &[f64]) -> f64 {
    data.iter().sum::<f64>() / data.len() as f64
}
