#!/usr/bin/env python3
"""The antigen documentation prose-linter — the doc-drift antigen, dogfooded.

The example-harness (run.py) proves every *example* still runs. This proves every
*claim in the prose* still holds the line: no internal names leaked into a user
doc, no stale version pin, no `(planned)` future promised in present tense, no
bare test-count standing in for evidence. Together they are the standing machinery
the mandate names — "no test-counts, no internal names, no (planned), every fence
verified-run."

This linter is antigen's own thesis turned on its own docs: a claim whose truth
silently flips when the world moves (a version tags, a role-name leaks out of the
team, a promised feature ships) is the silent-failure class. The linter is the
scan; a flagged line is a fingerprint match to inspect, not an audited verdict —
some hits are load-bearing and a human keeps them (the same filter/proof split
the tool itself teaches).

AUDIENCE-AWARE (the captain's ruling, ADR-style). The full leak-scrub applies to
USER docs only. `docs/internal/**` and `docs/decisions.md` are CONTRIBUTOR docs —
internal by design; ADR-numbers and role-names there are correct, not leaks. The
linter exempts them from the us-leak rules but still flags a personal name and a
stale version pin (those leak regardless of audience).

USAGE
-----
    python tools/doc-harness/prose_lint.py [--docs-root DIR] [--json]
                                           [--rule RULE]...

    --docs-root  repo root (default: cwd)
    --json       machine-readable report
    --rule       restrict to named rules (repeatable); default: all

EXIT CODES
----------
    0   no findings (in the selected rules)
    1   at least one finding
    2   linter error

Every rule is conservative by design: a noisy linter is a linter writers learn to
ignore. A rule fires only where the signal is high; uncertain cases are left for
the human pass rather than drowning the report.
"""

from __future__ import annotations

import argparse
import dataclasses
import json
import os
import re
import sys
from pathlib import Path
from typing import Callable, Optional

if hasattr(sys.stdout, "reconfigure"):
    try:
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")
    except Exception:
        pass


# --------------------------------------------------------------------------- #
# Audience classification
# --------------------------------------------------------------------------- #

def audience(rel_path: str) -> str:
    """USER vs CONTRIBUTOR, by path. Contributor docs are internal by design."""
    p = rel_path.replace(os.sep, "/")
    if p.startswith("docs/internal/") or p == "docs/decisions.md":
        return "contributor"
    return "user"


# --------------------------------------------------------------------------- #
# Fence tracking — a rule may want to ignore code-fence content
# --------------------------------------------------------------------------- #

_FENCE_RE = re.compile(r"^(\s*)(`{3,}|~{3,})(.*)$")


def line_kinds(text: str) -> list[tuple[int, str, str]]:
    """Yield (1-based-lineno, kind, line) where kind is 'prose' or 'code'.

    A rule about *prose* (a future-promise, an internal name) should skip 'code'
    lines: a `--version 0.5.0-beta.1` inside a command fence is an example the
    harness owns, not a prose claim. A rule about version pins in install
    snippets, by contrast, wants the code lines.
    """
    out = []
    in_fence = False
    fence_char = ""
    fence_len = 0
    for i, line in enumerate(text.splitlines(), start=1):
        m = _FENCE_RE.match(line)
        if m:
            ticks = m.group(2)
            if not in_fence:
                in_fence = True
                fence_char = ticks[0]
                fence_len = len(ticks)
                out.append((i, "fence", line))
                continue
            elif ticks[0] == fence_char and len(ticks) >= fence_len and m.group(3).strip() == "":
                in_fence = False
                out.append((i, "fence", line))
                continue
        out.append((i, "code" if in_fence else "prose", line))
    return out


# --------------------------------------------------------------------------- #
# Rules
# --------------------------------------------------------------------------- #

@dataclasses.dataclass
class Finding:
    doc: str
    line: int
    rule: str
    text: str
    note: str


# Each rule is (name, applies_to_audience, fn). fn(rel, line_kinds) -> [Finding].

def _user_only(aud: str) -> bool:
    return aud == "user"


# A pin is ACTIONABLE in an install context — a `[dependencies]` toml line
# (`name = "x.y.z"`), a `cargo install`/`cargo add` command, or the prose form of
# either. A bare version-string in running prose (a case study quoting a drift, a
# versioning-policy discussion) is NARRATIVE — flagging it as a pin-to-edit is a
# false positive that breaks the doc's own story. The rule splits the two so the
# report separates "fix these" (version-pin) from "review these" (version-mention).
_INSTALL_CONTEXT_RE = re.compile(
    r"""(?x)
    (?: ^\s*[A-Za-z0-9_-]+ \s* = \s* ["'] )        # toml dep:  name = "
      | cargo \s+ (?:install|add) \b               # cargo install / add
    """
)


def rule_version_pin(rel: str, aud: str, lk: list[tuple[int, str, str]]) -> list[Finding]:
    """A hard pre-release version pin. The CHANGELOG carries versions; an install
    snippet should not pin a beta. The derived `antigen_version` JSON field is not
    an install pin (it carries a JSON key) — excluded. A version-string in an
    install context is the real edit (`version-pin`); one in running prose is a
    narrative mention (`version-mention`) the human reviews, not auto-edits."""
    findings = []
    pin = re.compile(r"\b\d+\.\d+\.\d+-(?:beta|alpha|rc)[\w.]*\b")
    for lineno, kind, line in lk:
        if kind == "fence":
            continue
        if "antigen_version" in line:
            continue  # derived-from-byte, correct-now, couples to the version bump
        if not pin.search(line):
            continue
        if _INSTALL_CONTEXT_RE.search(line):
            findings.append(Finding(rel, lineno, "version-pin", line.strip(),
                                    "hard pre-release pin in an install context — drop it / point at crates.io"))
        else:
            findings.append(Finding(rel, lineno, "version-mention", line.strip(),
                                    "a version-string in prose — likely narrative (case study / policy); review, don't auto-edit"))
    return findings


# us-leak terms: internal project/person/role/tool names that a stranger never
# meets. USER docs only (contributor docs are internal by design) — EXCEPT the
# personal name, which leaks regardless of audience.
_LEAK_TERMS = {
    "tambear":      ("tambear", "cross-project name — say 'a computational-mathematics project'"),
    "the-captain":  (r"\bthe captain\b", "internal role-name"),
    "jbd":          (r"\bJBD\b", "internal methodology name"),
}
_PERSONAL_NAME = (r"\bTekgy\b", "personal name — leaks regardless of audience")
_ROLE_NAMES = re.compile(r"\b(the naturalist|the adversarial agent|the pathmaker|the navigator|the aristotle|the scout)\b", re.I)


def rule_us_leak(rel: str, aud: str, lk: list[tuple[int, str, str]]) -> list[Finding]:
    findings = []
    for lineno, kind, line in lk:
        if kind == "fence":
            continue
        # Personal name: every audience.
        if re.search(_PERSONAL_NAME[0], line):
            findings.append(Finding(rel, lineno, "personal-name", line.strip(), _PERSONAL_NAME[1]))
        if not _user_only(aud):
            continue  # contributor docs are internal by design for the rest
        for key, (pat, note) in _LEAK_TERMS.items():
            if re.search(pat, line, re.I if key == "tambear" else 0):
                findings.append(Finding(rel, lineno, f"us-leak:{key}", line.strip(), note))
        if _ROLE_NAMES.search(line):
            findings.append(Finding(rel, lineno, "us-leak:role-name", line.strip(),
                                    "internal team role-name"))
    return findings


def rule_planned_future(rel: str, aud: str, lk: list[tuple[int, str, str]]) -> list[Finding]:
    """A `(planned)` / `(coming)` / `will ship` promise in present-tense prose.

    Conservative: only the parenthetical-promise forms and a few explicit
    future-of-OURS phrasings. Honest-scope ("the v0.7 frontier", "not yet wired")
    is present-fact about a boundary, not a promise — NOT flagged. USER docs only.
    """
    if not _user_only(aud):
        return []
    findings = []
    promise = re.compile(r"\((?:planned|coming|coming soon|future|todo|tbd|not yet built)\)", re.I)
    for lineno, kind, line in lk:
        if kind != "prose":
            continue
        if promise.search(line):
            findings.append(Finding(rel, lineno, "planned-future", line.strip(),
                                    "a promise in present-tense prose — move to the roadmap or state the present boundary"))
    return findings


def rule_test_count(rel: str, aud: str, lk: list[tuple[int, str, str]]) -> list[Finding]:
    """A bare test-count as performed evidence in prose ("all 1187 tests pass",
    "passes A1/A2/B/C/D/E/F/G"). Conservative: the `running N tests` form inside a
    fence is a real transcript (the harness's job), so we skip code; we flag the
    prose forms. USER docs only."""
    if not _user_only(aud):
        return []
    findings = []
    count = re.compile(r"\b(all\s+)?\d{2,}\s+tests\b|\bpasses\s+[A-G](?:\d)?(?:/[A-G]\d?){2,}", re.I)
    for lineno, kind, line in lk:
        if kind != "prose":
            continue
        if count.search(line):
            findings.append(Finding(rel, lineno, "test-count", line.strip(),
                                    "a bare test-count is performed-honesty — name a runnable test instead"))
    return findings


RULES: dict[str, Callable[[str, str, list], list[Finding]]] = {
    "version-pin": rule_version_pin,
    "us-leak": rule_us_leak,
    "planned-future": rule_planned_future,
    "test-count": rule_test_count,
}


# --------------------------------------------------------------------------- #
# Driver
# --------------------------------------------------------------------------- #

def doc_targets(root: Path) -> list[Path]:
    docs = sorted((root / "docs").rglob("*.md"))
    readmes = [root / "README.md"]
    for crate in ("antigen", "antigen-fingerprint", "antigen-attestation", "antigen-macros", "cargo-antigen"):
        readmes.append(root / crate / "README.md")
    return [p for p in docs + readmes if p.exists()]


def run(root: Path, selected: Optional[set[str]]) -> list[Finding]:
    findings: list[Finding] = []
    rules = {k: v for k, v in RULES.items() if not selected or k in selected}
    for path in doc_targets(root):
        rel = str(path.relative_to(root)).replace(os.sep, "/")
        aud = audience(rel)
        lk = line_kinds(path.read_text(encoding="utf-8"))
        for fn in rules.values():
            findings.extend(fn(rel, aud, lk))
    findings.sort(key=lambda f: (f.doc, f.line, f.rule))
    return findings


def print_human(findings: list[Finding]) -> None:
    from collections import Counter
    by_rule = Counter(f.rule.split(":")[0] for f in findings)
    print(f"antigen prose-linter — {len(findings)} finding(s)")
    for rule, n in sorted(by_rule.items()):
        print(f"  {rule}: {n}")
    print()
    cur = None
    for f in findings:
        if f.doc != cur:
            print(f"\n{f.doc}")
            cur = f.doc
        print(f"  {f.line:>5}  [{f.rule}]  {f.note}")
        print(f"         {f.text[:140]}")


def print_json(findings: list[Finding]) -> None:
    print(json.dumps([dataclasses.asdict(f) for f in findings], indent=2))


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="antigen documentation prose-linter")
    ap.add_argument("--docs-root", default=".")
    ap.add_argument("--json", action="store_true")
    ap.add_argument("--rule", action="append", default=None,
                    help="restrict to named rules (repeatable)")
    args = ap.parse_args(argv)

    root = Path(args.docs_root).resolve()
    selected = set(args.rule) if args.rule else None
    if selected:
        unknown = selected - set(RULES)
        if unknown:
            print(f"error: unknown rule(s): {', '.join(sorted(unknown))}\n"
                  f"known: {', '.join(sorted(RULES))}", file=sys.stderr)
            return 2

    findings = run(root, selected)
    if args.json:
        print_json(findings)
    else:
        print_human(findings)
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
