# VAULT_ARCHITECTURE.md

# Comprehensive Reference — Granular Per-File Forensic Vault Architecture

This is the detailed technical design document describing the current architecture for per-file `.raa` reports, dated job folders, and the `~RAA-CONTROL-Manifest.log`.

It was previously named RAA-NEWPATH-FORWARD.txt during the initial design phase.

================================================================================
## 1. CORE VISION & WHY WE ARE DOING THIS
================================================================================

Primary Goal:
Move RAA-Gatekeeper from "one big .raa report per job" to a true **ONE FILE = ONE REPORT** model.

Every individual file that receives an AI forensic audit (whether it lives loose in a folder or inside a ZIP) will produce its own self-contained `.raa` vault file. That vault entry will contain:
- The full oracle "Analysis" paragraph (not just verdict)
- A clear verdict
- The file's DNA (SHA-256 content hash)

All artifacts from one user-initiated run (Certify folder or Archive) are grouped inside a single dated **Job Folder** inside the user's RAA-Vault.

As the very first action on "Start", we create:
- The dated Job Folder
- A special `~RAA-CONTROL-Manifest.log` (static name — always the same in every job folder) as the audit control sheet / master inventory inside it

This gives Joe (and the creator) a clean, professional, forensic-grade package that is easy to review later, easy to reason about after the fact, and gives every file strong, independent DNA ownership.

User's explicit reasons (captured during deep reasoning):
- Each file owning its own hash in its own vault entry is easier to deal with long-term for DNA verification.
- After a run of 80 files (75 reports + 5 skipped), it is far easier to do post-facto analysis, after-action reporting, or find a singular culprit when you have discrete files + a master control manifest than when you have to parse one giant monolithic report.
- Job folders + hierarchy mirroring make the vault feel organized and make duplicate filename situations safe.
- The right pane provides real-time comfort (COMPLETED feed) while a long job is running.

This is a personal hobby / re-learning / "masterpiece" project. Coding cost and long-term maintenance burden are secondary to building something the creator is proud of and that genuinely protects "Joe".

================================================================================
## 2. KEY ARCHITECTURAL DECISIONS (LOCKED IN DURING REASONING)
================================================================================

1. ONE FILE = ONE REPORT
   - No more single aggregated report with dozens of "--- RAA FILE ANALYSIS ---" blocks + one OVERALL header as the primary persisted artifact.
   - Each audited file gets its own `.raa`.

2. Job Folders (dated containers)
   - Example: `MyProject-20251001-143022/` or `cool-repo.zip-20251001-143022/`
   - All per-file .raa reports + the control manifest live inside this folder.
   - Old flat naming style (`Gatekeeper-certify-... .raa` directly in vault root) is being phased toward this model.

3. Hierarchy Mirroring (strongly preferred)
   - When writing per-file reports, mirror the original source folder structure inside the job folder whenever possible.
   - This provides natural duplicate-name protection and makes after-action navigation familiar.

4. ~RAA-CONTROL-Manifest (First Stage)
   - Created immediately on job start, before any file reading or LLM work.
   - Uses the **static name** `~RAA-CONTROL-Manifest.log` in *every* job folder (no dynamic naming or variables needed).
   - Contains the complete inventory + full hierarchy of every file that will be audited.
   - The `~` prefix ensures it always sorts to the very top of the job folder when viewed in Finder/Explorer.
   - Acts as the permanent "audit control sheet" and scope declaration for that job.

5. Archive Scanning Is Preserved
   - We will continue to support pointing at a .zip and getting per-file analysis without forcing manual extraction first.
   - The in-memory ZipArchive path stays.
   - It will now emit per-file .raa reports inside a job folder (instead of one aggregated archive report).
   - DNA verification for container-origin files will still need special handling (📦 states, limited re-verifiability).

6. Right Pane Role (Current vs Future)
   - **Current (near-term):** Pure live COMPLETED comfort feed.
     - Shows .raa files being created in real time as the job progresses.
     - Gives the user visible forward progress and psychological comfort during long Certify or Archive runs.
     - Not intended for click-to-open or deep interaction yet (the "YET" <wink>).
   - **Future possibilities (explicitly parked):**
     - Right pane could become a live view of the actual vault content.
     - Right pane could dynamically show skipped files + reasons (extension filter, junk, failure, etc.).
     - Eventually a dedicated Reports section (possibly with PDF output) that summarizes an entire job using the control manifest + the individual .raa files.

7. Skipped Files Handling
   - All existing skippedFiles state, collection logic, reset logic, and the current bottom comma-separated bar are to be preserved exactly as-is for now (this was the explicit "temporary holding pattern" request).
   - Future stages may move or enhance skipped display into the right pane.

================================================================================
## 3. IMPLEMENTATION STATUS (SUMMARY)
================================================================================

**Note:** This document has been condensed. Detailed historical narratives for completed early stages of the "New Path Forward" have been summarized in ROADMAP.md (see the "Granular Vault Architecture" section there). The original full staging details are preserved in git history.

Current focus (see also ROADMAP.md and Open Questions below):
- Stabilizing per-file .raa emission + job folder creation (Certify and Archive).
- Finalizing the `~RAA-CONTROL-Manifest.log` (inventory + DNA Registry) as the reliable single source of truth.
- Right pane as live comfort feed.
- (Parked) Full job-folder-aware vault browser (Stage 5 in historical numbering).

Full constraints, locked decisions, and remaining open questions are in the sections below. Success criteria for active work are referenced in ROADMAP.md.

================================================================================
## 4. TECHNICAL MAPPING (HISTORICAL / WHERE THINGS LIVE)
================================================================================

**Note:** This section is retained for reference from the original planning. Many "will need to" items have since been implemented or are in progress (see ROADMAP.md for current status). The core mapping of responsibilities remains useful.

Rust side (src-tauri/src/lib.rs):
- generate_manifest (the big bucketing + LLM loop for Certify)
- scan_compressed_archive (the per-file in-memory ZIP walker)
- log_to_raa (the writer — evolved to support job folders)
- resolve_vault_root + related vault helpers
- list_vault_files, read_single_vault_file, delete_vault_file
- FileJob struct, bucket logic, FileAnalysis JSON struct
- RAAReport return type (hardened for is_error)

**Junk / Exclusion Filter (shared constant `JUNK_NAMES` in lib.rs):**
This list controls which files and directories are completely ignored during auditing and control manifest generation.
Current contents (as of latest update):
- node_modules
- .git
- target
- dist
- __MACOSX
- .DS_Store          ← Added 2026-05-30 (macOS Finder metadata)

This list should be kept in sync between `generate_manifest` and control manifest building.

Svelte side (src/routes/+page.svelte):
- handleCertifyFolder and handleBrowseArchive
- resetResults (clears activeFiles + skippedFiles)
- scan-event listener (Active vs Skipped)
- activeFiles / skippedFiles state and the dual-pane + right-pane rendering
- extractRecordedHashes + DNA verification logic (now handles per-file entries)
- currentReport, certMsg, timers, etc.

CSS (src/app.css):
- .dual-pane-monitor and .pane rules
- Styles for right-pane comfort feed and vault list

Other important files:
- docs/PROJECT_MEMORY.md (permanent mission + push discipline + capture points)
- docs/ROADMAP.md (high-level plan and status)

================================================================================
## 5. NAMING & CONVENTIONS
================================================================================

Job Folder naming (proposed, confirm during implementation):
- Pattern: <sanitized-target>-<YYYYMMDD-HHMMSS>/
  Examples:
  - MyCoolProject-20251001-143022/
  - cool-repo.zip-20251001-143022/

Control Manifest:
- Always uses the **static name** `~RAA-CONTROL-Manifest.log` in every job folder.
- The `~` prefix guarantees it sorts to the top.
- Chosen as `.log` for clarity (forensic log / audit control sheet).

Per-file reports inside the job folder:
- Should be short and meaningful (ideally just the original basename + .raa, or the mirrored relative path).
- The job folder name carries the timestamp and job identity, so individual filenames can be much cleaner than the old flat style.

================================================================================
## 6. JOE EXPERIENCE TARGETS
================================================================================

- Joe downloads a repo zip or folder.
- He clicks Start.
- He immediately sees a job folder appear in his RAA-Vault containing a prominent `~RAA-CONTROL-Manifest.log` at the very top. This tells him "the Gatekeeper has taken a complete inventory of what you asked it to look at."
- As the job runs, the right pane shows individual reports being created in real time — he feels progress instead of staring at a spinner.
- When finished, he has a clean dated package: one control manifest + many small, high-quality, per-file .raa reports (with hierarchy preserved).
- Later, doing after-action work is dramatically easier than digging through one giant file.

================================================================================
## 7. IMPORTANT CONSTRAINTS & PRESERVATIONS
================================================================================

- Do NOT delete or refactor the existing skippedFiles collection/processing logic during early stages. It is being held in the bottom bar as a temporary pattern.
- Keep the existing report colorization removal (plain <pre> / raw text) until we explicitly decide to bring highlighting back.
- Maintain zero warnings / zero errors discipline.
- Follow strict git push rules (local commits frequent, pushes only at clear milestones or end of day, ask first).
- Enduring principle: A Certify job must never be allowed to appear successful if any audited file contains a violation. This must be enforced in the new per-file + job folder model.
- Vault browser is currently flat-only (no job folder navigation). Full directory-style browsing of the vault is explicitly tabled until the ~RAA-CONTROL-Manifest.log experience is finalized. See the detailed note in Stage 5 and the TODO in list_vault_files.

================================================================================
## 8. OPEN / TABLED QUESTIONS (FROM REASONING SESSIONS)
================================================================================

- Exact naming pattern for job folders (tabled the zip-specific variant for now).
- Internal format and content of the static `~RAA-CONTROL-Manifest.log` (plain text hierarchy? Include pre-computed hashes? Job metadata header?).
- How much metadata should live in the control manifest vs. being reconstructible from the individual .raa files.
- Whether the control manifest should ever be "live updated" later or remain a pure point-in-time snapshot.
- Right pane future evolution timing and scope.
- How the old flat .raa files in existing vaults will be handled during transition (coexistence period?).
- Whether we eventually want any kind of job-level summary .raa in addition to the per-file ones and the control manifest.

================================================================================
## 9. REFERENCES & RELATED DOCUMENTS
================================================================================

- ROADMAP.md (the public staged plan)
- PROJECT_MEMORY.md (permanent mission, push rules, capture points)
- src-tauri/src/lib.rs (core engine)
- src/routes/+page.svelte (UI and event handling)
- src/app.css (layout rules for panes)

================================================================================
## 10. SUCCESS CRITERIA FOR STAGE 1 (~RAA-CONTROL-Manifest) — HISTORICAL
================================================================================

**Note:** This section is retained for historical reference. Stage 1 success criteria have largely been met (initial implementation landed; see ROADMAP.md for current status of related work). Full original text preserved below for completeness.

When Stage 1 is complete, the following should be true:
- Clicking Start on either Certify or Archive immediately creates a dated job folder in the vault.
- The static file `~RAA-CONTROL-Manifest.log` is the very first thing written into that folder.
- The manifest accurately lists the full hierarchy of every file that will be audited (using the same filtering logic the rest of the job will use).
- The right pane (once Stage 2 is done) can announce the creation of this manifest.

================================================================================
## 11. FUTURE / DEFERRED WORK (Outside Core "New Path Forward" Planning)
================================================================================

### Uniform Integrity Checks Revamp (Deferred)
- **Goal**: Later revamp ALL Integrity checks (the 7-item self-check dashboard) so they follow a single, uniform structure and execution model.
- **Current State (as of now)**: Mix of actual runtime tests (parallel_hashing via rayon, bucket_traversal via walkdir, vault_path existence) and hardcoded `true` assertions (ai_reasoning, terminal_input_lock, zip_safety, disk_first_verification). The "📁 Hidden Vault" / vault_path item is currently just a directory existence check.
- **Desired Outcome**:
  - Consistent pattern for every check (e.g. same way of computing, same way of reporting status, same "guarded by design" semantics).
  - Clearer distinction between "real runtime verification" vs. "architectural guarantee".
  - Easier to extend, test, and document.
  - Potentially rename/restructure "Hidden Vault" into "Forensic Vault" to better reflect the new granular architecture (static subs, job folders, etc.).
- **Why Deferred**: Not blocking the current New Path Forward stages (manifests, per-file reports, job folders). This is a polish + architectural consistency task for the Integrity Guard area.

**Additional Note (Tabled for Discussion):** The Forensic Vault safeguard check (subs existence + exercising the creation logic) lives inside the dev-only Integrity Guard. It is intentionally invisible in release builds. See the expanded dev-only enforcement notes and tabled suggestions in ROADMAP.md and the code comments in +page.svelte.

See CERTIFY_ZIP_BUCKETING.md for the preserved historical details of how .zip files were specially collected and given dedicated buckets inside the Certify bucketing logic (before the shift to container-only treatment + separate Archive audits).
- **Related**: See also the "Known Polish Items" section in ROADMAP.md.
- No existing skipped logic, DNA logic, or permanent anchors have been broken.
- Joe can open the job folder in Finder and immediately see the control sheet at the top.

================================================================================
This document (condensed from its original "New Path Forward" form) exists so that the full vision, locked architectural decisions, constraints, and rationale of the Granular Vault Architecture are preserved and easily referenced, even across long gaps or if the main AI context is reset.

Historical implementation details for early stages have been summarized in ROADMAP.md to reduce redundancy on finished work. See git history for the original full text if needed.

================================================================================
## Cross-Reference: Tabled Suggestions
See the new "📋 Tabled Suggestions for Future Discussion (Credible Ideas — Do Not Implement)" section at the end of ROADMAP.md. It captures several vault-related, integrity-safeguard, versioning, and dev-only enforcement ideas that surfaced during the granular architecture work. No content was removed or altered in this file; the tabled items are for future discussion only and must not be acted upon until reviewed and approved in a dedicated session.
================================================================================