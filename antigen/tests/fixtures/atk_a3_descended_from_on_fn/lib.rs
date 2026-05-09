// A3 hardening fixture: #[descended_from] on a function (non-type) item.
//
// Per ADR-013, #[descended_from] is meaningful only on antigen-type
// declarations (unit struct + class enum). Other placements must surface
// as parse_failures rather than be silently dropped — fractal-pattern
// guard: silent no-op on a "wrong" item kind would hide user error.

#[antigen(name = "real-antigen", fingerprint = "item: struct")]
pub struct RealAntigen;

#[descended_from(RealAntigen)]
pub fn this_is_not_a_type_declaration() {}
