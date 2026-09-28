# `::@safe` — the full binder

> Companion to **ShorthandQL v1.3 beta** (`SHQL-v1.3b.md`, §2.2 binder map).
> The spec stays lean; **this file is the howto**. A model that reads this
> top-to-bottom can implement `::@safe` from scratch. This is the pattern
> going forward: lean ruleset + one external binder per command.

---

## 0. Full spec (read first — this is the contract)

**Name:** `@safe` — the key to the model's vault. The vault is the storage; `@safe` is the command that opens it.
**Default:** OFF. Opt-in, like SHQL itself. No strength (`^N`) on the command name.

| Operator | Form | Effect | Ack |
|---|---|---|---|
| `on` | `::@safe/on/` | Arm for the session. When a task touches vault-covered ground: read the vault index first, then pull only matching slices. Cite what was pulled. | `Safe on. I'll check the vault when it helps.` |
| `off` | `::@safe/off/` | Disarm. The vault is not touched unasked. | `Safe off.` |
| `dado` | `::@safe/dado/` | One-shot digest & discuss of vault contents relevant to the current topic. **Read-only**: report, take no action, propose nothing unless asked. Does not arm the ON state. | The digest itself. |
| `status` | `::@safe/status/` | Read the index only (~1KB). Report what the vault contains. No deep pulls. | One-line-per-section listing. |

Bare `::@safe/` = `::@safe/status/`.

**Syntax:**
```
::@safe/on/
::@safe/off/
::@safe/dado/
::@safe/status/
```

**Parser:**
1. Command token exactly `safe`. Anything else (`safe-on`, `safe^2`) → **MalformedCommand**.
2. Optional one operator segment: `on`, `off`, `dado`, `status`. Unknown operator → **MalformedCommand**. Bare `::@safe/` = `status`.
3. Query after first `/` is ignored. Prepend `**Note:**` if present.

**Errors (hard stop):**
- **MalformedCommand** — `` `@safe^2` is not a valid command. Valid forms: `::@safe/on/`, `::@safe/off/`, `::@safe/dado/`, `::@safe/status/`. Strength modifiers apply only to tags. ``

**State:**
- Locked: run vault pulls only when the current message's task touches vault ground. Stay locked.
- DADO: `::@safe/dado/` *is* the DADO flavor — digest & discuss only, no action.
- `::/` / `@clear`: disarms `@safe` (back to OFF).
- `@unload`: `@safe` off.
- Vault archives are write-once: new state → new file, never overwrite.

---

## 1. What the vault is

The vault is the model's **external cold storage**: a folder on disk holding
everything too bulky, raw, or rarely-needed for the live context window.

`::@safe` is the command that opens it — the key, not the room.

Standard layout:

```
<vault-root>/
  INDEX.md            ← the live trail. Tiny (~1KB). The ONLY warm file.
  <topic>/...         ← frozen archives, logs, reference. All cold.
  scratch/            ← disposable work. Clean periodically.
  logs/               ← append-only run logs, including safe-use.log
  cold-storage/       ← bulky reference. Read on demand, never preload.
```

The routing trick (why this works instead of bloating context):

- The live session holds **pointers**, not payloads.
- `INDEX.md` is the trail: topic → path, one line each.
- Payload loads **on fire only**, and only the matching slice.
- Each layer is an order of magnitude smaller than what it points to.

Token math: ~4 bytes ≈ 1 token. Every KB kept out of the live window
saves ~250 tokens per turn it would otherwise have occupied.

---

## 2. Self-prompt: how to work `@safe`

### When `::@safe/on/` is armed

On **every** subsequent user message, before acting:

1. **Does this task touch vault-covered ground?** Compare the task's domain
   against the INDEX trail. No match → proceed normally, touch nothing.
2. **Trail first.** Read `INDEX.md` (cheap). Never skip straight to payload.
3. **Extract, don't inhale.** Search the vault for the topic
   (e.g. `grep -i "<topic>" -r <vault-root>/ | head -20`). Pull matching
   **lines**, not whole files. Cost = exactly the context you used.
4. **Cite the pull.** State what was read (`<topic>/…`, N files / N lines)
   so the human can audit it.
5. **Respect write-once.** Never edit a frozen archive. New state → new file.

### When `::@safe/dado/` fires

1. Read the INDEX, find what's relevant to the current topic.
2. Pull the minimal slices.
3. **Digest and discuss only.** Report what the vault holds and what it means.
   Take no action on it. Propose nothing unless asked. Then done —
   the ON state is untouched.

### When `::@safe/status/` fires (or bare `::@safe/`)

Read INDEX.md only. Reply with a one-line-per-section listing of what the
vault contains. No deep pulls.

---

## 3. Retrieval discipline (the law)

- **Never preload** the vault. Not at session start, not "just in case."
- **Never read a whole cold file** when a slice answers the question.
- **INDEX before payload**, always.
- **Cite every pull.** The human audits the vault; silent reads are a breach.
- If the vault doesn't cover the task's domain, say so once and move on —
  don't go digging.

---

## 4. Worked examples

**Armed session:**
> Human: `::@safe/on/`
> Model: `Safe on. I'll check the vault when it helps.`
> Human: `how did last week's moltbook check go?`
> Model: *(task touches `moltbook/` → reads INDEX → greps logs for the check-up)*
> `Pulled 12 lines from logs/moltbook-checkup-2026-09-22.log. Last check: 4/5 posts still pending, bug-report post born pending…`

**DADO glance:**
> Human: `::@safe/dado/` *what do we have on the bouncer tunnel?*
> Model: *(reads INDEX, pulls matching lines, reports — takes no action)*
> `From the vault: tunnel URL recorded 2026-09-21, cloudflared restart = new URL… (digest only, no action taken)`

**Status:**
> Human: `::@safe/status/`
> Model: `Vault contains: moltbook/ (5 frozen posts + check logs), scratch/ (empty), logs/ (3 files), cold-storage/ (2 binders).`

---

## 5. Logging its use

Every `::@safe` invocation appends one line to `<vault-root>/logs/safe-use.log`:

```
<utc-timestamp> | <operator> | <what was pulled> | <why>
```

Example:
```
2026-09-22T23:55:00Z | on/fire | logs/moltbook-checkup-2026-09-22.log:12 lines | weekly check-up question
```

The log is how the human audits the vault. It is also how the *model*
audits itself — "logs for logs," all the way down.

---

## 6. Giving this to every model

All of the human's models share the same pattern: a lean ruleset up front,
binders on disk behind it. To hand `::@safe` to a new model:

1. Give it `SHQL-v1.3b.md` (the spec — §2 is the whole contract).
2. Give it this file (the binder — the howto).
3. Tell it where its vault lives (its own `<vault-root>`; every model gets
   its own vault — vaults are not shared).
4. The model seeds its own `INDEX.md` and grows from there.

Spec in the ruleset, howto in the binder, help one `::@help/` away.
That's the pattern. Repeat per command.

---

*Binder v1.0 — 2026-09-22. Paired with SHQL v1.3 beta.*
*The vault stays lean so the mind can stay sharp.*
