// Fixture: #[diagnostic] with genuinely independent WitnessClass modalities.
//
// Purpose: verify that TWO GENUINELY DIFFERENT WitnessClass variants do NOT
// trigger the `diagnostic-modalities-class-collapsed` hint.
//
// This is the NEGATIVE case for class-collapse detection. When witnesses come
// from truly independent classes (e.g., StaticAnalysis + PropertyTest), the
// audit must NOT flag class-collapse — they are independent by class definition.
//
// min_independent = 2 with [StaticAnalysis, PropertyTest] MUST PASS:
//   - StaticAnalysis (clippy/rustc analysis) is one class
//   - PropertyTest (proptest/quickcheck) is a different class
//   - Two distinct classes = min_independent=2 satisfied
//
// If this fixture DOES trigger class-collapse, audit has a false-positive.
//
// ADR-024 §WitnessClass — 6 distinct classes, independence is class-level.

#[antigen(
    name = "OverfitModel",
    fingerprint = "item: fn"
)]
pub struct OverfitModel;

/// A function with witnesses from two genuinely independent WitnessClass categories.
/// StaticAnalysis (clippy) and PropertyTest (proptest) are different classes —
/// this claim of 2-class independence is TRUE and must not be flagged.
#[presents(OverfitModel)]
#[diagnostic(
    modalities = [WitnessClass::StaticAnalysis, WitnessClass::PropertyTest],
    min_independent = 2
)]
pub fn train_model(data: &[f64]) -> f64 {
    data.iter().sum::<f64>() / data.len() as f64
}
