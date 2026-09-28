# AI-Shorthand-RAA (compact - for .grok/rules/)

Compact rules for automatic injection. Extends the general AI-Shorthand system (see `docs/AI-Shorthand-Grok-v1.md` and the full `docs/AI-Shorthand-RAA-Gatekeeper.md` for details).

Use short RAA-specific tags to invoke strict enforcement of high-value, normally token-heavy rules without restating them.

## Tags & Enforcement (strict when tag is active)

**::raa::mission::** / **::raa::philosophy::** / **::raa::joe::**
- Everything serves "Joe" (regular end user downloading untrusted/AI code). Enforce "Trust, then Certify.", local-first, read-only, privacy-respecting, advisory-only. Human is always the final decision maker.

**::raa::permanent::** / **::raa::anchors::**
- Never modify user files. All verdicts are diagnostic only. Use dynamic Base URL (no hard-coded providers). Always use `fs::canonicalize` for paths. Respect user `vaultRootPath`. Keep `JUNK_NAMES` perfectly in sync everywhere. *A Certify job must never be allowed to appear successful if any audited file contains a violation.*

**::raa::vault::** / **::raa::granular::** / **::raa::onefile::**
- ONE FILE = ONE REPORT. Every audited file produces its own self-contained `.raa` (full analysis + clear verdict + DNA/SHA-256). All artifacts go inside a dated Job Folder. Hierarchy mirroring is strongly preferred.

**::raa::manifest::** / **::raa::control::**
- The static `~RAA-CONTROL-Manifest.log` (created first, `~` prefix) is the single source of truth for the entire job: inventory, hierarchy, and DNA Registry. Per-file `.raa` reports are secondary.

**::raa::certify::** / **::raa::archive::**
- Certify handles loose files + container flagging only. Deep per-file ZIP analysis happens exclusively in dedicated Archive mode (in-memory scan, reports emitted under `/Archive/`). No special ZIP bucketing logic inside normal Certify.

**::raa::integrity::** / **::raa::devonly::**
- Integrity Guard (including Forensic Vault safeguard) is **strictly dev-only**. It must remain completely invisible in production/release builds. Currently gated by `isDev` + `.dev-tab` class. Any change that would expose it in release builds is forbidden.

**::raa::git::** / **::raa::discipline::** / **::raa::push::**
- Commit locally very frequently (every coherent piece, ~15-40 min). Push to GitHub sparingly — only at clear milestones, end of session/day, or when explicitly requested. Ask before pushing mid-session. Always write good commit messages. Keep working tree reasonably clean. Respect real token/API cost of pushes.

**::raa::grok::**
- Run `grok inspect` when starting work in a new context or after major changes. Use `/flush` before compaction or at end of productive work. Cross-reference the exact files listed in AGENTS.md Documentation Map before touching auditing flows, manifests, per-file reports, job folders, or vault browser.

## Usage

Prefix queries with one or more tags:
`::raa::vault::granular::manifest/ How should I finalize the control manifest?`

`::raa::permanent::anchors::integrity::devonly/ Should this new check go in the Integrity Guard?`

Reset all tags with `::/`

Combine with general or other tags:
`::expert::rust::tauri::raa::permanent:: / ...`

When any `::raa::` tag is active, treat the corresponding rules in `AGENTS.md` and the referenced `docs/` files as hard constraints. Do not relax or go outside scope unless the user explicitly asks.

This compact file is intended for placement in `.grok/rules/ai-shorthand-raa.md` for automatic low-overhead injection. The full explanatory version lives in `docs/AI-Shorthand-RAA-Gatekeeper.md`.