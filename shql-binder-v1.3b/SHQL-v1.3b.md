# ShorthandQL v1.3 beta (SHQL-v1.3b) — Loadable Language Definition

**Name:** ShorthandQL version 1.3 beta  
**File:** `SHQL-v1.3b.md`  
**Status:** Project-local loadable definition (Project Newborn)  
**Activation:** **Opt-in.** Default OFF in sessions. User turns ON/OFF via `AGENTS.md` toggles (`::@load::SHQL-v1.3b.md/`, “turn on/off shorthand”, `::@unload/`, etc.).  
**Dialect:** ShorthandQL management commands use `::@…/`. **DADO is not SHQL prefix syntax** — the command is the plain word **`DADO`** only (not `::DADO/`, not `@DADO`, not `::@DADO/`). Negation uses `!` on **tags only**.

**Binder layout:** this file is the lean core — syntax, parser, errors, and command *contracts*. Full howtos for substantial commands live in `help-files/`, one binder per command group. See §2.2 for the map.

---

## Strict System Instructions (when this file is loaded / ON)

You natively implement ShorthandQL. This document is the active definition. When the user has **not** turned shorthand ON, this file is inactive — do not enforce it.

On **every** user message:

1. Fully validate and parse any leading `::…/` prefix **before** reading or reasoning about text after the first `/`.
2. If the prefix is invalid → return the exact error message and **stop** (do not answer the query).
3. Update the active tag set and lock state; execute any management command.
4. If a command and a query share one message: run the command first, then answer the query under the **resulting** state.
5. When tags are active: open with `Focusing on ::chain::/.` then answer inside positive domains only (scale by strength; suppress negated tags).
6. Prefer raw short form. Do not expand tags into natural-language meta.
7. Do not mention “ShorthandQL” or this system unless the user uses `::@help/` or explicitly asks about it.
8. If the user says **DADO**, apply Digest and Discuss Only (see §2.1). Tags still apply (Focusing line still required when tags are active).

**Lock (default: locked = true)**  
- `::@lock/` → locked. `::@unlock/` → unlocked.  
- `::/` and `::@clear/` clear tags **and** reset lock to **locked**.  
- While locked: only actions **explicitly** requested in the current message. No unsolicited tools, fixes, refactors, or “helpful” continuations. You may still *suggest* ShorthandQL commands (e.g. capture).

**When SHQL unloaded / OFF:** Tag/`::@` rules apply only while SHQL is ON. After unload, only SHQL on/off toggles (and minimal help) remain for shorthand. **DADO** is not SHQL state — it is just an acronym/command meaning Digest and Discuss Only when the user says it.

---

## 1. Core Syntax

```
::tag1::tag2^N::!neg^M/ optional query text
```

| Rule | Detail |
|------|--------|
| Start | Must begin with `::`, else **Plain** (answer under current tags, if any) |
| Chain | Further tags separated by `::` |
| Terminator | **First** `/` ends the prefix; rest (lstrip) is the query (later `/` in URLs is fine) |
| Case | Tag names are case-insensitive → store **lowercase** |
| Strength | `^N` with integer **N ≥ 1**; omit → 1. Display omits `^1` |
| Negation | `!name` or `!name^N` on a **tag** segment |
| Reset | `::/` or `::@clear/` clears all tags and resets lock to locked |
| Reserved | Plain **`DADO`** is an acronym outside `::…/` (see §2.1), not SHQL syntax. |
| Preference | Raw short form when this definition is loaded |

**Tag model:** ordered unique triples `(name, negated, strength)`.  
**Last-write-wins (Lwins)** by name (later occurrence replaces earlier name, including neg/strength).  
**Normal tag prefix** → **replace** entire active set.  
**Plain text** (no leading `::`) → keep active set; answer under it.

Display form: `name`, `name^N`, `!name`, `!name^N`.  
Focus chain form: `::expert^3::rust::!async^2::/` (omit `^1`).

---

## 2. Management Commands (`::@…/`)

Commands start with `@` on the first segment. Execute + minimal ack. Strength is **not** allowed on command names. (**DADO** is plain English/acronym, not `::@…/` — §2.1.)

Commands marked with a binder path keep only their contract here; the full howto (syntax variants, parser steps, state tables) lives in that binder file. Pull the binder when implementing the command from scratch or when the contract below is not enough.

| Command | Effect | Ack (typical) | Binder |
|---------|--------|----------------|--------|
| `::@list/` · `::@list::filter/` | List active tags; optional substring filter (first arg only). If locked, prefix with `Locked.` | `Locked. Active tags: expert^3, rust` | — |
| `::@add::t1::!neg^2/` | **Mutate:** add tags (neg/strength). Lwins within add | `Added. Current: ::…/` | — |
| `::@remove::t1::t2/` | Remove by **name** only (ignore `!`/`^` on args) | `Removed. Current: ::…/` | — |
| `::@save::name/` | Snapshot current set → conversation preset (name lowercased) | `Saved preset 'name'.` | — |
| `::@preset::name/` | **Replace** active set with preset | `Preset 'name' applied. Current: ::…/` · or `Unknown preset 'name'.` | — |
| `::@clear/` or `::/` | Clear tags; reset lock → locked | `Tags cleared.` | — |
| `::@lock/` | Enter locked mode | `Locked. Strict mode active — no unsolicited changes.` | — |
| `::@unlock/` | Exit locked mode | `Unlocked. Normal behavior restored.` | — |
| `::@load::SHQL-v1.3b.md/` | Load this definition (aliases: `spec`, `SHQL-v1.3b`, `SHQL-v1.3b.md`). Requires a definition name — missing → **MissingCommandArgument** | `Loaded ShorthandQL v1.3 beta.` | — |
| `::@unload/` | Unload / turn OFF this definition (optional filename arg) | `Unloaded ShorthandQL v1.3 beta.` | — |
| `::@help/` | Loaded: concise command/syntax ref (include plain **`DADO`**). Unloaded (host): command names + pointer to this file | (minimal when unloaded) | — |
| `::@safe/on/` · `::@safe/off/` · `::@safe/dado/` · `::@safe/status/` | The model's private vault ("the safe"). Arm/disarm session vault checks; `dado` = digest & discuss vault contents (read-only); `status` = list vault contents via the index | `Safe on.` · `Safe off.` · (digest) · (listing) | `help-files/SHQL-help-@safe.md` |
| `::@capture[-lN][+]::file/` | **Host-only.** Capture last N content turns to file. Bare = `-l1`. `+` = full raw scaffolding | `Capture saved <file>.` | `help-files/SHQL-help-@context.md` |
| `::@compact[-lN]::file/` | **Host-only.** Capture then shrink the live window to a digest of the file | `Compact saved <file>.` | `help-files/SHQL-help-@context.md` |
| `::@restore/[::file/]` | **Host-only.** Put a compact file back into the live window | `Restored <file>.` | `help-files/SHQL-help-@context.md` |
| `::@scrape[-dN][::selectors]/ URL` | **Host-only.** Fetch URL (+ same-host depth), extract with selector tags | `Scrape URL.` · `Scrape URL depth N.` | `help-files/SHQL-help-@scrape.md` |
| `::@return/[::full/outline/digest/]` | **Host-only.** Shape the pending scrape extract. Does not fetch | `Return digest.` · `Return full.` · `Return outline.` | `help-files/SHQL-help-@scrape.md` |
| `::@temperature::N/` · `::@temperature::default/` · `::@temperature::clear/` · `::@temperature/` | **Host-only.** Session sampling temperature, real number in host clamp (default **0.0–2.0**). `default` / `clear` / bare = project default | `Temperature 0.7.` · `Temperature default.` | — |
| `::@context::N/` · `::@context::default/` · `::@context::clear/` · `::@context/` | **Host-only.** Effective context length, integer 1 … max_seq_len (cannot exceed model max). `default` / `clear` / bare = project default | `Context 64 (max 128).` · `Context default.` | — |
| `::@effort::N/` · `::@effort::default/` | **Host-only.** Session reasoning depth, integer **1–5** | `Effort 3.` · `Effort default.` | — |

**Notes**

- `@load` / `@unload` = **language definition**, not presets. Presets use `@save` / `@preset`.
- `@save` does not change the active set; query (if any) uses pre-save tags.
- Extra segments after a valid command arg (e.g. list filter, add/remove tags): still execute; prepend  
  `**Note:** …` (commands only — never on normal tag queries).
- Unknown `@name` that is not a valid command → **MalformedCommand**.
- **`@temperature` / `@context` / `@effort` = host runtime knobs** (sampling, truncation, reasoning depth). The model does not “feel” them unless the host applies them. `N` for `@effort` = integer 1–5; `default` / `clear` / bare = project default. Invalid → **InvalidEffort**. Empty `::@effort::/` → **MissingCommandArgument**.
- Strength (`^N`) is **not** allowed on command names (including `temperature` / `context` / `effort` / `scrape` / `compact` / `restore` / `return` / `capture` / `safe`).

---

## 2.1 DADO (permanent knowledge; not SHQL syntax)

**DADO** = **D**igest **a**nd **D**iscuss **O**nly.

Always-known acronym (Grok rules / AGENTS). Not a toggle, not SHQL load state, not `::DADO/` or `@DADO`.

When the user says **DADO**, resolve and apply for that request: digest, discuss, options; no silent default execute; no implementation.  
When they do not say DADO, normal build rules apply.

---

## 2.2 Command binders (the map)

Substantial commands keep their full howto outside this spec. One binder per command group, under `help-files/`:

| Binder | Commands | Contents |
|--------|----------|----------|
| `help-files/SHQL-help-@safe.md` | `@safe` | Vault routing: index-before-payload, pull discipline, write-once, worked examples, self-prompt |
| `help-files/SHQL-help-@context.md` | `@capture`, `@compact`, `@restore` | The context pipeline: turn counting, file paths, capture→compact→restore flow, failure semantics |
| `help-files/SHQL-help-@scrape.md` | `@scrape`, `@return` | The fetch pipeline: URL/depth/selectors, extract shaping, DADO fetch rules |
| `help-files/SHQL-help-fredcrumb.md` | — (project convention) | fredcrumb.md: the project's map and operating contract — placement, contents, authority |

Pattern: lean ruleset up front, binders on disk behind it. To hand a command to a new model: give it this spec (the contract) + the binder (the howto). Spec in the ruleset, howto in the binder, help one `::@help/` away.

---
## 3. Negation & Strength

- Positive tags: focus domains; higher strength → more authority/detail/priority.  
  1 = baseline · 2 = strong · 3+ = dominant.
- Negated tags: de-prioritize/exclude; higher strength → stronger avoidance.
- Example: `::expert^3::rust::!async^2::perf/` = strong expert + rust + perf; strongly avoid async.
- Negation and strength persist through `@save` / `@preset` / `@add` / `@remove` (remove is by name only).

---

## 4. Parser (prefix-first)

SHQL path:

1. Trim input.  
2. If not starting with `::` → **Plain**.  
3. If no `/` → **MissingTerminator**.  
4. Prefix = through first `/`; query = after `/` (lstrip).  
5. Inner = prefix without leading `::` and trailing `/`.  
6. Empty inner → **Reset** (`::/`) — clear tags + lock reset.  
7. Split inner on `::`.  
8. If first segment starts with `@`:  
   - If `^` appears in the command token → **MalformedCommand**.  
   - `cmd` = after `@`, lowercased.  
   - If `cmd` ∈ {list, add, remove, save, preset, load, unload, clear, help, lock, unlock, temperature, context, effort, restore, return, safe} **or** `cmd` is `capture` / starts with `capture-` **or** `cmd` is `scrape` / starts with `scrape-` **or** `cmd` is `compact` / starts with `compact-` → **Command** (validate required args; see the command's binder for full arg grammar).  
   - Else → **MalformedCommand**.  
9. Else every segment is a tag: parse `[!]name[^N]`.  
10. Empty name → **EmptyTag**. Strength not integer ≥ 1 → **InvalidStrength**.  
11. Lowercase names; **normalize** Lwins; return **Query(tags, query)**.

`scrape`: query’s first token must be `http://` or `https://` (else **MissingCommandArgument** / **InvalidScrapeTarget**). Required args: `save`/`preset` need name; `load` needs definition name; `add`/`remove` need ≥1 tag. `effort`: bare `::@effort/` = default; empty `::@effort::/` → **MissingCommandArgument**; other present arg must be integer 1–5, `default`, or `clear` (else **InvalidEffort**). Missing required args → **MissingCommandArgument**. `compact`: bare = whole live content window; `-lN` same count as capture; no `+`. `restore`: bare = last compact file. `return`: bare = `digest`; form must be `full`, `outline`, or `digest` (else **InvalidReturn**); no pending scrape extract → **MissingCommandArgument**. Full arg grammars live in each command's binder (§2.2).

---

## 5. Errors (hard stop)

On any of these: emit the message and **stop** (no query processing).

1. **MissingTerminator** — starts with `::` but no `/`  
   `Missing terminating '/'. ShorthandQL prefixes must end with `/`. Example: \`::expert::python/ your question\``

2. **EmptyTag** — empty segment (`::::`, `::!::`, `::^3::`)  
   `Empty tag detected. Tags cannot be empty. Check for double \`::\` or missing names.`

3. **InvalidStrength** — not integer ≥ 1  
   `Invalid strength. Strength must be a positive integer (e.g. \`tag^2\` or \`tag^3\`). Zero or negative values are not allowed.`

4. **MissingCommandArgument** — required arg missing  
   `The \`@save\` command requires a preset name. Example: \`::@save::my-preset/\`` (adapt command name)

5. **MalformedCommand** — bad/unknown `@…` or strength on a command name  
   `\`@save^2\` is not a valid command. Valid commands are: list, add, remove, save, preset, load, unload, clear, help, lock, unlock, capture, temperature, context, effort, scrape, compact, restore, return, safe. Strength modifiers apply only to domain tags. (DADO is a plain-word acronym, not an @-command.)`

6. **InvalidEffort** — `@effort` value is not 1–5, `default`, or `clear`  
   `Invalid effort. @effort takes an integer 1–5, or default/clear. Example: ::@effort::3/`

7. **InvalidScrapeTarget** — first token after `/` on `@scrape` is not an http(s) URL  
   `Invalid scrape target. After \`/\` the first token must be an http:// or https:// URL.`

8. **InvalidReturn** — `@return` form is not `full`, `outline`, or `digest`  
   `Invalid return form. Use full, outline, or digest. Example: ::@return::digest/`

9. **MalformedInput** — other invalid prefix  
   `This input does not appear to be valid ShorthandQL. Please check the prefix format (must start with \`::\` and end with \`/\`).`

---

## 6. Response Behavior

| Situation | Behavior |
|-----------|----------|
| Active tags | Lead with `Focusing on ::chain::/.` then domain-bound answer |
| No tags (after clear/reset) | Normal answer; no Focusing line |
| Plain text | Use current tags if any (with Focusing if tags remain) |
| Pure command | Minimal ack + state when useful |
| Command + query | Command first; answer under new state |
| `@effort` + query | Run `@effort` first; no required ack; answer the query |
| `::/` or `@clear` | Clear tags; lock = locked; effort unchanged |
| Locked | Explicit-only actions (see Strict System Instructions) |
| User said DADO | Digest and discuss only for that request (see §2.1). Focusing line still if tags active. `@scrape` does not fetch; ack target, depth, selectors only |
| `@scrape` / `@return` | See `help-files/SHQL-help-@scrape.md` |
| `@compact` / `@restore` / `@capture` | See `help-files/SHQL-help-@context.md` |
| `@safe` | See `help-files/SHQL-help-@safe.md` |
| DADO + `@compact` / `@restore` / `@return` | Ack the plan only. Do not write, restore, or emit the extract |

Stay inside **positive** tags unless the user explicitly asks to leave scope. Blend multi-tags. Scale by strength.

---

## 7. Session toggle bootstrap (required while OFF)

Per project **`AGENTS.md`**, even when this definition is **OFF**:

1. Always recognize **turn ON / turn OFF** phrases and `::@load::SHQL-v1.3b.md/` / `::@unload/` (and aliases).  
2. On ON: read and enforce this file. On OFF: stop enforcing it.  
3. Optional: route `::@capture…/` to a project-local capture path if the host supports it; LLM does not invent out-of-project paths.  
4. While OFF, `::@help/` may list: load/unload toggles + pointer to `SHQL-v1.3b.md`. While ON, include `effort (1–5 | default | clear)`, `scrape [-dN] [selectors] / URL`, and `safe [on|off|dado|status]`.  
5. While SHQL ON, keep tag/lock state until unload.

After unload, other `::@` commands and `::tag::` syntax are plain text until load again.

Optional session persistence (not required): in-memory or project-local tag/preset notes only — never outside this project.

---

## 8. Minimal Examples

**Focus + strength + negation**
```
User: ::expert^3::rust::perf::!async^2/ How do I profile a sync service?
You: Focusing on ::expert^3::rust::perf::!async^2::/.
[Sync-oriented expert answer; avoid async.]
```

**State + preset + lock**
```
User: ::expert^3::mechanic::v8::electrical::diag/
You: Focusing on ::expert^3::mechanic::v8::electrical::diag::/.

User: ::@save::hemi-diag/
You: Saved preset 'hemi-diag'.

User: ::@clear/
You: Tags cleared.

User: ::@preset::hemi-diag/
You: Preset 'hemi-diag' applied. Current: ::expert^3::mechanic::v8::electrical::diag::/.

User: ::@list/
You: Locked. Active tags: expert^3, mechanic, v8, electrical, diag
```

**Clear / unfocused**
```
User: ::/ What time is it in Tokyo?
You: [Normal answer — no Focusing line]
```

**DADO (acronym command)**
```
User: …topic…
DADO
You: [Digest + discuss + options only; no implementation]
```

**Binder-backed commands** (full forms in each binder):
```
::@capture::note.md/          → last 1 content turn  (help-files/SHQL-help-@context.md)
::@scrape/ https://example.com/ → Scrape https://example.com/.  (help-files/SHQL-help-@scrape.md)
::@safe/on/                   → Safe on.  (help-files/SHQL-help-@safe.md)
::@effort::3/                 → Effort 3.
```

---

**End of ShorthandQL v1.3 beta (`SHQL-v1.3b.md`)**
