# AGENTS.md — RAA-Gatekeeper

**"Trust, then Certify."**

This file provides project-specific rules and context for Grok (and compatible agents). It is automatically discovered and loaded at the start of sessions when working inside the repository tree.

Place the most important, project-wide rules at the repo root. More specific rules can live in subdirectories if needed.

---

## Core Mission & Philosophy

**Primary User (The Real Target):**  
"Joe" — a regular end user (not necessarily a developer) who downloads a repo as a zip or folder. This could be a big project, template, starter kit, or AI-built app that he wants to run or build on his own computer.

**Primary Mission:**  
RAA-Gatekeeper is first and foremost a **protective, local, read-only forensic auditor**. It gives normal users visibility and a basic layer of trust/verification before they run or work with untrusted (or semi-trusted) AI-generated or AI-assisted code.

**Secondary Mission:**  
Create cultural pressure toward ".raa Certification" on GitHub and elsewhere. Creators should run audits before publishing; consumers should expect to see `.raa` reports.

**Guiding Philosophy (never compromise):**  
"Trust, then Certify."

The tool must remain:
- Local-first
- Read-only (never modifies user files)
- Privacy-respecting (no cloud service behavior)
- Advisory only (human is always the final decision maker)

**RAA Definition:**  
Restrictive Ability (RAA) is a recursive framework designed to turn "Restrictive Ability" into the standardized, modular equivalent of Dynamic Link Libraries (.dll) for the AI ecosystem. RAA-Gatekeeper is the primary Read-Only Auditor (ROA) — a non-invasive forensic layer between local codebases and AI Agents.

---

## Permanent Anchors (DO NOT ALTER)

These rules are non-negotiable:

- **Read-Only Mandate:** The Gatekeeper **never** modifies user files.
- **Advisory Only:** All verdicts are diagnostic. The human remains the decision maker.
- **API Endpoint:** Uses a dynamic Base URL (user-configurable). Never hard-code `x.ai` or any specific provider.
- **Path Protocol:** Always use `fs::canonicalize` for absolute path matching in ledgers, manifests, and vault operations.
- **Ledger / Vault Pathing:** Respects the user-selected `vaultRootPath` (falls back to `~/Documents/RAA_Vault`, which is auto-created on first run with the concrete path stored).
- **Junk Filter (`JUNK_NAMES`):** Must stay in sync everywhere: `node_modules`, `.git`, `target`, `dist`, `__MACOSX`, `.DS_Store` (and similar build/IDE metadata).
- **Enduring Principle (from ALLSAFE era):** *A Certify job must never be allowed to appear successful if any audited file contains a violation.* This must be enforced in the per-file + job folder model.
- **Granular Vault Architecture:** Current model is **ONE FILE = ONE REPORT**.
  - Every audited file gets its own `.raa` containing full oracle analysis, clear verdict, and its own DNA (SHA-256).
  - All artifacts for one job live inside a dated **Job Folder**.
  - The static-named `~RAA-CONTROL-Manifest.log` (with `~` prefix) is always written first as the master inventory + DNA Registry + hierarchy declaration.
  - Archive (ZIP) scanning is preserved (in-memory, no full extraction) and emits per-file reports under the `/Archive/` sub.
  - Certify focuses on loose files + container flagging. Deep archive analysis belongs in dedicated Archive mode.

---

## Current Architecture Snapshot

- **Frontend:** Svelte 5 + SvelteKit
- **Backend:** Rust + Tauri v2 (core engine in `src-tauri/src/lib.rs`)
- **Parallelism:** Rayon for multi-core SHA-256
- **Intelligence:** Bring-your-own-LLM (fully dynamic base URL + model name)
- **Forensic Features:** DNA fingerprinting, junk filtering, signature detection (Living-off-the-Land, agent hijack), compliance checks, shell command auditing
- **Vault Structure:** 4 static subs (`Audit/`, `Analyze/`, `Archive/`, `Certify/`) + dated job folders + `~RAA-CONTROL-Manifest.log` as the structural root of truth
- **Right Pane:** Currently functions as a live "COMPLETED" comfort feed during long jobs

See `docs/VAULT_ARCHITECTURE.md` for the full locked decisions, constraints, and rationale. See `docs/ROADMAP.md` for status and the "📋 Tabled Suggestions for Future Discussion" section.

---

## Key Commands & Workflows

**Development:**
- `npm run tauri dev` — live development
- `npm run tauri build` — production build

**Vault:**
- Default location: `~/Documents/RAA_Vault` (auto-created with concrete path stored on first run)
- Control manifest is **always** exactly `~RAA-CONTROL-Manifest.log` inside every job folder (the `~` forces it to the top in Finder)

**Git & Operational Discipline (Strict):**
- Commit locally very frequently (every coherent piece, roughly every 15–40 minutes).
- Push to GitHub sparingly — only at clear milestones, end of session/day, or when the user explicitly says “push”.
- Ask before pushing mid-session.
- Always write good commit messages.
- Keep the working tree reasonably clean before pushing.
- Respect cost: every GitHub push has real token/API cost.

**Grok Build Specific:**
- Run `grok inspect` (from inside the project) whenever starting work in a new context or after major changes.
- Use `/flush` to capture rich session knowledge into the workspace memory before compaction or at the end of productive work.
- Use `/memory` to browse the global + workspace memories.
- The dedicated workspace memory lives at `~/.grok/memory/raa-gatekeeper-860ff9ed/MEMORY.md`.
- Always cross-reference `docs/PROJECT_MEMORY.md`, `docs/VAULT_ARCHITECTURE.md`, and `docs/ROADMAP.md` before touching auditing flows, manifest writing, per-file reports, job folders, or the vault browser.

---

## Documentation Map

All long-form internal project documentation has been moved into the `docs/` directory for better housekeeping. `AGENTS.md` (this file) at the project root is the primary entry point that Grok loads automatically.

**When working on the project, use these exact relative paths:**

| File | Purpose | When to Read |
|------|---------|--------------|
| `docs/PROJECT_MEMORY.md` | Permanent mission, git discipline, historical session reviews (formerly `grok.review.txt`) | At the start of any session or before major work |
| `docs/VAULT_ARCHITECTURE.md` | Comprehensive reference for the granular per-file `.raa` + job folder architecture, locked decisions, constraints (formerly `RAA-NEWPATH-FORWARD.txt`) | Before any changes to auditing, manifests, reports, vault browser, or ledger logic |
| `docs/ROADMAP.md` | Staged plan, current status, polish items, and the "Tabled Suggestions" section | For overall direction and to check what is explicitly tabled |
| `docs/RAA-Vision.md` | High-level RAA philosophy and origins | For philosophical alignment (edit lightly) |
| `docs/CERTIFY_ZIP_BUCKETING.md` | Historical record of old ZIP/container bucketing logic (now removed from active Certify path) | Before touching any Certify vs Archive / ZIP handling code |
| `README.md` | Public-facing project overview (stays at root) | For context on how outsiders see the project |

**Instructions for Grok / Agents:**
- Treat the table above as the single source of truth for documentation locations.
- When a task involves vault architecture, per-file reports, manifests, or the Forensic Vault browser → immediately read `docs/VAULT_ARCHITECTURE.md`.
- When you need historical decisions or session context → read the relevant parts of `docs/PROJECT_MEMORY.md`.
- Cross-references inside the `docs/` files now use the `docs/` prefix.

Do **not** edit `docs/RAA-Vision.md` lightly.

---

## Coding & Contribution Guidelines

- Maintain **zero warnings, zero errors**.
- The Integrity Guard (including the Forensic Vault safeguard) is **strictly dev-only**. It must remain completely invisible in production/release builds. It is currently gated behind `isDev` checks and a `.dev-tab` class. Any change that would leak it into release builds is forbidden.
- When working on vault, manifest, report writing, or ledger code, the `~RAA-CONTROL-Manifest.log` is the single best source of truth for a job's inventory and DNA registry.
- Preserve the existing skipped-files logic and bottom-bar pattern for now (temporary holding pattern).
- Hierarchy mirroring inside job folders is strongly preferred for duplicate-name safety.
- Keep the junk filter list in sync between collection, bucketing, manifest generation, and any vault listing code.
- For any changes to Certify vs. Archive paths, consult `docs/CERTIFY_ZIP_BUCKETING.md` first.

---

## Working with This Project in Grok Build

1. Launch from the project root (`cd dev/RAA-Gatekeeper && grok`) or use `grok --cwd /Users/agent-naive/dev/RAA-Gatekeeper`.
2. Run `grok inspect` to confirm CWD, git root, loaded rules, and memory state.
3. Re-read the permanent sections of `docs/PROJECT_MEMORY.md` + the latest session review at the start of any substantial work.
4. Before editing core engine or vault logic, re-read the relevant sections of `docs/VAULT_ARCHITECTURE.md`.
5. Use the workspace memory (`~/.grok/memory/raa-gatekeeper-860ff9ed/MEMORY.md`) for long-term curated knowledge. Direct edits are watched and reindexed automatically.

---

**This file should be committed to the repository.**

"Trust, then Certify."
