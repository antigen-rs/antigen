#!/usr/bin/env python3
"""The byte-unchanged test — pins antigen's read-only behavioral claims.

The single most damaging behavioral lie antigen's docs could tell is that a
command only *observes* when in fact it writes. The whole trust model rests on
two such claims:

  - `propose` — observe-don't-declare (ADR-044): it renders a ratifiable
    suggestion and "leaves the source tree byte-unchanged" (cli-reference.md:145,
    examples/propose-demo/README.md, glossary.md:1216). This is why a human
    ratifies what the machine drafts.
  - `scan` + `audit` — "two read-only inspection commands. Neither mutates your
    code" (deployment-ci-integration.md:12).

Neither claim is catchable by reading the source — the write-suppression is the
*absence* of a write across the whole run, not a line you can point at. The only
way to know is to RUN it and diff the tree. So this test does exactly that: for
each claim it copies the demo fixtures to a throwaway dir, runs every documented
invocation (for propose: route-to-human, promote — the path that constructs a
fingerprint, the one most likely to write — and both `--format json` variants),
and hashes every file before and after. If a single byte moves, the claim is a
lie and the test fails loudly, naming the doc that made it.

USAGE
-----
    python tools/doc-harness/byte_unchanged.py [--bin PATH] [--docs-root DIR]

EXIT CODES
----------
    0   every read-only claim held — the tree was byte-unchanged
    1   a claim is FALSE — the tree changed under a command documented read-only
    2   harness error (binary or demo fixtures not found)
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import platform
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Optional

if hasattr(sys.stdout, "reconfigure"):
    try:
        sys.stdout.reconfigure(encoding="utf-8")
        sys.stderr.reconfigure(encoding="utf-8")
    except Exception:
        pass


def find_binary(explicit: Optional[str], root: Path) -> Optional[Path]:
    if explicit:
        p = Path(explicit)
        return p if p.exists() else None
    exe = "cargo-antigen.exe" if platform.system() == "Windows" else "cargo-antigen"
    for cand in (root / "target" / "debug" / exe, root / "target" / "release" / exe):
        if cand.exists():
            return cand
    from shutil import which
    found = which("cargo-antigen")
    return Path(found) if found else None


def snapshot(tree: Path) -> dict[str, str]:
    """Map every file under `tree` to its sha256, relative-path keyed."""
    out = {}
    for p in sorted(tree.rglob("*")):
        if p.is_file():
            rel = str(p.relative_to(tree)).replace("\\", "/")
            out[rel] = hashlib.sha256(p.read_bytes()).hexdigest()
    return out


# A read-only claim: a doc-anchored assertion that a set of invocations leaves the
# tree byte-unchanged. Each carries the doc that makes the claim (so a failure
# names the lying doc), and the invocations to run against a throwaway copy.
@dataclasses.dataclass
class Claim:
    name: str
    doc_anchor: str          # where the doc asserts read-only-ness
    invocations: list[list[str]]


def claims(demo: Path) -> list["Claim"]:
    cluster = str(demo / "cluster")
    clean = str(demo / "clean")
    near = str(demo / "clean-near-miss")
    # propose — every invocation the demo documents: route-to-human (cluster+clean),
    # promote (cluster+near-miss, constructs a fingerprint = riskiest), + both json.
    propose = ["antigen", "propose", "--cluster-root", cluster, "--clean-root"]
    # scan + audit — deployment-ci-integration.md:12 "two read-only inspection
    # commands. Neither mutates your code." Run against the marked cluster tree.
    return [
        Claim(
            "propose observe-don't-declare",
            "cli-reference.md:145 / examples/propose-demo/README.md / glossary.md:1216",
            [propose + [clean], propose + [near],
             propose + [clean, "--format", "json"], propose + [near, "--format", "json"]],
        ),
        Claim(
            "scan + audit read-only",
            "deployment-ci-integration.md:12 (\"Neither mutates your code\")",
            [["antigen", "scan", "--root", cluster],
             ["antigen", "audit", "--root", cluster],
             ["antigen", "scan", "--root", cluster, "--message-format", "json"]],
        ),
    ]


def run(root: Path, binary: Path) -> int:
    demo_src = root / "examples" / "propose-demo"
    if not demo_src.is_dir():
        print(f"error: demo fixtures not found at {demo_src}", file=sys.stderr)
        return 2

    all_pass = True
    for claim in claims(demo_src):
        with tempfile.TemporaryDirectory() as td:
            work = Path(td) / "propose-demo"
            shutil.copytree(demo_src, work)
            # Re-point the claim's invocations at THIS copy (the paths above were
            # built from demo_src; rebuild relative to the fresh work tree).
            invs = _retarget(claim.invocations, demo_src, work)

            before = snapshot(work)
            for argv in invs:
                proc = subprocess.run([str(binary)] + argv, capture_output=True, timeout=120)
                if proc.returncode != 0:
                    print(f"  note: `{' '.join(argv)}` exited {proc.returncode} "
                          f"(read-only outcomes are exit-0; investigate)", file=sys.stderr)
            after = snapshot(work)

        if before == after:
            print(f"PASS — {claim.name}: tree byte-unchanged across "
                  f"{len(claim.invocations)} runs ({claim.doc_anchor})")
        else:
            all_pass = False
            print(f"FAIL — {claim.name}: the tree CHANGED — the claim at "
                  f"{claim.doc_anchor} is FALSE.")
            for k in sorted(set(after) - set(before)):
                print(f"  + WROTE NEW FILE: {k}")
            for k in sorted(set(before) - set(after)):
                print(f"  - DELETED FILE:   {k}")
            for k in sorted(k for k in before.keys() & after.keys() if before[k] != after[k]):
                print(f"  ~ MODIFIED FILE:  {k}")

    if all_pass:
        print("\nobserve-don't-declare holds: antigen's read-only commands render, "
              "they do not write.")
        return 0
    return 1


def _retarget(invocations: list[list[str]], old_root: Path, new_root: Path) -> list[list[str]]:
    """Rewrite any arg that points inside `old_root` to point inside `new_root`."""
    old = str(old_root)
    new = str(new_root)
    return [[a.replace(old, new) for a in argv] for argv in invocations]


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="antigen read-only byte-unchanged keystone test")
    ap.add_argument("--bin", default=None)
    ap.add_argument("--docs-root", default=".")
    args = ap.parse_args(argv)

    root = Path(args.docs_root).resolve()
    binary = find_binary(args.bin, root)
    if binary is None:
        print("error: cargo-antigen binary not found. Build it:\n"
              "    cargo build -p cargo-antigen\n"
              "or pass --bin PATH.", file=sys.stderr)
        return 2

    return run(root, binary)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
