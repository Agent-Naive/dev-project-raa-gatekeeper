# fredcrumb — dev-project-raa-gatekeeper

**SHQL (ShorthandQL)** — the command language and agent ruleset for this project.
- If the task involves SHQL: read the ACTIVE binder first — `shql-binder-v1.3b/SHQL-v1.3b.md`. The binder is the authority, not memory or these notes.
- Per-command how-tos live in `shql-binder-v1.3b/help-files/`.
- Binders are versioned directories; the ACTIVE one is named in the tree below. Upgrades add a new versioned dir — never edit a binder in place.

RAA-Gatekeeper: the read-only forensic auditor for the AI-agent era. Built on
the Restrictive Access Ability protocol — no code, command, or archive reaches
an LLM without a forensic safety certification. Read-only, advisory-only,
everything lands in a `.raa` vault entry. (Tauri v2 + Rust + Svelte 5.)

```
dev-project-raa-gatekeeper/
├── src/                              ← SvelteKit frontend
├── src-tauri/                        ← Rust backend: forensic engine, OS integration
├── docs/                             ← ROADMAP, VAULT_ARCHITECTURE, PROJECT_MEMORY
│                                        (4 AI-Shorthand docs pruned 2026-09-28 ahead of the v1.3b upgrade;
│                                        Grok-v1 + v3 hash-verified against the Vault)
├── target/                           ← Rust build output (regenerate; not source)
├── node_modules/                     ← frontend deps (not catalogued)
├── test-directory01/ · test-directory02/ ← isolated test dirs
├── AGENTS.md                         ← agent operating notes
├── README.md                         ← setup, architecture, the RAA mandate
├── NEED-TO-UPGRADE-SHQL-v1.3b.md     ← SHQL flow needs v1.3b rework; new version coming shortly
├── shql-binder-v1.3b/               ← ACTIVE SHQL binder (v1.3b). Versioned dirs: upgrades add new, never edit in place.
└── fredcrumb.md                      ← this file: what lives here and why
```

Fred was here.
