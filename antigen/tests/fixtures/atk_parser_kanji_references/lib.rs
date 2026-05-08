// ATK-PARSER-4: Kanji / non-ASCII characters in the references array.
// The name and fingerprint must not be corrupted by non-ASCII references.

#[antigen(
    name = "test-kanji-refs",
    fingerprint = "fp",
    references = ["参考文献-001", "DEC-042", "論文-2026", "ADR-010"]
)]
pub struct TestKanjiRefs;
