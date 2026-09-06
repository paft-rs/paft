#!/usr/bin/env python3
"""Check a release tag against an explicit Cargo format-version 1 metadata file.

This module has no Cargo, network, or GitHub environment dependency. Generate
its input with `cargo metadata --locked --no-deps --format-version 1`.
"""

import argparse
import json
from pathlib import Path
import sys


def check_release_tag(metadata: dict, tag: str) -> list[tuple[str, str]]:
    """Return the release set, or reject with discovered package diagnostics."""
    members = set(metadata["workspace_members"])
    selected = sorted(
        (package["name"], package["version"])
        for package in metadata["packages"]
        if package["id"] in members
        # Cargo: null is unrestricted; [] forbids publication; a nonempty list
        # restricts registries. In particular, do not test null for truthiness.
        and (
            package["publish"] is None
            or (isinstance(package["publish"], list) and len(package["publish"]) > 0)
        )
    )
    discovered = "Publishable workspace packages:\n" + (
        "\n".join(f"  {name} {version}" for name, version in selected)
        if selected
        else "  (none)"
    )
    if not selected:
        raise ValueError(f"release set is empty\n{discovered}")
    versions = {version for _, version in selected}
    if len(versions) != 1:
        raise ValueError(f"release package versions differ\n{discovered}")
    expected = "v" + selected[0][1]
    if tag != expected:
        raise ValueError(f"tag {tag!r} does not equal {expected!r}\n{discovered}")
    return selected


def main(argv=None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--metadata", type=Path, required=True)
    parser.add_argument("--tag", required=True)
    args = parser.parse_args(argv)
    try:
        metadata = json.loads(args.metadata.read_text(encoding="utf-8"))
        selected = check_release_tag(metadata, args.tag)
    except (OSError, UnicodeError, ValueError, KeyError, TypeError) as error:
        print(f"Release preflight failed: {error}", file=sys.stderr)
        return 1
    print(f"Release tag {args.tag!r} matches {len(selected)} publishable workspace packages:")
    for name, version in selected:
        print(f"  {name} {version}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
