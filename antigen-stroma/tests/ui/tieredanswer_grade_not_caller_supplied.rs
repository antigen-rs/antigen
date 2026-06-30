// ATK-FRAME-TIER-CAP-CONSTRUCTION (compile-state half) — `TieredAnswer`'s grade is DERIVED from the
// source tier, NEVER caller-supplied. A struct literal that sets the grade directly MUST NOT compile.
//
// WHY: the false-quiet cardinal sin is a syntactic source carrying a presents-grade answer. The
// builder closed this by deriving grade = tier.detection_ceiling() inside the private constructor, so
// `{ tier: Syntactic, grade: Presents }` is unconstructible. This fixture guards that the FIELDS stay
// private — a future refactor that exposed `grade` as a public field, or added a caller-supplied-grade
// constructor, would silently reopen the hole. The private `value`/`tier`/`grade` fields forbid the
// literal.

use antigen_stroma::read::answer::TieredAnswer;
use antigen_stroma::read::tier::{DetectionGrade, ResolutionTier};

fn main() {
    // Forge a tier-dishonest answer: a Syntactic source claiming Presents grade. The private fields
    // make this struct literal a privacy error — it MUST NOT compile.
    let _dishonest: TieredAnswer<&str> = TieredAnswer {
        value: "x",
        tier: ResolutionTier::Syntactic,
        grade: DetectionGrade::Presents, // a syntactic source can NOT carry presents-grade
    };
}
