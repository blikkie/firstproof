# Architecture Decision Records (ADRs)

This directory documents the foundational architectural decisions for First Proof. Each record captures the context, considered options, decision rationale, and accepted trade-offs for a non-trivial choice.

---

## ⚡ Agent Token-Defense & Usage Protocol
1. **Never read all ADR files indiscriminately**: Reading every ADR will rapidly deplete your token context.
2. **Read the Index Table first**: When working on an existing subsystem, consult only the table below to check if a relevant ADR exists.
3. **Targeted Reading**: If (and only if) an ADR directly impacts your current task, read that single file (`docs/adr/NNNN-....md`).
4. **When to Record a New ADR**: When proposing or implementing a significant architectural choice (new technology, serialization format, inter-process protocol, or rejecting an obvious convention), author a new ADR using [`template.md`](template.md), assign the next sequential number, and register it in the table below.

---

## ADR Index

| Number | Title | Status | Date | Subsystems / Scope | Summary |
| :--- | :--- | :--- | :--- | :--- | :--- |
| [0001](0001-record-architecture-decisions.md) | Record Architecture Decisions | Accepted | 2026-11-03 | All / Docs / Agents | Establishes lightweight ADR practice with token-defense rules for agents. |
| [0002](0002-linear-rec2020-working-gamut-and-terminal-rgb-encoding.md) | Linear Rec.2020 Gamut & Terminal RGB | Accepted | 2026-03-15 | Color Science, Engine, Shaders | Develop pipeline runs in Linear Rec.2020; display conversion is strictly terminal. |
| [0003](0003-canonical-wgsl-naga-transpilation.md) | Canonical WGSL Naga Transpilation | Accepted | 2026-04-10 | Shaders, macOS Metal, WebGL | Author shaders once in WGSL; transpile to MSL and GLSL via Rust Naga CLI. |
| [0004](0004-dual-domain-performance-asymmetry.md) | Dual-Domain Performance Asymmetry | Accepted | 2026-03-20 | Viewport, Engine, Export | 2×2 Bayer box binning for 60 FPS viewport; full float CPU demosaicing for print export. |
| [0005](0005-xmp-sidecars-archival-truth-sqlite-ephemeral-index.md) | XMP Sidecars as Archival Truth | Accepted | 2026-03-25 | Metadata, Storage, Database | Standard `.xmp` files are durable truth; SQLite / localStorage are ephemeral indexes. |

