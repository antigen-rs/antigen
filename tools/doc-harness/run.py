#!/usr/bin/env python3
"""The antigen documentation example-harness.

Extracts every shell command-fence from the documentation, pairs each with the
output the doc *claims* it produces, runs the deterministically-runnable ones
against the real `cargo antigen` binary, and reports any drift between the
claimed output and the actual output.

This is the mechanical replacement for a heroic human "re-prove every example"
pass. Run it once per release. A non-reproducing example is a located finding —
the report names the doc and line so a writer can fix it.

WHY THIS SHAPE
--------------
The docs already encode their examples co-natively: a ```sh fence holds the
command, and the fence that follows holds the output the reader is told to
expect. The harness reads that existing convention rather than imposing a new
markup — no doc has to change to be checked.

Not every command is runnable in a vacuum. `cargo install`, `cd /path/to/your
/project`, and `cargo antigen scan` against an unspecified tree need a world the
harness can't conjure deterministically. Those are classified ILLUSTRATIVE and
reported (so they are *visible*, never silently skipped) but not asserted byte-
equal. The RUNNABLE-HERE set — every `--help` surface, `--version` — is run for
real and diffed. That set is exactly where CLI-output drift hides, because it is
the surface that changes when a subcommand is added or a help string is reworded.

USAGE
-----
    python tools/doc-harness/run.py [--bin PATH] [--docs-root DIR] [--json]

    --bin        path to the cargo-antigen binary
                 (default: target/debug/cargo-antigen[.exe], else PATH)
    --docs-root  repo root holding docs/ and the crate READMEs (default: cwd)
    --json       emit a machine-readable report instead of the human one

EXIT CODES
----------
    0   every RUNNABLE-HERE example reproduced (ILLUSTRATIVE ones don't gate)
    1   at least one RUNNABLE-HERE example drifted from its claimed output
    2   harness error (binary not found, etc.)
"""

from __future__ import annotations

import argparse
import dataclasses
import difflib
import json
import os
import platform
import re
import subprocess
import sys
from pathlib import Path
from typing import Iterator, Optional

# The console on Windows defaults to cp1252, which cannot encode the em-dash and
# section-sign that antigen's help strings use. Force UTF-8 so the report prints
# the real characters rather than dying on them.
if hasattr(sys.stdout, "reconfigure"):
    try:
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")
    except Exception:
        pass

# --------------------------------------------------------------------------- #
# Fence extraction
# --------------------------------------------------------------------------- #

# A fenced block as it sits in the source: its info-string (``` ```sh `` ->
# "sh"), its body lines, and the 1-based line number of the opening fence (so a
# finding can point a writer at the exact spot).
@dataclasses.dataclass
class Fence:
    info: str
    body: str
    open_line: int


_FENCE_RE = re.compile(r"^(\s*)(`{3,}|~{3,})(.*)$")


def iter_fences(text: str) -> Iterator[Fence]:
    """Yield every fenced code block in a markdown document, in order.

    Handles both ``` and ~~~ fences and tolerates indentation. A closing fence
    must use the same character and be at least as long as the opener (the
    CommonMark rule), so a ```` ``` ```` inside a ~~~ block does not close it.
    """
    lines = text.splitlines()
    i = 0
    n = len(lines)
    while i < n:
        m = _FENCE_RE.match(lines[i])
        if not m:
            i += 1
            continue
        indent, ticks, info = m.group(1), m.group(2), m.group(3).strip()
        open_line = i + 1  # 1-based
        body_lines: list[str] = []
        i += 1
        while i < n:
            cm = _FENCE_RE.match(lines[i])
            if cm and cm.group(2)[0] == ticks[0] and len(cm.group(2)) >= len(ticks) and cm.group(3).strip() == "":
                i += 1  # consume the closing fence
                break
            body_lines.append(lines[i])
            i += 1
        yield Fence(info=info.lower(), body="\n".join(body_lines), open_line=open_line)


# --------------------------------------------------------------------------- #
# Command / output pairing
# --------------------------------------------------------------------------- #

_SH_INFOS = {"sh", "shell", "bash", "console", "shell-session", "sh-session"}
# An output fence is normally untagged; "text"/"console" sometimes carry it too.
_OUTPUT_INFOS = {"", "text", "console", "output", "txt"}

# A command line we care about — anything driving the antigen binary, plus the
# install line (illustrative, but worth surfacing).
_CMD_LINE_RE = re.compile(r"\b(cargo\s+antigen\b|cargo\s+install\s+cargo-antigen\b)")


@dataclasses.dataclass
class Example:
    doc: str
    open_line: int
    command: str          # the cargo-antigen invocation, normalized
    raw_command_line: str  # the line as it appeared (may have a `$ ` prompt)
    claimed_output: Optional[str]   # the following output fence, if any
    claimed_output_line: Optional[int]
    # A self-contained transcript fence (MODE 2) may legitimately show only an
    # *excerpt* of help — a README routinely trims the Usage/Options apparatus.
    # An exact-line-subsequence of the real output is treated as a valid excerpt
    # (EXCERPT), not drift. A command-then-output pair (MODE 1) is held to exact
    # match: it claims to be the whole output.
    excerpt_ok: bool = False


def _strip_prompt(line: str) -> str:
    """Drop a leading shell prompt (`$ `, `> `, `PS> `) from a command line."""
    return re.sub(r"^\s*(\$|>|PS[^>]*>)\s+", "", line).rstrip()


_PROMPT_CMD_RE = re.compile(r"^\s*\$\s+(cargo\s+antigen\b.*)$")


def extract_examples(doc: str, text: str) -> list[Example]:
    """Extract every antigen example a doc claims an output for.

    Two extraction modes cover the two conventions the docs use:

    MODE 1 — command-fence then output-fence: an ```sh fence holds the command,
    and the fence right after holds the output ("Verify:" / "You should see:").

    MODE 2 — self-contained transcript: a single ```text fence whose first line
    is a `$ cargo antigen ...` prompt, with the output below it in the same
    fence. Crate READMEs use this. The command is the prompt line; the claimed
    output is the rest of the fence.

    A fence consumed as MODE-1 output is not re-read as a MODE-2 transcript.
    """
    fences = list(iter_fences(text))
    examples: list[Example] = []
    consumed_as_output: set[int] = set()  # fence indices used as a command's output

    # MODE 1 — sh command fence paired with the following output fence.
    for idx, f in enumerate(fences):
        if f.info not in _SH_INFOS:
            continue
        cmd_line = None
        for raw in f.body.splitlines():
            if _CMD_LINE_RE.search(raw):
                cmd_line = raw
                break
        if cmd_line is None:
            continue
        command = _strip_prompt(cmd_line)

        claimed = None
        claimed_line = None
        if idx + 1 < len(fences):
            nxt = fences[idx + 1]
            # The output fence may echo the command on its first line (`$ cargo
            # antigen --help`) — still output. Only another *command* (sh) fence
            # disqualifies it (two commands in a row, no output between).
            if nxt.info in _OUTPUT_INFOS and nxt.info not in _SH_INFOS:
                claimed = nxt.body
                claimed_line = nxt.open_line
                consumed_as_output.add(idx + 1)

        examples.append(
            Example(
                doc=doc, open_line=f.open_line, command=command,
                raw_command_line=cmd_line.strip(),
                claimed_output=claimed, claimed_output_line=claimed_line,
            )
        )

    # MODE 2 — self-contained `$ cargo antigen ...` transcript fence.
    for idx, f in enumerate(fences):
        if idx in consumed_as_output or f.info in _SH_INFOS:
            continue
        if f.info not in _OUTPUT_INFOS:
            continue
        body_lines = f.body.splitlines()
        if not body_lines:
            continue
        m = _PROMPT_CMD_RE.match(body_lines[0])
        if not m:
            continue
        command = m.group(1).rstrip()
        # The claimed output is everything after the prompt line. (normalize()
        # also strips a leading echoed-command line, so passing the whole body is
        # safe — but pass only the tail to keep the diff anchored on the output.)
        claimed = "\n".join(body_lines[1:])
        examples.append(
            Example(
                doc=doc, open_line=f.open_line, command=command,
                raw_command_line=body_lines[0].strip(),
                claimed_output=claimed, claimed_output_line=f.open_line,
                excerpt_ok=True,
            )
        )

    return examples


# --------------------------------------------------------------------------- #
# Classification: runnable-here vs illustrative
# --------------------------------------------------------------------------- #

# A command is RUNNABLE-HERE only if it is deterministic in this repo with no
# user project, no network, and no mutation. The help/version surface qualifies;
# anything that needs a target tree or the network does not.
_RUNNABLE_RE = re.compile(
    r"^cargo\s+antigen\s+"
    r"(?:[a-z-]+\s+)?"          # optional subcommand
    r"(?:--help|-h|--version|-V)\s*$"
)
_HELP_TAIL_RE = re.compile(r"(--help|-h|--version|-V)\s*$")

_ILLUSTRATIVE_MARKERS = ("cd ", "/path/to", "your-project", "your/rust", "cargo install")


def classify(command: str) -> str:
    """Return 'runnable' or 'illustrative' for a command string."""
    c = command.strip()
    if any(mark in c for mark in _ILLUSTRATIVE_MARKERS):
        return "illustrative"
    if _RUNNABLE_RE.match(c):
        return "runnable"
    # `cargo antigen --help` with trailing args, or a bare help on the root.
    if c.startswith("cargo antigen") and _HELP_TAIL_RE.search(c):
        return "runnable"
    return "illustrative"


# --------------------------------------------------------------------------- #
# Running + normalization
# --------------------------------------------------------------------------- #

def _split_argv(command: str) -> list[str]:
    """Turn a `cargo antigen ...` string into argv for the binary.

    The binary is invoked as the cargo subcommand: `cargo-antigen antigen ...`.
    So `cargo antigen scan --help` -> [bin, "antigen", "scan", "--help"].
    """
    parts = command.split()
    assert parts[0] == "cargo" and parts[1] == "antigen", command
    return parts[1:]  # drop "cargo", keep "antigen" + the rest


def run_command(binary: Path, command: str) -> str:
    argv = [str(binary)] + _split_argv(command)
    # Capture BYTES and decode as UTF-8 ourselves. The binary writes UTF-8 (its
    # help strings carry em-dashes and section-signs); `text=True` would decode
    # with the platform locale (cp1252 on Windows), turning `—` into mojibake and
    # producing a FALSE drift on every line that contains one. Decoding the bytes
    # as UTF-8 is the only way the comparison sees the same characters the doc has.
    proc = subprocess.run(
        argv,
        capture_output=True,
        timeout=60,
    )
    out = proc.stdout.decode("utf-8", errors="replace")
    err = proc.stderr.decode("utf-8", errors="replace")
    # Help goes to stdout; clap prints --help to stdout, errors to stderr.
    return out if out.strip() else err


def normalize(text: str) -> list[str]:
    """Normalize output for comparison.

    Drops a leading `$ <command>` echo line (docs prefix their output blocks with
    the command for the reader), trims trailing whitespace per line, and drops
    trailing blank lines. The version string is held volatile (it changes at
    release) and masked so a pre-bump binary doesn't false-positive every block.
    """
    lines = []
    for ln in text.splitlines():
        # Skip a doc's echoed-command line ("$ cargo antigen --help").
        if re.match(r"^\s*\$\s+cargo\s+antigen\b", ln):
            continue
        ln = ln.rstrip()
        # Mask the volatile version string (carried by the CHANGELOG, not pinned
        # in help output for comparison purposes).
        ln = re.sub(r"\b\d+\.\d+\.\d+(?:-[\w.]+)?\b", "<VERSION>", ln)
        lines.append(ln)
    # Drop leading/trailing blank lines.
    while lines and lines[0] == "":
        lines.pop(0)
    while lines and lines[-1] == "":
        lines.pop()
    return lines


# --------------------------------------------------------------------------- #
# Driver
# --------------------------------------------------------------------------- #

@dataclasses.dataclass
class Result:
    example: Example
    status: str   # PASS | EXCERPT | DRIFT | ILLUSTRATIVE | NO-CLAIM
    diff: Optional[str] = None


def is_subsequence(claimed: list[str], actual: list[str]) -> bool:
    """True if every non-blank claimed line appears in `actual`, in order.

    This is the excerpt test: a README that shows a *truthful subset* of help —
    trimming the Usage/Options apparatus, dropping rows — passes. A README that
    *modifies* a line (an abbreviated description the binary doesn't emit
    verbatim) does NOT pass: that line isn't found in actual, so the subsequence
    breaks and it surfaces as drift.
    """
    it = iter(actual)
    for c in claimed:
        if c == "":
            continue  # blank lines don't have to line up
        if not any(c == a for a in it):  # advances the shared iterator
            return False
    return True


def find_binary(explicit: Optional[str], root: Path) -> Optional[Path]:
    if explicit:
        p = Path(explicit)
        return p if p.exists() else None
    exe = "cargo-antigen.exe" if platform.system() == "Windows" else "cargo-antigen"
    for cand in (root / "target" / "debug" / exe, root / "target" / "release" / exe):
        if cand.exists():
            return cand
    # PATH fallback.
    from shutil import which
    found = which("cargo-antigen")
    return Path(found) if found else None


def doc_targets(root: Path) -> list[Path]:
    docs = sorted((root / "docs").rglob("*.md"))
    readmes = [root / "README.md"]
    for crate in ("antigen", "antigen-fingerprint", "antigen-attestation", "antigen-macros", "cargo-antigen"):
        readmes.append(root / crate / "README.md")
    return [p for p in docs + readmes if p.exists()]


def run(root: Path, binary: Path) -> list[Result]:
    results: list[Result] = []
    for path in doc_targets(root):
        rel = str(path.relative_to(root)).replace(os.sep, "/")
        text = path.read_text(encoding="utf-8")
        for ex in extract_examples(rel, text):
            kind = classify(ex.command)
            if kind == "illustrative":
                results.append(Result(ex, "ILLUSTRATIVE"))
                continue
            if ex.claimed_output is None:
                results.append(Result(ex, "NO-CLAIM"))
                continue
            actual = run_command(binary, ex.command)
            claimed_n = normalize(ex.claimed_output)
            actual_n = normalize(actual)
            if claimed_n == actual_n:
                results.append(Result(ex, "PASS"))
            elif ex.excerpt_ok and is_subsequence(claimed_n, actual_n):
                # A self-contained transcript showing a truthful subset of help.
                results.append(Result(ex, "EXCERPT"))
            else:
                diff = "\n".join(
                    difflib.unified_diff(
                        claimed_n, actual_n,
                        fromfile=f"{ex.doc}:{ex.claimed_output_line} (claimed)",
                        tofile="binary (actual)",
                        lineterm="",
                    )
                )
                results.append(Result(ex, "DRIFT", diff))
    return results


def print_human(results: list[Result], binary: Path) -> None:
    n_pass = sum(r.status == "PASS" for r in results)
    n_excerpt = sum(r.status == "EXCERPT" for r in results)
    n_drift = sum(r.status == "DRIFT" for r in results)
    n_illus = sum(r.status == "ILLUSTRATIVE" for r in results)
    n_noclaim = sum(r.status == "NO-CLAIM" for r in results)

    print(f"antigen doc-harness — binary: {binary}")
    print(f"  examples found: {len(results)}")
    print(f"  RUNNABLE-HERE:  {n_pass} pass, {n_excerpt} excerpt, {n_drift} drift, {n_noclaim} no-claim (command present, no output block)")
    print(f"  ILLUSTRATIVE:   {n_illus} (needs a user project/network — surfaced, not asserted)")
    print()

    if n_excerpt:
        print("=== EXCERPT — a truthful subset of help (README trim); not drift ===")
        for r in results:
            if r.status == "EXCERPT":
                print(f"  {r.example.doc}:{r.example.open_line}  `{r.example.command}`")
        print()

    if n_drift:
        print("=== DRIFT — claimed output does not match the binary ===")
        for r in results:
            if r.status == "DRIFT":
                print(f"\n--- {r.example.doc}:{r.example.open_line}  `{r.example.command}`")
                print(r.diff)
        print()

    if n_noclaim:
        print("=== RUNNABLE command with NO claimed-output block (informational) ===")
        for r in results:
            if r.status == "NO-CLAIM":
                print(f"  {r.example.doc}:{r.example.open_line}  `{r.example.command}`")
        print()

    print("=== ILLUSTRATIVE commands (surfaced for visibility) ===")
    for r in results:
        if r.status == "ILLUSTRATIVE":
            print(f"  {r.example.doc}:{r.example.open_line}  `{r.example.command}`")


def print_json(results: list[Result], binary: Path) -> None:
    payload = {
        "binary": str(binary),
        "results": [
            {
                "doc": r.example.doc,
                "open_line": r.example.open_line,
                "command": r.example.command,
                "status": r.status,
                "claimed_output_line": r.example.claimed_output_line,
                "diff": r.diff,
            }
            for r in results
        ],
    }
    print(json.dumps(payload, indent=2))


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="antigen documentation example-harness")
    ap.add_argument("--bin", default=None, help="path to the cargo-antigen binary")
    ap.add_argument("--docs-root", default=".", help="repo root (holds docs/ and crate READMEs)")
    ap.add_argument("--json", action="store_true", help="emit machine-readable report")
    args = ap.parse_args(argv)

    root = Path(args.docs_root).resolve()
    binary = find_binary(args.bin, root)
    if binary is None:
        print("error: cargo-antigen binary not found. Build it first:\n"
              "    cargo build -p cargo-antigen\n"
              "or pass --bin PATH.", file=sys.stderr)
        return 2

    results = run(root, binary)
    if args.json:
        print_json(results, binary)
    else:
        print_human(results, binary)

    drift = any(r.status == "DRIFT" for r in results)
    return 1 if drift else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
