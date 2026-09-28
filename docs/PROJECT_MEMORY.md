# PROJECT_MEMORY.md

# RAA-Gatekeeper — Project Knowledge Base & Session Reviews

This document serves as the persistent project memory, including permanent rules, mission, push discipline, and historical session reviews. It is the primary reference to help future contributors (and AI sessions) maintain context across long gaps.

It was previously named grok.review.txt.

================================================================================
### PERMANENT KNOWLEDGE — DO NOT EDIT THIS SECTION ###
================================================================================

## Core Mission & User Positioning

**Primary User (The Real Target):**  
"Joe" — a regular end user (not necessarily a developer) who downloads a repo as a zip or folder. This could be a big project, template, starter kit, or AI-built app that he wants to run or build on his own computer.

**Core Problem We Solve:**  
99.9% of AI coding is text-based. When Joe downloads someone else's code (or a project that was heavily AI-assisted), it is nearly impossible for him to manually scour every line to check for:
- Harmful or malicious code injection
- Data-stealing behavior
- Rogue commands that could run without approval
- Unauthorized access to parts of his system

Joe does not have the time, skill, or patience to audit large amounts of AI-generated code.

**Primary Mission:**  
RAA-Gatekeeper exists **first and foremost** as a protective utility for end users who are bringing other people's AI-generated or AI-assisted code onto their machines. It acts as a local, read-only forensic auditor that gives normal users visibility and a basic layer of trust/verification before they run or work with untrusted (or semi-trusted) code.

**Secondary Mission:**  
Create cultural pressure and a de facto standard on GitHub and other platforms. The goal is that:
- Creators feel it is in their best interest to run an LLM audit before publishing.
- Consumers start expecting to see `.raa` reports alongside repos.
- "Trust, then Certify" becomes a recognized practice in the AI-assisted development ecosystem.

**Guiding Philosophy:**  
"Trust, then Certify."

The tool should remain local-first, read-only, and privacy-respecting. It is not a cloud service. It is a forensic layer that helps users make informed decisions about code they did not write themselves.

================================================================================
### END PERMANENT KNOWLEDGE — DO NOT EDIT ABOVE THIS LINE ###
================================================================================

---

**This document is structured for long-term use:**

- The section above is **permanent** and should never be edited.
- All future session reviews and daily notes should be appended **below** this line.
- When starting a new day, read the Permanent Knowledge first, then review the most recent session notes.

---

## Strict Git & Push Rules (Operational Discipline)

**Critical Note:** GitHub pushes incur real token/API costs. We must be much more disciplined than typical AI-assisted development sessions.

### Core Rules

1. **Commit locally very frequently**  
   - Commit after every coherent piece of work or roughly every 15–40 minutes.  
   - Local commits are free and provide excellent safety + history.  
   - Never wait long between local commits.

2. **Push to GitHub sparingly**  
   - Only push at these times:
     - End of a clear, working milestone or feature
     - End of a work session / end of day (when user requests a clean stopping point)
     - When the user explicitly says “push this”, “let’s push”, or similar
   - Do **not** push after every small change, CSS tweak, or minor fix.

3. **Ask before pushing mid-session**  
   - Unless we are at a natural checkpoint, explicitly ask the user before doing a push during active work.

4. **Always write good commit messages**  
   - Even when we’re not pushing immediately, every local commit must have a clear, descriptive message.  
   - This makes later batch pushes much more useful.

5. **Keep the working tree reasonably clean before pushing**  
   - Avoid pushing with lots of unrelated or experimental junk in the working directory.

6. **Respect cost**  
   - Every push has a cost. Treat pushes as a deliberate action, not an automatic reflex.

### Recommended Daily Rhythm

- Commit locally often throughout the session.
- Push 2–5 times maximum per active day (typically at major checkpoints + end of day).
- One solid end-of-day push is usually the most important.

These rules exist specifically because this project uses Grok Build and GitHub pushes are not free.

---

# Session Reviews & Daily Notes

## 2026-05-30 — End of Day Review

**Session Focus:** Heavy UI/UX refinement on Vault view and Integrity Guard, CSS architecture improvements, delete report feature, and critical bug documentation.

---

**Note on Tabled Suggestions (added per user request):** Credible ideas for the vault finder (subs + job folders), Forensic Vault safeguard strengthening inside Integrity Guard, compile-time dev-only stripping, version sync across the 5 files, full "RAA-Gatekeeper" name hygiene, release-readiness checklist, and settings reorg have been captured in a dedicated "📋 Tabled Suggestions for Future Discussion" section in ROADMAP.md (with cross-refs from VAULT_ARCHITECTURE.md and this file). 

These are explicitly tabled for a later dedicated discussion session. No implementation, no code comments treating them as active work, and no purging of existing content in PROJECT_MEMORY.md or VAULT_ARCHITECTURE.md has occurred. Review that ROADMAP section when planning future turns. (This note added without altering any prior session content.)

### Major Accomplishments
- Restored and significantly improved the **Integrity Guard** page (it had lost most of its visual structure during earlier CSS cleanup). Now has a clean, card-based grid layout.
- Major improvements to the **Vault left pane**:
  - Width reduced to 300px (user confirmed "300px is nice").
  - Delete button (🗑) moved from the top line to the bottom line alongside the date (right-aligned).
  - Added consistent right-edge padding alignment so the filter input, filename truncation, and trashcan all share the same visual gutter.
- Added full **delete .raa report** functionality with a proper confirmation modal.
- Created `knownbug.ALLSAFE.txt` — a detailed, self-contained spec for the critical Certify bug.
- Updated ROADMAP.md with a visible "🚨 Critical Bugs" section.
- Continued external CSS migration (`app.css` + `+layout.svelte`).

**Current State:** Working tree is clean. All work pushed.

### Key Current UI State (Vault)
- **Left Pane:** 300px fixed width. Two-line row design (filename on top, date + trashcan on bottom).
- **Right Pane:** Min 480px / Max 620px.
- Delete confirmation uses a custom modal (not browser `confirm()`).
- Backend has a safe `delete_vault_file` command with vault boundary checks.

### Important Notes from This Session
- The user is very iterative and gives precise, direct feedback ("300px is nice", "move the trashcan to the bottom line").
- Strong preference for clean external CSS in `app.css` over component bloat.
- Strict rule: Never edit `RAA_VISION.md`.

---

**End of 2026-05-30 Review**

---

## 2026-05-30 — Vault System Update (Later in Session)

**Important Design Change:** Implemented proactive default vault creation on first run.

### What Changed
- Added new Rust command: `get_default_vault_path()`
- On first launch (when no `raa_vault_root_path` exists in localStorage), the app now automatically:
  1. Calls `get_default_vault_path` to get the resolved default location.
  2. Creates `~/Documents/RAA_Vault` (via `create_vault_directory`).
  3. Stores the **actual parent path** (e.g. `/Users/joe/Documents`) instead of leaving `vaultRootPath` empty.
- "Set Default to ~/Documents/RAA_Vault" button now resolves the real path and stores it (instead of setting empty string).
- Added a one-time session notification in Settings when the default vault is auto-created on first run.
- `ensureVault()` and `loadVaultData()` now behave more consistently because we prefer having a real path early.
- Centralized `normalizeVaultPath()` helper is used everywhere.

### Why This Was Done
The previous "empty string = default" convention was causing confusing state when users mixed the "Set Default" button with manual Finder selection of the default folder. This led to Vault not refreshing properly after restarts.

By creating the folder on first run and always storing a concrete path, we eliminate most of the fragile dual-state logic (`""` vs real path).

### Impact on Future Development
- `vaultRootPath = ""` is now mostly a fallback / reset state rather than the normal "default" representation.
- Most code paths can now assume `vaultRootPath` contains a real directory.
- This should make Vault loading, auditing, and path handling much more reliable.

---

**End of 2026-05-30 Vault Update**

---

## How to Use This Document Going Forward

1. Always read the **Permanent Knowledge** section first (top of file).
2. Then read the most recent session review(s).
3. Append new session notes at the bottom using the date format above.
4. Keep technical details, decisions, and "why" explanations — these are the things that get lost in context compaction.

Good luck. The project is making steady progress toward becoming a genuinely useful protective utility for people bringing AI-generated code onto their machines.

---

## Capture Value of Audit - Checkpoint

**Label:** Capture Value of Audit  
**Commit ID:** `ed5a45b707296d909a0ccd222b08c52b3fc80373`  
**Date of Capture:** 2026-05-30 (end of session)  
**Purpose:** Hard reference point before implementing structured output improvements for multi-file buckets in Certify. This checkpoint preserves the state of the bucketing + prompting logic after the isolation prompting revert and before moving to forced structured output (JSON / strict sections).

### Full Explanation of Options (Captured from session)

**Current Situation (What We Do Today)**

When a bucket contains multiple files (e.g. 5 small files), we now send a strong prompt telling the LLM to analyze them ONE AT A TIME, completely independently.

The LLM then returns **one single response** (the `report.reasoning` field). This response might be 800–2000+ tokens long.

After we get that response, this is what currently happens in the code:

```rust
for job in bucket {
    let analysis_block = format!(
        "--- RAA FILE ANALYSIS ---\n\
         File: {}\n\
         Hash: {}\n\
         Verdict: {}\n\
         Analysis:\n{}\n\
         ------------------------\n",
        job.path.display(),
        job.hash,
        report.verdict,
        report.reasoning.trim()     // ← The SAME reasoning is used for every file
    );
    report_entries.push_str(&analysis_block);
}
```

**Result:**  
Every single file in that bucket gets the **exact same block of text** written under its `Analysis:` section. The only thing that changes per file is the `File:`, `Hash:`, and `Verdict:` lines.

This is the part that needs improvement.

### The Three Options Discussed

**Option A: Try to Parse/Split the Model’s Response (Post-processing)**

After the LLM sends back one big blob of text, we could attempt to **parse** it and split it into per-file sections.

For example, if the model responds like this:

```
FILE 1 ANALYSIS:
[long analysis for file 1]

FILE 2 ANALYSIS:
[long analysis for file 2]
```

We could write code that tries to detect these sections and then assign the correct piece of reasoning to each file’s `Analysis:` block.

**Pros:**
- The vault entry would actually contain different, relevant analysis for each file.
- Much higher forensic value.

**Cons / Difficulties:**
- LLMs are inconsistent. Sometimes they follow the format, sometimes they don’t.
- They might use different headings.
- They might blend analyses together.
- Parsing this reliably is fragile and can get very complex.
- If parsing fails, we fall back to dumping the entire response under every file anyway.

**Option B: Force Better Structure from the Model (Prompt Engineering)**

Instead of trying to clean up the response *after* we receive it, we could change the prompt even more aggressively to force the model to return output in a parseable format (e.g. clearly labeled sections, or even JSON).

This is related to the prompting work we just did, but goes further.

**Option C (Strongest Recommendation at the time):** Force structured output from the model using e.g. JSON or very strict sections.

**Honest Opinion at Time of Capture:**
The prompting improvement (strong “one file at a time, independently” instructions) is the **highest leverage** change for the quality of the actual *analysis*.

However, even with great prompting, if we still blindly paste the entire model response under every file in the bucket, we’re leaving a lot of value on the table.

The question was whether we also want to try to **capture** that sequential analysis properly in the final `.raa` file (via structured output or parsing).

---

**End of Capture Value of Audit Section**

This section is intended as a permanent reference point for the state of the Certify multi-file bucket logic before moving into structured output implementation.
---

## 2026-05-30 — End of Day Report

**Session Focus:**  
Major architectural progress on the granular per-file report model + successful relocation of both toasts into the top header + significant watcher reliability improvements.

### Major Accomplishments

1. **Per-File .raa Reports (Audited Files)**
   - `generate_manifest` now writes individual `.raa` files for every audited file.
   - Reports are placed inside the dated job folder using the original source relative path (e.g. `job-folder/src/utils/helper.rs.raa`).
   - This completes the core "ONE FILE = ONE REPORT" goal for the audited content.
   - The old aggregated `certify-report-*.raa` is still written for backward compatibility during the transition.

2. **~RAA-CONTROL-Manifest.log Enhanced with DNA Registry**
   - The manifest is now finalized at the very end of the Certify process (after all hashing is complete).
   - It now contains a **"DNA Registry (File → Hash)"** section near the top with every file (audited + skipped) listed as:
     ```
     File: relative/path/to/file.ext
     Hash: <sha256>
     ```
   - This gives us a single, static-named file (`~RAA-CONTROL-Manifest.log`) per job that contains both the directory structure and all DNA hashes.
   - This is currently the best single source of truth for quick hash lookup per job.

3. **Top Header Restructured into 3-Column Layout**
   - `.top-bar` is now a CSS Grid: `auto 1fr auto`
   - **Left column**: 🛡️ + **GATEKEEPER** (in blue via `.gatekeeper-text`)
   - **Center column**: Dedicated space for toast popups (both the audit completion toast and the DNA watcher toast now live here)
   - **Right column**: Version tag (right-aligned, same styling as before)
   - This successfully "reclaimed space" in the header and stopped the toasts from covering lower content.

4. **Both Toasts Now Live in the Header Center Column**
   - Audit completion toast (`.mission-success-toast`)
   - DNA watcher toast (`.watcher-toast`)
   - Both were moved out of their old fixed bottom positions.
   - Added `min-width: 300px` to both toasts (user requested this for visual testing).
   - User confirmed the new location works well.

5. **Watcher Reliability Fixes**
   - Removed the overly aggressive `!path_str.contains("~")` filter from the watcher event handler.
   - The filter was silently dropping all events for any path containing a tilde character (very dangerous given how the user names test directories).
   - After removal, the watcher began firing events reliably, including on external drives and directories containing `~`.
   - User confirmed they are now seeing proper `⚡ SPARK` lines when editing files.

### Current Architecture Snapshot (Important for Context)

**Job Folder Structure (per Certify run):**
```
RAA-Vault/
└── test-directory01-20260530-191527/
    ├── ~RAA-CONTROL-Manifest.log          ← Static name, contains DNA Registry + trees
    ├── certify-report-20260530-191527.raa ← Aggregated report (still written)
    ├── threat.md.raa
    ├── safe01.txt.raa
    ├── threat01.txt.raa
    └── ~test-dir02/
        └── threat02.txt.raa
```

**~RAA-CONTROL-Manifest.log now contains (in order):**
- Header (Generated, Source, Job Folder)
- DNA Registry section (File + Hash for every file)
- Directory Structure (files to be audited) — tree view
- Directory Structure (files to be skipped) — tree view

### Known Gotchas & Lessons Learned

- The `contains("~")` filter in the watcher was extremely harmful and should probably never have been that broad. It was a quick hack that became technical debt.
- Moving or renaming test directories requires manually re-arming the watcher with the new paths. The watcher does not auto-update.
- Watcher behavior on external drives (`/Volumes/...`) can still be inconsistent due to FSEvents limitations on macOS.
- Editor temp files (`.sb-*` files from Sublime Text, etc.) generate a lot of noise in the watcher. We may want to filter more aggressively later.
- The vault browser (`list_vault_files`) is still completely flat and does not understand job folders yet. This was explicitly tabled until the manifest + per-file reports stabilized.
- DNA verification currently depends on the user having the correct report selected. With per-file reports now scattered inside job folders, this will become increasingly painful until we improve the vault UI.

### Current UI State (Header)

- 3-column grid in `.top-bar`
- Both toasts live in `.header-center`
- Toasts have `min-width: 300px` (added for testing)
- Forensic toast is back to `font-size: 12px` (reverted after testing `font-weight: 700`)
- User is currently happy with the visual structure of the control manifest.

### Files / Areas That Changed Significantly Today

- `src-tauri/src/lib.rs` — Major changes to `generate_manifest` (per-file report writing + DNA registry in manifest)
- `src/routes/+page.svelte` — Header restructured into 3 columns, both toasts moved into center
- `src/app.css` — New header grid, toast positioning + min-width, various small adjustments
- `ROADMAP.md` — Updated with new architecture progress and the mandatory future settings reorganization note
- `VAULT_ARCHITECTURE.md` — Contains detailed notes on the current architecture

### Open Questions / Next Priorities

1. **DNA Verification Flow** — Now that we have per-file reports + a DNA Registry in the manifest, how should the watcher + verification actually work? Should it look first in the manifest? Should it scan job folders?
2. **Vault Browser for Job Folders** — Still tabled, but becoming increasingly necessary.
3. **Watcher Noise** — Should we add more aggressive filtering for editor temp files (`.sb-*`, etc.)?
4. **Settings Page Scrolling** — User strongly dislikes scrolling in Settings on Mac. Future mandatory improvement: button-driven sections instead of one long page.
5. **Toast Polish** — User is testing `min-width: 300px`. May want further adjustments to width, height, or behavior.

### Recommendations for Future Sessions

- When testing the watcher, always confirm exactly which folders are currently listed in the "🕵️ Silent Watcher" section in Settings.
- After any directory move/rename, the watcher must be re-armed.
- The `~RAA-CONTROL-Manifest.log` is now the single best place to look for "what files were in this job and what were their hashes."
- The per-file reports are the future. The aggregated `certify-report-*.raa` is legacy during transition.

**Overall Feeling:**  
Today was a genuinely productive day. We made real, visible progress on the granular architecture the user has been driving toward. The combination of per-file reports + DNA registry in the static manifest + toasts living in the header feels like a meaningful step forward.

---

## 2026-06-06 — Project Focus Switch + Comprehensive Context Re-acquisition

**Session Focus:** Located the RAA-Gatekeeper project on disk, switched active Grok development context to it, and performed a full document review + status synthesis to establish strong continuity for future inference sessions. This was primarily a re-orientation and knowledge-loading session after moving focus from the home directory.

### Major Accomplishments & Context Established This Session
- Used terminal search (`find`) across ~ and ~/dev to locate the project root at `/Users/agent-naive/dev/RAA-Gatekeeper`.
- Confirmed Git details: remote `https://github.com/Agent-Naive/RAA-Gatekeeper.git`, branch `main`, clean working tree at time of switch.
- Established project context using the proper Grok invocation: `~/.grok/bin/grok --cwd /Users/agent-naive/dev/RAA-Gatekeeper inspect`. This correctly reported CWD + Git root, no project-local instructions/config yet, and user-level Grok config.
- Performed deep reads of the project's own long-term memory artifacts:
  - docs/ROADMAP.md (full) — captured exact current status and staged plan.
  - This file (docs/PROJECT_MEMORY.md) — re-read permanent knowledge, git rules, and 2026-05-30 session history for continuity.
  - docs/RAA-Vision.md (full) — high-level philosophy.
  - docs/VAULT_ARCHITECTURE.md (full) — the comprehensive internal reference document for the granular architecture (created as a fallback to prevent context loss).
  - Attempted direct read of the foundational `RAA Gatekeeper.pdf` on Desktop (the "Birth of RAA" origin doc from ~April 5, 2026). Access blocked by macOS ("Operation not permitted"). Relied on RAA-Vision.md + references in other docs as proxy.
- Synthesized and internalized "where we left off" for the entire project (detailed below).
- Confirmed no AGENTS.md or local `.grok/` directory exists in the repo yet (opportunity for future project rules).
- Noted existing artifacts in the tree: PROJECT_MEMORY.md, VAULT_ARCHITECTURE.md, ROADMAP.md, RAA-Vision.md, VAULT_ARCHITECTURE.md, previous review notes (PROJECT_MEMORY.md), test directories, and build artifacts.

### Current Project Status Snapshot (Synthesized for Future Sessions)
**From ROADMAP.md (as of this session):**
- **Current Status:** Active Development — Phase 3 + 4 Core Complete + **New Granular Architecture Path Initiated**.
- **Last Major Update:** Definition of the New Path Forward (ONE FILE = ONE REPORT + dated Job Folders + `~RAA-CONTROL-Manifest` as the first artifact).
- Phases 1+2 (Core Engine & Performance) and core Phase 3+4 (Forensic UI & Silent Monitoring) are marked COMPLETE (terminal auditing with Bible cache, deep file + archive scanning, parallel Rayon hashing, Silent Watcher with DNA alerts, Integrity Guard, user-selectable vault, real-time timers, etc.).
- "Currently In Progress / Next" items remain open: Rich Vault Browser, DNA Verification UI, Export Reports, Violation History Dashboard ("Wall of Shame"), Token Economy Counter.
- Significant polish note: Settings page reorganization into button-driven sections is called out as a **Mandatory UI Improvement (Deferred)** due to excessive scrolling pain on macOS.
- Dominant content is the detailed **New Path Forward** section with 6 explicit stages (see below). Vault browser work is *explicitly tabled* until `~RAA-CONTROL-Manifest` + per-file report writing inside job folders are stable. See TODO comments in `src-tauri/src/lib.rs` (especially `list_vault_files`) and the full details in VAULT_ARCHITECTURE.md.

**New Path Forward (the active architecture we are implementing):**
- Shift from monolithic "one big .raa per job" (with internal blocks) to **ONE FILE = ONE REPORT**.
- Every file that receives LLM forensic analysis (Certify folder or inside Archive) produces its own dedicated `.raa` containing full oracle analysis paragraph + clear verdict + its own DNA (SHA-256).
- All artifacts for one user-initiated job are grouped inside a **dated Job Folder** inside the user's RAA-Vault (e.g., `MyProject-20260606-123456/` or `cool-repo.zip-...`).
- As the very first action on "Start Certification" or "Select ZIP": create the dated job folder + write a static-named `~RAA-CONTROL-Manifest.log` (the `~` prefix forces it to sort to the top in Finder). This becomes the permanent audit control sheet / master inventory + hierarchy declaration for the entire job.
- Right pane is initially repurposed as a live COMPLETED comfort feed (shows .raa files being emitted in real time). Not yet clickable/interactive ("YET").
- Hierarchy mirroring inside job folders is strongly preferred for duplicate-name safety and familiar after-action review.
- Archive (ZIP) scanning must continue to work (in-memory, no full extraction) and emit per-file .raa inside the job folder.
- Enduring principle (from ALLSAFE era, must be enforced in new model): *A Certify job must never be allowed to appear successful if any audited file contains a violation.*
- Stages (detailed in ROADMAP and fully expanded in VAULT_ARCHITECTURE.md):
  1. ~RAA-CONTROL-Manifest (Current Focus / First Stage to stabilize).
  2. Real-Time Right Pane Comfort Feed.
  3. Per-File .raa Writing + Job Folder Hierarchy Mirroring (core of the granular model; partial implementation already landed in 2026-05-30 session).
  4. Archive Path Full Alignment.
  5. Vault Browser Evolution for Job Folders (currently tabled).
  6+. Future (right pane clickable, skipped files moved dynamically, job-level summaries, export, etc.).

**From 2026-05-30 Session History (previous entry in this file):**
- Major progress on the granular model: `generate_manifest` now writes individual `.raa` files per audited file (placed inside dated job folder using original relative source path). Legacy aggregated `certify-report-*.raa` still written for transition.
- `~RAA-CONTROL-Manifest.log` enhanced with "DNA Registry (File → Hash)" section near the top — now the single best point-in-time source for what files were in a job and their hashes.
- UI: Header restructured to 3-column CSS grid (left: logo/title, center: toasts, right: version). Both mission-success and watcher DNA toasts moved into `.header-center`.
- Watcher fixes (removed harmful broad `~` filter that was dropping events on test dirs and external volumes).
- Vault improvements (proactive default `~/Documents/RAA_Vault` creation + storing concrete path on first run).
- Vault UI polish (300px left pane, delete report with confirmation modal, consistent padding).
- "Capture Value of Audit" checkpoint captured the state of multi-file bucketing + prompting logic before structured output improvements.
- Open questions carried forward: DNA verification flow (manifest vs. per-file reports vs. job folders), vault browser for job folders, watcher noise filtering, Settings scrolling pain, toast width polish.
- Recommendations from that session are still highly relevant (re-arm watcher after dir moves, use manifest for hashes, per-file reports are the future, etc.).

**From RAA-Vision.md + PDF references:**
- Core philosophy: RAA (Restrictive Ability) as the modular "DLL standard" for AI security/trust. RAA-Gatekeeper is the Read-Only Auditor (ROA) — non-invasive forensic layer between local code and AI agents.
- DNA Fingerprinting (hashes guarantee "what was audited is what is passed"), junk filtering (especially macOS `__MACOSX` / `._`), LLM oracles for signature detection, compliance, and agent-hijack guarding.
- Target user "Joe": regular end-user (not dev) who downloads a zip/folder/AI-built project and needs protection before running it.
- "Trust, then Certify" as the cultural standard. Every interaction recorded in tamper-proof `.raa` vault entries. The PDF (inaccessible here) is the April 2026 origin document for these ideas.

**From VAULT_ARCHITECTURE.md (critical long-term reference — always re-read when working on the new architecture):**
- Full rationale, locked decisions, technical mapping (which functions live where: `generate_manifest`, `scan_compressed_archive`, `log_to_raa`, `list_vault_files`, `resolve_vault_root`, Svelte handlers, etc.), naming conventions, Joe experience targets, constraints (preserve skipped logic as temporary holding pattern for now, zero-mutation, etc.), success criteria for Stage 1, and open/tabled questions.
- Explicitly created so full context survives long gaps or context resets.
- Junk filter list (`JUNK_NAMES`) must stay in sync between manifest generation and control manifest building (includes node_modules, .git, target, dist, __MACOSX, .DS_Store — last added 2026-05-30).

**Permanent Anchors & Rules (re-affirmed):**
- Read-only mandate, advisory-only (human is always the decision maker), dynamic Base URL (no hard-coded x.ai), `fs::canonicalize` for paths, respect user `vaultRootPath` (fallback `~/Documents/RAA_Vault`).
- Strict git discipline from this file: local commits frequently, pushes only at clear milestones or end of day (ask first), good messages, clean trees before push. This project incurs real costs on GitHub pushes.
- "Trust, then Certify." + the enduring principle about never letting a job appear successful if violations exist.

### Current Environment & Tooling Notes for This Context
- Always invoke Grok with project focus: `grok --cwd /Users/agent-naive/dev/RAA-Gatekeeper` (or `cd` first).
- `grok inspect --cwd ...` (or via the full invocation) is the authoritative way to confirm context.
- No project-specific AGENTS.md, .grok/config, or local rules loaded yet.
- Memory is globally enabled (from user config); workspace-specific memory under `~/.grok/memory/` will use the git slug (`Agent-Naive/RAA-Gatekeeper`) once used in this CWD.
- PDF on Desktop remains inaccessible in current tool environment — use RAA-Vision.md + VAULT_ARCHITECTURE.md as authoritative proxies.

### Open Questions / Next Priorities (Carried Forward + New from This Session)
1. Stabilize **Stage 1 (~RAA-CONTROL-Manifest)**: Job folder creation on Start + writing the static `~RAA-CONTROL-Manifest.log` (full inventory + hierarchy using the same filters as auditing) as the very first artifact. Emit events for right-pane comfort feed.
2. **Stage 2**: Wire the right pane as live emission log for .raa files as they are created (left pane stays "📡 Audited", bottom bar preserves skipped logic).
3. Un-table and evolve **Vault Browser + DNA Verification** for job folders + per-file reports (currently the biggest gap for post-facto usability).
4. Address mandatory Settings page UX (reorganize to sections/tabs to eliminate long scrolling).
5. DNA verification flow redesign now that reports live per-file inside job folders (manifest as single source of truth?).
6. Watcher noise filtering improvements (editor temp files, etc.).
7. Continue following the detailed constraints and success criteria in VAULT_ARCHITECTURE.md.
8. Consider adding project-level AGENTS.md or .grok rules now that we have strong focus here.
9. **Future Revamp (Deferred)**: Revamp ALL Integrity checks so they are uniform in their structure and execution. Currently a heterogeneous mix of real runtime tests and hardcoded `true` assertions (see ROADMAP.md Known Polish Items and VAULT_ARCHITECTURE.md section 11 for details). Goal: single consistent pattern across the entire Integrity Guard dashboard.

**Tabled Suggestion (Dev-only Integrity Guard):** The entire Integrity area (button + panel, including the Forensic Vault code safeguard) must remain strictly dev-only. Current implementation uses `{#if isDev}`. Credible future improvements to table for discussion:
- Use Vite define / tree-shaking so the code is completely absent from production bundles.
- Switch from a hidden nav tab to a dev-only global keyboard shortcut (e.g. Ctrl+Shift+I) that opens an overlay.
- Add a CI/build test that verifies no "Integrity" or "dev-tab" artifacts exist in production builds.

### Recommendations for Future Grok Inference Sessions
- **Always begin by reading the PERMANENT KNOWLEDGE section (top of this file) + the most recent session review(s).**
- Re-read ROADMAP.md + the entire VAULT_ARCHITECTURE.md before making changes to auditing flows, manifest writing, or vault code.
- Cross-reference the staged plan and "locked decisions" when implementing.
- When testing, use the existing test directories (e.g. test-directory02) and re-arm the watcher after any renames/moves.
- Maintain zero warnings, zero errors, and the read-only + advisory-only invariants.
- For git: commit locally often; push only at natural checkpoints or explicit user request.
- This document + VAULT_ARCHITECTURE.md + ROADMAP.md are the primary mechanisms to survive context resets.

**Overall Feeling:**  
Excellent re-acquisition and focus switch. The project now has strong, self-contained long-term memory in its own files (especially this review log + the detailed NEWPATH-FORWARD spec). We have a clear picture of exactly where implementation left off and what the next concrete steps are in the New Path Forward. Ready to resume active development (most logically by tackling Stage 1 stabilization) with full context preserved for any future Grok session.

**Note added per user request (2026 session):**  
Future task logged: Later revamp **ALL Integrity checks** for uniformity in structure and execution (see added item #9 above, ROADMAP.md "Known Polish Items", and new section 11 in VAULT_ARCHITECTURE.md). This is explicitly decoupled from the .raa data/reports themselves and focused on the Integrity Guard dashboard's internal consistency.

---
**2026-06-10 Session Review (Current Work)**

**Major Focus Areas:**
- **Preservation of Historical ZIP Bucketing Logic in Certify**: Created dedicated `CERTIFY_ZIP_BUCKETING.md` file. This captures the exact procedures for how .zip files were handled/filled in the Certify routine:
  - Special FileJob construction (hash="ZIP", size=0, content="") during jobs collection.
  - Forcing ZIPs into their own dedicated singleton buckets (the `if job.hash == "ZIP"` rule that flushed pending buckets and isolated them).
  - Later container detection (`is_archive_container`) and override to produce the special "ARCHIVE CONTAINER DETECTED DURING CERTIFY" report with `ACTION:ARCHIVE_AUDIT:` link.
- **Purge of ZIP Routines from Active Certify Path**: Removed the above special ZIP handling from `generate_manifest` while preserving bucketing for all uncompressed/regular files:
  - No more special FileJob for .zip in the collection phase.
  - Removed the dedicated-bucket forcing rule from the bucketing loop.
  - Removed the is_archive_container branching from the regular per-file report writing.
  - ZIPs encountered during Certify are now collected separately (`zip_paths`) and written directly as clean container reports (no participation in main bucketing or Oracle calls during Certify).
  - This reduces token waste (no unnecessary Oracle calls for containers when a full separate Archive audit is recommended) and keeps the Certify path focused on loose files.
  - The dedicated Archive path (`scan_compressed_archive`) remains the proper place for deep per-file ZIP analysis (with its own job folder under /Archive/).
- **UI Label Cleanup**: Renamed the Stage 2 right-pane comfort feed from "Just Done" / "What I Just Did" to "COMPLETED" across the board:
  - Updated visible `<h4>` headers and associated HTML comments in both Certify and Archive dual-pane sections.
  - Updated all related code comments (in +page.svelte listener, launch function, lib.rs emit logic).
  - Propagated the change to ROADMAP.md, VAULT_ARCHITECTURE.md, PROJECT_MEMORY.md, and src/app.css for consistency.
  - The feed still functions as the live emission log for reports and manifests.
- **Dev-Only Debug Controls**: Added a new section in the Settings page (visible only when `isDev` / localhost):
  - Independent toggles for `[RAA]` operational logs (manifest/report writing, vault tracing, job folder creation) and `[RAA-ORACLE]` verbose LLM tracing.
  - Persisted via localStorage (`raa_debug_raa`, `raa_debug_oracle`).
  - Flags are passed on every major invoke (`generate_manifest`, `scan_compressed_archive`, `audit_command`, `scan_file_integrity`).
  - All relevant `eprintln!` statements are now gated behind the respective boolean (plus the `debug_raa`/`debug_oracle` command parameters).
  - This gives precise, individual control while keeping debug output strictly dev-only.
- **Full GitHub Push as Backup**: Performed a complete push (`git add -A`, detailed commit `c89c91b`, `git push origin main`) before any major vault browser work. Working tree was clean. Commit message captured the preservation, purge, rename, debug controls, and supporting doc/code updates.
- **Vault Browser Planning Discussions**:
  - Confirmed preference for using `~RAA-CONTROL-Manifest.log` as the primary "root designer" for the vault structure (leverages its guaranteed position at the top of every dated job folder + its inventory/hierarchy data).
  - Decision to start with Phase 2 (job-folder tree awareness / structural browser) for a cleaner experience, explicitly skipping Phase 1 items like per-row "Reveal in Finder" buttons.
  - Phase 1 polish (cards, search improvements, DNA visibility, etc.) can be backtracked and added selectively once the visual/tree structure feels right.
  - This approach keeps the UI less cluttered while directly advancing the "Rich Vault Browser" goal (searchable, parsed incident cards, per-file DNA verification) and un-tabling progress toward Stage 5.
- **Related Analysis & Insights**:
  - Confirmed that Archive routines still lack bucketing (one file = one individual LLM call), which explains why ZIP scans feel slower compared to Certify's bin-packing for regular files.
  - The Integrity Guard "🪣 Bucket Traversal" check remains a placeholder (simple WalkDir existence test, not real validation of the BUCKET_LIMIT logic).
  - All work maintains the permanent anchors (read-only, advisory-only, dynamic base URL, canonicalize paths, user vaultRootPath, etc.).

**Updates to Supporting Files**:
- New focused preservation file: `CERTIFY_ZIP_BUCKETING.md` (self-contained reference for the purged logic).
- Multiple updates to ROADMAP.md (renames, status notes, tabled section).
- Updates to VAULT_ARCHITECTURE.md (cross-references, status).
- Minor supporting changes in src-tauri/src/lib.rs (partitioning for ZIPs, simplified regular report writing, new container handling block, debug flag threading, comment cleanups).
- UI and CSS adjustments for the COMPLETED label.

**Git & Process Discipline**:
- Followed rules: local commits frequent, one solid push at a clear checkpoint (pre-vault-browser work), detailed commit message.
- No information was purged from PROJECT_MEMORY.md or VAULT_ARCHITECTURE.md (only appended).

**Open Questions / Next Priorities (Carried Forward + New)**:
- Proceed with Rich Vault Browser implementation using ~RAA-CONTROL-Manifest.log as structural root (Phase 2 tree first).
- Decide which (if any) Phase 1 items to re-introduce after the visual structure is solid (e.g., selective Reveal actions, improved cards).
- Bring real bucketing/bin-packing to the Archive path (to address ZIP scan performance).
- Continue polishing: Settings page reorg (still mandatory), make Integrity checks reflect real state, uniform Integrity Guard revamp.
- Monitor token usage / comment bloat in active code (new dedicated preservation files like CERTIFY_ZIP_BUCKETING.md are the preferred pattern going forward).
- Revisit tabled suggestions (e.g., dev-only tree-shaking, version sync) only after explicit user approval in a dedicated discussion.

**Recommendations for Future Sessions**:
- Re-read the full PERMANENT KNOWLEDGE section + the most recent review (this one + 2026-06-06).
- Always consult CERTIFY_ZIP_BUCKETING.md before touching any remaining or future ZIP/container logic in Certify vs. Archive paths.
- Re-read ROADMAP.md "Granular Vault Architecture" section and the full VAULT_ARCHITECTURE.md before vault UI or bucketing changes.
- Maintain the clean separation: Certify focuses on loose files + container flagging; dedicated Archive for deep ZIP analysis.

**Overall Feeling:**  
Strong progress on cleanup, preservation discipline, and architectural hygiene. The project now has even better long-term memory separation (main architecture doc + focused historical files like CERTIFY_ZIP_BUCKETING.md). The recent push provides a solid backup. Ready to move into the visual/structural work on the Rich Vault Browser with the ~RAA-CONTROL-Manifest.log as the root, taking a Phase-2-first approach for cleanliness. Context is fully captured for any future reset.

---
**End of 2026-06-10 Review**

---
