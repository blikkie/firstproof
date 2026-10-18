#!/usr/bin/env python3
"""
Atomic multi-ecosystem version bumper for First Proof.
Synchronizes Rust (Cargo), Apple (Info.plist), Web (package.json, index.html),
Makefile, and README.md.
"""

import sys
import re
from pathlib import Path
import subprocess

ROOT_DIR = Path(__file__).resolve().parent.parent

def get_current_version() -> str:
    cargo_toml = (ROOT_DIR / "Cargo.toml").read_text(encoding="utf-8")
    m = re.search(r'\[workspace\.package\][\s\S]*?version\s*=\s*"([^"]+)"', cargo_toml)
    if not m:
        raise ValueError("Could not find [workspace.package] version in root Cargo.toml")
    return m.group(1)

def parse_semver(v: str):
    m = re.match(r"^(\d+)\.(\d+)\.(\d+)$", v)
    if not m:
        raise ValueError(f"Version '{v}' does not match semver format X.Y.Z")
    return int(m.group(1)), int(m.group(2)), int(m.group(3))

def calculate_next_version(current: str, bump_type_or_version: str) -> str:
    major, minor, patch = parse_semver(current)
    if bump_type_or_version == "patch":
        return f"{major}.{minor}.{patch + 1}"
    elif bump_type_or_version == "minor":
        return f"{major}.{minor + 1}.0"
    elif bump_type_or_version == "major":
        return f"{major + 1}.0.0"
    else:
        parse_semver(bump_type_or_version) # Validate
        return bump_type_or_version

def update_file(path: Path, transform_fn):
    content = path.read_text(encoding="utf-8")
    new_content = transform_fn(content)
    if content != new_content:
        path.write_text(new_content, encoding="utf-8")
        print(f"  ✓ Updated {path.relative_to(ROOT_DIR)}")
    else:
        print(f"  - No changes in {path.relative_to(ROOT_DIR)}")

def bump_all(next_version: str):
    current = get_current_version()
    if current == next_version:
        print(f"Version is already {current}. Nothing to do.")
        return

    print(f"Bumping First Proof: v{current} -> v{next_version}")

    # 1. Root Cargo.toml
    def root_cargo(txt):
        return re.sub(
            r'(\[workspace\.package\][\s\S]*?version\s*=\s*)"([^"]+)"',
            rf'\g<1>"{next_version}"',
            txt,
            count=1
        )
    update_file(ROOT_DIR / "Cargo.toml", root_cargo)

    # 2. Member crates Cargo.toml
    member_crates = ["firstproof-core", "firstproof-engine", "firstproof-wasm"]
    for crate in member_crates:
        crate_toml = ROOT_DIR / crate / "Cargo.toml"
        def crate_fn(txt):
            return re.sub(
                r'(\[package\][\s\S]*?version\s*=\s*)"([^"]+)"',
                rf'\g<1>"{next_version}"',
                txt,
                count=1
            )
        update_file(crate_toml, crate_fn)

    # 3. FirstProofApp Info.plist
    plist_path = ROOT_DIR / "FirstProofApp" / "Resources" / "Info.plist"
    def plist_fn(txt):
        # Update CFBundleShortVersionString
        res = re.sub(
            r'(<key>CFBundleShortVersionString</key>\s*<string>)[^<]+(</string>)',
            rf'\g<1>{next_version}\g<2>',
            txt
        )
        # Increment CFBundleVersion integer build number
        m = re.search(r'(<key>CFBundleVersion</key>\s*<string>)(\d+)(</string>)', res)
        if m:
            build_num = int(m.group(2)) + 1
            res = res[:m.start(2)] + str(build_num) + res[m.end(2):]
        return res
    update_file(plist_path, plist_fn)

    # 4. Makefile
    def makefile_fn(txt):
        return re.sub(
            r'^(VERSION\s*\?=\s*).*$',
            rf'\g<1>{next_version}',
            txt,
            flags=re.MULTILINE
        )
    update_file(ROOT_DIR / "Makefile", makefile_fn)

    # 5. web/package.json
    web_pkg = ROOT_DIR / "web" / "package.json"
    def web_pkg_fn(txt):
        return re.sub(
            r'("version"\s*:\s*)"[^"]+"',
            rf'\g<1>"{next_version}"',
            txt,
            count=1
        )
    update_file(web_pkg, web_pkg_fn)

    # 6. web/index.html
    web_html = ROOT_DIR / "web" / "index.html"
    def web_html_fn(txt):
        # Replace occurrences of current version in index.html
        res = txt.replace(f"v{current}", f"v{next_version}")
        res = res.replace(f"Version {current}", f"Version {next_version}")
        res = res.replace(f"/{current}/", f"/{next_version}/")
        res = res.replace(f"-{current}-", f"-{next_version}-")
        return res
    update_file(web_html, web_html_fn)

    # 7. README.md
    readme_path = ROOT_DIR / "README.md"
    def readme_fn(txt):
        return txt.replace(f"-{current}-", f"-{next_version}-")
    update_file(readme_path, readme_fn)

    # 8. Sync Cargo.lock
    print("  Syncing Cargo.lock...")
    subprocess.run(
        ["cargo", "check", "--quiet", "--workspace"],
        cwd=ROOT_DIR,
        check=True
    )
    print("  ✓ Cargo.lock synced")

    print(f"\n✅ Successfully bumped to v{next_version} across all files.")

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: bump_version.py <patch|minor|major|X.Y.Z>")
        sys.exit(1)

    arg = sys.argv[1].strip()
    curr = get_current_version()
    target = calculate_next_version(curr, arg)
    bump_all(target)
