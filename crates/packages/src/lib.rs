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
    /// Git URL (for full git URLs)
    pub git: Option<String>,
    /// Local path
    pub path: Option<String>,
    /// Version/branch/tag/commit
    pub version: Option<String>,
    /// Whether this is a native extension (requires cargo build)
    #[serde(default)]
    pub extension: bool,
}

impl Dependency {
    /// Get the GitHub repo if this is a GitHub dependency
    pub fn github(&self) -> Option<&str> {
        match self {
            Dependency::Simple(_) => None,
            Dependency::Detailed(d) => d.github.as_deref(),
        }
    }

    /// Get the git URL if specified
    pub fn git(&self) -> Option<&str> {
        match self {
            Dependency::Simple(_) => None,
            Dependency::Detailed(d) => d.git.as_deref(),
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

    /// Check if this is a native extension (requires cargo build)
    pub fn is_extension(&self) -> bool {
        match self {
            Dependency::Simple(_) => false,
            Dependency::Detailed(d) => d.extension,
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

    // ========================================================================
    // Extension Support
    // ========================================================================

    /// Ensure an extension is fetched and built, return paths to library and module dir
    pub fn ensure_extension(&self, name: &str, dep: &Dependency) -> Result<ExtensionResult, String> {
        let git_url = if let Some(github) = dep.github() {
            format!("https://github.com/{}.git", github)
        } else if let Some(git) = dep.git() {
            git.to_string()
        } else if let Some(path) = dep.path() {
            // Local path extension - just build it
            let path = PathBuf::from(path);
            if !path.exists() {
                return Err(format!("Extension path not found: {}", path.display()));
            }
            let lib_path = self.build_extension(&path)?;
            return Ok(ExtensionResult {
                name: name.to_string(),
                library_path: lib_path,
                module_dir: path,
            });
        } else {
            return Err(format!("Extension '{}' has no source specified", name));
        };

        let version = dep.version().unwrap_or("master");

        // Cache path for extensions: ~/.nostos/extensions/repo-name/
        let ext_cache_dir = self.extensions_cache_dir();
        let repo_name = git_url
            .trim_end_matches(".git")
            .rsplit('/')
            .next()
            .unwrap_or(name);
        let ext_dir = ext_cache_dir.join(repo_name);

        // Check if we need to fetch
        if ext_dir.exists() {
            // Check if already built and up-to-date
            let lib_path = self.find_extension_library(&ext_dir);
            if lib_path.is_some() {
                eprintln!("Using cached extension: {}", name);
                return Ok(ExtensionResult {
                    name: name.to_string(),
                    library_path: lib_path.unwrap(),
                    module_dir: ext_dir,
                });
            }
        }

        // Fetch extension
        eprintln!("Fetching extension: {} from {}", name, git_url);
        self.fetch_git_repo(&git_url, version, &ext_dir)?;

        // Build extension
        let lib_path = self.build_extension(&ext_dir)?;

        Ok(ExtensionResult {
            name: name.to_string(),
            library_path: lib_path,
            module_dir: ext_dir,
        })
    }

    /// Get extensions cache directory
    fn extensions_cache_dir(&self) -> PathBuf {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(".nostos").join("extensions")
    }

    /// Fetch a git repository
    fn fetch_git_repo(&self, url: &str, version: &str, target: &Path) -> Result<(), String> {
        use std::process::Command;

        fs::create_dir_all(target)
            .map_err(|e| format!("Failed to create directory: {}", e))?;

        // If target exists and has .git, try to update
        if target.join(".git").exists() {
            eprintln!("  Updating existing repo...");
            let status = Command::new("git")
                .args(["fetch", "--all"])
                .current_dir(target)
                .status()
                .map_err(|e| format!("Failed to run git fetch: {}", e))?;

            if status.success() {
                let checkout_status = Command::new("git")
                    .args(["checkout", version])
                    .current_dir(target)
                    .status()
                    .map_err(|e| format!("Failed to run git checkout: {}", e))?;

                if checkout_status.success() {
                    return Ok(());
                }
            }
            // If update failed, remove and re-clone
            fs::remove_dir_all(target)
                .map_err(|e| format!("Failed to remove old repo: {}", e))?;
            fs::create_dir_all(target)
                .map_err(|e| format!("Failed to recreate directory: {}", e))?;
        }

        // Clone with specific branch/tag/commit
        eprintln!("  Cloning {}...", url);
        let status = Command::new("git")
            .args(["clone", "--depth", "1", "--branch", version, url, target.to_str().unwrap()])
            .status();

        match status {
            Ok(s) if s.success() => Ok(()),
            Ok(_) => {
                // Try without --branch (for commit hashes)
                let _ = fs::remove_dir_all(target);
                fs::create_dir_all(target)
                    .map_err(|e| format!("Failed to recreate directory: {}", e))?;

                let status = Command::new("git")
                    .args(["clone", url, target.to_str().unwrap()])
                    .status()
                    .map_err(|e| format!("Failed to run git clone: {}", e))?;

                if !status.success() {
                    return Err(format!("Failed to clone {}", url));
                }

                // Checkout specific commit
                let status = Command::new("git")
                    .args(["checkout", version])
                    .current_dir(target)
                    .status()
                    .map_err(|e| format!("Failed to run git checkout: {}", e))?;

                if !status.success() {
                    return Err(format!("Failed to checkout {} in {}", version, url));
                }

                Ok(())
            }
            Err(e) => Err(format!("Failed to run git clone: {}", e)),
        }
    }

    /// Build an extension with cargo
    fn build_extension(&self, ext_dir: &Path) -> Result<PathBuf, String> {
        use std::process::Command;

        eprintln!("  Building extension in {:?}...", ext_dir);

        let status = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(ext_dir)
            .status()
            .map_err(|e| format!("Failed to run cargo build: {}", e))?;

        if !status.success() {
            return Err(format!("Cargo build failed in {:?}", ext_dir));
        }

        // Find the library
        self.find_extension_library(ext_dir)
            .ok_or_else(|| format!("No .so/.dylib found after building in {:?}", ext_dir))
    }

    /// Find the compiled extension library in target/release/
    fn find_extension_library(&self, ext_dir: &Path) -> Option<PathBuf> {
        let release_dir = ext_dir.join("target").join("release");

        if !release_dir.exists() {
            return None;
        }

        for entry in fs::read_dir(&release_dir).ok()? {
            let entry = entry.ok()?;
            let path = entry.path();
            if let Some(ext) = path.extension() {
                if ext == "so" || ext == "dylib" {
                    let name = path.file_name()?.to_str()?;
                    // Match libname.so but not libname-hash.so (deps)
                    if name.starts_with("lib") && !name.contains('-') {
                        return Some(path);
                    }
                }
            }
        }

        None
    }
}

/// Result of ensuring an extension is available
#[derive(Debug, Clone)]
pub struct ExtensionResult {
    /// Extension name
    pub name: String,
    /// Path to the compiled .so/.dylib file
    pub library_path: PathBuf,
    /// Directory containing .nos wrapper files
    pub module_dir: PathBuf,
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
