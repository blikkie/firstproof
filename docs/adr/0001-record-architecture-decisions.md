# ADR-0001: Record Architecture Decisions

- **Status**: Accepted
- **Date**: 2026-11-03
- **Deciders**: blikkie, Antigravity
- **Consulted / Impacted Subsystems**: Entire codebase, agent workflows, documentation

## Context & Problem Statement
As First Proof scales across macOS native, WebGL darkroom, and Rust core, foundational technical choices (color science, shader pipeline, concurrency, storage) must remain stable and understood. Living architecture docs describe *what* the system does today, and invariants state *what not to do*, but neither captures *why* alternatives were rejected or what trade-offs were accepted. Furthermore, LLM agents risk wasting large amounts of context tokens loading composite architecture docs or re-litigating settled decisions.

## Decision Drivers
- **Token efficiency**: Keep agent context tight by providing small, targeted decision records (~50–80 lines) rather than requiring multi-thousand line architecture guides to understand a subsystem.
- **Defense against re-litigation**: Prevent agents and developers from repeatedly proposing previously rejected approaches (e.g. hand-authoring separate Metal shaders, working in sRGB, embedding proprietary edits in SQLite).
- **Immutability and auditability**: Record decisions at the point in time they are made; supercede rather than rewrite history.

## Considered Options
- **Option 1**: Rely only on living architecture docs (`docs/architecture/*.md`) and commit messages.
- **Option 2**: Heavy formal RFC process with extensive template ceremony and review gates.
- **Option 3**: Lightweight Architecture Decision Records (ADRs) stored in `docs/adr/` with an index table and strict token-budget guidelines for agents.

## Decision Outcome
Chosen option: **Option 3 (Lightweight ADRs)**. We maintain numbered markdown files in `docs/adr/NNNN-<slug>.md` indexed in `docs/adr/README.md`.

### Positive Consequences
- Targeted context loading: Agents only read `docs/adr/README.md` first, then load the single specific ADR relevant to the task if needed.
- Clear rationale for rejected alternatives and deliberate trade-offs.
- Seamless onboarding for both humans and AI pairing sessions.

### Negative Consequences & Accepted Trade-offs
- Slight overhead when making significant architectural choices: deciders must remember to author an ADR and update the index table.
- Needs enforcement in agent guidelines and code reviews to avoid drifting or forgetting to log decisions.

## Implementation Links & Code Anchors
- ADR Directory: [docs/adr/](./)
- Template: [template.md](./template.md)
- Index: [README.md](./README.md)

