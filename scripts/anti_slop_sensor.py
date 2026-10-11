#!/usr/bin/env python3
"""
First Proof Architectural Anti-Slop & Invariant Sensor
Harness Engineering: Agent = Model + Harness (Martin Fowler)

Deterministically enforces core architectural, color science, and typing invariants:
1. Anti-Hedge Invariant: No speculative 'typeof wasm.* === function' or shims on first-party WASM exports.
2. Boundary Anchoring Invariant: No additive floor lifts (+ 0.016, + 0.035) in develop/tone recovery shaders.
3. Strict Typing Gate: web/biome.json must maintain "noExplicitAny": "error".
4. Swift FFI Error Swallowing: No 'try? FirstProofCore' or 'try? FirstProofDatabase' silently dropping Rust errors.
"""

import sys
import re
import os
import subprocess
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parent.parent

def check_wasm_hedges():
    """Ensure web/src does not contain speculative 'typeof wasm.* === function' guards."""
    web_src = ROOT_DIR / "web" / "src"
    if not web_src.exists():
        return []
    
    violations = []
    pattern = re.compile(r"typeof\s+(wasm|wasmExt)\.[a-zA-Z0-9_]+\s*===?\s*['\"]function['\"]")
    pattern_ext = re.compile(r"wasmExt\.")
    
    for path in web_src.rglob("*.ts"):
        # Allow test mocks if any
        if "__tests__" in str(path):
            continue
        try:
            content = path.read_text(encoding="utf-8")
            for idx, line in enumerate(content.splitlines(), start=1):
                if pattern.search(line):
                    violations.append(f"{path.relative_to(ROOT_DIR)}:{idx} -> Speculative first-party WASM function check: {line.strip()}")
                elif pattern_ext.search(line):
                    violations.append(f"{path.relative_to(ROOT_DIR)}:{idx} -> Forbidden wasmExt shim usage: {line.strip()}")
        except Exception as e:
            violations.append(f"Could not read {path}: {e}")
            
    return violations

def check_boundary_anchoring():
    """Ensure no additive floor lifts (+ 0.016, + 0.035) in tone shaders."""
    violations = []
    targets = [
        ROOT_DIR / "FirstProofApp" / "Sources" / "Shaders",
        ROOT_DIR / "web" / "src" / "shaders",
        ROOT_DIR / "firstproof-engine" / "src",
    ]
    pattern = re.compile(r"\+\s*0\.016|\+\s*0\.035|\bbase_?refl\b", re.IGNORECASE)
    
    for target in targets:
        if not target.exists():
            continue
        for path in target.rglob("*"):
            if path.suffix in [".metal", ".ts", ".rs"]:
                try:
                    content = path.read_text(encoding="utf-8")
                    for idx, line in enumerate(content.splitlines(), start=1):
                        if pattern.search(line) and not "// allow-floor-lift" in line:
                            violations.append(f"{path.relative_to(ROOT_DIR)}:{idx} -> Additive floor lift violates f(0.0) == 0.0 boundary invariant: {line.strip()}")
                except Exception as e:
                    violations.append(f"Could not read {path}: {e}")
                    
    return violations

def check_biome_strictness():
    """Ensure web/biome.json strictly configures noExplicitAny: error."""
    biome_path = ROOT_DIR / "web" / "biome.json"
    if not biome_path.exists():
        return []
    
    try:
        content = biome_path.read_text(encoding="utf-8")
        if '"noExplicitAny": "error"' not in content:
            return [f"web/biome.json must maintain 'noExplicitAny': 'error'"]
    except Exception as e:
        return [f"Could not read {biome_path}: {e}"]
        
    return []

def check_swift_ffi_swallowing():
    """Ensure Swift callers do not silently drop Rust UniFFI errors via 'try? FirstProof'."""
    swift_src = ROOT_DIR / "FirstProofApp" / "Sources"
    if not swift_src.exists():
        return []
        
    violations = []
    pattern = re.compile(r"try\?\s+FirstProof(Database|Core)")
    
    for path in swift_src.rglob("*.swift"):
        try:
            content = path.read_text(encoding="utf-8")
            for idx, line in enumerate(content.splitlines(), start=1):
                if pattern.search(line):
                    violations.append(f"{path.relative_to(ROOT_DIR)}:{idx} -> Swift swallows Rust FFI error via try?: {line.strip()}")
        except Exception as e:
            violations.append(f"Could not read {path}: {e}")
            
    return violations

def check_no_liquid_glass():
    """Ensure views do not use translucent liquid glass / refractive materials that bleed background radiance."""
    violations = []
    
    # 1. Swift Views: No ultraThinMaterial / thinMaterial
    swift_views = ROOT_DIR / "FirstProofApp" / "Sources" / "Views"
    if swift_views.exists():
        pattern_swift = re.compile(r"(\.ultraThinMaterial|Material\.ultraThinMaterial|\.thinMaterial|Material\.thinMaterial)")
        for path in swift_views.rglob("*.swift"):
            try:
                content = path.read_text(encoding="utf-8")
                for idx, line in enumerate(content.splitlines(), start=1):
                    if pattern_swift.search(line):
                        violations.append(f"{path.relative_to(ROOT_DIR)}:{idx} -> Translucent liquid glass violates neutral chromatic adaptation: {line.strip()}")
            except Exception as e:
                violations.append(f"Could not read {path}: {e}")
                
    # 2. Web CSS: No backdrop-filter blur
    web_src = ROOT_DIR / "web" / "src"
    if web_src.exists():
        pattern_web = re.compile(r"backdrop-filter:\s*blur")
        for path in web_src.rglob("*.css"):
            try:
                content = path.read_text(encoding="utf-8")
                for idx, line in enumerate(content.splitlines(), start=1):
                    if pattern_web.search(line):
                        violations.append(f"{path.relative_to(ROOT_DIR)}:{idx} -> Web backdrop-filter blur violates darkroom matte slate standard: {line.strip()}")
            except Exception as e:
                violations.append(f"Could not read {path}: {e}")
                
    return violations

def main():
    all_violations = []
    
    wasm_violations = check_wasm_hedges()
    if wasm_violations:
        all_violations.extend(["[Anti-Hedge Wasm Violations]", *wasm_violations])
        
    boundary_violations = check_boundary_anchoring()
    if boundary_violations:
        all_violations.extend(["[Boundary Anchoring Violations]", *boundary_violations])
        
    biome_violations = check_biome_strictness()
    if biome_violations:
        all_violations.extend(["[Biome Strictness Violations]", *biome_violations])
        
    ffi_violations = check_swift_ffi_swallowing()
    if ffi_violations:
        all_violations.extend(["[Swift Rust-FFI Error Swallowing Violations]", *ffi_violations])

    glass_violations = check_no_liquid_glass()
    if glass_violations:
        all_violations.extend(["[Liquid Glass & Translucency Violations]", *glass_violations])

    if all_violations:
        print("❌ Architectural Anti-Slop & Invariant Sensor FAILED:")
        for v in all_violations:
            print(f"  {v}")
        return 1
    else:
        print("✅ Architectural Anti-Slop & Invariant Sensor: ALL INVARIANTS PASSED")
        return 0

if __name__ == "__main__":
    sys.exit(main())
