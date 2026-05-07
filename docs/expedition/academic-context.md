# Antigen — Academic Context

> Survey of academic prior art, positioning antigen relative to existing work in
> programming-language type systems, contracts, effects, security types, separation
> logic, lightweight specification, Rust verification, bug taxonomies, and
> recent venue (PLDI / OOPSLA / ICFP / ECOOP) literature.
>
> Source documents: `design-intent.md`, `api-shape.md`, `revolutionary-and-not.md`.

---

## Framing — where antigen sits in the academic landscape

Antigen makes four core claims of novelty (per `revolutionary-and-not.md`):

1. **Failure-class memory as a first-class structural artifact** (not documentation).
2. **Structural inheritance of immunity through composition** (`#[descended_from]`).
3. **A composition operation on failure-class declarations** (a refinement-lattice
   over named failure classes themselves, not over values).
4. **Vaccination as a development action** — a bulk operation that pushes a known
   immunity pattern across a structural family.

The research neighborhood is **specification-and-verification systems**, but antigen
sits at an unusual altitude within that neighborhood. It is **not** a refinement-type
system, **not** a contract language, **not** an effect system, **not** a separation-logic
framework. It is a **meta-layer** that names and composes the artifacts those systems
produce — closer in spirit to "the way deprecated annotations are composed across
the type graph" or "the way clippy lints are organized into structured taxonomies"
than to any individual verification approach.

Before reading the per-area sections, the headline conclusion: **antigen's individual
mechanisms are well-trodden — annotations, witnesses, structural pattern-matching,
delegation to existing checkers. Its novelty lives in three places: (a) treating
failure-class names themselves as composable first-class artifacts with inheritance,
(b) the vaccination operation as a developer-facing bulk transform on the
failure-class graph, and (c) the synthesis target — Rust ecosystem, AI-coding
context, and explicit cross-tool delegation rather than reinvention.**

If that synthesis is the contribution, then the honest framing for any future paper
is "the missing connective tissue between Rust's lint, test, and verification
ecosystems" rather than "a new kind of type system."

---

## 1. Refinement Types

### Nearest neighbors

- **Liquid Haskell** (Vazou, Rondon, Seidel, Jhala, et al.). Refinement types over
  Haskell with SMT-based discharge. Logical predicates are attached to types
  (`{v:Int | v > 0}`), and the typechecker discharges proof obligations to an SMT
  solver. Long line of work since ~2008; production-quality since ~2017.
- **F\*** (Microsoft Research, INRIA). Dependent + refinement types, monadic effects,
  and an extraction pipeline to OCaml/F#/C/WASM. Used in Project Everest (verified
  TLS / HTTP/3 / Bitcoin). The most ambitious working refinement-types system.
- **Stainless** (EPFL, Kuncak group). Refinement types and contracts for Scala,
  inductive function definitions, equational rewriting, SMT discharge.
- **Flux / LiquidRust** (Lehmann, Geller, Vazou, Jhala, et al., recent PLDI/POPL).
  Refinement types **for Rust**. Builds on Liquid types but engages the borrow
  checker — refinement predicates can talk about ownership, mutation, and lifetimes.
  The most direct neighbor in the Rust verification space.

### Distinguishing features

- Refinement types **prove value-level properties** (`x > 0`, `length(xs) == n`).
  Antigen does not prove value-level properties. It tracks **named failure-classes**
  at the level of artifacts (functions, types, traits) and asks whether each
  presentation has an associated witness.
- Refinement types are **automatic given annotations** — the SMT solver discharges
  obligations once predicates are written. Antigen's witnesses are **arbitrary** —
  a `proptest`, a `#[test]`, a kani harness, a phantom-type construction, even a
  clippy lint. Antigen does not have a single discharge engine.
- Refinement types do not have a **failure-class taxonomy**. They have a
  predicate language (the SMT logic). Antigen's primary artifact is not a predicate
  but a **named class with a fingerprint**.

### Relationship

**Complementary, not competitive.** Flux/LiquidRust is the natural witness backend
for any antigen whose failure-class is expressible as a refinement predicate. The
intended composition is `#[immune(X, witness = some_flux_proof)]`. Antigen
delegates; Flux discharges.

### Citation candidates

- Vazou, N., Seidel, E. L., Jhala, R., Vytiniotis, D., & Peyton-Jones, S. (2014).
  "Refinement Types for Haskell." *ICFP 2014*.
- Rondon, P. M., Kawaguchi, M., & Jhala, R. (2008). "Liquid Types." *PLDI 2008*.
- Swamy, N., Hriţcu, C., Keller, C., Rastogi, A., Delignat-Lavaud, A., Forest, S.,
  et al. (2016). "Dependent Types and Multi-Monadic Effects in F\*." *POPL 2016*.
- Lehmann, N., Geller, A. T., Vazou, N., & Jhala, R. (2023). "Flux: Liquid Types
  for Rust." *PLDI 2023*.

---

## 2. Design by Contract

### Nearest neighbors

- **Eiffel** (Bertrand Meyer, *Object-Oriented Software Construction*, 1988/1997).
  The original DbC system: preconditions, postconditions, class invariants,
  inheritance of contracts. The vocabulary that antigen's "presentation /
  immunity / witness" structurally rhymes with.
- **Spec#** (Microsoft Research; Barnett, Leino, Schulte). Extends C# with
  preconditions, postconditions, non-null types, and Boogie-based discharge.
  Predecessor to Code Contracts and to Dafny.
- **JML — Java Modeling Language** (Leavens et al.). Formal specifications in
  Java comments, multiple discharge backends (ESC/Java, OpenJML, KeY).
- **Code Contracts for .NET** (Fähndrich, Logozzo). Library-level DbC for C#.
  Discontinued but historically important.
- **Cofoja** for Java; **PyContracts**; **dbc-rust** crates — cross-language
  ecosystem of contract libraries.

### Distinguishing features

- DbC contracts are **per-method invariants** — preconditions, postconditions,
  invariants over values. Antigen's `#[presents]` and `#[immune]` are not
  invariants on values; they are **markers about which named failure-classes
  apply to this site, and whether evidence for immunity exists**.
- DbC contracts in Eiffel propagate via inheritance with **subtype variance
  rules** (preconditions weaken in subclasses, postconditions strengthen).
  Antigen's `#[descended_from]` propagation is **markerwise** — presentations
  pass through, immunities require witness re-validation.
- DbC has rich runtime semantics (assertion checking modes, contract violations
  as exceptions). Antigen has **no runtime semantics by default**. Witnesses can
  be runtime tests, but the antigen markers themselves are static-only.

### Relationship

**Conceptual ancestor.** Eiffel's covariance/contravariance rules for inherited
contracts are the closest existing analog to antigen's `#[descended_from]`
inheritance semantics. Future antigen design should explicitly engage with that
literature. The phrase "structural inheritance of immunity" should be honest
about the lineage.

### Citation candidates

- Meyer, B. (1992). "Applying 'Design by Contract'." *IEEE Computer*, 25(10).
- Meyer, B. (1997). *Object-Oriented Software Construction* (2nd ed.).
- Leavens, G. T., & Cheon, Y. (2006). "Design by Contract with JML."
- Barnett, M., Fähndrich, M., Leino, K. R. M., Müller, P., Schulte, W., & Venter,
  H. (2011). "Specification and Verification: The Spec# Experience." *CACM* 54(6).
- Fähndrich, M., & Logozzo, F. (2010). "Static Contract Checking with Abstract
  Interpretation." *FoVeOOS 2010*.

---

## 3. Named Effect Type Systems

### Nearest neighbors

- **Koka** (Daan Leijen, Microsoft Research). Row-typed effect system with
  algebraic effects and effect handlers. `int -> <exn,div> bool` — effects are
  named, declared in the type, and tracked by the compiler.
- **OCaml 5 effect handlers** (Sivaramakrishnan, Dolan, et al.). Production
  multicore OCaml ships effect handlers as a runtime mechanism; the typed
  surface remains research (e.g., Eff-tracking dialects).
- **Eff** (Bauer, Pretnar). Original algebraic-effects-and-handlers research
  language.
- **Frank** (McBride, Convent, Lindley). Effects as "shonky" function calls;
  ambient handlers.
- **Capability-based effects** in Scala 3 / Caprese (Odersky group).

### Distinguishing features

- Effect systems track **what a function may do at runtime** — throw, allocate,
  perform IO, yield, transact. Antigen tracks **what failure-class a function
  is structurally vulnerable to**. These are different categories: "may panic"
  is an effect; "may suffer frame-translation between meet=min and meet=max"
  is a failure-class.
- Effect systems **automatically infer** effects from function bodies (Koka
  infers `<exn>` from a `throw`). Antigen does **not infer**; presentations
  are **manually declared** by the developer who has named the failure-class.
- Effect systems' composition rule is **effect union** (a function calling two
  effectful functions has the union of their effects). Antigen's composition
  is **selective propagation through `#[descended_from]`** — derivation
  inherits, but caller-callee does not by default.

### Relationship

**Cousins, not competitors.** An effect system answers "what does this code do?";
antigen answers "what failure-class memory applies to this code?". Both are
named, structural, type-system-anchored. The shape is similar; the semantics
differ. A future paper on antigen should cite Koka as the reference effect
system for "named, declared, propagated through types."

### Citation candidates

- Leijen, D. (2014). "Koka: Programming with Row-Polymorphic Effect Types."
  *MSFP 2014*.
- Leijen, D. (2017). "Type Directed Compilation of Row-Typed Algebraic Effects."
  *POPL 2017*.
- Bauer, A., & Pretnar, M. (2015). "Programming with Algebraic Effects and
  Handlers." *Journal of Logical and Algebraic Methods in Programming*.
- Sivaramakrishnan, K. C., et al. (2021). "Retrofitting Parallelism onto OCaml."
  *ICFP 2020*.
- Convent, L., Lindley, S., McBride, C., & McLaughlin, C. (2020). "Doo Bee Doo
  Bee Doo." *JFP 30*.

---

## 4. Security-Typed Languages

### Nearest neighbors

- **FlowCaml** (Pottier, Simonet). Information-flow types for OCaml. Each value
  carries a security level; the type system prevents flows from high to low.
- **Jif — Java Information Flow** (Myers, Liskov). Decentralized label model;
  labeled types in Java.
- **Ur/Web** (Adam Chlipala). Strong information-flow guarantees for web apps,
  baked into a dependently-typed surface language.
- **Paragon** (Broberg, van Delft, Sands). Java extension with paralocks for
  flow-sensitive policies.

### Distinguishing features

- Security-typed languages track **information flow from sources to sinks**.
  Antigen does not track information flow.
- Security-typed languages have an **automatic discharge** (the type system
  proves no leaks). Antigen has **no automatic discharge** — the witness is
  arbitrary evidence.
- Security types use a **lattice of security levels** (low ⊑ high). Antigen
  has a "family" hierarchy among failure-classes that loosely lattice-shapes,
  but it is **not a security lattice** — there's no well-defined "join" that
  enforces a non-interference property.

### Relationship

**Distant cousin.** The shared shape is "named structural property attached to
program artifacts, propagated through composition." But the semantics
(information-flow non-interference vs failure-class memory) differ enough that
direct technical borrowing is unlikely. Worth citing for the structural rhyme.

### Citation candidates

- Pottier, F., & Simonet, V. (2003). "Information Flow Inference for ML." *ACM TOPLAS*.
- Myers, A. C., & Liskov, B. (2000). "Protecting Privacy Using the Decentralized
  Label Model." *ACM TOSEM*.
- Sabelfeld, A., & Myers, A. C. (2003). "Language-Based Information-Flow
  Security." *IEEE JSAC*. (Survey — important framing.)

---

## 5. Hoare Logic / Separation Logic

### Nearest neighbors

- **Classical Hoare logic** (Hoare 1969). `{P} C {Q}` triples.
- **Separation logic** (Reynolds 2002, O'Hearn, Yang). Heap reasoning with the
  separating conjunction `*`.
- **Concurrent separation logic** (Brookes, O'Hearn). Reasoning about shared
  mutable state under concurrency.
- **Iris** (Jung, Krebbers, Birkedal, Dreyer, et al.). A general-purpose
  higher-order concurrent separation logic, mechanized in Coq. The semantic
  foundation of RustBelt (see §9).

### Distinguishing features

- Separation logic reasons about **heap structure and resource ownership**
  through proof. Antigen does no such reasoning.
- Separation logic is a **proof system**; antigen is a **declaration system**.
  Antigen's witnesses can be Iris/RustBelt-style proofs (delegated), but
  antigen itself doesn't prove anything.
- Separation logic has a **formal soundness theorem**. Antigen has no
  soundness theorem because the witness mechanism is open — a wrong witness
  produces a wrong claim, and antigen relies on the witness being valid by
  whatever standard the witness type implies.

### Relationship

**Foundational underlay for some witnesses.** A `#[immune(X, witness = iris_proof)]`
borrows the soundness of the proof. Antigen orchestrates; Iris/separation logic
is one possible witness shape.

### Citation candidates

- Hoare, C. A. R. (1969). "An Axiomatic Basis for Computer Programming." *CACM*.
- Reynolds, J. C. (2002). "Separation Logic: A Logic for Shared Mutable Data
  Structures." *LICS 2002*.
- O'Hearn, P. W. (2007). "Resources, Concurrency, and Local Reasoning." *TCS*.
- Jung, R., Krebbers, R., Jourdan, J.-H., Bizjak, A., Birkedal, L., & Dreyer, D.
  (2018). "Iris from the Ground Up: A Modular Foundation for Higher-Order
  Concurrent Separation Logic." *JFP 28*.

---

## 6. Lightweight Specification Systems

### Nearest neighbors

- **Dafny** (Microsoft Research; Leino). Imperative-with-specifications language;
  `requires`, `ensures`, `invariant`, `decreases`. SMT (Z3) discharge via Boogie.
- **Why3** (INRIA). Multi-prover backend (Z3, Alt-Ergo, CVC4, Coq, Isabelle);
  WhyML language for spec + extraction.
- **Frama-C / ACSL** (CEA). Specification language for C, multi-plugin analyzers
  (WP, EVA, value analysis).
- **VeriFast** (Jacobs et al.). Separation-logic-based modular verifier for
  C and Java with explicit ghost code.
- **OpenJML** — direct JML successor, multi-prover.

### Distinguishing features

- These systems **prove specifications** through SMT or interactive proof.
  Antigen does not prove specifications.
- These systems are **single-tool, single-prover** in their default form.
  Antigen explicitly **delegates** to whatever discharge mechanism is appropriate:
  proptest for empirical evidence, kani for bounded model checking, prusti for
  refinement-style proofs, clippy for syntactic patterns, phantom types for
  compile-time impossibility.
- These systems have **a fixed specification language** (FOL, separation logic,
  etc.). Antigen has **no specification language** — it has names and witness
  pointers.

### Relationship

**Witness backends.** Each of Dafny / Why3 / Frama-C / VeriFast could in
principle host an antigen witness. The composition is "antigen names the
failure-class, the witness is a Dafny proof / Frama-C contract / Why3 derivation."

### Citation candidates

- Leino, K. R. M. (2010). "Dafny: An Automatic Program Verifier for Functional
  Correctness." *LPAR-16*.
- Filliâtre, J.-C., & Paskevich, A. (2013). "Why3 — Where Programs Meet Provers."
  *ESOP 2013*.
- Cuoq, P., Kirchner, F., Kosmatov, N., Prevosto, V., Signoles, J., & Yakobowski, B.
  (2012). "Frama-C: A Software Analysis Perspective." *SEFM 2012*.
- Jacobs, B., Smans, J., Philippaerts, P., Vogels, F., Penninckx, W., & Piessens, F.
  (2011). "VeriFast: A Powerful, Sound, Predictable, Fast Verifier for C and Java."
  *NFM 2011*.

---

## 7. Recent Venue Papers (PLDI / OOPSLA / ECOOP / ICFP)

### Type-system extensions for failure detection

- **"Detecting Anti-Patterns in Refactoring"** family (multiple ECOOP/ICSE).
  Pattern-matching for known structural anti-patterns. Closest to
  `cargo antigen scan`'s structural fingerprint matching.
- **"Pattern-Based Bug Detection"** lineage (FindBugs, SpotBugs, Infer). Pattern
  matchers for named bug classes — without the **structural inheritance** or
  **vaccination** primitives antigen claims as novel.

### Effect/contract systems for Rust specifically

- **Prusti** (Astrauskas, Müller, et al.). Refinement-style verification for Rust
  built on Viper. Reasons about ownership; specifications attached as Rust
  attributes. Possibly the most mature Rust-specific verification tool. A
  natural witness backend for antigen.
- **Creusot** (Denis, Jourdan, Marché). Foreshadow/Why3-based verifier for Rust.
  Specifications via attributes; produces Why3 proof obligations.
- **Verus** (Hance et al., MSR / CMU; OOPSLA/POPL papers 2023+). Rust extension
  with linear ghost code, SMT discharge, and Coq-like rich specifications.
- **Aeneas** (Ho, Protzenko; ICFP 2022). Translates Rust functional-fragment
  programs into pure F\* / Lean / Coq for verification. A different design
  point — translation rather than annotation.

### Lightweight verification approaches

- **Kani** (AWS / MSR). Bounded model checking for Rust via CBMC. Often
  presented at industry tracks; CAV/PLDI workshop papers.
- **MIRAI** (Facebook/Meta). Abstract interpreter for Rust MIR.
- **Stylus / cargo-mutants** mutation-testing line. Different vocabulary but
  shares antigen's "failure pattern as artifact" intuition.

### Memory of past bugs / regression antipatterns

- **"Bug-Driven Programming"** / regression-test mining literature.
- **"Learning to fix bugs from past commits"** ML/learning approaches (PLDI/ICSE).
  Probabilistic; antigen is symbolic. Different framing of "failure memory."
- **AOSD / aspect-oriented work** on cross-cutting concerns. Distant cousin —
  antigen's failure-class is a cross-cutting concern, but antigen is
  declaration-only, not weaving-based.

### Cross-project patterns

- **CWE (Common Weakness Enumeration)** community-curated taxonomy of failure
  patterns. Closest existing public artifact to a "stdlib antigens crate"
  envisioned in Phase 3 of antigen's adoption pathway.
- **OWASP Top Ten / SANS Top 25** — narrower domain, but the same pattern of
  "named, taxonomy-organized failure-class library."

### Distinguishing features (across recent venues)

The recent Rust-verification venue work (Prusti, Creusot, Verus, Aeneas, Flux,
Kani) is **specification-and-discharge** focused. None of them surface
**failure-class names as composable artifacts** — they all attach value-level
predicates to functions. None has a **vaccination operation** that bulk-applies
immunity across a structural family. None has explicit **`#[descended_from]`
inheritance** of failure-class memory through derivation.

This is the gap antigen claims to fill, **specifically as a Rust-ecosystem
connective tissue layer**.

### Citation candidates

- Astrauskas, V., Müller, P., Poli, F., & Summers, A. J. (2019). "Leveraging
  Rust Types for Modular Specification and Verification." *OOPSLA 2019*.
- Lehmann, N., Geller, A. T., Vazou, N., & Jhala, R. (2023). "Flux: Liquid Types
  for Rust." *PLDI 2023*.
- Ho, S., & Protzenko, J. (2022). "Aeneas: Rust Verification by Functional
  Translation." *ICFP 2022*.
- Lattuada, A., Hance, T., et al. (2023). "Verus: Verifying Rust Programs using
  Linear Ghost Types." *OOPSLA 2023*.
- Denis, X., Jourdan, J.-H., & Marché, C. (2022). "Creusot: A Foundry for the
  Deductive Verification of Rust Programs." *ICFM 2022*.
- VanHattum, A., Schwartz-Narbonne, D., Chong, N., & Sampson, A. (2022).
  "Verifying Dynamic Trait Objects in Rust" / Kani-related papers.

---

## 8. Taxonomies of Software Bugs

### Nearest neighbors

- **CWE — Common Weakness Enumeration** (MITRE). The canonical industry
  taxonomy of failure classes. Hundreds of named entries with structural
  fingerprints in many cases. The most direct precedent for antigen's
  "stdlib antigens" concept.
- **The Empirical Studies of Bugs literature** — long line of papers across
  ICSE/FSE/MSR studying real-world bug distributions in C, Java, JavaScript,
  Python. Examples:
  - Tan, L. et al. "Bug Characteristics in Open Source Software" (EMSE).
  - Zhong, H., & Su, Z. "An Empirical Study on Real Bug Fixes" (ICSE).
  - "Bugs in Rust" / "Empirical Study of Rust Bugs" (Qin et al., PLDI 2020) —
    direct empirical foundation for antigen-stdlib failure-class library.
- **The "Bug Taxonomies" lineage** (Beizer, Kaner, Bach). Software-testing-textbook
  taxonomies from the 1990s — wide coverage of failure modes, mostly
  process-oriented rather than type-system anchored.
- **ATC / Antipatterns literature** (Brown et al., *AntiPatterns*, 1998).
  Folkloric taxonomy.

### Distinguishing features

- CWE is a **taxonomy** — names and descriptions. It does not have
  language-level enforcement, witness mechanism, or composition primitives.
- Empirical bug studies provide **distributional evidence** — how often each
  class occurs — which is invaluable input to antigen-stdlib curation but is
  not itself a tool.
- The Rust empirical study (Qin et al. PLDI 2020) is the most directly relevant
  empirical foundation. Its findings on which Rust failure classes occur in
  practice should drive Phase 3 antigen-stdlib priorities.

### Relationship

**Empirical foundation for stdlib curation.** CWE is the structural model for
how an antigen library could grow at the ecosystem scale. Qin et al. is the
empirical priorities map for Rust-specific antigen-stdlib content.

### Citation candidates

- Qin, B., Chen, Y., Yu, Z., Song, L., & Zhang, Y. (2020). "Understanding Memory
  and Thread Safety Practices and Issues in Real-World Rust Programs." *PLDI 2020*.
- MITRE Corporation. *Common Weakness Enumeration (CWE) Specification*.
  https://cwe.mitre.org/
- Tan, L., Liu, C., Li, Z., Wang, X., Zhou, Y., & Zhai, C. (2014). "Bug
  Characteristics in Open Source Software." *Empirical Software Engineering*.
- Brown, W. J., Malveau, R. C., McCormick, H. W., & Mowbray, T. J. (1998).
  *AntiPatterns*. Wiley.

---

## 9. Rust-Specific Verification Work

### Nearest neighbors

- **RustBelt** (Jung, Jourdan, Krebbers, Dreyer; POPL 2018). Semantic model of
  Rust's type system in Iris. Proves soundness of Rust's ownership/borrow
  system, including unsafe primitives. Foundational.
- **Stacked Borrows** (Jung, Dang, Kang, Dreyer; POPL 2020). Operational model
  of Rust aliasing — formalizes what unsafe code may do.
- **Tree Borrows** (Villani, Jung; 2023+). Successor model to Stacked Borrows
  with finer-grained analysis.
- **Aeneas** (Ho, Protzenko; ICFP 2022). Functional translation of Rust safe
  fragment for downstream verification.
- **Prusti, Creusot, Verus, Flux, Kani** (covered in §7).

### Distinguishing features

- This entire literature is **soundness and verification** focused: prove that
  Rust's type system is sound, prove that specific programs satisfy specific
  predicates. Antigen does neither.
- This literature is dominated by **mechanized proof** in Coq, Lean, F\*, Why3,
  Iris. Antigen explicitly avoids depending on mechanized-proof toolchains
  (per `design-intent.md`: "Don't want: Dependency on heavy formal-verification
  toolchains").
- Antigen claims a different ergonomic point — **declaration-and-witness rather
  than specification-and-proof**. Witnesses can be cheap (proptest, clippy) or
  expensive (kani, prusti). The grade is up to the developer.

### Relationship

**Witness backends for high-rigor antigens.** A `#[immune(X, witness = rustbelt_lemma)]`
or `#[immune(X, witness = aeneas_translation)]` is the maximal-rigor end of the
spectrum. Antigen does not require this end; it makes it accessible when needed.

### Citation candidates

- Jung, R., Jourdan, J.-H., Krebbers, R., & Dreyer, D. (2018). "RustBelt:
  Securing the Foundations of the Rust Programming Language." *POPL 2018*.
- Jung, R., Dang, H.-H., Kang, J., & Dreyer, D. (2020). "Stacked Borrows: An
  Aliasing Model for Rust." *POPL 2020*.
- Ho, S., & Protzenko, J. (2022). "Aeneas: Rust Verification by Functional
  Translation." *ICFP 2022*.
- Astrauskas, V., Bílý, A., Fiala, J., Grannan, Z., Müller, P., Poli, F., &
  Summers, A. J. (2022). "The Prusti Project: Formal Verification for Rust
  (Tutorial)." *NFM 2022*.

---

## 10. Closest existing prior art — the 3-5 nearest neighbors

After surveying the landscape, the most relevant prior art for antigen's claimed
novelty is, in descending order of closeness:

1. **Eiffel's design-by-contract with inheritance** (Meyer 1992/1997). The
   structural rhyme is the strongest. Pre/post-conditions inherited through
   subclassing with covariance/contravariance is the closest existing analog
   to `#[descended_from]`'s presentation/immunity inheritance. Any honest
   antigen paper must engage with this lineage.

2. **CWE + Rust empirical bug studies (Qin et al. PLDI 2020)**. The taxonomy
   side of antigen has CWE as a direct precedent. Without engaging CWE,
   antigen-stdlib looks like reinvention.

3. **Koka's named effect types** (Leijen 2014/2017). For "named structural
   property declared on functions and propagated through types," Koka is the
   reference paradigm. Antigen tracks failure-classes rather than effects, but
   the design vocabulary is borrowed structure.

4. **Prusti / Flux / Kani / Verus** (recent Rust verification). These are the
   tools antigen explicitly delegates to via the witness mechanism. They are
   the cohabitants of the Rust verification ecosystem. Antigen is not in
   competition with them; antigen is the orchestration layer above them.

5. **FindBugs / SpotBugs / Infer pattern-bug-detection lineage**. The
   structural-fingerprint side of `cargo antigen scan` is well-precedented in
   this lineage. Antigen distinguishes itself by **inheritance** and
   **vaccination** — structural propagation of named patterns rather than
   per-site detection — but the recognition machinery is well-trodden.

---

## 11. Antigen's distinguishing claim — what is genuinely novel

After honest comparison, the following is what antigen contributes that does
**not** appear in the prior art surveyed:

### Genuinely novel

1. **Failure-class names as inherited first-class artifacts.** Eiffel inherits
   contracts (predicates over values). CWE has names but no inheritance. Koka
   inherits effects through type composition. **None inherits a named
   failure-class through structural derivation (`#[descended_from]`)** with
   the witness re-validation contract antigen describes.

2. **Vaccination as a first-class developer action.** `cargo antigen vaccinate`
   is a bulk transform on the failure-class graph — applying a known immunity
   pattern across a structural family. The closest analog in any tool surveyed
   is `cargo fix`, which is per-site. The closest analog in any verification
   system is "apply this lemma family to all instances of pattern P," which
   exists in proof assistants (Coq's `Hint Resolve`, etc.) but not as a
   developer-facing bulk operation on annotation graphs.

3. **The composition-as-orchestration design.** Antigen's design that the
   witness shape is **arbitrary** — a proptest, a clippy lint, a kani harness,
   a phantom type, a Prusti spec, an F\* lemma — and that antigen is the
   **connective tissue** that makes these heterogeneous artifacts cohere
   under named failure-classes is genuinely distinctive. The closest analog
   is the "multi-prover" architecture of Why3 (which orchestrates SMT solvers
   and proof assistants under a unified spec language), but Why3 has a single
   spec language; antigen has zero spec language and accepts heterogeneous
   evidence.

### Synthesis, not novelty

- The annotation mechanism (`#[antigen]`, `#[presents]`, `#[immune]`) is
  standard Rust attribute machinery. Spec# / Code Contracts / JML / Eiffel /
  Koka all do similar attribute-driven specification.
- The cargo subcommand pattern (`cargo antigen scan`) is unremarkable in
  cargo-extension culture (cargo-mutants, cargo-fuzz, etc.).
- The structural fingerprint matching is FindBugs / SpotBugs / Infer / clippy
  territory.
- The witness-as-proof concept is borrowed from proof-carrying-code
  (Necula 1997) and from JML-style specification + checker delegation.
- The 8-class taxonomy in `design-intent.md` is honestly disclosed as
  not-novel — each class has prior literature.

### The honest paper framing

If a future paper is written, the defensible framing is:

> *Antigen: Composing failure-class memory across the Rust verification ecosystem.
> We present antigen, an annotation system and cargo extension that names
> failure-classes as first-class structural artifacts, inherits them through
> derivation, supports bulk vaccination across structural families, and
> orchestrates heterogeneous existing witnesses (proptest, kani, prusti, clippy,
> phantom types) under a unified vocabulary. Our contribution is not a new
> verification technique but a connective-tissue layer that makes the existing
> Rust verification ecosystem composable through named failure-class memory,
> with explicit support for cross-session and AI-assisted development contexts
> where implicit failure-memory degrades.*

This framing claims composition, inheritance, vaccination, and ecosystem
orchestration — and disclaims being a new verification technique.

---

## 12. Related work — paper-ready section draft

> *The following is drafted for direct inclusion in a future antigen paper's
> Related Work section. It is honest about the synthesis nature of the
> contribution.*

Antigen sits at the intersection of several research traditions, none of which
fully captures its design point.

**Refinement and contract systems** (Liquid Haskell [Vazou et al. 2014; Rondon
et al. 2008], F\* [Swamy et al. 2016], Flux [Lehmann et al. 2023], Eiffel
[Meyer 1992, 1997], Spec# [Barnett et al. 2011], JML [Leavens & Cheon 2006],
Dafny [Leino 2010], Why3 [Filliâtre & Paskevich 2013]) attach value-level
predicates to program artifacts and discharge proof obligations through SMT
or interactive provers. Antigen does not attach value-level predicates;
instead, it names *failure-classes* — taxonomic categories of past bugs —
and accepts heterogeneous *witnesses* of immunity. Antigen's `#[descended_from]`
inheritance of failure-class markers is structurally analogous to Eiffel's
inheritance of pre/post-conditions but operates at the level of named
failure-classes rather than predicates.

**Effect type systems** (Koka [Leijen 2014, 2017], Eff [Bauer & Pretnar 2015],
Frank [Convent et al. 2020]) track named structural properties of function
behavior through composition. Antigen borrows this paradigm but applies it to
failure-classes rather than runtime effects, and explicitly does not infer —
declarations are manual, and inheritance follows structural derivation rather
than effect union.

**Rust verification ecosystem** (RustBelt [Jung et al. 2018], Stacked Borrows
[Jung et al. 2020], Prusti [Astrauskas et al. 2019, 2022], Creusot
[Denis et al. 2022], Verus [Lattuada et al. 2023], Aeneas [Ho & Protzenko 2022],
Kani, Flux [Lehmann et al. 2023]) provides a rich set of verification
technologies for Rust, but no orchestration layer that lets developers say
"this site is vulnerable to failure-class X" while delegating the proof
obligation to whichever tool is appropriate. Antigen is positioned as that
orchestration layer, with witness shapes for proptest, clippy, kani, prusti,
phantom types, and beyond.

**Bug taxonomies and empirical bug studies** (CWE [MITRE], Qin et al. [2020]
on Rust-specific bug patterns, Tan et al. on open-source bug characteristics)
provide the population from which antigen-stdlib is curated. Antigen's
contribution is to make these taxonomies *checkable* in the type system rather
than browsable in a registry.

**Pattern-based bug detection** (FindBugs / SpotBugs, Infer, the clippy lint
collection) provides the structural-fingerprint matching technology that
`cargo antigen scan` builds on. Antigen extends this with two primitives not
present in pattern-based detectors: (1) failure-class memory that propagates
through structural derivation via `#[descended_from]`, and (2) the
*vaccination* operation that bulk-applies a known immunity pattern across a
structural family.

The synthesis target — a Rust-ecosystem connective tissue that names
failure-classes as inheritable, vaccinable, and witness-orchestrating
artifacts, designed for AI-assisted development where implicit failure-memory
degrades — is, to our knowledge, novel.

---

## 13. Citation list (consolidated)

> Bibliographic stubs for the references throughout this document. Where exact
> publication venue / year is uncertain, this list flags which references should
> be verified before paper submission.

### Refinement types
- Rondon, P. M., Kawaguchi, M., & Jhala, R. (2008). "Liquid Types." *PLDI 2008*.
- Vazou, N., Seidel, E. L., Jhala, R., Vytiniotis, D., & Peyton-Jones, S. (2014).
  "Refinement Types for Haskell." *ICFP 2014*.
- Swamy, N., et al. (2016). "Dependent Types and Multi-Monadic Effects in F\*."
  *POPL 2016*.
- Lehmann, N., Geller, A. T., Vazou, N., & Jhala, R. (2023). "Flux: Liquid Types
  for Rust." *PLDI 2023*.

### Design by contract
- Meyer, B. (1992). "Applying 'Design by Contract'." *IEEE Computer* 25(10).
- Meyer, B. (1997). *Object-Oriented Software Construction* (2nd ed.). Prentice Hall.
- Leavens, G. T., & Cheon, Y. (2006). "Design by Contract with JML." Tutorial.
- Barnett, M., Fähndrich, M., Leino, K. R. M., Müller, P., Schulte, W., & Venter, H.
  (2011). "Specification and Verification: The Spec# Experience." *CACM* 54(6).

### Effect systems
- Leijen, D. (2014). "Koka: Programming with Row-Polymorphic Effect Types." *MSFP 2014*.
- Leijen, D. (2017). "Type Directed Compilation of Row-Typed Algebraic Effects."
  *POPL 2017*.
- Bauer, A., & Pretnar, M. (2015). "Programming with Algebraic Effects and Handlers."
  *Journal of Logical and Algebraic Methods in Programming*.
- Convent, L., Lindley, S., McBride, C., & McLaughlin, C. (2020). "Doo Bee Doo
  Bee Doo." *Journal of Functional Programming* 30.
- Sivaramakrishnan, K. C., et al. (2021). "Retrofitting Parallelism onto OCaml."
  *ICFP 2020*.

### Information flow
- Pottier, F., & Simonet, V. (2003). "Information Flow Inference for ML." *ACM TOPLAS*.
- Myers, A. C., & Liskov, B. (2000). "Protecting Privacy Using the Decentralized
  Label Model." *ACM TOSEM*.
- Sabelfeld, A., & Myers, A. C. (2003). "Language-Based Information-Flow Security."
  *IEEE Journal on Selected Areas in Communications*.

### Hoare / separation logic
- Hoare, C. A. R. (1969). "An Axiomatic Basis for Computer Programming." *CACM*.
- Reynolds, J. C. (2002). "Separation Logic: A Logic for Shared Mutable Data
  Structures." *LICS 2002*.
- O'Hearn, P. W. (2007). "Resources, Concurrency, and Local Reasoning."
  *Theoretical Computer Science*.
- Jung, R., Krebbers, R., Jourdan, J.-H., Bizjak, A., Birkedal, L., & Dreyer, D.
  (2018). "Iris from the Ground Up." *JFP 28*.

### Lightweight specification systems
- Leino, K. R. M. (2010). "Dafny: An Automatic Program Verifier for Functional
  Correctness." *LPAR-16*.
- Filliâtre, J.-C., & Paskevich, A. (2013). "Why3 — Where Programs Meet Provers."
  *ESOP 2013*.
- Cuoq, P., et al. (2012). "Frama-C: A Software Analysis Perspective." *SEFM 2012*.
- Jacobs, B., et al. (2011). "VeriFast: A Powerful, Sound, Predictable, Fast
  Verifier for C and Java." *NFM 2011*.

### Rust verification
- Jung, R., Jourdan, J.-H., Krebbers, R., & Dreyer, D. (2018). "RustBelt:
  Securing the Foundations of the Rust Programming Language." *POPL 2018*.
- Jung, R., Dang, H.-H., Kang, J., & Dreyer, D. (2020). "Stacked Borrows."
  *POPL 2020*.
- Astrauskas, V., Müller, P., Poli, F., & Summers, A. J. (2019). "Leveraging
  Rust Types for Modular Specification and Verification." *OOPSLA 2019*.
- Astrauskas, V., et al. (2022). "The Prusti Project: Formal Verification for
  Rust (Tutorial)." *NFM 2022*.
- Denis, X., Jourdan, J.-H., & Marché, C. (2022). "Creusot: A Foundry for the
  Deductive Verification of Rust Programs." *ICFM 2022*. (Verify venue.)
- Lattuada, A., Hance, T., et al. (2023). "Verus: Verifying Rust Programs using
  Linear Ghost Types." *OOPSLA 2023*.
- Ho, S., & Protzenko, J. (2022). "Aeneas: Rust Verification by Functional
  Translation." *ICFP 2022*.

### Bug taxonomies and empirical studies
- MITRE Corporation. *Common Weakness Enumeration (CWE) Specification*.
  https://cwe.mitre.org/
- Qin, B., Chen, Y., Yu, Z., Song, L., & Zhang, Y. (2020). "Understanding Memory
  and Thread Safety Practices and Issues in Real-World Rust Programs." *PLDI 2020*.
- Tan, L., Liu, C., Li, Z., Wang, X., Zhou, Y., & Zhai, C. (2014). "Bug
  Characteristics in Open Source Software." *Empirical Software Engineering*.
- Brown, W. J., Malveau, R. C., McCormick, H. W., & Mowbray, T. J. (1998).
  *AntiPatterns*. Wiley.

### Pattern-based bug detection
- Hovemeyer, D., & Pugh, W. (2004). "Finding Bugs Is Easy." *OOPSLA 2004*. (FindBugs.)
- Calcagno, C., Distefano, D., et al. (2015). "Moving Fast with Software
  Verification." *NFM 2015*. (Infer at Facebook.)

### Proof-carrying code (witness-as-proof concept lineage)
- Necula, G. C. (1997). "Proof-Carrying Code." *POPL 1997*.

---

## 14. Open verification items for the team

Items where this survey relied on background knowledge and which a paper-track
team should verify against current literature:

1. **Verus venue/year** — listed as OOPSLA 2023, multiple Verus papers exist;
   verify which is the canonical citation.
2. **Creusot venue** — likely *iFM* or *FM* 2022/2023, verify exact venue.
3. **Tree Borrows** — listed as 2023+; the formal publication may be 2024.
   Verify before citing.
4. **MIRAI** — engineering paper exists; verify whether there is a venue paper
   to cite or whether it should be cited as a tool URL.
5. **Sivaramakrishnan et al. on OCaml 5 effects** — multiple papers; 2021 ICFP
   listed but the canonical citation may be PLDI 2020 / 2021 / 2022.
6. **Recent FindBugs / SpotBugs / Infer citations** — the lineage spans 2004
   (Hovemeyer & Pugh) to 2015 (Calcagno et al.) to ongoing. Pick the relevant
   anchors for the angle being argued.

Until these are verified, the paper-track team should treat this document as a
**research-grade survey of nearest neighbors** rather than as a copy-pasteable
bibliography.

---

## Summary in one paragraph

Antigen's nearest academic neighbors are Eiffel's inheritable contracts,
Koka's named effect types, the CWE failure-class taxonomy, and the cohort of
recent Rust verification tools (Prusti, Creusot, Verus, Flux, Kani). None of
these inherits *named failure-class memory* through structural derivation;
none has a bulk *vaccination* operation across a structural family; none is
designed as *connective tissue* that orchestrates heterogeneous witnesses
under a single failure-class vocabulary. Antigen's defensible novelty is in
the synthesis and the three primitives — inheritance of failure-class
markers, vaccination as a developer action, and witness-shape pluralism — not
in the underlying annotation/scanning/verification technology, which is well
trodden. A future paper should claim composition and orchestration, not new
verification.
