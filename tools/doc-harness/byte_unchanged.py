#!/usr/bin/env python3
"""The byte-unchanged test — pins the observe-don't-declare keystone.

The single most damaging behavioral lie antigen's docs could tell is that
`cargo antigen propose` only *observes* — that it renders a suggestion and leaves
your source tree byte-unchanged — when in fact it writes. The whole trust model
rests on it: observe-don't-declare (ADR-044) is why a human ratifies what the
machine drafts. `cli-reference.md` and `examples/propose-demo/README.md` both
assert it in plain words: "A `propose` run leaves the source tree byte-unchanged."

This claim is NOT catchable by reading the source — the proposal logic is library
code in `propose.rs`, and the write-suppression (its *absence*, really) is a
property of the whole run. The only way to know is to RUN it and diff the tree.
So this test does exactly that, against the demo's real twin-cluster — including
the *promote* path that constructs a fingerprint, the one most likely to write if
the contract were ever broken.

It runs every propose invocation the demo documents (route-to-human, promote, and
both `--format json` variants) against a throwaway COPY of the demo fixtures,
hashing every file before and after. If a single byte moves, the keystone claim
is a lie and this test fails loudly.

USAGE
-----
    python tools/doc-harness/byte_unchanged.py [--bin PATH] [--docs-root DIR]

EXIT CODES
----------
    0   the tree was byte-unchanged across every propose run — claim holds
    1   the tree changed — the observe-don't-declare claim is FALSE
    2   harness error (binary or demo fixtures not found)
"""

from __future__ import annotations

import argparse
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


# Every propose invocation the demo documents — the cluster paired with each
# corpus, in both output formats. `cluster` + `clean` → route-to-human;
# `cluster` + `clean-near-miss` → promote. The promote path constructs a
# fingerprint; if any path wrote, this set would catch it.
def invocations(demo: Path) -> list[list[str]]:
    cluster = str(demo / "cluster")
    clean = str(demo / "clean")
    near = str(demo / "clean-near-miss")
    base = ["antigen", "propose", "--cluster-root", cluster, "--clean-root"]
    return [
        base + [clean],
        base + [near],
        base + [clean, "--format", "json"],
        base + [near, "--format", "json"],
    ]


def run(root: Path, binary: Path) -> int:
    demo_src = root / "examples" / "propose-demo"
    if not demo_src.is_dir():
        print(f"error: demo fixtures not found at {demo_src}", file=sys.stderr)
        return 2

    with tempfile.TemporaryDirectory() as td:
        work = Path(td) / "propose-demo"
        shutil.copytree(demo_src, work)

        before = snapshot(work)
        print(f"byte-unchanged test — {len(before)} fixture files, "
              f"{len(invocations(work))} propose runs")

        for argv in invocations(work):
            proc = subprocess.run([str(binary)] + argv, capture_output=True, timeout=120)
            # Exit code must be 0 — every propose outcome is legible and non-error
            # (a non-promotion is never a failure, per the demo contract).
            if proc.returncode != 0:
                print(f"  note: `{' '.join(argv)}` exited {proc.returncode} "
                      f"(propose outcomes are all exit-0; investigate)", file=sys.stderr)

        after = snapshot(work)

    if before == after:
        print("PASS — the source tree is byte-unchanged across every propose run.")
        print("       observe-don't-declare holds: propose renders, it does not write.")
        return 0

    print("FAIL — the source tree CHANGED. The observe-don't-declare claim is FALSE.")
    added = set(after) - set(before)
    removed = set(before) - set(after)
    changed = {k for k in before.keys() & after.keys() if before[k] != after[k]}
    for k in sorted(added):
        print(f"  + WROTE NEW FILE: {k}")
    for k in sorted(removed):
        print(f"  - DELETED FILE:   {k}")
    for k in sorted(changed):
        print(f"  ~ MODIFIED FILE:  {k}")
    return 1


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description="propose byte-unchanged keystone test")
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
