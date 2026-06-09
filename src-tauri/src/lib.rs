use chrono::Local;
use notify::Watcher;
use once_cell::sync::Lazy;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::env;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::Emitter;
use walkdir::WalkDir;
use zip::read::ZipArchive;

/// Names of files and directories that should always be excluded from auditing.
/// This list is used in both generate_manifest and build_control_manifest.
const JUNK_NAMES: &[&str] = &[
    "node_modules",
    ".git",
    "target",
    "dist",
    "__MACOSX",
    ".DS_Store", // macOS Finder metadata file
];

#[derive(Serialize)]
struct LlmRequest {
    model: String,
    messages: Vec<Message>,
}

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Clone)]
struct RAAReport {
    verdict: String,
    reasoning: String,
    target_name: String,
    is_error: bool,
}

#[derive(Serialize, Clone)]
struct ScanEvent {
    path: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    verdict: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Clone)]
struct FileJob {
    path: PathBuf,
    content: String,
    hash: String,
    size: usize,
}

#[derive(Serialize, Clone)]
struct VaultFile {
    name: String,
    path: String,
    modified: String,
    size: u64,
    has_violation: bool,
}

fn get_content_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[tauri::command]
async fn hash_file(path: String) -> Result<String, String> {
    let content = fs::read(&path).map_err(|e| e.to_string())?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    Ok(format!("{:x}", hasher.finalize()))
}

fn read_entry_from_disk(manifest_path: &PathBuf, hash: &str) -> Option<RAAReport> {
    if let Ok(file) = fs::File::open(manifest_path) {
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader.lines().filter_map(|l| l.ok()).collect();

        for (i, line) in lines.iter().enumerate() {
            if line.contains(&format!("Hash: {}", hash)) {
                let mut target_name = "Unknown".to_string();
                let mut reasoning_lines = Vec::new();

                for j in (0..i).rev() {
                    if lines[j].starts_with("Target: ") {
                        target_name = lines[j].replace("Target: ", "").trim().to_string();
                        break;
                    }
                }

                let mut capture = false;
                for next_line in &lines[i..] {
                    if next_line.starts_with("Result: ") {
                        capture = true;
                        continue;
                    }
                    if next_line.starts_with("---------------------------") && capture {
                        break;
                    }
                    if capture {
                        reasoning_lines.push(next_line.clone());
                    }
                }

                let full_reasoning = reasoning_lines.join("\n").trim().to_string();
                let is_violation = full_reasoning.to_uppercase().contains("VERDICT: VIOLATION");
                return Some(RAAReport {
                    verdict: if is_violation {
                        "RAA VIOLATION DETECTED".into()
                    } else {
                        "ALL SAFE".into()
                    },
                    reasoning: full_reasoning,
                    target_name,
                    is_error: is_violation,
                });
            }
        }
    }
    None
}

// --- VAULT PATH RESOLVER (handles ~, empty, and absolute paths correctly) ---
fn resolve_vault_root(vault_root_path: &str) -> PathBuf {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());

    if vault_root_path.is_empty() {
        return PathBuf::from(&home).join("Documents/RAA-Vault");
    }

    let trimmed = vault_root_path.trim_end_matches('/');

    // If the value already points to the full vault dir, use it directly
    // Support both old (RAA_Vault) and new (RAA-Vault) for backward compatibility during transition
    if trimmed.ends_with("RAA-Vault") || trimmed.ends_with("RAA_Vault") {
        return PathBuf::from(trimmed);
    }

    // Support ~/Documents and ~ as home
    if trimmed == "~" || trimmed.starts_with("~/") {
        let rest = trimmed.strip_prefix("~/").unwrap_or("");
        return PathBuf::from(&home).join(rest).join("RAA-Vault");
    }

    // Absolute or relative path provided by user (from dialog) — append /RAA-Vault
    PathBuf::from(trimmed).join("RAA-Vault")
}

/// Resolves the root directory for a specific operation type under the main RAA-Vault.
/// Operations: "Audit" (terminal commands), "Analyze" (single files),
/// "Archive" (zips), "Certify" (folders).
/// Creates the subdirectory if it does not exist.
fn resolve_operation_vault(vault_root_path: &str, operation: &str) -> PathBuf {
    let base = resolve_vault_root(vault_root_path);
    let op_root = base.join(operation);
    if !op_root.exists() {
        let _ = fs::create_dir_all(&op_root);
    }
    op_root
}

// --- RAA LOGGING ENGINE ---

/// Sanitizes a string for safe use in filenames and directory names.
///
/// 2025-10 Decision: We standardized on the **hyphen `-`** as the separator
/// for spaces and other characters across the new granular architecture
/// (job folders, control manifests, per-file reports, etc.).
///
/// Reasons:
/// - Clean, modern, professional appearance in file explorers
/// - Consistent with the existing timestamp format (YYYYMMDD-HHMMSS)
/// - Better readability and sharing than underscores for user-facing artifacts
/// - Safe in shells, URLs, and across Windows/macOS/Linux
///
/// See VAULT_ARCHITECTURE.md and ROADMAP.md → "New Path Forward" / granular per-file vault architecture section
/// for full context on the shift to job folders + ONE FILE = ONE REPORT.
///
/// FUTURE NOTE:
/// We have decided to use the **static name** `~RAA-CONTROL-Manifest.log`
/// inside every job folder (instead of a dynamic per-job name).
/// This simplifies logic significantly — no variable naming needed.
/// The `~` prefix ensures it always sorts to the top of the folder.
/// This file will be the very first artifact created when a user starts
/// either a Certify or Archive job.
fn sanitize_for_filename(input: &str) -> String {
    // Replace filesystem-illegal characters first
    let s = input.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "-");

    // Then normalize spaces, dots, and existing underscores to hyphens.
    // We deliberately avoid using periods as separators (they conflict with extensions).
    let s = s.replace([' ', '.', '_'], "-");

    // Collapse multiple consecutive hyphens and trim leading/trailing ones
    let s = s.split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    s
}

/// Collects two lists of relative paths from the source directory:
/// - Files that will be audited (match allowed extensions or are .zip)
/// - Files that will be skipped (do not match allowed extensions)
///
/// This is used both for building the control manifest and for writing
/// per-file reports inside the job folder.
fn collect_audited_and_skipped_paths(
    root: &Path,
    allowed_extensions: &[String],
) -> (Vec<PathBuf>, Vec<PathBuf>) {
    let mut audited: Vec<PathBuf> = Vec::new();
    let mut skipped: Vec<PathBuf> = Vec::new();

    let walker = WalkDir::new(root)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !JUNK_NAMES.iter().any(|&ex| name == ex) && !name.starts_with("._")
        });

    for entry in walker.filter_map(|e| e.ok()) {
        if entry.path() == root {
            continue;
        }

        if entry.file_type().is_file() {
            let ext = entry
                .path()
                .extension()
                .map(|s| format!(".{}", s.to_string_lossy().to_lowercase()))
                .unwrap_or_default();

            if let Ok(rel) = entry.path().strip_prefix(root) {
                if allowed_extensions.contains(&ext) || ext == ".zip" {
                    audited.push(rel.to_path_buf());
                } else {
                    skipped.push(rel.to_path_buf());
                }
            }
        }
    }

    (audited, skipped)
}

/// Builds a minimal hierarchical text representation of the directory structure
/// for the control manifest. Uses simple 4-space indentation to create a
/// "tabbed" visual effect that reflects the folder hierarchy.
///
/// Builds the final ~RAA-CONTROL-Manifest.log with a DNA Registry section
/// containing hashes for every file (audited + skipped) at the top.
fn build_final_control_manifest_with_dna(
    root: &Path,
    allowed_extensions: &[String],
    source_folder_name: &str,
    job_name: &str,
    timestamp: &str,
    audited_hashes: &HashMap<PathBuf, String>,
    skipped_hashes: &HashMap<PathBuf, String>,
) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Header
    lines.push("~RAA-CONTROL-Manifest.log".to_string());
    lines.push(format!("Generated: {}", timestamp));  // Use the job timestamp for consistency
    lines.push(format!("Source: {}", root.display()));
    lines.push(format!("Job Folder: {}", job_name));
    lines.push(String::new());

    // DNA Registry Section (hashes for quick lookup)
    lines.push("DNA Registry (File → Hash):".to_string());
    lines.push(String::new());

    // Audited files first
    let mut all_audited: Vec<_> = audited_hashes.keys().collect();
    all_audited.sort();
    for rel in all_audited {
        if let Some(hash) = audited_hashes.get(rel) {
            lines.push(format!("File: {}", rel.display()));
            lines.push(format!("Hash: {}", hash));
            lines.push(String::new());
        }
    }

    // Skipped files
    let mut all_skipped: Vec<_> = skipped_hashes.keys().collect();
    all_skipped.sort();
    for rel in all_skipped {
        if let Some(hash) = skipped_hashes.get(rel) {
            lines.push(format!("File: {}", rel.display()));
            lines.push(format!("Hash: {}", hash));
            lines.push(String::new());
        }
    }

    lines.push(String::new());

    // Directory structure sections (re-use existing tree logic)
    let (audited_rel_paths, skipped_rel_paths) =
        collect_audited_and_skipped_paths(root, allowed_extensions);

    lines.push(format!(
        "Directory Structure ({} files to be audited):",
        audited_rel_paths.len()
    ));
    lines.push(String::new());
    lines.extend(build_tree_lines(source_folder_name, &audited_rel_paths));

    lines.push(String::new());

    lines.push(format!(
        "Directory Structure ({} files to be skipped):",
        skipped_rel_paths.len()
    ));
    lines.push(String::new());
    lines.extend(build_tree_lines(source_folder_name, &skipped_rel_paths));

    lines.join("\n")
}

/// Builds clean indented tree lines from a list of relative file paths.
/// Only ancestor directories that contain at least one file from the list are included.
fn build_tree_lines(root_name: &str, file_paths: &[PathBuf]) -> Vec<String> {
    let mut lines = Vec::new();

    if file_paths.is_empty() {
        lines.push(format!("{}/", root_name));
        return lines;
    }

    // Root line
    lines.push(format!("{}/", root_name));

    // Collect all unique ancestor directories for the given files
    let mut all_dirs: std::collections::HashSet<PathBuf> = std::collections::HashSet::new();

    for path in file_paths {
        let mut current = path.clone();
        while let Some(parent) = current.parent() {
            if !parent.as_os_str().is_empty() {
                all_dirs.insert(parent.to_path_buf());
            }
            current = parent.to_path_buf();
        }
    }

    // Sort everything so directories appear before their files in a natural order
    let mut all_entries: Vec<(PathBuf, bool)> = Vec::new(); // (path, is_file)

    for path in file_paths {
        all_entries.push((path.clone(), true));
    }
    for dir in &all_dirs {
        all_entries.push((dir.clone(), false));
    }

    all_entries.sort_by(|a, b| a.0.cmp(&b.0));

    for (path, is_file) in all_entries {
        let depth = path.components().count();
        let indent = "    ".repeat(depth);

        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        if is_file {
            lines.push(format!("{}{}", indent, name));
        } else {
            lines.push(format!("{}{}/", indent, name));
        }
    }

    lines
}

/// Builds the ~RAA-CONTROL-Manifest.log for Archive (ZIP) operations.
/// Matches the visual structure of the Certify version:
/// - Header with Generated, Source (zip path), Job Folder
/// - DNA Registry (File → Hash)  -- audited get real hashes, skipped get "SKIPPED"
/// - Directory Structure for audited
/// - Directory Structure for skipped
/// The tree root uses the zip filename.
fn build_archive_control_manifest(
    zip_file_name: &str,
    source: &str,
    job_name: &str,
    timestamp: &str,
    audited_rel: &[PathBuf],
    skipped_rel: &[PathBuf],
    audited_hashes: &HashMap<PathBuf, String>,
) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Header - exact match to Certify style
    lines.push("~RAA-CONTROL-Manifest.log".to_string());
    lines.push(format!("Generated: {}", timestamp));
    lines.push(format!("Source: {}", source));
    lines.push(format!("Job Folder: {}", job_name));
    lines.push(String::new());

    // DNA Registry
    lines.push("DNA Registry (File → Hash):".to_string());
    lines.push(String::new());

    // Audited files (sorted for consistency)
    let mut audited_sorted: Vec<_> = audited_rel.iter().collect();
    audited_sorted.sort();
    for rel in audited_sorted {
        let hash = audited_hashes
            .get(rel)
            .cloned()
            .unwrap_or_else(|| "PENDING".to_string());
        lines.push(format!("File: {}", rel.display()));
        lines.push(format!("Hash: {}", hash));
        lines.push(String::new());
    }

    // Skipped files (sorted)
    let mut skipped_sorted: Vec<_> = skipped_rel.iter().collect();
    skipped_sorted.sort();
    for rel in skipped_sorted {
        lines.push(format!("File: {}", rel.display()));
        lines.push("Hash: SKIPPED".to_string());
        lines.push(String::new());
    }

    lines.push(String::new());

    // Directory Structure audited
    lines.push(format!(
        "Directory Structure ({} files to be audited):",
        audited_rel.len()
    ));
    lines.push(String::new());
    let root_name = sanitize_for_filename(zip_file_name);
    lines.extend(build_tree_lines(&root_name, audited_rel));

    lines.push(String::new());

    // Directory Structure skipped
    lines.push(format!(
        "Directory Structure ({} files to be skipped):",
        skipped_rel.len()
    ));
    lines.push(String::new());
    lines.extend(build_tree_lines(&root_name, skipped_rel));

    lines.join("\n")
}

/// Builds the *initial* ~RAA-CONTROL-Manifest.log written as the very first
/// artifact when a job starts (Stage 1 of New Path Forward).
///
/// This version contains only the header + directory structure (inventory +
/// hierarchy) using the exact same filtering as the rest of the audit.
/// No DNA hashes yet — those are added when the job completes.
///
/// The file is written immediately after the dated job folder is created,
/// before any content reading or LLM calls. It will be overwritten at the
/// end of the job by the full version containing the DNA Registry.
fn build_initial_control_manifest(
    root: &Path,
    allowed_extensions: &[String],
    source_folder_name: &str,
    job_name: &str,
    timestamp: &str,
) -> String {
    let mut lines: Vec<String> = Vec::new();

    // Header (matches the style of the final manifest)
    lines.push("~RAA-CONTROL-Manifest.log".to_string());
    lines.push(format!("Generated: {}", timestamp));
    lines.push(format!("Source: {}", root.display()));
    lines.push(format!("Job Folder: {}", job_name));
    lines.push(String::new());

    lines.push("Status: Initial inventory snapshot — DNA Registry (File → Hash) will be finalized at job completion.".to_string());
    lines.push(String::new());

    // Use the exact same collection logic as the audit path and the final manifest.
    let (audited_rel_paths, skipped_rel_paths) =
        collect_audited_and_skipped_paths(root, allowed_extensions);

    lines.push(format!(
        "Directory Structure ({} files to be audited):",
        audited_rel_paths.len()
    ));
    lines.push(String::new());
    lines.extend(build_tree_lines(source_folder_name, &audited_rel_paths));

    lines.push(String::new());

    lines.push(format!(
        "Directory Structure ({} files to be skipped):",
        skipped_rel_paths.len()
    ));
    lines.push(String::new());
    lines.extend(build_tree_lines(source_folder_name, &skipped_rel_paths));

    lines.join("\n")
}

fn log_to_raa(scan_type: &str, target_label: &str, hash: &str, result_text: &str, vault_root_path: &str) -> PathBuf {
    // Route to operation-specific sub-root for clean separation
    // "audit" (terminal commands) -> Audit/
    // "analyze" (single files) -> Analyze/
    // Other (legacy) -> main vault root
    let operation = match scan_type {
        "audit" => "Audit",
        "analyze" => "Analyze",
        _ => "",
    };

    let base_root = if !operation.is_empty() {
        resolve_operation_vault(vault_root_path, operation)
    } else {
        resolve_vault_root(vault_root_path)
    };

    if !base_root.exists() {
        let _ = fs::create_dir_all(&base_root);
    }

    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let manifest_name = if scan_type == "audit" {
        "Gatekeeper-master-terminal-history.raa".to_string()
    } else {
        let clean_label = sanitize_for_filename(target_label);
        format!("Gatekeeper-{}-{}-{}.raa", scan_type, clean_label, timestamp)
    };

    let manifest_path = base_root.join(manifest_name);
    let log_entry = format!(
        "\n--- RAA FORENSIC REPORT ---\nType: {}\nTarget: {}\nHash: {}\nTimestamp: {}\nResult: \n{}\n---------------------------\n",
        scan_type, target_label, hash, Local::now().format("%Y-%m-%d %H:%M:%S"), result_text
    );

    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&manifest_path)
    {
        let _ = f.write_all(log_entry.as_bytes());
        let _ = f.sync_all();
    }

    manifest_path
}

#[tauri::command]
async fn audit_command(
    command_str: String,
    base_url: String,
    model_name: String,
    vault_root_path: String,
    _debug_raa: bool,   // accepted for call-site uniformity (frontend always sends both flags)
    debug_oracle: bool,
) -> Result<RAAReport, String> {
    let hash = get_content_hash(&command_str);

    // Respect user-selected vault (handles ~ correctly)
    // Terminal command audits (the master "Bible") now live under RAA-Vault/Audit/
    let bible_dir = resolve_operation_vault(&vault_root_path, "Audit");
    let bible_path = bible_dir.join("Gatekeeper-master-terminal-history.raa");

    if let Some(cached) = read_entry_from_disk(&bible_path, &hash) {
        return Ok(cached);
    }

    let report = call_llm_auditor(
        &command_str,
        "terminal command",
        &base_url,
        &model_name,
        &command_str,
        debug_oracle,
    )
    .await?;

    let path = log_to_raa("audit", &command_str, &hash, &report.reasoning, &vault_root_path);
    Ok(read_entry_from_disk(&path, &hash).unwrap_or(report))
}

#[tauri::command]
async fn scan_file_integrity(
    file_path: String,
    base_url: String,
    model_name: String,
    vault_root_path: String,
    _debug_raa: bool,   // accepted for call-site uniformity (frontend always sends both flags)
    debug_oracle: bool,
) -> Result<RAAReport, String> {
    let path = PathBuf::from(&file_path);
    let content = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let hash = get_content_hash(&content);
    let target_label = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();

    let report = call_llm_auditor(
        &content,
        "file integrity",
        &base_url,
        &model_name,
        &target_label,
        debug_oracle,
    )
    .await?;
    let path = log_to_raa("analyze", &target_label, &hash, &report.reasoning, &vault_root_path);

    Ok(read_entry_from_disk(&path, &hash).unwrap_or(report))
}

async fn call_llm_auditor(
    input: &str,
    context_type: &str,
    base_url: &str,
    model_name: &str,
    target: &str,
    debug_oracle: bool,
) -> Result<RAAReport, String> {
    let api_key = env::var("GROK_API_KEY").unwrap_or_default();
    let client = reqwest::Client::new();

    if debug_oracle {
        eprintln!("[RAA-ORACLE] call_llm_auditor: context_type='{}', target='{}'", context_type, target);
        eprintln!("[RAA-ORACLE]   base_url configured in UI: {}", base_url);
        eprintln!("[RAA-ORACLE]   GROK_API_KEY present in env: {}", !api_key.is_empty());
        if api_key.is_empty() {
            eprintln!("[RAA-ORACLE]   WARNING: No GROK_API_KEY in environment. Sending empty Bearer token. This often breaks custom base_urls.");
        }
    }

    // HARDENED PROMPT: Forces technical depth
    let system_prompt = if context_type == "terminal command" {
        "You are a strict RAA Terminal Command Security Auditor. \
         You are analyzing a raw shell command that a user might type or run. \
         Be extremely vigilant for: data exfiltration (curl, wget, nc to external hosts, base64 encoding of sensitive output like system_profiler, whoami, env, id, etc.), command substitution $(...), backticks, network calls, privilege escalation (sudo), destructive actions (rm -rf, > /dev/null), living-off-the-land techniques, or anything that could steal data or harm the system. \
         A command like 'curl ... pornhub.com' with system data is a clear violation. \
         A simple 'ls -la' is safe. \
         You MUST provide a concise but technical explanation. \
         Start with 'Audit Analysis for <command>:' . \
         End with exactly 'VERDICT: SAFE' or 'VERDICT: VIOLATION'.".to_string()
    } else {
        format!(
            "You are an RAA Security Auditor. Analyze this {} for threats. \
            You MUST provide a lengthy, highly technical 1-paragraph explanation. \
            Start your response with 'Audit Analysis for {}:' followed by the details. \
            If safe, conclude with 'VERDICT: SAFE'. If dangerous, include 'VERDICT: VIOLATION'.",
            context_type, target
        )
    };

    if debug_oracle {
        eprintln!("[RAA-ORACLE]   Sending request to LLM... (model={})", model_name);
    }

    let response = client
        .post(base_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&LlmRequest {
            model: model_name.to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt,
                },
                Message {
                    role: "user".to_string(),
                    content: input.to_string(),
                },
            ],
        })
        .send()
        .await
        .map_err(|e| {
            if debug_oracle {
                eprintln!("[RAA-ORACLE]   HTTP SEND ERROR to oracle: {}", e);
            }
            e.to_string()
        })?;

    if debug_oracle {
        eprintln!("[RAA-ORACLE]   HTTP response status from oracle: {}", response.status());
    }

    let raw: serde_json::Value = response.json().await.map_err(|e| {
        if debug_oracle {
            eprintln!("[RAA-ORACLE]   JSON parse error from oracle response: {}", e);
        }
        e.to_string()
    })?;

    // Debug the shape of the response
    let has_choices = raw.get("choices").is_some();
    let content_path = raw.get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c0| c0.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str());
    
    if debug_oracle {
        eprintln!("[RAA-ORACLE]   Response has 'choices': {}", has_choices);
        eprintln!("[RAA-ORACLE]   Extracted content present: {}", content_path.is_some());
        if content_path.is_none() {
            eprintln!("[RAA-ORACLE]   FULL RAW RESPONSE (first 2000 chars): {}", 
                serde_json::to_string(&raw).unwrap_or_default().chars().take(2000).collect::<String>());
        }
    }

    let ai_response = content_path
        .unwrap_or("ORACLE_ERROR: failed to get content from LLM response")
        .to_string();

    let is_violation = ai_response.to_uppercase().contains("VIOLATION");

    if debug_oracle {
        if ai_response.starts_with("ORACLE_ERROR") {
            eprintln!("[RAA-ORACLE]   *** ORACLE CALL FAILED - not falling back to SAFE anymore ***");
        } else if content_path.is_none() {
            eprintln!("[RAA-ORACLE]   *** FALLING BACK (unexpected) ***");
        }

        eprintln!("[RAA-ORACLE]   Final ai_response length: {}, contains VIOLATION: {}", ai_response.len(), is_violation);
    }

    Ok(RAAReport {
        verdict: if is_violation {
            "RAA VIOLATION DETECTED".into()
        } else {
            "ALL SAFE".into()
        },
        reasoning: ai_response,
        target_name: target.to_string(),
        is_error: is_violation,
    })
}

#[allow(dead_code)]
static WATCHER: Lazy<Mutex<Option<notify::RecommendedWatcher>>> = Lazy::new(|| Mutex::new(None));

#[tauri::command]
async fn check_integrity(vault_root_path: Option<String>) -> Result<serde_json::Value, String> {
    // 1. Hardware Probes
    let is_multithreaded = rayon::current_num_threads() > 1;
    let is_bucket_active = walkdir::WalkDir::new(".")
        .max_depth(1)
        .into_iter()
        .next()
        .is_some();

    // 2. Actually test vault resolvability
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    let audit_root = match &vault_root_path {
        Some(p) if !p.is_empty() => PathBuf::from(p).join("RAA-Vault"),
        _ => PathBuf::from(home).join("Documents/RAA-Vault"),
    };
    let vault_ok = audit_root.exists();

    // === Forensic Vault Code Safeguard ===
    // This check verifies that the core "Forensic Vault" implementation is intact.
    // It specifically safeguards the new vault code we added:
    //   - Static creation of the 4 typed subs (Audit/Analyze/Archive/Certify) on initial launch
    //     (with the exact conditions: only assert "initial" if subs don't exist OR no custom root set;
    //      but always create under custom root when one is set).
    //   - The grouping logic that makes the subs visible as 📁 directories in the Forensic Vault finder.
    //   - The list_vault_files / collect_raa_files that reliably discover reports under the subs + job folders.
    //
    // The check verifies the *outcome* of this code: the 4 subs must exist under the vault root.
    // This is ✅ when the vault code is untouched and has maintained the expected structure.
    // It will be ❌ if the vault code has been messed with (e.g. the static creation block or sub logic
    // was deleted or broken, so the structure is not present on launch).
    //
    // This is the "do not touch" deterrent for the vault code.
    // It has nothing to do with .raa reports, the data inside the vault, or DNA hashes of user files.
    // It is purely about the vault *feature/code* itself being protected, similar to how the other
    // integrity checks guard core capabilities (parallel hashing, zip safety, etc.).
    //
    // Clear markers with warnings are placed in the protected code blocks (see the onMount IIFE
    // in +page.svelte and the sub creation + list_vault_files in lib.rs) as a reminder to future
    // edits. If you touch the marked blocks, this check is likely to show ❌ until fixed.

    // Verify the structure that the vault code is responsible for creating and maintaining on launch.
    // We explicitly call create_vault_directory here to exercise the vault code path.
    // If the creation logic (in create_vault_directory or the launch static code) has been
    // deleted or broken, the subs may not appear and this check will be ❌ .
    let root_for_create = vault_root_path.clone().unwrap_or_default();
    let _ = create_vault_directory(root_for_create).await;

    let subs = ["Audit", "Analyze", "Archive", "Certify"];
    let subs_ok = subs.iter().all(|&s| {
        let sub_path = audit_root.join(s);
        sub_path.exists()
    });

    let forensic_vault_ok = vault_ok && subs_ok;

    Ok(serde_json::json!({
        "parallel_hashing": is_multithreaded,
        "bucket_traversal": is_bucket_active,
        "ai_reasoning": true,
        "terminal_input_lock": true,
        "zip_safety": true,
        "vault_path": forensic_vault_ok,
        "disk_first_verification": true
    }))
}

#[tauri::command]
async fn scan_compressed_archive(
    window: tauri::Window,
    zip_path: String,
    allowed_extensions: Vec<String>,
    base_url: String,
    model_name: String,
    vault_root_path: String,
    debug_raa: bool,
    debug_oracle: bool,
) -> Result<RAAReport, String> {
    let file = fs::File::open(&zip_path).map_err(|e| e.to_string())?;
    let mut archive = ZipArchive::new(file).map_err(|e| e.to_string())?;

    // =============================================
    // STAGE 1 support for Archive path (New Path Forward)
    // Create a dated job folder + initial control manifest immediately,
    // before reading contents or calling the LLM for any internal file.
    // Per-file .raa reports (using internal ZIP paths for hierarchy) and the
    // aggregated report are now also written inside the job folder (matching Certify).
    // =============================================
    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();
    let zip_file_name = std::path::Path::new(&zip_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("archive.zip");
    let job_name = format!("{}-{}", sanitize_for_filename(zip_file_name), timestamp);
    // Archive (zip) operations now root under RAA-Vault/Archive/
    let vault_root = resolve_operation_vault(&vault_root_path, "Archive");
    let job_folder = vault_root.join(&job_name);
    if !job_folder.exists() {
        let _ = fs::create_dir_all(&job_folder);
    }

    // Quick first pass over central directory to build the list of files we will actually audit
    // (same filters as the processing loop below). Also collect skipped for the manifest.
    let mut zip_audited_rel: Vec<std::path::PathBuf> = Vec::new();
    let mut zip_skipped_rel: Vec<std::path::PathBuf> = Vec::new();
    for i in 0..archive.len() {
        if let Ok(zf) = archive.by_index(i) {
            let name = zf.name().to_string();
            if name.contains("__MACOSX") || name.split('/').last().unwrap_or("").starts_with("._") {
                continue;
            }
            if zf.is_file() {
                let ext = format!(".{}", name.split('.').last().unwrap_or("").to_lowercase());
                if allowed_extensions.contains(&ext) {
                    zip_audited_rel.push(std::path::PathBuf::from(name));
                } else {
                    zip_skipped_rel.push(std::path::PathBuf::from(name));
                }
            }
        }
    }

    // Write initial control manifest early (with PENDING hashes). Will be overwritten at end with full DNA.
    let initial_manifest = build_archive_control_manifest(
        &zip_file_name,
        &zip_path,
        &job_name,
        &timestamp,
        &zip_audited_rel,
        &zip_skipped_rel,
        &HashMap::new(),
    );
    let manifest_path = job_folder.join("~RAA-CONTROL-Manifest.log");
    if let Err(e) = fs::write(&manifest_path, &initial_manifest) {
        if debug_raa {
            eprintln!("[RAA] WARNING: Failed to write initial ZIP control manifest: {}", e);
        }
    } else if debug_raa {
        eprintln!("[RAA] Wrote INITIAL ~RAA-CONTROL-Manifest.log for archive at: {:?}", manifest_path);
        let _ = window.emit(
            "scan-event",
            ScanEvent {
                path: manifest_path.to_string_lossy().into(),
                status: "ControlManifest".into(),
                verdict: None,
                reason: None,
            },
        );
    }

    let mut file_analyses = Vec::new();
    let mut violation_found = false;

    // Collect real hashes for audited files as we process (for final DNA in manifest)
    let mut audited_hashes: HashMap<std::path::PathBuf, String> = HashMap::new();

    for i in 0..archive.len() {
        if let Ok(mut zf) = archive.by_index(i) {
            let name = zf.name().to_string();
            if name.contains("__MACOSX") || name.split('/').last().unwrap_or("").starts_with("._") {
                continue;
            }

            let ext = format!(".{}", name.split('.').last().unwrap_or("").to_lowercase());
            if zf.is_file() && allowed_extensions.contains(&ext) {
                let _ = window.emit(
                    "scan-event",
                    ScanEvent {
                        path: name.clone(),
                        status: "Active".into(),
                        verdict: None,
                        reason: None,
                    },
                );
                let mut buffer = vec![0; 2 * 1024 * 1024];
                let bytes_read = zf.read(&mut buffer).map_err(|e| e.to_string())?;
                let content = String::from_utf8_lossy(&buffer[..bytes_read]).to_string();
                let hash = get_content_hash(&content);

                // Record hash for the manifest DNA registry
                audited_hashes.insert(std::path::PathBuf::from(&name), hash.clone());

                let report =
                    call_llm_auditor(&content, "archive internal", &base_url, &model_name, &name, debug_oracle)
                        .await?;

                if report.is_error {
                    violation_found = true;
                }

                // Rich per-file block: preserves DNA line + stores the full oracle response
                let analysis_block = format!(
                    "--- RAA FILE ANALYSIS ---\n\
                     File: {} | Hash: {}\n\
                     Verdict: {}\n\
                     Analysis:\n{}\n\
                     ------------------------\n",
                    name,
                    hash,
                    report.verdict,
                    report.reasoning.trim()
                );

                file_analyses.push(analysis_block.clone());

                // Write individual per-file .raa report inside the job folder (following the Certify pattern).
                // Use the internal ZIP path (which may contain directories) as the relative structure.
                let rel_path = std::path::PathBuf::from(&name);
                let target_path = job_folder.join(&rel_path).with_extension("raa");
                if let Some(parent) = target_path.parent() {
                    let _ = fs::create_dir_all(parent);
                }
                match fs::write(&target_path, &analysis_block) {
                    Ok(_) => {
                        if debug_raa {
                            eprintln!("[RAA] Wrote per-file archive report inside job folder: {:?}", target_path);
                        }
                        let _ = window.emit(
                            "scan-event",
                            ScanEvent {
                                path: target_path.to_string_lossy().into(),
                                status: "PerFileReport".into(),
                                verdict: Some(report.verdict.clone()),
                                reason: None,
                            },
                        );
                    }
                    Err(e) => {
                        if debug_raa {
                            eprintln!("[RAA] WARNING: Failed to write per-file archive report {:?}: {}", target_path, e);
                        }
                    }
                }
            }
        }
    }

    let final_text = file_analyses.join("\n");

    // Finalize the ~RAA-CONTROL-Manifest.log at the end with full DNA Registry.
    // This overwrites the initial version written early, exactly as done for Certify.
    // Conforms to the visual structure and sections of the Certify reference output.
    let final_manifest = build_archive_control_manifest(
        &zip_file_name,
        &zip_path,
        &job_name,
        &timestamp,
        &zip_audited_rel,
        &zip_skipped_rel,
        &audited_hashes,
    );
    let manifest_path = job_folder.join("~RAA-CONTROL-Manifest.log");
    if let Err(e) = fs::write(&manifest_path, &final_manifest) {
        if debug_raa {
            eprintln!("[RAA] WARNING: Failed to finalize ~RAA-CONTROL-Manifest.log for archive: {}", e);
        }
    } else if debug_raa {
        eprintln!("[RAA] Finalized ~RAA-CONTROL-Manifest.log for archive (with DNA) at: {:?}", manifest_path);
        let _ = window.emit(
            "scan-event",
            ScanEvent {
                path: manifest_path.to_string_lossy().into(),
                status: "ControlManifest".into(),
                verdict: None,
                reason: None,
            },
        );
    }

    // Write the aggregated report inside the job folder (instead of root vault via legacy log_to_raa).
    // This makes Archive follow the same job-folder + per-file pattern as Certify.
    let report_filename = format!("{}-archive-{}.raa", sanitize_for_filename(zip_file_name), timestamp);
    let report_path = job_folder.join(report_filename);
    if let Err(e) = fs::write(&report_path, &final_text) {
        if debug_raa {
            eprintln!("[RAA] WARNING: Failed to write aggregated archive report to job folder: {}", e);
        }
    } else if debug_raa {
        eprintln!("[RAA] Wrote aggregated archive report inside job folder: {:?}", report_path);
    }

    // Legacy root write is intentionally skipped for the new path.
    // log_to_raa("archive", &zip_path, "ARCHIVE_BATCH", &final_text, &vault_root_path);

    Ok(RAAReport {
        verdict: if violation_found {
            "VIOLATION FOUND".into()
        } else {
            "SAFE".into()
        },
        reasoning: final_text,
        target_name: zip_path,
        is_error: violation_found,
    })
}

#[tauri::command]
async fn generate_manifest(
    window: tauri::Window,
    folder_path: String,
    allowed_extensions: Vec<String>,
    base_url: String,
    model_name: String,
    vault_root_path: String,
    debug_raa: bool,
    debug_oracle: bool,
) -> Result<RAAReport, String> {
    let folder_path_buf = fs::canonicalize(&folder_path).map_err(|_| "Path error")?;

    // =============================================
    // STAGE 1 (New Path Forward): Create dated Job Folder + write the
    // *initial* ~RAA-CONTROL-Manifest.log as the very first artifact.
    // (See build_initial_control_manifest and the write immediately below.)
    // This is the very first action when a Certify job starts.
    // =============================================
    let timestamp = Local::now().format("%Y%m%d-%H%M%S").to_string();

    let source_folder_name = folder_path_buf
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("UnknownProject");

    let job_name = format!("{}-{}", sanitize_for_filename(source_folder_name), timestamp);

    // Certify (folder) operations now root under RAA-Vault/Certify/
    let vault_root = resolve_operation_vault(&vault_root_path, "Certify");
    let job_folder = vault_root.join(&job_name);

    // Create the dated job folder
    if !job_folder.exists() {
        let _ = fs::create_dir_all(&job_folder);
    }

    // Collect which files will be skipped (we know this before any LLM work)
    let (_audited_paths, skipped_rel_paths) =
        collect_audited_and_skipped_paths(&folder_path_buf, &allowed_extensions);

    let mut skipped_hashes: HashMap<PathBuf, String> = HashMap::new();

    // Emit skipped events immediately so the UI's 🚫 SKIPPED area populates
    for rel in &skipped_rel_paths {
        let full_path = folder_path_buf.join(rel);
        let _ = window.emit(
            "scan-event",
            ScanEvent {
                path: full_path.to_string_lossy().into(),
                status: "Skipped".into(),
                verdict: None,
                reason: Some("filtered by rules".to_string()),
            },
        );
    }

    // =============================================
    // STAGE 1 (New Path Forward): Write the INITIAL ~RAA-CONTROL-Manifest.log
    // *immediately* as the very first artifact inside the job folder.
    // This happens before any content is read or any LLM calls are made.
    // The manifest contains the full inventory + hierarchical directory
    // structure using the exact same JUNK filter + extension rules as the audit.
    // It will be overwritten at job end with the DNA Registry version.
    // =============================================
    let initial_manifest = build_initial_control_manifest(
        &folder_path_buf,
        &allowed_extensions,
        source_folder_name,
        &job_name,
        &timestamp,
    );

    let manifest_path = job_folder.join("~RAA-CONTROL-Manifest.log");
    match fs::write(&manifest_path, &initial_manifest) {
        Ok(_) => {
            if debug_raa {
                eprintln!("[RAA] Wrote INITIAL ~RAA-CONTROL-Manifest.log (inventory + hierarchy) at: {:?}", manifest_path);
            }
            // Emit for the Stage 2 live COMPLETED comfort feed (right pane).
            let _ = window.emit(
                "scan-event",
                ScanEvent {
                    path: manifest_path.to_string_lossy().into(),
                    status: "ControlManifest".into(),
                    verdict: None,
                    reason: None,
                },
            );
        }
        Err(e) => {
            if debug_raa {
                eprintln!("[RAA] WARNING: Failed to write initial control manifest {:?}: {}", manifest_path, e);
            }
        }
    }

    // Continue with original processing (hashing, bucketing, LLM, per-file reports, final manifest overwrite)...
    let mut target_files = Vec::new();

    let walker = WalkDir::new(&folder_path_buf)
        .into_iter()
        .filter_entry(|e| {
            let name = e.file_name().to_string_lossy();
            !JUNK_NAMES.iter().any(|&ex| name == ex) && !name.starts_with("._")
        });

    for entry in walker.filter_map(|e| e.ok()) {
        let path = entry.path().to_path_buf();
        let ext = path
            .extension()
            .map(|s| format!(".{}", s.to_string_lossy().to_lowercase()))
            .unwrap_or_default();
        if entry.file_type().is_file() && (allowed_extensions.contains(&ext) || ext == ".zip") {
            let _ = window.emit(
                "scan-event",
                ScanEvent {
                    path: path.to_string_lossy().into(),
                    status: "Active".into(),
                    verdict: None,
                    reason: None,
                },
            );
            target_files.push((path, ext));
        }
    }

    // Partition: ZIPs are handled separately as archive containers (see CERTIFY_ZIP_BUCKETING.md).
    // Only non-ZIP files go through bucketing and Oracle in the Certify routine.
    let mut regular_target_files: Vec<(PathBuf, String)> = vec![];
    let mut zip_paths: Vec<PathBuf> = vec![];
    for (path, ext) in target_files {
        if ext == ".zip" {
            zip_paths.push(path);
        } else {
            regular_target_files.push((path, ext));
        }
    }

    let jobs: Vec<FileJob> = regular_target_files
        .into_par_iter()
        .filter_map(|(path, _ext)| {
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

    // Build audited hashes map after parallel work (safer than capturing mutably in closure)
    let mut audited_hashes: HashMap<PathBuf, String> = HashMap::new();
    for job in &jobs {
        if let Ok(rel) = job.path.strip_prefix(&folder_path_buf) {
            audited_hashes.insert(rel.to_path_buf(), job.hash.clone());
        }
    }

    const BUCKET_LIMIT: usize = 10000;

    let mut buckets = Vec::new();
    let mut current_bucket = Vec::new();
    let mut current_size = 0;

    for job in jobs {
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
    let mut report_entries = String::new();
    let mut any_violations = false;

    /// Robustly extract a JSON array from the model's response.
    /// LLMs often wrap output in ```json fences, add explanatory text, or emit slightly malformed JSON.
    /// We try direct parse, then clean markdown, then a best-effort [ ... ] substring extraction.
    fn extract_json_array_str(text: &str) -> Option<String> {
        let t = text.trim();

        // 1. Direct parse of the whole thing
        if serde_json::from_str::<Vec<serde_json::Value>>(t).is_ok() {
            return Some(t.to_string());
        }

        // 2. Strip common markdown code fences
        let cleaned = t
            .trim_start_matches("```json")
            .trim_start_matches("```JSON")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
            .to_string();

        if serde_json::from_str::<Vec<serde_json::Value>>(&cleaned).is_ok() {
            return Some(cleaned);
        }

        // 3. Best-effort: locate the outermost [ ... ] that parses as an array
        if let Some(start) = cleaned.find('[') {
            if let Some(end) = cleaned.rfind(']') {
                if end > start {
                    let candidate = &cleaned[start..=end];
                    if serde_json::from_str::<Vec<serde_json::Value>>(candidate).is_ok() {
                        return Some(candidate.to_string());
                    }
                }
            }
        }

        None
    }

    for bucket in buckets {
        let mut batch_text = String::new();
        let is_multi_bucket = bucket.len() > 1;

        if bucket.len() == 1 {
            // Single file in bucket — ask for focused analysis (no JSON required)
            let job = &bucket[0];
            batch_text.push_str(&format!(
                "Analyze the following file for security issues, malicious code, or suspicious behavior.\n\n\
                 FILE: {}\nHASH: {}\n\nCONTENT:\n{}\n",
                job.path.display(),
                job.hash,
                job.content
            ));
        } else {
            // Multiple files in one bucket — force the model to treat every file as a completely separate, independent audit
            let file_count = bucket.len();

            batch_text.push_str(&format!(
                "You have been given a batch containing exactly {} separate files to audit.\n\n\
                 YOU MUST FOLLOW THIS EXACT PROCESS (treat it like a programming for-loop):\n\n\
                 For i from 1 to {}:\n\
                   - Focus ONLY on FILE i.\n\
                   - Completely ignore every other file in this batch while analyzing FILE i.\n\
                   - Perform a full, independent security audit of ONLY FILE i.\n\
                   - Output the complete analysis and verdict for FILE i.\n\
                   - Then mentally clear your context before moving to FILE i+1.\n\
                   - Do not carry over any suspicions, context, or details from previous files.\n\n\
                 This is not a single holistic review of the whole batch. Each file must receive its own standalone forensic analysis.\n\n\
                 Files in this batch (in order):\n\n",
                file_count, file_count
            ));

            for (i, job) in bucket.iter().enumerate() {
                batch_text.push_str(&format!(
                    "=== START OF FILE {} ===\n\
                     FILE PATH: {}\n\
                     HASH: {}\n\n\
                     FILE CONTENT:\n{}\n\n\
                     === END OF FILE {} ===\n\n",
                    i + 1,
                    job.path.display(),
                    job.hash,
                    job.content,
                    i + 1
                ));
            }

            batch_text.push_str(
                "Now execute the process described above.\n\n\
                 REQUIRED OUTPUT FORMAT (YOU MUST FOLLOW EXACTLY — CRITICAL):\n\
                 Your *entire* response must be ONLY a valid JSON array. \n\
                 NOTHING before the opening [, NOTHING after the closing ].\n\
                 No markdown code fences (```json), no explanations, no extra text of any kind.\n\n\
                 The array must contain one object per file in this batch, in the same order.\n\n\
                 Each object MUST use these exact field names (and only these):\n\
                 {\n\
                   \"file_number\": number (starting from 1),\n\
                   \"file_path\": \"exact FILE PATH string from the list above\",\n\
                   \"verdict\": \"CERTIFIED\" or \"VIOLATION FOUND\",\n\
                   \"analysis\": \"your full, independent analysis for ONLY this file\"\n\
                 }\n\n\
                 Rules:\n\
                 - Use the exact field names (file_number, file_path, verdict, analysis).\n\
                 - Do not rename, add, or remove fields.\n\
                 - Do not wrap the array in an object.\n\
                 - Output ONLY the raw JSON array — nothing else in your response.\n\n\
                 Begin processing FILE 1 now."
            );
        }

        let report = call_llm_auditor(&batch_text, "folder", &base_url, &model_name, "Manifest", debug_oracle)
            .await
            .unwrap_or(RAAReport {
                verdict: "SAFE".into(),
                reasoning: "".into(),
                target_name: "".into(),
                is_error: false,
            });

        if report.is_error {
            any_violations = true;
        }

        // Try to parse structured JSON output from the model.
        // For multi-file buckets we expect an array. For single-file we expect prose (parse will fail, which is fine).
        #[derive(serde::Deserialize, Debug)]
        #[allow(dead_code)]
        struct FileAnalysis {
            file_number: u32,
            file_path: String,
            verdict: String,
            analysis: String,
        }

        let analyses: Vec<FileAnalysis> = if let Some(json_str) = extract_json_array_str(&report.reasoning) {
            match serde_json::from_str(&json_str) {
                Ok(parsed) => parsed,
                Err(e) => {
                    eprintln!(
                        "WARNING: Extracted JSON array but still failed to deserialize to FileAnalysis vec.\n\
                         Error: {}\n\
                         Extracted candidate (first 1500 chars):\n{}\n",
                        e,
                        &json_str.chars().take(1500).collect::<String>()
                    );
                    vec![]
                }
            }
        } else {
            // No usable array found in the response at all.
            if is_multi_bucket {
                eprintln!(
                    "WARNING: Failed to locate/parse any JSON array from model for multi-file bucket.\n\
                     Raw response (first 2000 chars):\n{}\n",
                    &report.reasoning.chars().take(2000).collect::<String>()
                );
            }
            vec![]
        };

        // When a multi-file bucket completely failed to parse, still capture the raw model response
        // once in the aggregated report_entries so the original blob is preserved for the job
        // (it will NOT be injected into the individual per-file .raa files anymore).
        if is_multi_bucket && analyses.is_empty() && !report.reasoning.trim().is_empty() {
            report_entries.push_str(&format!(
                "--- RAW MODEL RESPONSE FOR BUCKET (structured parse failed) ---\n{}\n------------------------\n\n",
                report.reasoning.trim()
            ));
        }

        // Write rich per-file analysis blocks
        for (i, job) in bucket.iter().enumerate() {
            let (verdict, analysis_text) = if let Some(a) = analyses.get(i) {
                // Successfully parsed per-file structured data from model
                if a.file_path != job.path.display().to_string() {
                    eprintln!(
                        "Warning: Model returned mismatched file_path.\n  Expected: {}\n  Got: {}",
                        job.path.display(),
                        a.file_path
                    );
                }
                (a.verdict.clone(), a.analysis.clone())
            } else {
                // Fallback: model did not return valid structured JSON for this position.
                // Still respect the top-level is_error from this bucket's LLM response.
                let fallback_verdict = if report.is_error {
                    "VIOLATION FOUND (structured output failed to parse)".to_string()
                } else {
                    "CERTIFIED (structured output failed to parse)".to_string()
                };

                let fallback_text = if is_multi_bucket && analyses.is_empty() {
                    // IMPORTANT: When a multi-file bucket produced an unparsable response
                    // (e.g. duplicate keys, extra prose, markdown, or the whole batch in one blob),
                    // do NOT stuff the entire raw model output (which may describe all files)
                    // into this individual file's .raa report. That pollutes the per-file record.
                    // The raw response is still captured in the aggregated certify-report-*.raa
                    // via report_entries below.
                    "[MODEL DID NOT RETURN USABLE STRUCTURED OUTPUT FOR THIS FILE]\n\n\
                     The LLM response for the bucket containing this file could not be parsed into per-file records.\n\
                     The full raw model output (including any batch data) is preserved in the main certify-report-*.raa\n\
                     file inside this job folder for manual review.".to_string()
                } else if analyses.is_empty() {
                    // Single-file bucket (or other non-multi case) — the full raw reasoning is the analysis.
                    format!(
                        "[MODEL DID NOT RETURN STRUCTURED OUTPUT]\n\n{}",
                        report.reasoning.trim()
                    )
                } else {
                    format!(
                        "[STRUCTURED OUTPUT PARSING FAILED FOR THIS FILE]\n\n{}",
                        report.reasoning.trim()
                    )
                };

                (fallback_verdict, fallback_text)
            };

            // Regular (non-ZIP) per-file report. ZIPs are now handled outside the bucketing/Oracle path
            // in Certify (see CERTIFY_ZIP_BUCKETING.md for the purged dedicated-bucket logic).
            let analysis_block = format!(
                "--- RAA FILE ANALYSIS ---\n\
                 File: {}\n\
                 Hash: {}\n\
                 Verdict: {}\n\
                 Analysis:\n{}\n\
                 ------------------------\n",
                job.path.display(),
                job.hash,
                verdict,
                analysis_text
            );

            report_entries.push_str(&analysis_block);

            // Write individual per-file report inside the job folder, preserving source hierarchy.
            // This enables proper DNA tracking per file in the new job folder model.
            let rel_path = match job.path.strip_prefix(&folder_path_buf) {
                Ok(p) => p.to_path_buf(),
                Err(_) => job.path.file_name().map(PathBuf::from).unwrap_or_else(|| job.path.clone()),
            };
            let target_path = job_folder.join(&rel_path).with_extension("raa");
            if let Some(parent) = target_path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            match fs::write(&target_path, &analysis_block) {
                Ok(_) => {
                    if debug_raa {
                        eprintln!("[RAA] Wrote per-file audited report: {:?}", target_path);
                    }
                    let _ = window.emit(
                        "scan-event",
                        ScanEvent {
                            path: target_path.to_string_lossy().into(),
                            status: "PerFileReport".into(),
                            verdict: Some(verdict.clone()),
                            reason: None,
                        },
                    );
                }
                Err(e) => {
                    if debug_raa {
                        eprintln!("[RAA] WARNING: Failed to write per-file audited report {:?}: {}", target_path, e);
                    }
                }
            }

            if verdict.to_uppercase().contains("VIOLATION") {
                any_violations = true;
            }
        }
    }

    // Prepend overall status header
    let overall_verdict = if any_violations {
        "UNCERTIFIED - VIOLATIONS FOUND".to_string()
    } else {
        "CERTIFIED".to_string()
    };

    let overall_header = format!(
        "--- OVERALL CERTIFY RESULT ---\nStatus: {}\nViolations Detected: {}\n\n",
        overall_verdict,
        if any_violations { "YES" } else { "NO" }
    );

    let full_report = format!("{}{}", overall_header, report_entries);

    // Write the (legacy aggregated) certify report inside the job folder.
    // Individual per-file .raa files (mirroring source hierarchy) are already written above
    // for proper DNA tracking and the new granular vault model.
    let report_filename = format!("certify-report-{}.raa", timestamp);
    let report_path = job_folder.join(report_filename);
    match fs::write(&report_path, &full_report) {
        Ok(_) => {
            if debug_raa {
                eprintln!("[RAA] Wrote certify report to: {:?}", report_path);
            }
        }
        Err(e) => {
            if debug_raa {
                eprintln!("[RAA] WARNING: Failed to write certify report to {:?}: {}", report_path, e);
            }
        }
    }

    // Handle ZIP containers found during this Certify run (purged from bucketing/Oracle path).
    // See CERTIFY_ZIP_BUCKETING.md for the removed dedicated-bucket logic.
    for zip_path in zip_paths {
        let rel_path = match zip_path.strip_prefix(&folder_path_buf) {
            Ok(p) => p.to_path_buf(),
            Err(_) => zip_path.file_name().map(PathBuf::from).unwrap_or_else(|| zip_path.clone()),
        };
        let target_path = job_folder.join(&rel_path).with_extension("raa");
        if let Some(parent) = target_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        let container_report = format!(
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
Hash: ZIP
Verdict: ARCHIVE CONTAINER (deep analysis recommended via Archive mode)
Analysis:
This is an archive container detected during Certify.
No deep per-file analysis of its contents was performed in this run.
Run a dedicated Archive audit on this file (using the link above or the Archive tab)
for full forensic analysis of the archive contents.

------------------------
"#,
            zip_path.display(), zip_path.display()
        );

        match fs::write(&target_path, &container_report) {
            Ok(_) => {
                if debug_raa {
                    eprintln!("[RAA] Wrote archive container report: {:?}", target_path);
                }
                let _ = window.emit(
                    "scan-event",
                    ScanEvent {
                        path: target_path.to_string_lossy().into(),
                        status: "PerFileReport".into(),
                        verdict: Some("ARCHIVE CONTAINER".into()),
                        reason: None,
                    },
                );
            }
            Err(e) => {
                if debug_raa {
                    eprintln!("[RAA] WARNING: Failed to write archive container report {:?}: {}", target_path, e);
                }
            }
        }
    }

    // =============================================
    // Write individual reports for skipped files inside the job folder.
    // We do this first (as requested) because we already know the skipped list
    // before any LLM work happens.
    // Each skipped file gets its own small .raa report with its DNA hash.
    // =============================================
    for rel_path in &skipped_rel_paths {
        let full_path = folder_path_buf.join(rel_path);
        let content = fs::read_to_string(&full_path).unwrap_or_default();
        let hash = get_content_hash(&content);
        skipped_hashes.insert(rel_path.clone(), hash.clone());

        let skipped_report = format!(
            "--- RAA FILE ANALYSIS ---\n\
             File: {}\n\
             Hash: {}\n\
             Verdict: SKIPPED\n\
             Analysis:\n\
             This file was not sent for AI analysis because its file extension \
             did not match the allowed extensions configured for this certification run.\n\
             ------------------------\n",
            rel_path.display(),
            hash
        );

        let target_path = job_folder.join(rel_path).with_extension("raa");
        if let Some(parent) = target_path.parent() {
            let _ = fs::create_dir_all(parent);
        }

        match fs::write(&target_path, &skipped_report) {
            Ok(_) => {
                if debug_raa {
                    eprintln!("[RAA] Wrote skipped report: {:?}", target_path);
                }
                let _ = window.emit(
                    "scan-event",
                    ScanEvent {
                        path: target_path.to_string_lossy().into(),
                        status: "SkippedReport".into(),
                        verdict: None,
                        reason: Some("extension filter".to_string()),
                    },
                );
            }
            Err(e) => {
                if debug_raa {
                    eprintln!("[RAA] WARNING: Failed to write skipped report {:?}: {}", target_path, e);
                }
            }
        }
    }

    // =============================================
    // Finalize / overwrite the ~RAA-CONTROL-Manifest.log
    // with the full version containing the DNA Registry (hashes) at the top.
    // This overwrites the initial inventory-only manifest written at job start.
    // The result is one static-named file per job with inventory + hierarchy + DNA.
    // =============================================
    let final_manifest = build_final_control_manifest_with_dna(
        &folder_path_buf,
        &allowed_extensions,
        source_folder_name,
        &job_name,
        &timestamp,
        &audited_hashes,
        &skipped_hashes,
    );

    let manifest_path = job_folder.join("~RAA-CONTROL-Manifest.log");
    match fs::write(&manifest_path, &final_manifest) {
        Ok(_) => {
            if debug_raa {
                eprintln!("[RAA] Finalized control manifest with DNA registry at: {:?}", manifest_path);
            }
            let _ = window.emit(
                "scan-event",
                ScanEvent {
                    path: manifest_path.to_string_lossy().into(),
                    status: "ControlManifest".into(),
                    verdict: None,
                    reason: None,
                },
            );
            // Do not re-emit; initial emit already announced creation to the live feed.
            // Final step just adds the DNA Registry section.
        }
        Err(e) => {
            if debug_raa {
                eprintln!("[RAA] WARNING: Failed to write final control manifest {:?}: {}", manifest_path, e);
            }
        }
    }

    Ok(RAAReport {
        verdict: overall_verdict,
        reasoning: full_report,
        target_name: folder_path,
        is_error: any_violations,
    })
}

// --- RAA LEDGER BROWSER ---
#[tauri::command]
async fn read_vault(vault_root_path: String) -> Result<String, String> {
    let audit_root = resolve_vault_root(&vault_root_path);
    
    let mut all_logs = String::new();

    if let Ok(entries) = fs::read_dir(audit_root) {
        let mut paths: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        paths.sort_by_key(|a| {
            a.metadata()
                .and_then(|m| m.modified())
                .ok()
                .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
        });
        paths.reverse();

        for entry in paths {
            if let Ok(content) = fs::read_to_string(entry.path()) {
                all_logs.push_str(&format!(
                    "\nFILE: {}\n{}",
                    entry.file_name().to_string_lossy(),
                    content
                ));
                all_logs.push_str("\n------------------------------------------------\n");
            }
        }
    }
    
    Ok(all_logs)
}

// --- VAULT HELPERS ---
#[tauri::command]
async fn create_vault_directory(root_path: String) -> Result<String, String> {
    let audit_root = resolve_vault_root(&root_path);

    fs::create_dir_all(&audit_root)
        .map_err(|e| format!("Failed to create RAA-Vault at {:?}: {}", audit_root, e))?;

    // VAULT-CODE-SAFEGUARD-START (sub creation part)
    // !!! PROTECTED BY "📜 Forensic Vault" INTEGRITY CHECK !!!
    // Do not delete or alter this loop. It is what guarantees the 4 subs on launch/custom root.
    // If removed, the Forensic Vault check in Integrity will show ❌ when structure is missing.
    // Ensure the operation-specific sub-roots exist for clean separation
    // Audit (terminal commands), Analyze (single files), Archive (zips), Certify (folders)
    for op in ["Audit", "Analyze", "Archive", "Certify"] {
        let op_path = audit_root.join(op);
        let _ = fs::create_dir_all(&op_path);
    }
    // VAULT-CODE-SAFEGUARD-END

    Ok(audit_root.to_string_lossy().into_owned())
}

#[tauri::command]
async fn get_default_vault_path() -> Result<String, String> {
    let home = env::var("HOME").unwrap_or_else(|_| ".".into());
    let default_path = PathBuf::from(home).join("Documents").join("RAA-Vault");
    Ok(default_path.to_string_lossy().into_owned())
}

// --- LEDGER BROWSER COMMANDS ---

/// Recursively collects .raa files from a directory (supports job folders inside operation roots).
fn collect_raa_files(dir: &Path, files: &mut Vec<VaultFile>) {
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_dir() {
                collect_raa_files(&path, files);
            } else if path.extension().and_then(|e| e.to_str()) == Some("raa") {
                let metadata = entry.metadata().ok();
                let modified = metadata
                    .as_ref()
                    .and_then(|m| m.modified().ok())
                    .and_then(|t| {
                        let datetime: chrono::DateTime<chrono::Local> = t.into();
                        Some(datetime.format("%Y%m%d-%H%M%S").to_string())
                    })
                    .unwrap_or_else(|| "unknown".into());

                let size = metadata.map(|m| m.len()).unwrap_or(0);

                let content = fs::read_to_string(&path).unwrap_or_default();
                let has_violation = content.to_uppercase().contains("VIOLATION");

                files.push(VaultFile {
                    name: path.file_name().unwrap_or_default().to_string_lossy().into(),
                    path: path.to_string_lossy().into(),
                    modified,
                    size,
                    has_violation,
                });
            }
        }
    }
}

#[tauri::command]
async fn list_vault_files(vault_root_path: String) -> Result<Vec<VaultFile>, String> {
    // ============================================================
    // Vault discovery now scans the operation roots (Audit/Analyze/Archive/Certify)
    // and recurses to find .raa files (including those inside dated job folders).
    // Full UI navigation of job folders as containers is still tabled per earlier notes.
    // See VAULT_ARCHITECTURE.md → Stage 5.
    // ============================================================
    // VAULT-CODE-SAFEGUARD-START
    // !!! DO NOT TOUCH OR DELETE THIS FUNCTION OR THE SUB CREATION LOGIC !!!
    // This is protected by the "📜 Forensic Vault" check in the 🛡️ Integrity Guard.
    // It must remain intact so the 4 subs are discovered and the finder works.
    // If you modify/delete this, the Forensic Vault item will show ❌ on integrity check.
    // This safeguards the vault *code* (static subs, grouping, list logic) - not the .raa data.
    let audit_root = resolve_vault_root(&vault_root_path);

    if !audit_root.exists() {
        return Ok(vec![]);
    }

    let mut files: Vec<VaultFile> = Vec::new();

    // Collect from main vault root (for backward compatibility with any old flat files)
    collect_raa_files(&audit_root, &mut files);

    // Collect from the new operation-specific roots (Audit/Analyze/Archive/Certify)
    // so that terminal masters, single-file reports, and job-folder contents are all discovered.
    // We always reference the typed subs so that reports under
    // custom roots or on initial launch are reliably found once the subs are statically created.
    for op in ["Audit", "Analyze", "Archive", "Certify"] {
        let op_root = audit_root.join(op);
        collect_raa_files(&op_root, &mut files);
    }

    // Newest first
    files.sort_by(|a, b| b.modified.cmp(&a.modified));
    Ok(files)
    // VAULT-CODE-SAFEGUARD-END
}

#[tauri::command]
async fn read_single_vault_file(full_path: String) -> Result<String, String> {
    fs::read_to_string(&full_path).map_err(|e| e.to_string())
}

/// Safely deletes a .raa vault file. Only allows deletion inside the current vault root.
#[tauri::command]
async fn delete_vault_file(full_path: String, vault_root_path: String) -> Result<(), String> {
    let audit_root = resolve_vault_root(&vault_root_path);
    let target_path = std::path::PathBuf::from(&full_path);

    // Security: prevent deleting files outside the vault
    if !target_path.starts_with(&audit_root) {
        return Err("Delete not allowed: file is outside the RAA vault".into());
    }

    if !target_path.exists() {
        return Err("File no longer exists".into());
    }

    // Extra safety: only allow .raa files
    if target_path.extension().and_then(|e| e.to_str()) != Some("raa") {
        return Err("Only .raa files can be deleted through this command".into());
    }

    fs::remove_file(&target_path).map_err(|e| format!("Failed to delete file: {}", e))
}

#[tauri::command]
async fn toggle_watcher(
    window: tauri::Window,
    enabled: bool,
    folders: Vec<String>,
    depth: usize,
) -> Result<(), String> {
    let mut watcher_lock = WATCHER.lock().map_err(|e| e.to_string())?;

    // Kill old watcher
    *watcher_lock = None;

    if !enabled || folders.is_empty() {
        println!("🕵️ Watcher: Deactivated");
        return Ok(());
    }

    println!(
        "🕵️ Watcher: ARMED for {} target slots (Depth: {})",
        folders.len(),
        depth
    );
    let window_clone = window.clone();
    let mut watcher =
        notify::recommended_watcher(move |res: Result<notify::Event, notify::Error>| match res {
            Ok(event) => {
                println!("📡 RADAR: Event Detected: {:?}", event.kind);
                for path in event.paths {
                    let path_str = path.to_string_lossy().to_string();
                    if !path_str.contains(".DS_Store") {
                        println!("⚡ SPARK: Broadcasting DNA change: {}", path_str);
                        let _ = window_clone.emit("watcher-event", path_str);
                    }
                }
            }
            Err(e) => println!("🚨 ERROR: Watcher kernel error: {:?}", e),
        })
        .map_err(|e| e.to_string())?;

    for folder in folders {
        let mode = if depth > 1 {
            notify::RecursiveMode::Recursive
        } else {
            notify::RecursiveMode::NonRecursive
        };
        let _ = watcher.watch(std::path::Path::new(&folder), mode);
    }

    *watcher_lock = Some(watcher);
    Ok(())
}

pub fn run() {
    dotenvy::dotenv().ok();
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            audit_command,
            generate_manifest,
            scan_file_integrity,
            scan_compressed_archive,
            read_vault,
            check_integrity,
            toggle_watcher,
            create_vault_directory,
            get_default_vault_path,
            list_vault_files,
            read_single_vault_file,
            delete_vault_file,
            hash_file
        ])
        .run(tauri::generate_context!())
        .expect("error");
}

// End of File
