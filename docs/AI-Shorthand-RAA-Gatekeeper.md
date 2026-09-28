# AI-Shorthand-RAA-Gatekeeper.md

This extends the general AI-Shorthand system (see `docs/AI-Shorthand-Grok-v1.md`) with project-specific tags for RAA-Gatekeeper.

## RAA-Gatekeeper Tag Vocabulary

Use these tags (alone or combined) to activate strict, high-priority focus on the corresponding permanent rules and architecture. The tags act as lightweight but powerful enforcers.

### Core Mission & Philosophy
- `::raa::mission::`
- `::raa::philosophy::`
- `::raa::joe::` (primary user focus)

**When used:** Treat as highest priority. Everything must serve "Joe" (regular end-user who downloads untrusted/AI-generated code). Enforce "Trust, then Certify." The tool is local-first, read-only, privacy-respecting, and advisory-only. Human is always the final decision maker. Never optimize for developer convenience at the expense of end-user protection.

### Permanent Anchors (Non-Negotiable)
- `::raa::permanent::`
- `::raa::anchors::`

**When used:** Strictly enforce all permanent anchors from AGENTS.md:
- Read-Only Mandate (never modify user files)
- Advisory Only (verdicts are diagnostic only)
- Dynamic Base URL (never hard-code any provider)
- Path Protocol (`fs::canonicalize` for all absolute matching)
- Vault pathing respects user `vaultRootPath` (default `~/Documents/RAA_Vault`)
- Junk Filter (`JUNK_NAMES`) must stay perfectly in sync everywhere
- Enduring Principle: *A Certify job must never be allowed to appear successful if any audited file contains a violation.*

### Granular Vault Architecture
- `::raa::vault::`
- `::raa::granular::`
- `::raa::onefile::`

**When used:** Enforce ONE FILE = ONE REPORT model. Every audited file gets its own self-contained `.raa` (full analysis + verdict + DNA/SHA-256). All artifacts live inside a dated Job Folder. Hierarchy mirroring is strongly preferred.

### Control Manifest (Source of Truth)
- `::raa::manifest::`
- `::raa::control::`
- `::raa::manifest::truth::`

**When used:** The `~RAA-CONTROL-Manifest.log` (static name, `~` prefix) is created as the very first artifact on job start. It is the single source of truth for the entire job (inventory, hierarchy, DNA Registry). Per-file `.raa` reports are secondary. Never treat any other file as authoritative for what was audited or what the hashes were.

### Certify vs Archive Separation
- `::raa::certify::`
- `::raa::archive::`
- `::raa::certify::archive::`

**When used:** Certify = loose files + container flagging only. Deep archive analysis belongs exclusively in dedicated Archive mode (in-memory, per-file reports emitted under `/Archive/`). Do not apply special ZIP bucketing or container logic inside the regular Certify path (see `docs/CERTIFY_ZIP_BUCKETING.md` for historical reference only).

### Dev-Only Enforcement
- `::raa::integrity::`
- `::raa::devonly::`
- `::raa::integrity::guard::`

**When used:** The Integrity Guard (including Forensic Vault safeguard) is **strictly dev-only**. It must remain completely invisible in production/release builds. It is gated by `isDev` and `.dev-tab` class. Any change that would expose it in release builds is forbidden.

### Git & Operational Discipline
- `::raa::git::`
- `::raa::discipline::`
- `::raa::push::`

**When used:** Enforce strict discipline:
- Commit locally very frequently (every coherent piece, roughly every 15–40 minutes).
- Push to GitHub sparingly — only at clear milestones, end of session/day, or when the user explicitly says “push”.
- Ask before pushing mid-session.
- Always write good commit messages.
- Keep the working tree reasonably clean before pushing.
- Respect cost: every push has real token/API cost.

### Grok Build Practices
- `::raa::grok::`
- `::raa::grok::inspect::`

**When used:** Run `grok inspect` when starting work in a new context. Use `/flush` before compaction or at end of productive work. Cross-reference the exact `docs/` files listed in AGENTS.md before touching auditing flows, manifest writing, per-file reports, job folders, or the vault browser. Use the dedicated workspace memory at `~/.grok/memory/raa-gatekeeper-860ff9ed/MEMORY.md`.

## Usage Examples

`::raa::vault::granular::manifest/ How should the control manifest be finalized at the end of a job?`

`::raa::permanent::anchors::integrity/ We need to add a new check to the Integrity Guard.`

`::raa::certify::archive/ A user selected a folder that contains a .zip — what should happen during Certify?`

`::raa::git::discipline::push/ I want to push the current vault browser changes.`

`::raa::devonly::integrity/ Should I expose the Forensic Vault check in the UI for easier testing?`

`::raa::vault::onefile:: / Explain the current job folder structure and why we use the ~ prefix.`

`::raa::grok:: / Before I edit generate_manifest, what files should I re-read?`

`::raa::permanent:: / A Certify job just finished with one violation. What must be true about the final state the user sees?`

## Reset & Combination

Use the general reset `::/` to clear all tags (including RAA-specific ones).

You can freely combine general and RAA tags:
`::expert::rust::tauri::raa::vault::granular/ ...`

When RAA tags are active, treat the corresponding sections of `AGENTS.md` and the referenced `docs/` files as hard constraints. Do not relax them unless the user explicitly asks to step outside the scope.

**This file should be committed. Place a compiled/minimal version in `.grok/rules/` for automatic injection if desired.**
