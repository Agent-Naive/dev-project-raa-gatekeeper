# fredcrumb — dev-project-raa-gatekeeper

RAA-Gatekeeper: the read-only forensic auditor for the AI-agent era. Built on
the Restrictive Access Ability protocol — no code, command, or archive reaches
an LLM without a forensic safety certification. Read-only, advisory-only,
everything lands in a `.raa` vault entry. (Tauri v2 + Rust + Svelte 5.)

```
dev-project-raa-gatekeeper/
├── src/                              ← SvelteKit frontend
├── src-tauri/                        ← Rust backend: forensic engine, OS integration
├── docs/                             ← ROADMAP, VAULT_ARCHITECTURE, PROJECT_MEMORY +
│                                        4 AI-Shorthand docs (sweep pending: Vault or here?)
├── target/                           ← Rust build output (regenerate; not source)
├── node_modules/                     ← frontend deps (not catalogued)
├── test-directory01/ · test-directory02/ ← isolated test dirs
├── AGENTS.md                         ← agent operating notes
├── README.md                         ← setup, architecture, the RAA mandate
└── fredcrumb.md                      ← this file: what lives here and why
```

Fred was here.
