# `::@scrape` / `::@return` — the full binder

> Companion to **ShorthandQL v1.3 beta** (`SHQL-v1.3b.md`, §2.2 binder map).
> The spec stays lean; **this file is the howto**. A model that reads this
> top-to-bottom can implement the fetch pipeline from scratch.
> The pipeline: `@scrape` fetches a URL and extracts text with selector tags,
> `@return` shapes a pending scrape extract without fetching.

---

## `@scrape` (host-only)

**Host-only.** Fetch the URL after `/` and extract text using optional selector tags on the command.
Not a session tag prefix. Does not replace or add session tags.
Strength (`^N`) is not allowed on the command name.
Does not change lock or DADO.

### Syntax

```
::@scrape/ URL
::@scrape::sel1::sel2^N::!sel3/ URL
::@scrape-dN/ URL
::@scrape-dN::sel1::sel2^N::!sel3/ URL
```

`N` in `-dN` = integer **≥ 1**. Depth = extra path folders under the URL's directory, same host only.
Bare `@scrape` = depth **0** (that page only).

Selector tags are optional. Same tag grammar as spec §1: `name`, `name^N`, `!name`, `!name^N`. Case-insensitive; store lowercase. Lwins by name.

Command + extra text after the URL: use the first URL token; ignore the rest; prepend `**Note:**`.

### Effect

1. Host fetches the URL (and same-host folder pages if `-dN`).
2. Apply selector tags only to that fetch:
   - positive `name^N` → keep / detail; higher N = more weight
   - omitted name → not required (no selector list means the page's readable text, not a required vocabulary)
   - `!name^N` → avoid or drop; higher N = stronger drop
3. Answer from the extracted text. If session tags are already active, also follow those for how to write the answer (Focusing line if session tags exist).
4. Scrape selector tags end when the reply ends. They do not stay in `@list`.

If the host cannot fetch: `Scrape failed (host ignored).` Do not invent page content.

### Depth (`-dN`)

Same pattern as `capture-lN`.

| Form | Pages |
|------|--------|
| `@scrape` | the URL only |
| `@scrape-d1` | the URL + files/folders **one** slash deeper on the same host |
| `@scrape-d2` | two slashes deeper |
| `@scrape-dN` | N slashes deeper |

Do not follow other hosts. Do not use `-dN` as tag strength. Cap is host-defined; if `N` is over the cap, use the cap and ack `Depth N (capped M).`

`^` on depth is illegal: `@scrape-d3^2` → **MalformedCommand**.

### Parser

`scrape` and any token that starts with `scrape-` are in the spec §4 step 8 command set (`capture` rule).

1. Command token `scrape` or `scrape-dN` (`N` integer ≥ 1). Other `scrape-…` → **MalformedCommand**.
2. Remaining segments: parse each as a tag `[!]name[^N]`. Empty name → **EmptyTag**. Bad strength → **InvalidStrength**.
3. Query after first `/`: trim. First token must be a URL (`http://` or `https://`). Missing → **MissingCommandArgument**. Not a URL → **InvalidScrapeTarget**.
4. Extra `::` after valid selectors is already consumed as selectors. No `**Note:**` for those.

### Errors (hard stop)

**MissingCommandArgument**  
`` The `@scrape` command requires a URL after `/`. Example: `::@scrape::h1^5/ https://example.com/` ``

**InvalidScrapeTarget**  
`` Invalid scrape target. After `/` the first token must be an http:// or https:// URL. ``

**MalformedCommand**  
`` `@scrape^3` is not a valid command. Valid forms: `::@scrape/` or `::@scrape-d2/`. Strength modifiers apply only to tags. ``

**InvalidStrength** / **EmptyTag** / **MissingTerminator** — same messages as spec §5.

### Ack

Pure command (URL only, no question text):  
`Scrape URL.` or `Scrape URL depth N.`

Then the extract.

Command + question after the URL: no extra ack; answer from the extract.

### State

- Locked: run `@scrape` (explicit). Stay locked. Fetch only this command's URL/depth. No other tools.
- DADO: do not fetch. Ack the plan only: target, depth, selectors.
- `::/` / `@clear`: session tags + lock reset; does not cancel an in-flight host fetch already started.
- `@unload`: `@scrape` off.

### Help

`scrape [-dN] [selectors] / URL`

---

## `@return` (host-only)

**Host-only.** Shape a scrape extract already in hand. Does not fetch. Does not change tags, lock, or DADO.
Strength (`^N`) is not allowed on the command name.

### Syntax

```
::@return/
::@return::full/
::@return::outline/
::@return::digest/
```

Bare = `digest`.
Only those three forms. Selector tags are not return args (those belong on `@scrape`).

### Effect

1. Host must already hold a successful `@scrape` extract from this session.
2. Shape it:
   - `full` — cleaned page text, as scraped
   - `outline` — headings and short lines only
   - `digest` — short packed facts
3. Emit that shaped text. It does not become session tags.
4. A later `@scrape` replaces the pending extract. `@return` does not.

If no extract is waiting: do not invent page content. Hard stop.

### Parser

`return` is in the spec §4 step 8 command set.

1. Command token exactly `return`.
2. Optional one form segment: `full`, `outline`, or `digest`. Other token → **InvalidReturn**.
3. Empty form segment (`::@return::/`) → **MissingCommandArgument**.
4. Query after first `/`: if present, answer from the shaped text (no extra ack). No URL required.

### Errors (hard stop)

**MissingCommandArgument**  
`` The `@return` command needs a scraped extract, or was given an empty form. Example: `::@return::digest/` ``

**InvalidReturn**  
`` Invalid return form. Use full, outline, or digest. Example: `::@return::digest/` ``

**MalformedCommand**  
`` `@return^2` is not a valid command. Valid forms: `::@return/` or `::@return::digest/`. Strength modifiers apply only to tags. ``

### Ack

Pure command: `Return full.` · `Return outline.` · `Return digest.`

Then the shaped text.

Command + question after `/`: no extra ack; answer from the shaped text.

### State

- Locked: run `@return` only when the current message asks. Stay locked. Do not fetch.
- DADO: do not emit the extract. Ack the form only.
- `::/` / `@clear`: does not drop a pending scrape extract.
- `@unload`: `@return` off. Pending extract drops.

### Help

`return [full|outline|digest]`

---

## Pipeline notes

- `@scrape` fetches; `@return` shapes. `@return` never fetches — no pending extract means hard stop, never invented content.
- Scrape selector tags are per-fetch: they die with the reply and never enter `@list` or the session tag set.
- A later `@scrape` replaces the pending extract; `@return` never does.
- Under DADO: `@scrape` acks target/depth/selectors without fetching; `@return` acks the form without emitting.

---

*Binder v1.0 — 2026-09-28. Paired with SHQL v1.3 beta.*
*Fetch the page, shape the take.*
