# CERTIFY_ZIP_BUCKETING.md

**Purpose:** This file preserves the exact historical procedure (as of late 2026) for how `.zip` files were collected and specially bucketed during Certify runs in `generate_manifest`. 

It documents the "how they are handled/filled" logic in the shared Certify bucketing machinery so that this behavior can be referenced, understood, or restored in the future without bloating the active source code or the main `VAULT_ARCHITECTURE.md` with long historical detail.

This logic existed before (and during the transition away from) the decision to treat embedded archives strictly as containers that recommend a separate dedicated Archive audit.

---

## 1. Special FileJob Creation for ZIPs (Collection Phase)

In the jobs collection (after walking the folder and filtering allowed extensions):

```rust
let jobs: Vec<FileJob> = target_files
    .into_par_iter()
    .filter_map(|(path, ext)| {
        if ext == ".zip" {
            return Some(FileJob {
                path,
                size: 0,
                hash: "ZIP".into(),
                content: "".into(),
            });
        }
        let content = fs::read_to_string(&path).ok().unwrap_or_default();
        let hash = get_content_hash(&content);

        Some(FileJob {
            path,
            size: content.len(),
            hash,
            content,
        })
    })
    .collect();
```

**Key points:**
- ZIPs were deliberately turned into a sentinel `FileJob` with `hash = "ZIP"`, `size = 0`, and empty `content`.
- This allowed the later bucketing logic to recognize them as archive containers rather than regular files whose contents should be deep-audited in the current Certify run.
- Regular files received real content, size, and hash.

---

## 2. Dedicated Bucket Rule (Bucketing Phase)

After the jobs list was built, the bucketing logic (using `BUCKET_LIMIT = 10000`) contained this explicit early rule:

```rust
for job in jobs {
    if job.hash == "ZIP" {
        // ZIP files always get their own dedicated bucket
        if !current_bucket.is_empty() {
            buckets.push(current_bucket);
            current_bucket = Vec::new();
            current_size = 0;
        }
        buckets.push(vec![job]);
        continue;
    }

    // Large file that exceeds the entire bucket limit
    if job.size > BUCKET_LIMIT {
        // Flush current bucket first if it has anything
        if !current_bucket.is_empty() {
            buckets.push(current_bucket);
            current_bucket = Vec::new();
            current_size = 0;
        }
        // The large file will be split across multiple buckets
        let size = job.size;
        current_bucket.push(job);
        current_size += size;
        continue;
    }

    // Normal case: only add the file if it completely fits in the remaining space.
    // This prevents "just fits and wastes the bucket" which leads to bad splits.
    if current_size + job.size > BUCKET_LIMIT && !current_bucket.is_empty() {
        buckets.push(current_bucket);
        current_bucket = Vec::new();
        current_size = 0;
    }

    let size = job.size;
    current_bucket.push(job);
    current_size += size;
}

if !current_bucket.is_empty() {
    buckets.push(current_bucket);
}
```

**Key points:**
- Every ZIP was forced into its own singleton bucket (`vec![job]`).
- This flushed any pending regular files and prevented ZIPs from being mixed into multi-file buckets.
- The comment "ZIP files always get their own dedicated bucket" was the canonical marker for this behavior.
- After this, buckets (including singleton ZIP buckets) flowed into the normal LLM batch processing and later report-writing phase.

---

## 3. Later Container Detection and Special Report Output

After the LLM calls for buckets, during the per-file report writing loop:

```rust
let is_archive_container = job.hash == "ZIP" || job.path.extension().map_or(false, |e| e.to_string_lossy().to_lowercase() == "zip");

let report_content = if is_archive_container {
    let action_path = job.path.display();
    // Special clean format for archives hit during Certify.
    // We still use the (useful) container-level reasoning the model produced,
    // but wrap it with a clear actionable header instead of the noisy parse-failure wrapper.
    // This keeps the feed clean and gives the user an immediate path to deep analysis.
    format!(
        r#"### ARCHIVE CONTAINER DETECTED DURING CERTIFY
This ZIP was encountered while certifying the folder.
Deep per-file analysis of its *contents* was not performed during this Certify run
(to keep the broad certification focused and avoid partial structured parses).

To perform full per-file forensic analysis of the files inside this archive
(with its own dated job folder under the Archive/ sub), run a dedicated Archive audit on this file:

ACTION:ARCHIVE_AUDIT:{}

(The dedicated Archive mode will emit clean per-file .raa reports inside a proper job folder.)

--- RAA FILE ANALYSIS ---
File: {}
Hash: {}
Verdict: ARCHIVE CONTAINER (deep analysis recommended via Archive mode)
Analysis:
{}

------------------------
"#,
        action_path, job.path.display(), job.hash, analysis_text
    )
} else {
    // normal per-file report construction
    format!( ... )
};

match fs::write(&target_path, &report_content) {
    ...
}
```

The event emitted for these items also used a cleaned `event_verdict = "ARCHIVE CONTAINER"`.

---

## 4. Why This Logic Existed (Context for Future Reference)

- This was the mechanism that gave ZIPs special treatment inside the shared Certify bucketing machinery.
- It allowed a container-level LLM call on the ZIP itself (even with empty content) while ensuring the ZIP did not participate in normal multi-file batching of loose files.
- The special "ARCHIVE CONTAINER" report + `ACTION:ARCHIVE_AUDIT:` marker was later added (to address structured parse failures and to keep Certify focused) so that deep per-file analysis of archive *contents* would be performed only in a dedicated Archive run.
- The dedicated-bucket rule for ZIPs was a holdover from when we still intended (or partially intended) to analyze archive contents inline during a broad Certify.

This entire pattern (special FileJob + dedicated bucket + later container override) is what is being considered for removal or simplification once the policy of "offload all deep archive analysis to separate Archive audits" is fully settled.

---

## Preservation Notes

- This file exists so the exact collection + dedicated-bucket behavior can be pulled and re-applied later if needed (e.g., if we ever decide to re-enable limited inline deep analysis of certain archives during Certify).
- The active code in `src-tauri/src/lib.rs` should contain only minimal pointers (e.g. a one-line comment referencing this file) rather than long historical explanations.
- See also `VAULT_ARCHITECTURE.md` for the broader Granular Vault Architecture context and `ROADMAP.md` for status of related work.

**Last preserved state:** The logic described above (special ZIP FileJob + forced dedicated bucket + container report override) as implemented in the Certify path before any removal of the dedicated-bucket rule for compressed files.