# `::@capture` / `::@compact` / `::@restore` — the full binder

> Companion to **ShorthandQL v1.3 beta** (`SHQL-v1.3b.md`, §2.2 binder map).
> The spec stays lean; **this file is the howto**. A model that reads this
> top-to-bottom can implement the context pipeline from scratch.
> The pipeline: `@capture` writes turns to a file, `@compact` captures then
> shrinks the live window to a digest, `@restore` puts a compact file back.

---

## `@capture` (host-only)

**Host-only.** Capture the last N content turns to a file. The LLM does not write files — the host strips the command and routes it to a project-local capture path.
Strength (`^N`) is not allowed on the command name.
Does not change tags, lock, or DADO.

### Syntax

```
::@capture::file/
::@capture-lN::file/
::@capture-lN+::file/
```

`N` in `-lN` = integer **≥ 1**. Bare `@capture` = `-l1` (last 1 content turn).
`+` = full raw scaffolding (not the cleaned writer). Optional `file` is a name only; the host picks the directory (e.g. `logs/captures/`).

### Effect

1. Host counts **content turns only** — the resolver skips prior command turns when counting N.
2. Host writes those turns to the named file (or a host-chosen name).
3. If the write fails, the command fails; do not invent file contents.

### Parser

`capture` and any token that starts with `capture-` are in the §4 step 8 command set.

1. Command token `capture`, `capture-lN` (`N` integer ≥ 1), or `capture-lN+`. Other `capture-…` → **MalformedCommand**.
2. Optional one file segment after `::`. Empty file segment → **MissingCommandArgument**.
3. Query after first `/` is ignored. Prepend `**Note:**` if present.

### Errors (hard stop)

**MalformedCommand**  
`` `@capture^2` is not a valid command. Valid forms: `::@capture::file/`, `::@capture-l3::file/`. Strength modifiers apply only to tags. ``

**MissingCommandArgument**  
`` The `@capture` command was given an empty file name. Omit it, or pass a name. ``

### Ack

`Capture saved <file>.`

### State

- Locked: run `@capture` only when the current message asks. Stay locked.
- DADO: do not write. Ack which turns and which file would be captured.
- `::/` / `@clear`: does not delete capture files.
- `@unload`: `@capture` off.

### Help

`capture [-lN][+] [file]`

---

## `@compact` (host-only)

**Host-only.** Shrink the live window. First run `@capture` on content turns, write that file, then replace the live window with a short digest of the written file.
Does not change tags, lock, or DADO. Does not add inferred tags to `@list`.
Strength (`^N`) is not allowed on the command name. `+` is not a compact form.

### Syntax

```
::@compact/
::@compact::file/
::@compact-lN/
::@compact-lN::file/
```

`N` in `-lN` = integer **≥ 1**. Same count as `@capture`: content turns only; skip prior command turns.
Bare `@compact` = the whole live content window (not last-1).
Optional `file` is a name only. The host picks the directory (e.g. `logs/compacts/`).

### Effect

1. Host captures the chosen content turns with the `@capture` writer (no `+` / raw scaffolding).
2. If that write fails: `Compact failed (host ignored).` Do not shrink the live window. Do not invent a digest.
3. If the write succeeds: replace the live window with a short digest of that file. Active session tags stay. The digest is facts, not new tags.

### Parser

`compact` and any token that starts with `compact-` are in the §4 step 8 command set (`capture` rule).

1. Command token `compact` or `compact-lN` (`N` integer ≥ 1). Other `compact-…`, or `+` → **MalformedCommand**.
2. Optional one file segment after `::`. Empty file segment (`::@compact::/`) → **MissingCommandArgument**.
3. Query after first `/` is ignored. Prepend `**Note:**` if present.

### Errors (hard stop)

**MalformedCommand**  
`` `@compact^2` is not a valid command. Valid forms: `::@compact/` or `::@compact-l3/`. Strength modifiers apply only to tags. ``

**MissingCommandArgument**  
`` The `@compact` command was given an empty file name. Omit it, or pass a name. Example: `::@compact::pack.md/` ``

### Ack

`Compact saved <file>.`

Then the digest.

### State

- Locked: run `@compact` only when the current message asks. Stay locked.
- DADO: do not write and do not shrink. Ack the plan only: which turns, which file.
- `::/` / `@clear`: session tags + lock reset. Does not undo a compact already written.
- `@unload`: `@compact` off.

### Help

`compact [-lN] [file]`

---

## `@restore` (host-only)

**Host-only.** Put a compact file back into the live window.
Does not change tags, lock, or DADO. Does not promote inferred tags from the file into `@list`.
Strength (`^N`) is not allowed on the command name.

### Syntax

```
::@restore/
::@restore::file/
```

Bare = the last compact file this session wrote.
`file` names that compact. Host looks in the compact directory.

### Effect

1. Host reads the named compact file (or the last one).
2. If missing: `Restore failed (host ignored).` Do not invent the text.
3. If found: replace the live window with that file's captured turns. Session tags stay as they are now.

### Parser

`restore` is in the §4 step 8 command set.

1. Command token exactly `restore`. Anything else (`restore-l1`, `+`) → **MalformedCommand**.
2. Optional one file segment. Empty file segment → **MissingCommandArgument**.
3. Query after first `/` is ignored. Prepend `**Note:**` if present.

### Errors (hard stop)

**MalformedCommand**  
`` `@restore^2` is not a valid command. Valid forms: `::@restore/` or `::@restore::pack.md/`. Strength modifiers apply only to tags. ``

**MissingCommandArgument**  
`` The `@restore` command was given an empty file name. Omit it, or pass a name. Example: `::@restore::pack.md/` ``

### Ack

`Restored <file>.`

Then the restored turns are the live window (host-side). Do not reprint the whole file unless asked.

### State

- Locked: run `@restore` only when the current message asks. Stay locked.
- DADO: do not restore. Ack which file would be restored.
- `::/` / `@clear`: does not delete compact files.
- `@unload`: `@restore` off.

### Help

`restore [file]`

---

## Pipeline notes

- `@compact` always writes through `@capture` first (cleaned writer, never `+`). A failed write never shrinks the window.
- `@restore` never invents: missing file = `Restore failed (host ignored).`
- None of the three touch tags, lock, or DADO. Under DADO, all three ack the plan only.
- Capture files and compact files are host-managed under project-local directories (e.g. `logs/captures/`, `logs/compacts/`). The LLM never invents out-of-project paths.

---

*Binder v1.0 — 2026-09-28. Paired with SHQL v1.3 beta.*
*Capture the past, compact the present, restore on demand.*
