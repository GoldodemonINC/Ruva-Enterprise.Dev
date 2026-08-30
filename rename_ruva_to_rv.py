#!/usr/bin/env python3
"""Rename all `.ruva` source files to `.rve` (recursively).

The Ruva toolchain accepts `.rve` as a source extension, so once these are
renamed everything still compiles, runs, and parses identically.

Usage:
    python rename_ruva_to_rv.py                 # dry-run preview (no changes)
    python rename_ruva_to_rv.py --apply         # actually rename files
    python rename_ruva_to_rv.py --root DIR      # start from DIR instead of repo root
    python rename_ruva_to_rv.py --apply --root DIR

Safeguards:
  * Skips build/vendored directories (.git, target, node_modules, .freebuff).
  * Never overwrites an existing `.rve` file (skips with a warning).
  * Defaults to a dry run so you can see exactly what would change first.
"""

import argparse
import os
import sys

# The target extension for renamed files.
NEW_EXT = ".rve"

# Directories we never rename inside, regardless of depth.
SKIP_DIRS = {".git", ".hg", ".svn", "target", "node_modules", ".freebuff", "vendor"}


def iter_candidates(root: str):
    """Yield absolute paths of every `.ruva` file under `root` (case-insensitive)."""
    for dirpath, dirnames, filenames in os.walk(root):
        # Mutate in place so os.walk does not descend into skipped dirs.
        dirnames[:] = [d for d in dirnames if d not in SKIP_DIRS]
        for fname in filenames:
            if fname.lower().endswith(".ruva"):
                yield os.path.join(dirpath, fname)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--root",
        default=os.path.dirname(os.path.abspath(__file__)),
        help="Root directory to scan (default: this script's directory).",
    )
    parser.add_argument(
        "--apply",
        action="store_true",
        help="Actually rename files. Without this the script only previews.",
    )
    args = parser.parse_args()

    root = os.path.abspath(args.root)
    if not os.path.isdir(root):
        print(f"error: root is not a directory: {root}", file=sys.stderr)
        return 1

    files = sorted(iter_candidates(root))
    if not files:
        print(f"No `.ruva` files found under {root}")
        return 0

    renamed = 0
    skipped = 0
    for src in files:
        stem = src[: -len(".ruva")]  # strip the ".ruva" suffix
        dst = stem + NEW_EXT
        if os.path.exists(dst):
            print(f"SKIP  {rel(root, src)}  (would overwrite existing {rel(root, dst)})")
            skipped += 1
            continue
        if args.apply:
            os.rename(src, dst)
            print(f"RENAMED {rel(root, src)}  ->  {rel(root, dst)}")
        else:
            print(f"  {rel(root, src)}  ->  {rel(root, dst)}")
        renamed += 1

    print("")
    print(f"mode:       {'APPLY (files renamed)' if args.apply else 'DRY RUN (no changes)'}")
    print(f"files:      {len(files)} found")
    print(f"renamed:    {renamed}")
    print(f"skipped:    {skipped}")
    if not args.apply:
        print("Re-run with --apply to perform the rename.")

    return 0


def rel(root: str, path: str) -> str:
    try:
        return os.path.relpath(path, root)
    except ValueError:  # different drive on Windows
        return path


if __name__ == "__main__":
    sys.exit(main())