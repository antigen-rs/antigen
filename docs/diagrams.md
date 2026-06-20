# Antigen — Visual Reference

> Diagrams of antigen's architecture, flow, and key concepts. Mermaid
> source renders natively on GitHub and most modern markdown viewers.
> If you're reading on docs.rs (which may not render mermaid), the
> diagram source is itself reasonably parseable as structured text.

---

## The five vocabulary primitives

```mermaid
graph LR
    A["#[antigen]<br/>declare failure-class"] -->|named by| F["fingerprint pattern"]
    P["#[presents]<br/>mark vulnerable site"] -->|references| A
    I["#[defended_by]<br/>register defense"] -->|references| A
    I -->|witnessed by| W["witness function /<br/>phantom type /<br/>external tool"]
    D["#[descended_from]<br/>inheritance"] -->|references| A
    T["#[antigen_tolerance]<br/>accept match"] -->|references| A
    T -->|requires| R["rationale"]
```

The vocabulary is the shared coordination layer. Each primitive
references the antigen type and adds different structure.

---

## Scan flow

```mermaid
graph TD
    Start[cargo antigen scan] --> Walk[walk workspace files]
    Walk --> Collect{collect}
    Collect -->|"#[antigen]"| Decls[antigen declarations]
    Collect -->|"#[presents]"| Pres[explicit presentations]
    Collect -->|"#[defended_by]"| Imm[defense registrations]
    Collect -->|"#[antigen_tolerance]"| Tol[tolerances]
    Collect -->|"#[descended_from]"| Lin[lineage edges]
    Collect -->|fingerprint match| FP[passive matches]

    Decls --> Build[build ScanReport]
    Pres --> Build
    Imm --> Build
    Tol --> Build
    Lin --> Build
    FP --> Build

    Lin --> Cycle{cycle detection<br/>DFS + depth limit}
    Cycle -->|cycle found| Err[parse_failure]
    Cycle -->|clean| Prop[propagation walk]
    Prop --> Build

    Build --> Out[scan report<br/>human or JSON]
```

The scan is a single pass over the workspace, with cycle detection
and propagation walk for `#[descended_from]` chains.

---

## Audit flow

```mermaid
graph TD
    Start[cargo antigen audit] --> Scan[run scan]
    Scan --> Loop{for each<br/>#[defended_by]}
    Loop --> Resolve[resolve witness identifier]
    Resolve --> Kind{witness kind?}
    Kind -->|"phantom turbofish<br/>Foo::<T>::ctor"| FP[FormalProof tier<br/>PhantomTypeShapeRecognized]
    Kind -->|"#[test] fn"| Reach1[Reachability tier<br/>TestAttributePresentNotInvoked]
    Kind -->|"proptest! fn"| Reach2[Reachability tier<br/>ProptestPresentNotInvoked]
    Kind -->|"clippy::lint"| Reach3[Reachability tier<br/>ExternalToolPrefixRecognized]
    Kind -->|"kani:: / prusti:: / ..."| Reach3
    Kind -->|"helper fn"| Reach4[Reachability tier<br/>FunctionResolves]
    Kind -->|not found| None[None tier<br/>WitnessNotFound]
    Kind -->|ambiguous| Amb[None tier<br/>WitnessAmbiguous]
    Kind -->|missing| Miss[None tier<br/>WitnessMissing]

    FP --> Report[audit report<br/>tier-honest]
    Reach1 --> Report
    Reach2 --> Report
    Reach3 --> Report
    Reach4 --> Report
    None --> Report
    Amb --> Report
    Miss --> Report
```

Per ADR-005 Amendment 3 (audit-tier-honesty), the audit reports the
*actual* verification strength, never a stronger one. The `Execution` tier is
reserved in the `WitnessTier` enum but not emitted today (it awaits harness
invocation — a recorded graduation path, see [`roadmap.md`](roadmap.md)); the audit
reports test/proptest functions at `Reachability` with disambiguating hints.

---

## Witness tier gradient

```mermaid
graph LR
    F[FormalProof] -->|"phantom-type today<br/>+ external proof: roadmap"| Strong[strongest guarantee]
    E[Execution<br/>reserved] -->|"harness invoked + passed: roadmap"| Mid[empirical guarantee]
    R[Reachability] -->|"function resolves<br/>behavior unverified"| Weak[existence verified]
    N[None] -->|"missing / not found / ambiguous"| Gap[honest gap]

    style F fill:#90EE90
    style E fill:#FFFFE0
    style R fill:#FFE4B5
    style N fill:#FFB6B6
```

In v0.1, FormalProof is reached only by phantom-type witnesses.
Execution tier is reserved (no harness invocation yet). All test /
proptest / external-tool witnesses report Reachability with hints
disambiguating the case. See [`witness-tiers.md`](witness-tiers.md).

---

## Multi-component architecture

```mermaid
graph TB
    V[antigen-the-vocabulary<br/>emergent practice]

    V --> C1[C1: Dev-in-the-loop<br/>you write declarations]
    V --> C2[C2: Passive scan/tools<br/>cargo antigen scan/audit]
    V --> C3[C3: Test integration<br/>witnesses link to tests]
    V --> C4[C4: Knowledge-ecosystem<br/>references to PRs, ADRs, CVEs]
    V --> C5[C5: Cross-version/lineage<br/>descended_from + canonical_path]
    V --> C6[C6: Cross-crate/ecosystem<br/>antigen-stdlib + propagation]
    V --> C7[C7: Real-time/CI feedback<br/>future PR-scope tooling]

    C2 -.->|consumes| C1
    C3 -.->|consumes| C1
    C4 -.->|extends| C1
    C5 -.->|extends| C1
    C5 <-->|coupled| C6
    C7 -.->|consumes| C2

    style C1 fill:#E8F4F8
    style C2 fill:#E8F4F8
    style C3 fill:#E8F4F8
    style C5 fill:#E8F4F8
    style C6 fill:#E8F4F8
    style C7 fill:#E8F4F8
    style C4 fill:#FFE8B5
```

Seven components compose under the shared vocabulary. C1, C2, C3, C5,
C6, C7 are biology-tier (immune-system analogs); C4 is
engineered-boundary tier (human knowledge ecosystem, beyond biology).
See [`concepts.md`](concepts.md#multi-component-architecture) for
deeper framing.

---

## Adoption pathways

```mermaid
graph TD
    Adopter{who is adopting?}

    Adopter -->|junior:<br/>learning Rust+antigen together| Junior[developmental immunology cognate<br/>tool produces discipline through use]
    Adopter -->|senior:<br/>has tribal knowledge,<br/>lacks structural memory| Senior[vaccination cognate<br/>tool amplifies existing discipline]
    Adopter -->|mature org:<br/>has ADR culture,<br/>post-mortem rigor| Mature[immune surveillance cognate<br/>tool formalizes existing discipline]

    Junior --> Floor1[Floor: declare one antigen]
    Senior --> Floor2[Floor: install scan, see what's there]
    Mature --> Floor3[Floor: weave into existing ADR process]

    Floor1 --> Grow[grow components 2-7 as discipline matures]
    Floor2 --> Grow
    Floor3 --> Grow
```

Three pathways, three different biology cognates, all real adoption
shapes. The "ideal user" property is replicable for the junior path
through onboarding, extended for the senior path through the tool,
formalized for the mature path through structural enforcement.

---

## Biology cognate map

```mermaid
graph LR
    subgraph Biology
        PRR["Pattern-Recognition Receptors (PRRs)"]
        MHC["MHC Class I/II presentation"]
        TCR["T-cell receptors"]
        Ab["Antibodies"]
        BMem["B-cell memory"]
        BLin["B-cell lineage"]
        Treg["Peripheral tolerance / Tregs"]
        Drift["Antigenic drift / shift"]
        Epi["Epidemiological surveillance"]
        DC["Dendritic-cell processing"]
    end

    subgraph Antigen
        FE["Fingerprint engine"]
        Pres["#[presents]"]
        Fp["Named failure-class fingerprints"]
        W["Test / proptest / phantom-type / lint witnesses"]
        Decl["#[antigen] declarations"]
        DF["#[descended_from]"]
        Tol["#[antigen_tolerance]"]
        Ver["Version-boundary recognition (ADR-017)"]
        Cross["Cross-crate scan + propagation (C6)"]
        Audit["Audit pipeline"]
    end

    PRR -.->|maps to| FE
    MHC -.->|maps to| Pres
    TCR -.->|maps to| Fp
    Ab -.->|maps to| W
    BMem -.->|maps to| Decl
    BLin -.->|maps to| DF
    Treg -.->|maps to| Tol
    Drift -.->|maps to| Ver
    Epi -.->|maps to| Cross
    DC -.->|maps to| Audit
```

The biology metaphor is load-bearing, not decorative. When biology
predicts a primitive, the project builds it. When biology breaks at
a boundary (e.g., the knowledge-ecosystem references field doesn't
have a clean biology cognate — see C4 in
[`concepts.md`](concepts.md)), that silence is honest information.

---

## What antigen relates to

```mermaid
graph TB
    Code["your Rust code"]

    Code --> Tests["tests + proptest<br/>(verify this code does X)"]
    Code --> Docs["documentation<br/>(records we decided X)"]
    Code --> Antigen["antigen<br/>(captures this class has historically failed in this way)"]

    Antigen -.->|composes with| Tests
    Antigen -.->|complements| Docs
    Antigen -.->|delegates verification to| Verifiers["clippy / kani / prusti / verus / creusot / flux"]
    Antigen -.->|cross-references| Knowledge["PRs / ADRs / CVEs / post-mortems / RFCs"]

    style Antigen fill:#E8F4F8
    style Tests fill:#FFFFE0
    style Docs fill:#FFFFE0
    style Verifiers fill:#FFE4B5
    style Knowledge fill:#FFE4B5
```

Three pillars: testing checks specific behavior; documentation records
decisions; antigen captures structural failure-class memory. Antigen
composes with the others; it doesn't replace them.

---

## What a failure-class lifecycle looks like

```mermaid
graph TD
    Bug[bug found in production]
    Bug --> Fix[fixed in PR]
    Fix --> Tradi{traditional path}
    Tradi -->|commit message| LostA[lost to commit archive]
    Tradi -->|Slack thread| LostB[lost when channel archives]
    Tradi -->|docstring| LostC[drifts as code evolves]
    Tradi -->|post-mortem blog| LostD[platform dies in 5 years]

    Fix --> Antigen{antigen path}
    Antigen -->|declare #[antigen] with fingerprint| Decl[structural memory in src/antigens.rs]
    Antigen -->|references = [pr, blog, adr]| Bridge[bridge to lived context]
    Antigen -->|#[defended_by] on the fixed site| Imm[witness binds the fix to the lesson]

    Decl --> Future{6 months later}
    Bridge --> Future
    Imm --> Future

    Future -->|new dev writes similar code| Detect[cargo antigen scan detects it]
    Future -->|new dev refactors fixed site| Audit[audit catches lost immunity]
    Future -->|LLM agent generates code| LLM[LLM reads antigen declarations, respects them]

    style LostA fill:#FFB6B6
    style LostB fill:#FFB6B6
    style LostC fill:#FFB6B6
    style LostD fill:#FFB6B6
    style Decl fill:#90EE90
    style Bridge fill:#90EE90
    style Imm fill:#90EE90
```

The lesson learned in a single bug fix can either decay through
traditional carriers (none drift-resistant) or persist as structural
memory (durable; checkable by tooling; co-native for human + LLM
collaborators).

> **The efferent arc has its own diagrams.** The lifecycle above is the *afferent*
> half — how a class is born and detected. Once a class exists, the v0.6 organs let
> it *live*: a life-record, the two senses, the classifier, and CURATE. Those are
> diagrammed in [the v0.6 anatomy](the-v06-anatomy.md) (the organ topology) and
> [the learning loop](the-learning-loop.md) (the whole sense → classify → act loop,
> with the wired-vs-library-vs-frontier tiers marked honestly).

---

## See also

- [`the-v06-anatomy.md`](the-v06-anatomy.md) — the v0.6 organ-topology diagrams
- [`the-learning-loop.md`](the-learning-loop.md) — the afferent→efferent loop as one picture
- [`concepts.md`](concepts.md) — what antigen IS architecturally
- [`tutorial.md`](tutorial.md) — first 15 minutes
- [`macros.md`](macros.md) — full macro reference
- [`witness-tiers.md`](witness-tiers.md) — tier semantics
- [`output-formats.md`](output-formats.md) — scan/audit output
- [`index.md`](index.md) — full documentation map

---

*Diagrams as substrate. When something doesn't render, the mermaid
source is still parseable structured text. When something is wrong,
the docs are authoritative; this is a visual aid, not a primary
reference.*
