#!/usr/bin/env python3
"""
Semantic Version Tag Generator and Release Notes Renderer for WaDesk CI/CD.
Reads base version from Cargo.toml and bumps patch automatically based on existing git tags.
"""

import argparse
import os
import re
import subprocess
import sys

def get_next_version() -> str:
    # 1. Read base version from Cargo.toml
    cargo_path = os.path.join(os.path.dirname(__file__), "..", "..", "Cargo.toml")
    if not os.path.exists(cargo_path):
        cargo_path = "Cargo.toml"
        
    with open(cargo_path, "r", encoding="utf-8") as f:
        cargo_content = f.read()

    match = re.search(r'version\s*=\s*"([^"]+)"', cargo_content)
    if not match:
        raise ValueError("Could not find package version in Cargo.toml")

    version_str = match.group(1).strip()
    parts = version_str.split(".")
    if len(parts) != 3:
        raise ValueError(f"Version {version_str} is not valid SemVer (MAJOR.MINOR.PATCH)")

    major, minor, patch = int(parts[0]), int(parts[1]), int(parts[2])

    # 2. Get existing git tags
    try:
        raw_tags = subprocess.check_output(
            ["git", "tag", "-l"],
            stderr=subprocess.DEVNULL
        ).decode("utf-8", errors="ignore").splitlines()
    except Exception:
        raw_tags = []

    # 3. Find semver tags matching current major.minor
    pattern = re.compile(rf"^v{major}\.{minor}\.(\d+)$")
    existing_patches = []
    for tag in raw_tags:
        m = pattern.match(tag.strip())
        if m:
            existing_patches.append(int(m.group(1)))

    # 4. Calculate next clean SemVer tag
    if not existing_patches:
        return f"v{major}.{minor}.{patch}"

    highest_patch = max(existing_patches)
    if patch > highest_patch:
        next_patch = patch
    else:
        next_patch = highest_patch + 1

    return f"v{major}.{minor}.{next_patch}"

def render_release_notes(version: str, output_path: str):
    template_path = os.path.join(os.path.dirname(__file__), "..", "release_template.md")
    if not os.path.exists(template_path):
        raise FileNotFoundError(f"Template not found at {template_path}")

    with open(template_path, "r", encoding="utf-8") as f:
        template = f.read()

    rendered = template.replace("{{VERSION}}", version)
    with open(output_path, "w", encoding="utf-8") as f:
        f.write(rendered)
    print(f"Rendered release notes for {version} to {output_path}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="SemVer Tag and Release Helper")
    parser.add_argument("--render-notes", help="Path to write rendered release notes")
    parser.add_argument("--version", help="Explicit version to use for rendering")
    args = parser.parse_args()

    if args.render_notes:
        target_version = args.version or get_next_version()
        render_release_notes(target_version, args.render_notes)
    else:
        print(get_next_version())
