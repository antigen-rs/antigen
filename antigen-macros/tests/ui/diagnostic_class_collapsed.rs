//! CE-1 fixture: `#[diagnostic]` with duplicate WitnessClass and min_independent=2.
//!
//! Per ADR-024 adversarial C1: min_independent counts distinct CLASSES, not
//! raw witness count. [StaticAnalysis, StaticAnalysis] has 1 distinct class.
//! Requesting min_independent=2 must fail with a COMPILE ERROR.
//!
//! This fixture locks the W4 error message for CE-1 class-collapse enforcement.

use antigen_macros::diagnostic;

/// ATK-CE-1: Two witnesses of the same class claiming independence.
/// clippy and rustc are BOTH StaticAnalysis — duplicate, not independent.
#[diagnostic(
    modalities = [WitnessClass::StaticAnalysis, WitnessClass::StaticAnalysis],
    min_independent = 2
)]
pub fn train_model(data: &[f64]) -> f64 {
    data.iter().sum::<f64>() / data.len() as f64
}

fn main() {}
