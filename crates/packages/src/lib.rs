//! Package management for Nostos
//!
//! Handles nostos.toml manifest parsing and GitHub package fetching.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

// ============================================================================
// Manifest Types (nostos.toml)
// ============================================================================

/// Project manifest (nostos.toml)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Manifest {
    /// Project metadata (uses [project] section for compatibility)
    #[serde(default)]
    pub project: ProjectInfo,
    /// Dependencies
    #[serde(default)]
    pub dependencies: HashMap<String, Dependency>,
}

/// Project metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProjectInfo {
    /// Project name
    #[serde(default)]
    pub name: String,
    /// Project version
    #[serde(default)]
    pub version: String,
    /// Project description
    pub description: Option<String>,
}

/// A dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Dependency {
    /// Simple version string (for future registry support)
    Simple(String),
    /// Detailed specification
    Detailed(DependencyDetail),
}

/// Detailed dependency specification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DependencyDetail {
    /// GitHub repository (e.g., "pegesund/nostos-utils")
    pub github: Option<String>,
    /// Git URL
    pub git: Option<String>,
    /// Local path
    pub path: Option<String>,
    /// Version/branch/tag/commit
    pub version: Option<String>,
}

impl Dependency {
    /// Get the GitHub repo if this is a GitHub dependency
    pub fn github(&self) -> Option<&str> {
        match self {
            Dependency::Simple(_) => None,
            Dependency::Detailed(d) => d.github.as_deref(),
        }
    }

    /// Get the version/ref
    pub fn version(&self) -> Option<&str> {
        match self {
            Dependency::Simple(v) => Some(v.as_str()),
            Dependency::Detailed(d) => d.version.as_deref(),
        }
    }

    /// Get the local path if this is a path dependency
    pub fn path(&self) -> Option<&str> {
        match self {
            Dependency::Simple(_) => None,
            Dependency::Detailed(d) => d.path.as_deref(),
        }
    }
}

// ============================================================================
// Package Manager
// ============================================================================

/// Package manager for fetching and caching dependencies
pub struct PackageManager {
    /// Root directory for cached packages (~/.nostos/packages/)
    cache_dir: PathBuf,
}

impl PackageManager {
    /// Create a new package manager
    pub fn new() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let cache_dir = PathBuf::from(home).join(".nostos").join("packages");
        PackageManager { cache_dir }
    }

    /// Create with a custom cache directory
    pub fn with_cache_dir(cache_dir: PathBuf) -> Self {
        PackageManager { cache_dir }
    }

    /// Load manifest from a project directory
    pub fn load_manifest(project_dir: &Path) -> Result<Manifest, String> {
        let manifest_path = project_dir.join("nostos.toml");
        if !manifest_path.exists() {
            return Ok(Manifest::default());
        }

        let content = fs::read_to_string(&manifest_path)
            .map_err(|e| format!("Failed to read nostos.toml: {}", e))?;

        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse nostos.toml: {}", e))
    }

    /// Ensure a dependency is fetched and return its local path
    pub fn ensure_dependency(&self, name: &str, dep: &Dependency) -> Result<PathBuf, String> {
        if let Some(path) = dep.path() {
            // Local path dependency
            let path = PathBuf::from(path);
            if path.exists() {
                return Ok(path);
            } else {
                return Err(format!("Path dependency not found: {}", path.display()));
            }
        }

        if let Some(github) = dep.github() {
            return self.fetch_github(name, github, dep.version());
        }

        Err(format!("Dependency '{}' has no source specified", name))
    }

    /// Fetch a GitHub repository
    fn fetch_github(&self, name: &str, repo: &str, version: Option<&str>) -> Result<PathBuf, String> {
        let version = version.unwrap_or("master");

        // Cache path: ~/.nostos/packages/github.com/owner/repo/version/
        let cache_path = self.cache_dir
            .join("github.com")
            .join(repo)
            .join(version);

        // Check if already cached
        if cache_path.exists() && cache_path.join(".nostos-pkg").exists() {
            eprintln!("Using cached package: {}", name);
            return Ok(cache_path);
        }

        eprintln!("Fetching package: {} from github.com/{} ({})", name, repo, version);

        // Create cache directory
        fs::create_dir_all(&cache_path)
            .map_err(|e| format!("Failed to create cache directory: {}", e))?;

        // Fetch files from GitHub
        self.download_github_archive(repo, version, &cache_path)?;

        // Write metadata file
        let meta = PackageMeta {
            name: name.to_string(),
            source: format!("github.com/{}", repo),
            version: version.to_string(),
            fetched_at: chrono_lite_now(),
        };
        let meta_path = cache_path.join(".nostos-pkg");
        let meta_content = toml::to_string_pretty(&meta)
            .map_err(|e| format!("Failed to serialize metadata: {}", e))?;
        fs::write(&meta_path, meta_content)
            .map_err(|e| format!("Failed to write metadata: {}", e))?;

        Ok(cache_path)
    }

    /// Download and extract a GitHub archive
    fn download_github_archive(&self, repo: &str, version: &str, dest: &Path) -> Result<(), String> {
        // Try to download individual .nos files via raw.githubusercontent.com
        // This is simpler than dealing with zip archives

        // First, get the file list from the GitHub API
        let api_url = format!(
            "https://api.github.com/repos/{}/contents?ref={}",
            repo, version
        );

        let client = reqwest::blocking::Client::builder()
            .user_agent("nostos-package-manager")
            .build()
            .map_err(|e| format!("Failed to create HTTP client: {}", e))?;

        let response = client.get(&api_url)
            .send()
            .map_err(|e| format!("Failed to fetch file list: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("GitHub API returned status: {}", response.status()));
        }

        let files: Vec<GitHubFile> = response.json()
            .map_err(|e| format!("Failed to parse GitHub API response: {}", e))?;

        // Download each .nos file
        for file in files {
            if file.name.ends_with(".nos") {
                let raw_url = format!(
                    "https://raw.githubusercontent.com/{}/{}/{}",
                    repo, version, file.name
                );

                eprintln!("  Downloading: {}", file.name);

                let content = client.get(&raw_url)
                    .send()
                    .map_err(|e| format!("Failed to download {}: {}", file.name, e))?
                    .text()
                    .map_err(|e| format!("Failed to read {}: {}", file.name, e))?;

                let file_path = dest.join(&file.name);
                fs::write(&file_path, content)
                    .map_err(|e| format!("Failed to write {}: {}", file.name, e))?;
            }
        }

        Ok(())
    }

    /// Get the path to a cached package (if it exists)
    pub fn get_cached_path(&self, _name: &str, dep: &Dependency) -> Option<PathBuf> {
        if let Some(path) = dep.path() {
            let path = PathBuf::from(path);
            if path.exists() {
                return Some(path);
            }
        }

        if let Some(github) = dep.github() {
            let version = dep.version().unwrap_or("master");
            let cache_path = self.cache_dir
                .join("github.com")
                .join(github)
                .join(version);

            if cache_path.exists() && cache_path.join(".nostos-pkg").exists() {
                return Some(cache_path);
            }
        }

        None
    }

    /// List all .nos files in a package directory
    pub fn list_package_files(package_path: &Path) -> Result<Vec<PathBuf>, String> {
        let mut files = Vec::new();

        if package_path.is_dir() {
            for entry in fs::read_dir(package_path)
                .map_err(|e| format!("Failed to read package directory: {}", e))?
            {
                let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
                let path = entry.path();
                if path.extension().map(|e| e == "nos").unwrap_or(false) {
                    files.push(path);
                }
            }
        }

        Ok(files)
    }
}

impl Default for PackageManager {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Helper Types
// ============================================================================

/// Package metadata stored in .nostos-pkg
#[derive(Debug, Serialize, Deserialize)]
struct PackageMeta {
    name: String,
    source: String,
    version: String,
    fetched_at: String,
}

/// GitHub API file entry
#[derive(Debug, Deserialize)]
struct GitHubFile {
    name: String,
    #[serde(rename = "type")]
    file_type: String,
}

/// Simple timestamp without chrono dependency
fn chrono_lite_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_manifest() {
        let toml = r#"
[project]
name = "my-project"
version = "1.0.0"

[dependencies]
utils = { github = "pegesund/nostos-utils" }
local-lib = { path = "../local-lib" }
"#;

        let manifest: Manifest = toml::from_str(toml).unwrap();
        assert_eq!(manifest.project.name, "my-project".to_string());
        assert_eq!(manifest.dependencies.len(), 2);

        let utils = &manifest.dependencies["utils"];
        assert_eq!(utils.github(), Some("pegesund/nostos-utils"));

        let local = &manifest.dependencies["local-lib"];
        assert_eq!(local.path(), Some("../local-lib"));
    }
}
