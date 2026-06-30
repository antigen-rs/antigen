# `tests/ui_engine/` — ENGINE-epoch compile-fail fixtures (born-red PLACEHOLDERS)

These trybuild fixtures are NOT wired by the frame-epoch `compile_fail.rs` harness. They reference
engine-fill types (`CondensedGraph`, the `Semiring` trait, `StromaGraph`) that do not exist in the
frame epoch. They are the born-red obligations **recorded forward**, not built — the way ADR-067 names
deferred defenses rather than minting dangling ones.

When the ENGINE wave lands the `CondensedGraph` type-state:
1. Move the fixture from `tests/ui_engine/` to `tests/ui/`.
2. Wire it in the (engine-epoch) `compile_fail.rs` harness.
3. Confirm it was RED (the raw-graph counting path compiled) against a pre-type-state build, then GREEN
   (it now fails to compile) against the type-state.
4. Bless the `.stderr` and de-placeholder it in ATK-REGISTRY.md.

Current placeholder:
- `nonidem_semiring_without_condensation.rs` — ATK-FRAME-NONIDEM (ADR-068 clause-3 / ADR-070 §4.6 A9).
