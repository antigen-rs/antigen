// ATK-CE-1: Class-collapse in #[diagnostic].
//
// Per ADR-024 C1: min_independent = distinct WitnessClass CATEGORIES, not count.
// Two witnesses of the SAME class count as 1 independent class, not 2.
//
// Attack: provide [StaticAnalysis, StaticAnalysis] with min_independent = 2.
// This appears to satisfy count ≥ 2, but the CLASS COUNT is only 1.
// Audit MUST emit `diagnostic-modalities-class-collapsed`.
//
// This is the key correctness invariant for the diagnostic primitive.
// If the implementation counts WITNESSES instead of CLASSES, this passes
// silently — the canonical silent failure for CE-1.

use antigen::diagnostic;
use antigen::WitnessClass;

/// ATK-CE-1: Same class used twice.
/// min_independent = 2 BUT only 1 distinct WitnessClass.
/// Audit MUST NOT treat this as satisfying the independence requirement.
/// Expected hint: diagnostic-modalities-class-collapsed.
#[diagnostic(
    modalities = [WitnessClass::StaticAnalysis, WitnessClass::StaticAnalysis],
    min_independent = 2
)]
pub fn atk_ce1_same_class_twice() {}

/// ATK-CE-1-B: Three witnesses, all same class.
/// min_independent = 3 BUT distinct classes = 1.
/// Should also emit diagnostic-modalities-class-collapsed.
#[diagnostic(
    modalities = [
        WitnessClass::PropertyTest,
        WitnessClass::PropertyTest,
        WitnessClass::PropertyTest,
    ],
    min_independent = 3
)]
pub fn atk_ce1_three_same_class() {}

/// ATK-CE-1-C: The CORRECT case — different classes.
/// Should NOT emit class-collapsed hint.
#[diagnostic(
    modalities = [WitnessClass::StaticAnalysis, WitnessClass::PropertyTest],
    min_independent = 2
)]
pub fn atk_ce1_correct_two_classes() {}
