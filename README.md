# 🛡️ RAA-Gatekeeper
**The Read-Only Forensic Auditor for the AI-Agent Era.**

[![Project Status: Active](https://shields.io/badge/status-active-green)](https://github.com/Agent-Naive/RAA-Gatekeeper)
[![Version: 0.4.0](https://shields.io/badge/version-0.4.0-blue)](https://github.com/Agent-Naive/RAA-Gatekeeper)
[![Architecture: Tauri v2](https://shields.io/badge/architecture-Tauri_v2-blue)](https://tauri.app)
[![Backend: Rust](https://shields.io/badge/backend-Rust-orange)](https://rust-lang.org)

## 📖 Overview
**RAA-Gatekeeper** is a specialized security middleware designed to solve the "Trust Gap" between local codebases and AI Agents. Built on the **Restrictive Access Ability (RAA)** protocol, it serves as a **Read-Only Auditor**, ensuring that no code, terminal command, or archive is passed to an LLM without a forensic safety certification.

---

## 🛠️ Prerequisites & Dependencies

Before building the project, ensure the environment meets the following requirements:

### **1. System Requirements**
*   **Rust Toolchain:** [Install Rust](https://rust-lang.orgtools/install) (stable version).
*   **Node.js:** [Install Node.js](https://nodejs.org) (v18 or higher recommended).
*   **Package Manager:** `npm`, `yarn`, or `pnpm`.
*   **Tauri Dependencies:** Follow the [Tauri Setup Guide](https://tauri.appv1/guides/getting-started/prerequisites) for your OS (Windows C++ Build Tools, Linux `libwebkit2gtk`, etc.).

### **2. Recommended IDE Setup**
*   [VS Code](https://visualstudio.com)
*   **Extensions:**
    *   [Svelte for VS Code](https://visualstudio.com)
    *   [Tauri Extension](https://visualstudio.com)
    *   [rust-analyzer](https://visualstudio.com)

---

## 🚀 Getting Started

### **Installation**
1. Clone the repository:
   ```bash
   git clone https://github.com/Agent-Naive/RAA-Gatekeeper.git
   cd RAA-Gatekeeper
   ```
2. Install frontend dependencies:
   ```bash
   npm install
   ```

### **Development Mode**
Run the application in a live development window:
```bash
npm run tauri dev
```

### **Building for Production**
Generate a bundled executable (MSI, AppImage, or DMG):
```bash
npm run tauri build
```

---

## 🧠 Technical Architecture

*   **Frontend:** Svelte 5 / SvelteKit (UI Logic & State Management).
*   **Backend:** Rust / Tauri v2 (Forensic Engine & OS Integration).
*   **Parallelism:** `Rayon` for multi-core SHA-256 file hashing.
*   **Intelligence:** Integrated with xAI Grok for heuristic auditing (fully dynamic Base URL + Model Name).
*   **Architecture:** Granular per-file `.raa` reports + dated job folders (see [docs/VAULT_ARCHITECTURE.md](docs/VAULT_ARCHITECTURE.md) for the full design).

---

## ⚓ The RAA Mandate
1.  **READ-ONLY:** The Gatekeeper **never** modifies user files.
2.  **ADVISORY ONLY:** The app provides forensic diagnostics; the human pilot remains the decision-maker.
3.  **AUDITABLE:** Every interaction is recorded in a `.raa` forensic vault entry.

---

## 📂 Project Structure
*   `src/` - SvelteKit frontend routes and components.
*   `src-tauri/` - Rust backend, commands (`lib.rs`), and configuration.
*   `raa-test/` - Isolated directory for generated forensic vault entries.

---

## 📚 Documentation & References

- [docs/ROADMAP.md](docs/ROADMAP.md) — High-level status, phases, and current plan
- [docs/VAULT_ARCHITECTURE.md](docs/VAULT_ARCHITECTURE.md) — Detailed technical design for the granular per-file reports + job folder architecture
- [docs/PROJECT_MEMORY.md](docs/PROJECT_MEMORY.md) — Persistent project knowledge, rules, and session history

## ⚖️ License
Proprietary / RAA-Certified.

*"Trust, then Certify."*
