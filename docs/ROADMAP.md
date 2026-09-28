# 🛡️ RAA Gatekeeper Roadmap

**Current Status:** Active Development — Phase 3 + 4 Core Complete + Granular Vault Architecture (per-file reports + job folders) in active use  
**Last Major Update:** Architecture now documented in VAULT_ARCHITECTURE.md. "New Path Forward" planning phase complete; ongoing implementation and polish work.

---

## ⚓ PERMANENT ANCHORS (DO NOT ALTER)

- **API Endpoint:** Uses dynamic Base URL (user-configurable). No hard-coded `x.ai`.
- **Path Protocol:** Always use `fs::canonicalize` for absolute path matching in vault entries.
- **Vault Pathing:** Respects user-selected `vaultRootPath` (falls back to `~/Documents/RAA-Vault`).
- **Read-Only Mandate:** The Gatekeeper never modifies user files.
- **Advisory Only:** All verdicts are diagnostic. Human remains the decision maker.

---

## ✅ Phase 1 + 2: Core Engine & Performance — COMPLETE

- [x] Terminal command auditing with Bible instant-recall (SHA-256 cache)
- [x] Deep file integrity scanning (hash + LLM)
- [x] Archive deep-peeking (ZIP contents audited without full extraction, 2MB safety valve)
- [x] Parallel hashing with Rayon across all CPU cores
- [x] Smart bin-packing / batching to reduce API calls
- [x] macOS junk filter (`__MACOSX`, `._` files)
- [x] Binary safety (string-read bypass)
- [x] Real-time dual-pane collection UI (Active vs Skipped)
- [x] High-visibility mission success toasts
- [x] "Bring Your Own LLM" (fully dynamic Base URL + Model Name)
- [x] Persistent settings via LocalStorage
- [x] Performance telemetry (LOCAL handoff ms + ORACLE total time)

---

## ✅ Phase 3 + 4: Forensic UI & Silent Monitoring — CORE COMPLETE

- [x] **Silent Watcher** — Kernel-level file monitoring with DNA change alerts
- [x] **DNA Toast + Forensic Queue** — "Teleport to Analyze" from watcher events
- [x] **Configurable Watcher** — Up to 5 folders, depth control (1-5), persisted
- [x] **Vault Tab** — Basic access to historical `.raa` files
- [x] **Integrity Guard** — 7/7 self-check dashboard (parallel hashing, disk-first, etc.). **This entire feature is intentionally dev-only** (the button is hidden behind `{#if isDev}` and a `.dev-tab` class). It must remain completely invisible in production/release builds. See the code comment in +page.svelte for enforcement details.
- [x] **User-Selectable Vault** — Full `vaultRootPath` propagation to all audit types
- [x] **Real-time Timers** — Handoff + total oracle timing on every operation
- [x] **Disk-First Verification** — Reports pulled from SSD, not just memory

### Currently In Progress / Next

- [ ] **Rich Vault Browser** — Proper file list, searchable, parsed incident cards, per-file DNA verification
- [ ] **DNA Verification UI** — "Does current hash match last vault entry?" indicator
- [ ] **Export Reports** — Clean Markdown export of any audit or full manifest
- [ ] **Violation History Dashboard** ("Wall of Shame")
- [ ] **Token Economy Counter** — Live batch density + estimated savings

---

## 🔐 Phase 4+ (Future)

- [ ] Local LLM support (Ollama / LocalAI toggle)
- [ ] Quarantine mode for high-confidence violations
- [ ] Cryptographic signing of manifests

---

## 🛠️ Phase 5 (v1.0 Vision)

- [ ] AI-suggested auto-remediation for common issues
- [ ] Dependency CVE scanning
- [ ] Environment hardening helpers

---

## 🚨 Critical Bugs / High Priority Fixes

*(No open critical bugs at this time. Historical issues from the monolithic report era have been addressed.)*

---

## 📝 Known Polish Items

- Settings layout: Model selector as dropdown + wider Base URL field
- **Mandatory UI Improvement (Deferred)**: Reorganize the entire Settings page into a button-driven / section-based layout (e.g. buttons or tabs for “AI Configuration”, “Silent Watcher”, “Vault & Paths”, etc.) instead of one long scrolling page. Goal is to eliminate excessive scrolling, especially painful on macOS Magic Keyboard. This is considered mandatory for better UX.
- Remove duplicated vault-fallback blocks in UI
- Make Integrity Guard reflect real vault path state instead of hardcoded true
- **Future Revamp (Deferred)**: Revamp ALL Integrity checks so they are uniform in their structure and execution. Currently a mix of real runtime tests (e.g. parallel hashing, bucket traversal, vault path existence) and hardcoded `true` assertions (e.g. ai_reasoning, zip_safety, disk_first_verification, terminal_input_lock). Goal: consistent pattern, clearer "guarded by design" semantics, easier testing/maintainability, and a single execution model across the entire Integrity Guard dashboard.
- **Critical (Dev-only enforcement)**: The Integrity Guard (including the new Forensic Vault safeguard) must remain strictly hidden in production builds. It is currently gated behind `isDev` (localhost only) in the nav. Any change that would expose it in release builds must be prevented. This is documented in code comments and here for future contributors.

**Tabled for Future Discussion (Integrity Guard hardening):**
- Use Vite `define` / `import.meta.env.DEV` + conditional imports to completely tree-shake the Integrity Guard code and styles out of production bundles (current `isDev` only hides the UI at runtime).
- Replace the hidden nav tab with a global dev-only keyboard shortcut (e.g. Ctrl+Shift+I or a custom combo) that opens an overlay. This keeps the main navigation clean even during development.
- Add an automated build test that asserts the Integrity Guard (and "dev-tab" class) is absent from production builds.
- Consider a minimal, safe "production health check" mode that can be enabled without leaking internal implementation details.
- Layout deep clean (global CSS consolidation, reduce inline styles, improve long-term maintainability and contributor friendliness)

---

## 🚀 Granular Vault Architecture: Per-File Forensic Reports + Job Folders

**Note:** This is the current architecture (previously referred to internally during planning as the "New Path Forward").  
See [docs/VAULT_ARCHITECTURE.md](docs/VAULT_ARCHITECTURE.md) for the full detailed technical reference, locked decisions, constraints, success criteria, and open questions. The summary below is kept for high-level visibility only.

**Core Philosophy**  
"Trust, then Certify." — taken to its logical conclusion: every individual file audited deserves its own first-class, self-contained forensic artifact.

**Major Architectural Shift**
- Move from "one large .raa file per job" (with many internal blocks) to **ONE FILE = ONE REPORT**.
- Every file that receives AI analysis (whether from a Certify folder run or from inside an Archive) produces its own dedicated `.raa` vault entry containing:
  - Full oracle analysis paragraph
  - Clear verdict
  - Its own DNA (SHA-256) hash
- All artifacts from a single user-initiated job are grouped inside a **dated Job Folder** inside the RAA-Vault.
- A master **~RAA-CONTROL-Manifest** (inventory control sheet) is created as the very first artifact on job start.

**Key Benefits Targeted**
- Much stronger long-term DNA ownership per file
- Dramatically easier post-facto analysis, after-action reporting, and singular culprit identification (filesystem + job folder becomes the index)
- Clearer forensic packages that Joe can understand and share
- Better real-time comfort UX during long jobs
- Natural hierarchy preservation for duplicate filename safety

**Important Commitments**
- Archive scanning ability is preserved (in-memory analysis of ZIP contents still works and will emit per-file .raa reports inside job folders).
- The existing skipped-files logic and bottom-bar temporary holding pattern are preserved for future stages.
- The right pane is initially repurposed strictly as a live COMPLETED comfort feed (showing .raa files as they are created in real time). It is not yet interactive/clickable.

**Enduring Principle (from the ALLSAFE era)**
> A Certify job must never be allowed to appear successful if any audited file contains a violation. This principle must be enforced in the new per-file + job folder model.

---

### Staged Implementation Plan (Summary)

Full technical details, locked decisions, constraints, and in-depth rationale live in [docs/VAULT_ARCHITECTURE.md](docs/VAULT_ARCHITECTURE.md). The summary below is for high-level visibility only.

- **Stage 1: ~RAA-CONTROL-Manifest** — Create dated job folder + write static `~RAA-CONTROL-Manifest.log` (inventory + hierarchy) as the very first artifact. **Status: Initial version complete** (inventory written early; later finalized with DNA Registry at job end).
- **Stage 2: Real-Time Right Pane Comfort Feed (COMPLETED pane)** — Show individual .raa files being emitted in real time during long jobs. **Status: In progress / partial**.
- **Stage 3: Per-File .raa Writing + Job Folders** — Emit individual rich `.raa` reports (with DNA) inside the job folder, mirroring source hierarchy where possible. **Status: Core emission working for Certify and Archive**.
- **Stage 4: Archive Path Adaptation** — Ensure ZIP scanning emits per-file .raa inside job folders with appropriate special handling. **Status: In progress**.
- **Stage 5: Vault Browser Evolution for Job Folders** — Make the vault UI understand job folders as containers, with the control manifest prominently visible. **Status: Explicitly tabled** until prior stages are stable. See VAULT_ARCHITECTURE.md.
- **Stage 6+: Future** — Clickable right pane, dynamic skipped display, job-level summaries, export, etc. (Parked).

See VAULT_ARCHITECTURE.md for the complete staged details, success criteria, and open questions.

---

*"Trust, then Certify."*

---

## 📋 Tabled Suggestions for Future Discussion (Credible Ideas — Do Not Implement)

**Purpose:** During recent vault architecture stabilization, Integrity Guard + Forensic Vault safeguard work, dev-only enforcement reviews, and name/version hygiene checks, several credible suggestions surfaced. Per user direction, these are captured here for structured discussion in a dedicated later session. 

**Rules for this section:**
- These items are **explicitly tabled**. No code changes, no roadmap checkboxes, no "implement this next" comments may reference them until reviewed and approved.
- Credibility is based on observed friction (e.g. vault subs + job folder population, safeguard addition to integrity, repeated version drift, dev/prod leakage risk, full "RAA-Gatekeeper" branding in UI/docs).
- Each entry includes a short "Why credible" note tied to current artifacts (VAULT_ARCHITECTURE.md, +page.svelte VAULT-CODE-SAFEGUARD markers, integrity checks, 5 version locations, etc.).

### 1. Single Source of Truth + Automated Sync for Version Numbers (5+ locations)
**Locations currently observed:** root `Cargo.toml` (0.3.0), `src-tauri/Cargo.toml` (0.4.0), `package.json` (0.4.0), `src-tauri/tauri.conf.json` (0.4.0), hardcoded `v0.4.0` in `+page.svelte` version-tag, plus README badge and any future about dialog.
**Suggestion:** Choose one canonical source (e.g. tauri.conf.json "version" or a new `.raa-version` file) and either (a) a `npm run version:sync` script or (b) build-time injection via Vite/Rust env. Add a pre-commit or CI step that fails on drift.
**Why credible:** Root Cargo already lagged after 0.4 work; manual edits are error-prone on half-level bumps. Directly supports the "are we ready for a version change" and "confirm edits of all 5 files" workflow.

### 2. Make the "📜 Forensic Vault" Integrity Check Actually Exercise the Categorized Architecture
**Current state:** The vault_path item in `check_integrity` / integrity grid is essentially "does the root dir exist?". The 4 static subs (Certify/Archive/Analyze/Audit) + dated job folder + ~RAA-CONTROL-Manifest logic now exist in both Rust writer and Svelte finder grouping.
**Suggestion:** Enhance (still dev-only) the vault safeguard check to:
- Verify or lazily ensure the 4 static subdirectories exist under vaultRootPath.
- Detect presence of at least one dated job folder pattern or a ~RAA-CONTROL-Manifest.log.
- Optionally return lightweight stats (job folder count, total .raa under subs) for the UI badge.
**Why credible:** The safeguard was added precisely because vault code (job folders, per-file .raa, subs) is now non-trivial. A basic dir-exists check no longer meaningfully "guards" the new architecture. See VAULT-CODE-SAFEGUARD markers in +page.svelte and the "📁 Hidden Vault" / Forensic Vault item.

### 3. Compile-Time / Bundle Stripping for Integrity Guard + All Vault Safeguard Code (Beyond Runtime isDev)
**Current state:** Strong double guards (`{#if isDev}` on button + `isDev && activeTab === "integrity"` on content) + extensive comments exist in +page.svelte. `isDev` is set from `window.location.hostname === "localhost"`.
**Suggestion:** 
- Use Vite `define: { __DEV_INTEGRITY__: 'false' }` (or `import.meta.env.DEV`) + dynamic `import()` for the integrity tab contents / runIntegrityCheck so the code and its CSS are never emitted in production `build/`.
- Add a small post-build verifier (Node script) that greps the final assets for forbidden tokens ("Integrity Guard", "dev-tab", "VAULT-CODE-SAFEGUARD", "Forensic Vault:") and fails `tauri build` in release mode.
**Why credible:** Runtime hiding is good but not sufficient for "must remain strictly hidden in final releases" requirement. Tree-shaking + assertion gives stronger guarantee and reduces prod bundle size. Already partially tabled in the integrity tab comment block itself.

### 4. Vault Finder Robustness: Auto-Refresh + Better Initial Population Diagnostics
**Current state:** `loadVaultData` + `getGroupedVaultReports` groups under the 4 subs and handles job folders inside them. Manual ⟳ refresh exists. History showed initial population issues when only subs + dated folders were present (0 reports shown despite files on disk).
**Suggestion (tabled):** 
- Wire a lightweight Tauri fs watcher (or use existing notify) on the vault root + 4 subs so new job folders from a just-finished Certify/Archive appear automatically in the vault tab.
- Improve empty-state messaging with explicit "expected structure" hint and a "Create default subs" helper (if user somehow has a bare vault root).
- Persist last-used search filter + last-selected sub across tab switches.
**Why credible:** The "New Path Forward" is now the live architecture. A finder that requires manual refresh after every job feels incomplete for Joe. The subdir creation + population logic is now central; making the UI reflect it reliably without extra clicks improves the "trust" part of "Trust, then Certify."

### 5. Introduce Explicit `raa_format_version` (or equivalent) in Control Manifests and Per-File Reports
**Current state:** Control manifests and .raa files carry rich data (inventory, DNA, verdicts) but no declared schema version.
**Suggestion:** Add a small header field (e.g. `raa_format_version: "1.1"` or `"per-file-job-folder-v1"`) at the top of ~RAA-CONTROL-Manifest.log (and inside the JSON structure of .raa reports if they become structured). On read, gate future parsing logic behind the version.
**Why credible:** The granular architecture is a breaking change from the old monolithic report era. Future evolutions (more metadata, signing, different DNA placement, container file handling) will happen. A version marker prevents silent mis-parsing of mixed-era vaults and makes the "DNA ownership per file" story future-proof. Fits naturally with the locked decisions in VAULT_ARCHITECTURE.md.

### 6. User-Facing Name Hygiene & Header/Title Consistency Pass
**Current state:** Window title and productName are "RAA-Gatekeeper" (good). Some strings use "RAA Gatekeeper" (space) or "RAA-Gatekeeper". app.html still has the generic Tauri title. Version badge is hardcoded. Header has a 3-column layout (logo/title | toasts | version).
**Suggestion:** 
- Canonicalize on the exact string "RAA-Gatekeeper" (hyphen, title-case) in all user-visible UI text, toasts, subtitles, about, error messages, and docs that Joe might read.
- Update `<title>` in app.html to "RAA-Gatekeeper".
- Make the version tag in the header pull from a single source (see item 1) instead of being duplicated.
- Review header-center for any remaining generic or shorthand text.
**Why credible:** User has repeatedly asked about "the app itself as it appears top right hand corner of RAA-Gatekeeper", "how about the app itself", and "confirm edits of all 5 files" for name/version. Inconsistent casing leaks in UI strings undermine the professional forensic presentation.

### 7. Add an Explicit "Release Readiness — Dev-Only Areas" Checklist Item to ROADMAP
**Suggestion:** Create (or expand the existing "Critical (Dev-only enforcement)" polish note) a short, copy-pasteable checklist under a new "Release vX Readiness" heading:
- [ ] Integrity Guard button and tab content are double-guarded by `isDev` (or stronger).
- [ ] No "Integrity", "dev-tab", "VAULT-CODE-SAFEGUARD", or Forensic Vault safeguard strings appear in production build artifacts (add build assertion).
- [ ] All VAULT-CODE-SAFEGUARD comments have matching implementation that still passes the integrity check when run in dev.
- [ ] Full "RAA-Gatekeeper" name appears in window, header, and about; no user-facing shorthand.
**Why credible:** The requirement "integrity area is only visible during dev sessions and not visible during final release" has been stated multiple times. Capturing the verification steps in one place reduces risk of regression on future bumps or refactors. Complements the existing permanent anchors and Known Polish sections.

### 8. (Reinforced Polish) Settings Page Non-Scrolling Organization
(Already called out as "Mandatory UI Improvement (Deferred)" and "Future Revamp".) Re-iterate here for visibility: reorganize Settings into immediately reachable sections or a left-nav + right-panel layout (AI Core, Vault & Paths, Silent Watcher, Filters, etc.). Long vertical scroll is painful on laptop + Magic Keyboard/trackpad.
**Why credible:** Directly improves the daily driver experience for the primary protective workflow. Low risk, high Joe benefit.

**End of tabled section.** When discussing, reference the specific item numbers and the "Why credible" context above. After approval of any subset, move the approved items into the appropriate Phase / Polish / Future section with owners and success criteria, then delete from here.

---

*"Trust, then Certify."*