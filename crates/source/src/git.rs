//! Git integration for .nostos repository

use std::path::Path;
use std::process::Command;

/// Information about a git commit
#[derive(Debug, Clone)]
pub struct CommitInfo {
    /// Full commit hash
    pub hash: String,
    /// Short commit hash (7 chars)
    pub short_hash: String,
    /// Commit message (first line)
    pub message: String,
    /// Commit date (ISO format)
    pub date: String,
    /// Author name
    pub author: String,
}

/// Initialize git repository if not already initialized
pub fn init_repo(nostos_dir: &Path) -> Result<(), String> {
    let git_dir = nostos_dir.join(".git");
    if git_dir.exists() {
        return Ok(());
    }

    // Create .nostos directory if needed
    std::fs::create_dir_all(nostos_dir)
        .map_err(|e| format!("Failed to create .nostos: {}", e))?;

    // git init
    let output = Command::new("git")
        .args(["init"])
        .current_dir(nostos_dir)
        .output()
        .map_err(|e| format!("Failed to run git init: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "git init failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // Create .gitignore
    let gitignore = nostos_dir.join(".gitignore");
    std::fs::write(&gitignore, "*.tmp\n*.swp\n")
        .map_err(|e| format!("Failed to create .gitignore: {}", e))?;

    // Initial commit
    add_and_commit(nostos_dir, &[".gitignore"], "Initialize .nostos repository")?;

    Ok(())
}

/// Stage files and commit
fn add_and_commit(nostos_dir: &Path, files: &[&str], message: &str) -> Result<(), String> {
    // git add
    let mut add_cmd = Command::new("git");
    add_cmd.arg("add").current_dir(nostos_dir);
    for file in files {
        add_cmd.arg(file);
    }

    let output = add_cmd
        .output()
        .map_err(|e| format!("Failed to run git add: {}", e))?;

    if !output.status.success() {
        return Err(format!(
            "git add failed: {}",
            String::from_utf8_lossy(&output.stderr)
        ));
    }

    // git commit
    let output = Command::new("git")
        .args(["commit", "-m", message])
        .current_dir(nostos_dir)
        .output()
        .map_err(|e| format!("Failed to run git commit: {}", e))?;

    // Commit can "fail" if there's nothing to commit, which is fine
    // Note: "nothing to commit" message goes to stdout, not stderr
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stderr.contains("nothing to commit") && !stdout.contains("nothing to commit") {
            return Err(format!("git commit failed: {}{}", stderr, stdout));
        }
    }

    Ok(())
}
