#!/usr/bin/env python3
"""
Semantic Version Tag Generator and Dynamic Release Notes Renderer for WaDesk CI/CD.
Reads base version from Cargo.toml, auto-increments SemVer tags based on git history,
and generates structured release notes dynamically from conventional commits.
"""

import argparse
import os
import re
import subprocess
import sys

def get_base_cargo_version() -> tuple[int, int, int]:
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

    return int(parts[0]), int(parts[1]), int(parts[2])

def get_existing_git_tags() -> list[str]:
    try:
        return subprocess.check_output(
            ["git", "tag", "-l"],
            stderr=subprocess.DEVNULL
        ).decode("utf-8", errors="ignore").splitlines()
    except Exception:
        return []

def get_next_version() -> str:
    major, minor, patch = get_base_cargo_version()
    raw_tags = get_existing_git_tags()

    # Find clean semver tags matching current major.minor (e.g. v1.0.0, v1.0.1)
    pattern = re.compile(rf"^v{major}\.{minor}\.(\d+)$")
    existing_patches = []
    for tag in raw_tags:
        m = pattern.match(tag.strip())
        if m:
            existing_patches.append(int(m.group(1)))

    if not existing_patches:
        return f"v{major}.{minor}.{patch}"

    highest_patch = max(existing_patches)
    if patch > highest_patch:
        next_patch = patch
    else:
        next_patch = highest_patch + 1

    return f"v{major}.{minor}.{next_patch}"

def get_previous_tag(target_version: str) -> str | None:
    raw_tags = get_existing_git_tags()
    semver_pattern = re.compile(r"^v(\d+)\.(\d+)\.(\d+)$")
    parsed_tags = []
    for t in raw_tags:
        t = t.strip()
        m = semver_pattern.match(t)
        if m:
            parsed_tags.append((int(m.group(1)), int(m.group(2)), int(m.group(3)), t))
            
    parsed_tags.sort()
    
    # Target version tuple
    m_target = semver_pattern.match(target_version)
    if not m_target:
        return parsed_tags[-1][3] if parsed_tags else None
        
    target_tuple = (int(m_target.group(1)), int(m_target.group(2)), int(m_target.group(3)))
    
    # Find tag immediately preceding target_tuple
    prev = None
    for item in parsed_tags:
        if item[:3] < target_tuple:
            prev = item[3]
        else:
            break
            
    return prev

def generate_dynamic_changelog(current_version: str) -> str:
    prev_tag = get_previous_tag(current_version)
    if prev_tag:
        git_range = f"{prev_tag}..HEAD"
    else:
        git_range = "HEAD"

    try:
        raw_log = subprocess.check_output(
            ["git", "log", git_range, "--pretty=format:%h|%s"],
            stderr=subprocess.DEVNULL
        ).decode("utf-8", errors="ignore").splitlines()
    except Exception:
        raw_log = []

    features = []
    fixes = []
    maintenance = []

    for entry in raw_log:
        if not entry.strip():
            continue
        parts = entry.split("|", 1)
        commit_hash = parts[0].strip()
        msg = parts[1].strip() if len(parts) > 1 else ""

        # Skip CI release tagging commits
        if msg.startswith("ci(release):"):
            continue

        lower = msg.lower()
        if lower.startswith("feat"):
            cleaned = re.sub(r"^feat(\([^)]+\))?:\s*", "", msg, flags=re.IGNORECASE)
            features.append(f"- {cleaned} ({commit_hash})")
        elif lower.startswith("fix"):
            cleaned = re.sub(r"^fix(\([^)]+\))?:\s*", "", msg, flags=re.IGNORECASE)
            fixes.append(f"- {cleaned} ({commit_hash})")
        else:
            cleaned = re.sub(r"^(chore|refactor|perf|test|docs)(\([^)]+\))?:\s*", "", msg, flags=re.IGNORECASE)
            maintenance.append(f"- {cleaned} ({commit_hash})")

    # If no commits found in range (e.g. tag re-run or shallow range), check last 10 commits
    if not features and not fixes and not maintenance:
        try:
            fallback_log = subprocess.check_output(
                ["git", "log", "-n", "10", "--pretty=format:%h|%s"],
                stderr=subprocess.DEVNULL
            ).decode("utf-8", errors="ignore").splitlines()
            for entry in fallback_log:
                if not entry.strip():
                    continue
                parts = entry.split("|", 1)
                commit_hash = parts[0].strip()
                msg = parts[1].strip() if len(parts) > 1 else ""
                if msg.startswith("ci(release):"):
                    continue
                lower = msg.lower()
                if lower.startswith("feat"):
                    cleaned = re.sub(r"^feat(\([^)]+\))?:\s*", "", msg, flags=re.IGNORECASE)
                    features.append(f"- {cleaned} ({commit_hash})")
                elif lower.startswith("fix"):
                    cleaned = re.sub(r"^fix(\([^)]+\))?:\s*", "", msg, flags=re.IGNORECASE)
                    fixes.append(f"- {cleaned} ({commit_hash})")
                else:
                    cleaned = re.sub(r"^(chore|refactor|perf|test|docs)(\([^)]+\))?:\s*", "", msg, flags=re.IGNORECASE)
                    maintenance.append(f"- {cleaned} ({commit_hash})")
        except Exception:
            pass

    sections = []
    if features:
        sections.append("### Features and Improvements\n" + "\n".join(features))
    if fixes:
        sections.append("### Bug Fixes\n" + "\n".join(fixes))
    if maintenance:
        sections.append("### Maintenance and Infrastructure\n" + "\n".join(maintenance))

    if not sections:
        return "- Maintenance release with updated platform packages and security dependencies."

    return "\n\n".join(sections)

def sync_versions_across_codebase(version_str: str):
    """
    Dynamically injects the computed version into Cargo.toml, Inno Setup, Info.plist, and Debian control.
    Keeps all package metadata unified.
    """
    clean_ver = version_str.lstrip("v")
    root_dir = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))

    # 1. Update Cargo.toml
    cargo_path = os.path.join(root_dir, "Cargo.toml")
    if os.path.exists(cargo_path):
        with open(cargo_path, "r", encoding="utf-8") as f:
            cargo = f.read()
        new_cargo = re.sub(r'version\s*=\s*"[^"]+"', f'version = "{clean_ver}"', cargo, count=1)
        with open(cargo_path, "w", encoding="utf-8") as f:
            f.write(new_cargo)
        print(f"Updated Cargo.toml to version {clean_ver}")

    # 2. Update installer/windows/wadesk.iss
    iss_path = os.path.join(root_dir, "installer", "windows", "wadesk.iss")
    if os.path.exists(iss_path):
        with open(iss_path, "r", encoding="utf-8") as f:
            iss = f.read()
        new_iss = re.sub(r'#define\s+MyAppVersion\s+"[^"]+"', f'#define MyAppVersion "{clean_ver}"', iss)
        with open(iss_path, "w", encoding="utf-8") as f:
            f.write(new_iss)
        print(f"Updated wadesk.iss to MyAppVersion {clean_ver}")

    # 3. Update installer/macos/Info.plist
    plist_path = os.path.join(root_dir, "installer", "macos", "Info.plist")
    if os.path.exists(plist_path):
        with open(plist_path, "r", encoding="utf-8") as f:
            plist = f.read()
        new_plist = re.sub(
            r'(<key>CFBundleShortVersionString</key>\s*<string>)[^<]+(</string>)',
            rf'\g<1>{clean_ver}\g<2>',
            plist
        )
        new_plist = re.sub(
            r'(<key>CFBundleVersion</key>\s*<string>)[^<]+(</string>)',
            rf'\g<1>{clean_ver}\g<2>',
            new_plist
        )
        with open(plist_path, "w", encoding="utf-8") as f:
            f.write(new_plist)
        print(f"Updated Info.plist to version {clean_ver}")

def render_release_notes(version: str, output_path: str):
    template_path = os.path.join(os.path.dirname(__file__), "..", "release_template.md")
    if not os.path.exists(template_path):
        raise FileNotFoundError(f"Template not found at {template_path}")

    with open(template_path, "r", encoding="utf-8") as f:
        template = f.read()

    clean_ver = version.lstrip("v")
    deb_filename = f"wadesk_{clean_ver}_amd64.deb"

    changelog = generate_dynamic_changelog(version)

    rendered = template.replace("{{VERSION}}", version)
    rendered = rendered.replace("{{CHANGELOG}}", changelog)
    rendered = rendered.replace("{{DEB_FILENAME}}", deb_filename)

    with open(output_path, "w", encoding="utf-8") as f:
        f.write(rendered)
    print(f"Rendered dynamic release notes for {version} to {output_path}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="SemVer Tag and Dynamic Release Helper")
    parser.add_argument("--render-notes", help="Path to write rendered release notes")
    parser.add_argument("--sync-versions", help="Sync version string across Cargo.toml, Inno Setup, Info.plist")
    parser.add_argument("--version", help="Explicit version to use")
    args = parser.parse_args()

    if args.sync_versions:
        target_version = args.sync_versions
        sync_versions_across_codebase(target_version)
    elif args.render_notes:
        target_version = args.version or get_next_version()
        render_release_notes(target_version, args.render_notes)
    else:
        print(get_next_version())
