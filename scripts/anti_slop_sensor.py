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
        ROOT_DIR / "firstproof-engine" / "shaders",
    ]
    pattern = re.compile(r"\+\s*0\.016|\+\s*0\.035|\bbase_?refl\b", re.IGNORECASE)
    
    for target in targets:
        if not target.exists():
            continue
        for path in target.rglob("*"):
            if path.suffix in [".metal", ".ts", ".rs", ".wgsl"]:

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

def check_inspector_undo_handling():
    """Ensure Inspector views do not bind directly to '$viewModel.<property>' without undo handling."""
    inspector_dir = ROOT_DIR / "FirstProofApp" / "Sources" / "Views" / "Inspector"
    if not inspector_dir.exists():
        return []
        
    violations = []
    pattern = re.compile(r"Toggle\([^,]+,\s*isOn:\s*\$viewModel\.[a-zA-Z0-9_]+")
    
    for path in inspector_dir.rglob("*.swift"):
        try:
            content = path.read_text(encoding="utf-8")
            for idx, line in enumerate(content.splitlines(), start=1):
                if pattern.search(line):
                    violations.append(
                        f"{path.relative_to(ROOT_DIR)}:{idx} -> Direct $viewModel toggle binding bypasses darkroom Undo/Redo history! Use viewModel.editToggleBinding(...) or a dedicated viewModel.toggle...() action: {line.strip()}"
                    )
        except Exception as e:
            violations.append(f"Could not read {path}: {e}")
            
    return violations

def check_edit_settings_parity():
    """Ensure EditSettings fields in Rust have 1:1 parity with currentEdits and applyEdits in Swift."""
    settings_rs = ROOT_DIR / "firstproof-engine" / "src" / "settings.rs"
    editor_swift = ROOT_DIR / "FirstProofApp" / "Sources" / "ViewModels" / "AppViewModel+Editor.swift"
    if not settings_rs.exists() or not editor_swift.exists():
        return []

    violations = []
    try:
        content_rs = settings_rs.read_text(encoding="utf-8")
        match = re.search(r"pub\s+struct\s+EditSettings\s*\{([^}]+)\}", content_rs)
        if not match:
            return ["Could not find EditSettings struct in settings.rs"]
            
        rust_fields = []
        for line in match.group(1).splitlines():
            line = line.strip()
            if line.startswith("pub ") and ":" in line:
                field_name = line.split()[1].rstrip(":")
                rust_fields.append(field_name)

        def to_camel(s):
            parts = s.split("_")
            return parts[0] + "".join(p.capitalize() for p in parts[1:])

        content_swift = editor_swift.read_text(encoding="utf-8")
        
        # Extract the entire body of currentEdits using balanced braces
        ce_idx = content_swift.find("public var currentEdits: EditSettings {")
        if ce_idx == -1:
            return ["Could not find currentEdits in AppViewModel+Editor.swift"]
        brace_start = content_swift.find("{", ce_idx)
        brace_count = 1
        i = brace_start + 1
        while i < len(content_swift) and brace_count > 0:
            if content_swift[i] == "{":
                brace_count += 1
            elif content_swift[i] == "}":
                brace_count -= 1
            i += 1
        current_edits_text = content_swift[brace_start:i]

        # Extract the entire body of applyEdits using balanced braces
        ae_idx = content_swift.find("public func applyEdits(_ edits: EditSettings) {")
        if ae_idx == -1:
            return ["Could not find applyEdits in AppViewModel+Editor.swift"]
        brace_start = content_swift.find("{", ae_idx)
        brace_count = 1
        i = brace_start + 1
        while i < len(content_swift) and brace_count > 0:
            if content_swift[i] == "{":
                brace_count += 1
            elif content_swift[i] == "}":
                brace_count -= 1
            i += 1
        apply_edits_text = content_swift[brace_start:i]

        for r_field in rust_fields:
            c_field = to_camel(r_field)
            if c_field not in current_edits_text:
                violations.append(f"EditSettings.{r_field} ({c_field}) is missing from AppViewModel.currentEdits in AppViewModel+Editor.swift")
            if f"edits.{c_field}" not in apply_edits_text and c_field not in apply_edits_text:
                violations.append(f"EditSettings.{r_field} ({c_field}) is not restored by AppViewModel.applyEdits in AppViewModel+Editor.swift")

        # 1. EditDelta in firstproof-engine/src/command.rs
        command_rs = ROOT_DIR / "firstproof-engine" / "src" / "command.rs"
        if command_rs.exists():
            content_command = command_rs.read_text(encoding="utf-8")
            delta_match = re.search(r"pub\s+struct\s+EditDelta\s*\{([^}]+)\}", content_command)
            if delta_match:
                delta_body = delta_match.group(1)
                delta_fields = set()
                for line in delta_body.splitlines():
                    line = line.strip()
                    if line.startswith("pub ") and ":" in line:
                        delta_fields.add(line.split()[1].rstrip(":"))
                for r_field in rust_fields:
                    if r_field not in delta_fields:
                        violations.append(f"EditSettings.{r_field} is missing from EditDelta in command.rs")
            else:
                violations.append("Could not find EditDelta struct in command.rs")

        # 2. EditSettingsPayload in web/src/types.ts
        web_types_ts = ROOT_DIR / "web" / "src" / "types.ts"
        if web_types_ts.exists():
            content_web_types = web_types_ts.read_text(encoding="utf-8")
            types_match = re.search(r"export\s+interface\s+EditSettingsPayload\s*\{([^}]+)\}", content_web_types)
            if types_match:
                types_body = types_match.group(1)
                payload_fields = set()
                for line in types_body.splitlines():
                    line = line.strip()
                    if ":" in line:
                        field_name = line.split(":")[0].rstrip("?").strip()
                        payload_fields.add(field_name)
                for r_field in rust_fields:
                    if r_field not in payload_fields:
                        violations.append(f"EditSettings.{r_field} is missing from EditSettingsPayload in web/src/types.ts")
            else:
                violations.append("Could not find EditSettingsPayload interface in web/src/types.ts")

        # 3. getDefaultEdits in web/src/utils/edits.ts
        web_edits_ts = ROOT_DIR / "web" / "src" / "utils" / "edits.ts"
        if web_edits_ts.exists():
            content_web_edits = web_edits_ts.read_text(encoding="utf-8")
            gde_idx = content_web_edits.find("export function getDefaultEdits(")
            if gde_idx != -1:
                brace_start = content_web_edits.find("{", gde_idx)
                brace_count = 1
                i = brace_start + 1
                while i < len(content_web_edits) and brace_count > 0:
                    if content_web_edits[i] == "{":
                        brace_count += 1
                    elif content_web_edits[i] == "}":
                        brace_count -= 1
                    i += 1
                gde_body = content_web_edits[brace_start:i]
                for r_field in rust_fields:
                    if not re.search(rf"\b{re.escape(r_field)}\s*:", gde_body):
                        violations.append(f"EditSettings.{r_field} is missing from getDefaultEdits in web/src/utils/edits.ts")
            else:
                violations.append("Could not find getDefaultEdits in web/src/utils/edits.ts")

    except Exception as e:
        violations.append(f"Failed to check EditSettings parity: {e}")

    return violations

def check_xmp_sidecar_parity():
    """Ensure every EditSettings field in Rust is serialized and deserialized in XMP sidecars."""
    settings_rs = ROOT_DIR / "firstproof-engine" / "src" / "settings.rs"
    xmp_rs = ROOT_DIR / "firstproof-engine" / "src" / "xmp.rs"
    if not settings_rs.exists() or not xmp_rs.exists():
        return []

    violations = []
    try:
        content_settings = settings_rs.read_text(encoding="utf-8")
        match = re.search(r"pub\s+struct\s+EditSettings\s*\{([^}]+)\}", content_settings)
        if not match:
            return ["Could not find EditSettings struct in settings.rs"]

        rust_fields = []
        for line in match.group(1).splitlines():
            line = line.strip()
            if line.startswith("pub ") and ":" in line:
                field_name = line.split()[1].rstrip(":")
                rust_fields.append(field_name)

        content_xmp = xmp_rs.read_text(encoding="utf-8")

        # Extract generate_xmp_sidecar body
        gen_match = re.search(r"pub fn generate_xmp_sidecar\([^)]*\)\s*->\s*String\s*\{", content_xmp)
        if not gen_match:
            return ["Could not find generate_xmp_sidecar in xmp.rs"]
        brace_start = gen_match.end() - 1
        brace_count = 1
        i = brace_start + 1
        while i < len(content_xmp) and brace_count > 0:
            if content_xmp[i] == "{":
                brace_count += 1
            elif content_xmp[i] == "}":
                brace_count -= 1
            i += 1
        gen_text = content_xmp[brace_start:i]

        # Extract parse_xmp_sidecar body
        parse_match = re.search(r"pub fn parse_xmp_sidecar\([^)]*\)\s*->\s*Result<XmpData,\s*String>\s*\{", content_xmp)
        if not parse_match:
            return ["Could not find parse_xmp_sidecar in xmp.rs"]
        brace_start = parse_match.end() - 1
        brace_count = 1
        i = brace_start + 1
        while i < len(content_xmp) and brace_count > 0:
            if content_xmp[i] == "{":
                brace_count += 1
            elif content_xmp[i] == "}":
                brace_count -= 1
            i += 1
        parse_text = content_xmp[brace_start:i]

        for r_field in rust_fields:
            # Check serialization
            if f"data.edits.{r_field}" not in gen_text:
                violations.append(f"EditSettings.{r_field} is NOT serialized into XMP in firstproof-engine/src/xmp.rs generate_xmp_sidecar")
            # Check deserialization
            if f"data.edits.{r_field} =" not in parse_text and f"data.edits.{r_field}." not in parse_text:
                violations.append(f"EditSettings.{r_field} is NOT deserialized from XMP in firstproof-engine/src/xmp.rs parse_xmp_sidecar")

    except Exception as e:
        violations.append(f"Failed to check XMP sidecar parity: {e}")

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

    inspector_violations = check_inspector_undo_handling()
    if inspector_violations:
        all_violations.extend(["[Inspector Undo/Redo Missing Handling Violations]", *inspector_violations])

    parity_violations = check_edit_settings_parity()
    if parity_violations:
        all_violations.extend(["[EditSettings Darkroom History Parity Violations]", *parity_violations])

    xmp_violations = check_xmp_sidecar_parity()
    if xmp_violations:
        all_violations.extend(["[XMP Sidecar EditSettings Parity Violations]", *xmp_violations])

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
