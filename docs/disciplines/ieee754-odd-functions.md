---
version: 1.0
title: IEEE-754 sign preservation in odd functions
steward: math-researcher
ratified: 2026-05-26
---

# IEEE-754 sign preservation in odd functions

This discipline governs the implementation of mathematically **odd** functions
(`f(-x) = -f(x)`) over IEEE-754 floating point, where the identity must hold
*including at the signed zeros* `+0.0` and `-0.0`.

It is the ratified reference for the `SignedZeroDiscipline` antigen and is the
`ratified_doc` substrate-witness target for
[`antigen/examples/substrate_witness.rs`](../../antigen/examples/substrate_witness.rs).

## The invariant

For an odd function `f` implemented over `f64`:

- `f(-0.0)` MUST return `-0.0` (a negative zero), not `+0.0`.
- `f(+0.0)` MUST return `+0.0`.
- `f(NaN)` returns `NaN` (sign of NaN is unspecified and not part of this
  discipline).

IEEE-754 distinguishes `+0.0` from `-0.0` (they compare `==` but have different
sign bits, observable via `f64::to_bits` / `copysign` / `1.0 / x` → `±∞`). A
naive odd-function body computed purely arithmetically frequently *loses* the
sign at zero, because the algebraic expression evaluates to a positive zero even
when the input was negative zero.

## The failure mode (what `SignedZeroDiscipline` names)

Take `sinh`. The natural body

```rust
0.5 * (x.exp() - (-x).exp())
```

evaluates, at `x = -0.0`, to `0.5 * (1.0 - 1.0) = 0.5 * 0.0 = +0.0` — the
negative sign is silently dropped. `sinh(-0.0)` should be `-0.0`. The arithmetic
is *numerically* correct everywhere except at the signed zero, where it violates
the oddness identity in its IEEE-754 form.

## The remedy

Short-circuit the signed zero before the arithmetic runs, returning the input
unchanged so its sign bit is preserved:

```rust
pub fn signed_zero_preserving_sinh(x: f64) -> f64 {
    if x.is_nan() {
        return f64::NAN;
    }
    if x == 0.0 {
        // Matches BOTH +0.0 and -0.0; returning `x` preserves the sign bit.
        return x;
    }
    0.5 * (x.exp() - (-x).exp())
}
```

The `if x == 0.0 { return x; }` line is load-bearing: `==` matches both zeros,
and returning the original `x` carries its sign bit through untouched.

## Verifying compliance

A compliant implementation satisfies, for every odd function it provides:

- `f(-0.0).to_bits() == (-0.0_f64).to_bits()` (the sign bit survives), and
- `f(+0.0).to_bits() == (0.0_f64).to_bits()`.

A reviewer attesting `SignedZeroDiscipline` against a function is asserting they
have checked the implementation preserves the signed zero per the invariant
above — not merely that it is numerically accurate for non-zero inputs.

## References

- IEEE 754-2019 §5.5.1 (sign-symmetric operations and signed zero).
- Kahan, "Branch Cuts for Complex Elementary Functions" (1987), on the
  semantic significance of signed zero.
