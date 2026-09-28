# SHQL help — fredcrumb

fredcrumb.md is a project's map and operating contract. Any agent entering a
project reads it first, before listing directories or guessing at structure.

## Placement

- Exactly one per project, at the project root. Never nested in subdirectories.
- A nested fredcrumb is slop: fold its content up into the root map, then
  remove the nested file.

## Contents

- A visual tree (or list) of the directory: one line per entry, with a `←`
  annotation saying what it is and why it matters.
- Near the top, under the title: the SHQL block — what SHQL is, when to read
  the ACTIVE binder, where the help-files live.
- The ACTIVE `shql-binder-vX.Yb/` named in the tree. Binder versioning: new
  versions land as new directories beside the old; never edit a versioned
  binder in place. A version seals once its upgrade is announced — before
  that, pre-seal amendments are allowed (then re-pushed to all projects).
- A Rules section for project constraints: things an agent must never do here
  (never delete, read-only paths, historical / do-not-restructure).
- No timestamp lines in the content. Filesystem mtime is the stamp.

## Authority

- The fredcrumb is the authority for project structure — not memory, not
  guesswork.
- When the map and the directory disagree, the directory wins; fix the map.
