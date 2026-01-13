//! Package-related data models.

use std::path::PathBuf;
use std::time::Duration;

use crate::models::detection::DetectionMetadata;
use crate::models::error::{PackageError, PackageResult};

/// Verbosity level for output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Verbosity {
    /// Normal mode - show progress and messages
    #[default]
    Normal,
    /// Quiet mode - no prompts, overwrite existing
    Quiet,
    /// Silent mode - no console output at all
    Silent,
}

impl Verbosity {
    /// Returns true if prompts should be suppressed.
    pub fn suppress_prompts(&self) -> bool {
        matches!(self, Verbosity::Quiet | Verbosity::Silent)
    }

    /// Returns true if all output should be suppressed.
    pub fn suppress_output(&self) -> bool {
        matches!(self, Verbosity::Silent)
    }

    /// Returns true if progress should be shown.
    pub fn show_progress(&self) -> bool {
        matches!(self, Verbosity::Normal | Verbosity::Quiet)
    }
}

/// A file within the source package.
#[derive(Debug, Clone)]
pub struct SourceFile {
    /// Path relative to source root
    pub relative_path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// Whether this is the setup file
    pub is_setup_file: bool,
}

/// The collection of files to be packaged.
#[derive(Debug, Clone)]
pub struct SourcePackage {
    /// Root folder path
    pub root: PathBuf,
    /// Setup file path relative to root
    pub setup_file: PathBuf,
    /// All files to include (relative paths)
    pub files: Vec<SourceFile>,
    /// Total uncompressed size in bytes
    pub total_size: u64,
}

impl SourcePackage {
    /// Create a new source package from a root folder and setup file.
    pub fn new(root: PathBuf, setup_file: PathBuf) -> Self {
        Self {
            root,
            setup_file,
            files: Vec::new(),
            total_size: 0,
        }
    }

    /// Add a file to the package.
    pub fn add_file(&mut self, relative_path: PathBuf, size: u64, is_setup_file: bool) {
        self.files.push(SourceFile {
            relative_path,
            size,
            is_setup_file,
        });
        self.total_size += size;
    }

    /// Get the number of files in the package.
    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

/// Request to create an IntuneWin package.
#[derive(Debug, Clone)]
pub struct PackageRequest {
    /// Path to the source folder containing files to package
    pub source_folder: PathBuf,
    /// Name of the setup file within the source folder
    pub setup_file: String,
    /// Path to the output folder where .intunewin will be created
    pub output_folder: PathBuf,
    /// Optional custom output filename (without extension)
    pub output_name: Option<String>,
    /// Verbosity level for output
    pub verbosity: Verbosity,
}

impl PackageRequest {
    /// Create a new package request.
    pub fn new(source_folder: PathBuf, setup_file: String, output_folder: PathBuf) -> Self {
        Self {
            source_folder,
            setup_file,
            output_folder,
            output_name: None,
            verbosity: Verbosity::default(),
        }
    }

    /// Set custom output filename.
    pub fn with_output_name(mut self, name: String) -> Self {
        self.output_name = Some(name);
        self
    }

    /// Set verbosity level.
    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = verbosity;
        self
    }

    /// Validate the package request.
    pub fn validate(&self) -> PackageResult<()> {
        // Check source folder exists
        if !self.source_folder.exists() {
            return Err(PackageError::SourceFolderNotFound {
                path: self.source_folder.clone(),
            });
        }

        if !self.source_folder.is_dir() {
            return Err(PackageError::SourceFolderNotFound {
                path: self.source_folder.clone(),
            });
        }

        // Check setup file exists in source folder
        let setup_path = self.source_folder.join(&self.setup_file);
        if !setup_path.exists() {
            return Err(PackageError::SetupFileNotFound {
                file: self.setup_file.clone(),
                folder: self.source_folder.clone(),
            });
        }

        Ok(())
    }

    /// Get the output file path.
    pub fn output_path(&self) -> PathBuf {
        let base_name = self
            .output_name
            .as_ref()
            .map(|n| n.trim_end_matches(".intunewin").to_string())
            .unwrap_or_else(|| {
                // Use setup file name without extension
                PathBuf::from(&self.setup_file)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("package")
                    .to_string()
            });

        self.output_folder.join(format!("{}.intunewin", base_name))
    }
}

/// The final output package.
#[derive(Debug, Clone)]
pub struct IntuneWinPackage {
    /// Output file path
    pub output_path: PathBuf,
    /// Detection metadata
    pub metadata: DetectionMetadata,
    /// Size of the final .intunewin file
    pub package_size: u64,
    /// Time taken to create the package
    pub creation_time: Duration,
}

/// Request to unpack an IntuneWin package.
#[derive(Debug, Clone)]
pub struct UnpackRequest {
    /// Path to the .intunewin file to unpack
    pub input_file: PathBuf,
    /// Path to the output folder where files will be extracted
    pub output_folder: PathBuf,
    /// Verbosity level for output
    pub verbosity: Verbosity,
}

impl UnpackRequest {
    /// Create a new unpack request.
    pub fn new(input_file: PathBuf, output_folder: PathBuf) -> Self {
        Self {
            input_file,
            output_folder,
            verbosity: Verbosity::default(),
        }
    }

    /// Set verbosity level.
    pub fn with_verbosity(mut self, verbosity: Verbosity) -> Self {
        self.verbosity = verbosity;
        self
    }

    /// Validate the unpack request.
    pub fn validate(&self) -> PackageResult<()> {
        // Check input file exists
        if !self.input_file.exists() {
            return Err(PackageError::InvalidIntunewinFile {
                path: self.input_file.clone(),
                reason: "File does not exist".to_string(),
            });
        }

        if !self.input_file.is_file() {
            return Err(PackageError::InvalidIntunewinFile {
                path: self.input_file.clone(),
                reason: "Path is not a file".to_string(),
            });
        }

        Ok(())
    }
}

/// Result of unpacking an IntuneWin package.
#[derive(Debug, Clone)]
pub struct UnpackResult {
    /// Path to the output folder containing extracted files
    pub output_folder: PathBuf,
    /// Number of files extracted
    pub file_count: usize,
    /// Total size of extracted files in bytes
    pub total_size: u64,
    /// Time taken to unpack
    pub unpack_time: Duration,
    /// Original setup file name
    pub setup_file: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_verbosity_suppress_prompts() {
        assert!(!Verbosity::Normal.suppress_prompts());
        assert!(Verbosity::Quiet.suppress_prompts());
        assert!(Verbosity::Silent.suppress_prompts());
    }

    #[test]
    fn test_verbosity_suppress_output() {
        assert!(!Verbosity::Normal.suppress_output());
        assert!(!Verbosity::Quiet.suppress_output());
        assert!(Verbosity::Silent.suppress_output());
    }

    #[test]
    fn test_verbosity_show_progress() {
        assert!(Verbosity::Normal.show_progress());
        assert!(Verbosity::Quiet.show_progress());
        assert!(!Verbosity::Silent.show_progress());
    }

    #[test]
    fn test_output_path_default() {
        let req = PackageRequest::new(
            PathBuf::from("/source"),
            "setup.exe".to_string(),
            PathBuf::from("/output"),
        );
        assert_eq!(req.output_path(), PathBuf::from("/output/setup.intunewin"));
    }

    #[test]
    fn test_output_path_custom_name() {
        let req = PackageRequest::new(
            PathBuf::from("/source"),
            "setup.exe".to_string(),
            PathBuf::from("/output"),
        )
        .with_output_name("MyApp-v2.0".to_string());
        assert_eq!(
            req.output_path(),
            PathBuf::from("/output/MyApp-v2.0.intunewin")
        );
    }

    #[test]
    fn test_output_path_custom_name_with_extension() {
        let req = PackageRequest::new(
            PathBuf::from("/source"),
            "setup.exe".to_string(),
            PathBuf::from("/output"),
        )
        .with_output_name("MyApp.intunewin".to_string());
        assert_eq!(req.output_path(), PathBuf::from("/output/MyApp.intunewin"));
    }

    #[test]
    fn test_source_package_add_file() {
        let mut pkg = SourcePackage::new(PathBuf::from("/source"), PathBuf::from("setup.exe"));
        pkg.add_file(PathBuf::from("setup.exe"), 1024, true);
        pkg.add_file(PathBuf::from("data.dll"), 2048, false);

        assert_eq!(pkg.file_count(), 2);
        assert_eq!(pkg.total_size, 3072);
    }
}
