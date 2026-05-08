// ATK-A2-002 fixture: two antigen attribute declarations in the same file.
// Demonstrates the line_of_attr first-occurrence bug. Both attribute structs
// below will be reported at the same line number by the current implementation.
// The test asserts they must report different lines.
// (Note: the comment above intentionally avoids the literal trigger string.)

#[antigen(
    name = "panicking-in-drop",
    fingerprint = "impl Drop with panic"
)]
pub struct PanickingInDropAntigen;

#[antigen(
    name = "frame-translation",
    fingerprint = "class enum with meet method"
)]
pub struct FrameTranslationAntigen;
