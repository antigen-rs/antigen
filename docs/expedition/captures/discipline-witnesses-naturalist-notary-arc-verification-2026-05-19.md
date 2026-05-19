# Capture — Naturalist Verification of Scout's Notary 800-Year Arc

> **Date**: 2026-05-19
> **Author**: team-naturalist
> **Status**: append-only capture
> **Builds on**: scout S4 (notary institutions cross-domain pull), capture
> `discipline-witnesses-scout-pass-2026-05-19.md` §"Notary institutions and
> the witness problem"
> **What this is**: web-verified historical check of scout's structural claim,
> with one significant addition scout missed and one claim I couldn't fully
> verify but which doesn't affect scout's structural argument.

---

## Scout's claim to verify

Scout's S4 (paraphrased):
> The notary public institution (dating to Roman scribes, formalized in
> medieval Europe) is one of the oldest human solutions to "how do we make
> attestation portable and trusted?" It's been running for ~800 years.
> Notarial attestation is trusted not because the notary's word is inherently
> credible, but because the notary is a BOUNDED PARTY WITH KNOWN INCENTIVE
> ALIGNMENT: professional liability if they attest falsely; certification
> body that can revoke their commission; personally identifiable.

This is load-bearing for scout's ADR-019 long-arc prediction: "git-trust is
the floor; the accountability escalation path is OIDC + transparency log
(notary-accountability without the professional licensing)."

---

## What survives verification

**The 800-year arc**: confirmed. Web-checked.

- Roman scribal precedent (1st-5th century AD) is real but discontinuous —
  the notary as a continuous institution doesn't run unbroken from Rome
- Late 11th century: Bologna university revived study of Roman law
- Late 12th-early 13th century: notaries in northern Italian towns formed
  guilds; town administrations supervised them
- End of 12th century: Bologna had a trained, examined, guild-regulated
  notarial profession
- Rolandino of Bologna's *Summa Artis Notariae* (~1300) is a datable
  institutional milestone
- 13th century: notaries became customary in England (Northern Europe lag)

Net: continuous institutional history of ~800-900 years from 12th-13th
century formalization to present. Scout's arc is correctly named.

**The accountability structure**: confirmed at the institutional layer.

- Guilds controlled entry, training, examination
- Town administrations supervised guilds (governance over governance)
- Papal and imperial licensing distinguished notary-public (universal
  authority) from civic notary (place-bounded authority)
- The accountability apparatus existed in concrete institutional form

This validates scout's structural claim — accountability structures around
attestation existed and the institutional layer was load-bearing.

---

## What scout missed (and what I want to add to the substrate)

**The strongest structural rhyme for antigen is one scout didn't fully
articulate**: medieval Italian city courts treated notarial documents as
**near-self-authenticating proof**.

> "If a merchant produced a properly drafted instrument, the court did not
> have to re-investigate whether the transaction had happened."
> (Genoa, Venice, Florence court practice)

This IS antigen's tier-honesty made concrete in a 12th-15th century
institutional setting:

- The properly-drafted notarial instrument = Execution-tier attestation
- The court treats it as evidence WITHIN THAT TIER without re-deriving
- Higher assurance requires a different verification surface
- Lower assurance comes from non-notarized witness testimony

**The audit-time savings is the operational value the whole notarial
institution exists to underwrite.** Without the institution, the court would
have to re-investigate every transaction; with the institution, properly-
drafted instruments carry their own warrant.

For antigen: every `.attest/` sidecar at Execution tier is making the same
structural claim — "the discipline was applied; you can trust this without
re-deriving." The notarial precedent says this works at scale when the
institutional structure supports it. The institutional structure for antigen
is:
- Signed identity (git-trust v0.1; OIDC v0.4+)
- Schema-validated attestation (sidecar JSON against serde-derived type)
- Ratchet-asymmetry on tier promotions (audit reports lower bound; promotions
  require evidence; downgrades automatic)

---

## Sharpened ADR-019 prediction

Scout said: "antigen's tier-honesty discipline is structurally right, but
tier-Execution attestation from git-trust-only signers will be contested
once antigen serves high-stakes domains."

Sharper version (notary-historic):

**Antigen's tier-honesty is structurally right. The audit-time-savings it
provides will be the value proposition users actually feel. The accountability
structure determines what TIER OF AUDIENCE the audit-time-savings holds for.**

Direct rhyme to the medieval civic-vs-public distinction:
- **Civic notary** (place-bounded authority) ↔ **antigen with git-trust +
  CODEOWNERS** (workspace-bounded audit-time-savings)
- **Notary public** (papal/imperial license, universal authority) ↔
  **antigen with OIDC + transparency log** (cross-project audit-time-savings)

This is the actual rhyme. The medieval distinction between place-bounded and
universally-licensed notaries maps directly to antigen's git-trust-only
(workspace-bounded) vs OIDC-bound (cross-project) signing escalation. Scout
had the arc; this names the mechanism.

For ADR-019: name this directly. The escalation path isn't just "stronger
signing" — it's "stronger AUDIENCE for whom the audit-time-savings hold."
git-trust + CODEOWNERS works for the workspace's own developers; OIDC works
across organizations.

---

## What I couldn't verify

Scout claimed "professional liability if they attest falsely."

Searched specifically for medieval notary false-attestation sanctions, guild
revocation procedures, and personal-liability frameworks. Web search returned
organizational structure (guilds, supervision, licensing) but no direct
evidence of specific false-attestation liability mechanisms in medieval
practice.

This doesn't mean scout's claim is wrong — it's plausible that guilds did
sanction false attestation and revoke commissions, but I haven't directly
verified it from accessible web sources within this work-session's scope.

**Structural argument doesn't depend on this detail.** Even without verified
false-attestation sanctions, the notary institution had accountability via:
- Guild membership (revocable)
- Certification (revocable via licensing body)
- Documentary trail (notarized instruments signed, dated, recorded;
  falsification mechanically detectable in records)

The principle ("trust without accountability structures is fragile") holds
even at the conservative interpretation.

---

## Implication for v0.1 vs v0.4+

**For v0.1**: scout's current position holds. git-trust + CODEOWNERS is the
floor; the workspace's own developers are the audience; audit-time-savings
hold within that bounded audience. This is biology-rhyming and historically-
rhyming both: civic notary, BCR on local memory B-cell, accountability via
workspace-bounded structure.

**For v0.4+ DSSE + Sigstore**: this is the "notary public" escalation. The
audit-time-savings extend to cross-organization audience. Sigstore's OIDC-
binding + Rekor transparency log IS the notary-public-with-imperial-license
analog without the 13th-century jurisdictional politics.

The escalation is principled, biology-aligned, AND historically-precedented.
The 800-year arc isn't decoration; it's evidence that THIS SPECIFIC
ESCALATION (place-bounded → universally-recognized authority) is what
attestation institutions actually do as they mature.

---

## Going to message scout and navigator

Routing:
- **Scout**: your 800-year arc verified; the court-treatment-as-self-
  authenticating finding extends your argument; the civic-vs-public-notary
  mapping to git-trust-vs-OIDC sharpens the escalation prediction
- **Navigator**: this is substrate for ADR-019's "design arc" framing —
  name the historical precedent specifically (civic notary → notary public →
  papal/imperial license) as the structural rhyme for antigen's git-trust →
  CODEOWNERS-role → OIDC escalation

---

## Posture

Pure verification + extension work. No architecture-prescription (per
framing-call correction). Biology-question equivalent: does the historical
precedent scout cited hold up? Answer: yes, and with sharper structural
content than scout articulated. The civic-vs-public distinction maps directly
to antigen's escalation path; the court-treatment-as-self-authenticating
property is exactly antigen's audit-time-savings.

Sources:
- [A History of the Notary – Medieval Europe (Malaysian Notary Public)](https://malaysiannotarypublic.wordpress.com/2013/06/12/a-history-of-the-notary-medieval-europe/)
- [The Medieval Notary (Subtle Juxtaposition)](https://subtlejuxtaposition.substack.com/p/the-medieval-notary) — source of court-treatment-as-self-authenticating quote
- [Notary Public History (Notary Locator)](https://www.notarylocator.com.au/notary-public-history)
- [98.12.07 Murray, Notarial Instruments (The Medieval Review)](https://scholarworks.iu.edu/journals/index.php/tmr/article/view/14684/20802)
