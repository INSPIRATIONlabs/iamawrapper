//! macOS package-related data models.

use std::path::PathBuf;
use std::time::Duration;

use crate::models::package::Verbosity;

/// Request to create a macOS flat package (.pkg).
#[derive(Debug, Clone)]
pub struct MacosPkgRequest {
    /// Path to the source folder containing files to package
    pub source_folder: PathBuf,
    /// Package identifier (e.g., "com.company.app")
    pub identifier: String,
    /// Package version (e.g., "1.0.0")
    pub version: String,
    /// Installation target path on macOS
    pub install_location: PathBuf,
    /// Path to the output folder where .pkg will be created
    pub output_folder: PathBuf,
    /// Optional custom output filename (without extension)
    pub output_name: Option<String>,
    /// Optional folder containing preinstall/postinstall scripts
    pub scripts_folder: Option<PathBuf>,
    /// Verbosity level for output
    pub verbosity: Verbosity,
}

impl MacosPkgRequest {
    /// Create a new macOS package request with required fields.
    pub fn new(
        source_folder: PathBuf,
        identifier: String,
        version: String,
        output_folder: PathBuf,
    ) -> Self {
        Self {
            source_folder,
            identifier,
            version,
            install_location: PathBuf::from("/"),
            output_folder,
            output_name: None,
            scripts_folder: None,
            verbosity: Verbosity::default(),
        }
    }

    /// Set custom installation location.
    pub fn with_install_location(mut self, path: PathBuf) -> Self {
        self.install_location = path;
        self
    }

    /// Set custom output filename.
    pub fn with_output_name(mut self, name: String) -> Self {
        self.output_name = Some(name);
        self
    }

    /// Set scripts folder.
    pub fn with_scripts_folder(mut self, path: PathBuf) -> Self {
        self.scripts_folder = Some(path);
        self
    }

    /// Set verbosity level.
    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = verbosity;
        self
    }

    /// Get the output file path.
    pub fn output_path(&self) -> PathBuf {
        let base_name = self
            .output_name
            .as_ref()
            .map(|n| n.trim_end_matches(".pkg").to_string())
            .unwrap_or_else(|| format!("{}-{}", self.identifier, self.version));

        self.output_folder.join(format!("{}.pkg", base_name))
    }

    /// Check if identifier follows reverse-DNS convention.
    pub fn is_valid_identifier(&self) -> bool {
        let parts: Vec<&str> = self.identifier.split('.').collect();
        parts.len() >= 2 && parts.iter().all(|p| !p.is_empty())
    }
}

/// Result of successful macOS package creation.
#[derive(Debug, Clone)]
pub struct MacosPkgResult {
    /// Full path to created .pkg file
    pub output_path: PathBuf,
    /// Size of final .pkg in bytes
    pub package_size: u64,
    /// Number of files in payload
    pub file_count: usize,
    /// Time to create package
    pub creation_time: Duration,
}

/// A file to include in the package payload.
#[derive(Debug, Clone)]
pub struct PayloadFile {
    /// Path relative to install_location
    pub relative_path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// Unix permissions
    pub mode: u32,
}

/// The package payload (files to install).
#[derive(Debug, Clone)]
pub struct PackagePayload {
    /// Files included in payload
    pub files: Vec<PayloadFile>,
    /// Uncompressed total size
    pub total_size: u64,
}

impl PackagePayload {
    /// Create a new empty payload.
    pub fn new() -> Self {
        Self {
            files: Vec::new(),
            total_size: 0,
        }
    }

    /// Add a file to the payload.
    pub fn add_file(&mut self, relative_path: PathBuf, size: u64, mode: u32) {
        self.files.push(PayloadFile {
            relative_path,
            size,
            mode,
        });
        self.total_size += size;
    }

    /// Get the number of files.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

impl Default for PackagePayload {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_pkg_request_new() {
        let req = MacosPkgRequest::new(
            PathBuf::from("/source"),
            "com.test.app".to_string(),
            "1.0.0".to_string(),
            PathBuf::from("/output"),
        );

        assert_eq!(req.source_folder, PathBuf::from("/source"));
        assert_eq!(req.identifier, "com.test.app");
        assert_eq!(req.version, "1.0.0");
        assert_eq!(req.install_location, PathBuf::from("/"));
        assert_eq!(req.output_folder, PathBuf::from("/output"));
        assert!(req.output_name.is_none());
        assert!(req.scripts_folder.is_none());
    }

    #[test]
    fn test_output_path_default() {
        let req = MacosPkgRequest::new(
            PathBuf::from("/source"),
            "com.test.app".to_string(),
            "1.0.0".to_string(),
            PathBuf::from("/output"),
        );

        assert_eq!(
            req.output_path(),
            PathBuf::from("/output/com.test.app-1.0.0.pkg")
        );
    }

    #[test]
    fn test_output_path_custom_name() {
        let req = MacosPkgRequest::new(
            PathBuf::from("/source"),
            "com.test.app".to_string(),
            "1.0.0".to_string(),
            PathBuf::from("/output"),
        )
        .with_output_name("MyApp".to_string());

        assert_eq!(req.output_path(), PathBuf::from("/output/MyApp.pkg"));
    }

    #[test]
    fn test_is_valid_identifier() {
        let req = MacosPkgRequest::new(
            PathBuf::from("/source"),
            "com.test.app".to_string(),
            "1.0.0".to_string(),
            PathBuf::from("/output"),
        );
        assert!(req.is_valid_identifier());

        let req_invalid = MacosPkgRequest::new(
            PathBuf::from("/source"),
            "myapp".to_string(),
            "1.0.0".to_string(),
            PathBuf::from("/output"),
        );
        assert!(!req_invalid.is_valid_identifier());
    }

    #[test]
    fn test_payload_add_file() {
        let mut payload = PackagePayload::new();
        payload.add_file(PathBuf::from("file1.txt"), 1024, 0o644);
        payload.add_file(PathBuf::from("file2.txt"), 2048, 0o755);

        assert_eq!(payload.file_count(), 2);
        assert_eq!(payload.total_size, 3072);
    }
}
