//! Archive creation and file collection.

use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::models::error::{PackageError, PackageResult};
use crate::models::package::SourcePackage;

/// Collect all files from the source folder.
///
/// This includes:
/// - All files recursively (including subdirectories)
/// - Hidden files (dotfiles on Unix, hidden attribute on Windows)
/// - Follows symbolic links
pub fn collect_source_files(
    source_folder: &Path,
    setup_file: &str,
) -> PackageResult<SourcePackage> {
    if !source_folder.exists() {
        return Err(PackageError::SourceFolderNotFound {
            path: source_folder.to_path_buf(),
        });
    }

    let mut package = SourcePackage::new(source_folder.to_path_buf(), PathBuf::from(setup_file));

    let mut found_setup = false;

    // Walk directory, following symlinks
    for entry in WalkDir::new(source_folder)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        // Skip directories
        if entry.file_type().is_dir() {
            continue;
        }

        let full_path = entry.path();

        // Get relative path from source folder
        let relative_path = full_path
            .strip_prefix(source_folder)
            .map_err(|_| PackageError::SourceReadError {
                path: full_path.to_path_buf(),
                reason: "Failed to compute relative path".to_string(),
            })?
            .to_path_buf();

        // Get file size
        let metadata = entry
            .metadata()
            .map_err(|e| PackageError::SourceReadError {
                path: full_path.to_path_buf(),
                reason: e.to_string(),
            })?;

        let size = metadata.len();

        // Check if this is the setup file
        let is_setup = is_setup_file(&relative_path, setup_file);
        if is_setup {
            found_setup = true;
        }

        package.add_file(relative_path, size, is_setup);
    }

    // Verify setup file was found
    if !found_setup {
        return Err(PackageError::SetupFileNotFound {
            file: setup_file.to_string(),
            folder: source_folder.to_path_buf(),
        });
    }

    // Sort files for deterministic output
    package
        .files
        .sort_by(|a, b| a.relative_path.cmp(&b.relative_path));

    Ok(package)
}

/// Check if a relative path matches the setup file name.
fn is_setup_file(relative_path: &Path, setup_file: &str) -> bool {
    // The setup file should be at the root level
    if relative_path.components().count() != 1 {
        return false;
    }

    // Compare filenames (case-sensitive on Unix, case-insensitive on Windows)
    #[cfg(windows)]
    {
        relative_path
            .to_string_lossy()
            .eq_ignore_ascii_case(setup_file)
    }

    #[cfg(not(windows))]
    {
        relative_path.to_string_lossy() == setup_file
    }
}

/// Normalize path separators to forward slashes for ZIP compatibility.
pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_collect_source_files_basic() {
        let temp = TempDir::new().unwrap();
        let source = temp.path();

        // Create setup file
        let mut setup = File::create(source.join("setup.exe")).unwrap();
        setup.write_all(b"setup content").unwrap();

        // Create another file
        let mut data = File::create(source.join("data.dll")).unwrap();
        data.write_all(b"dll content").unwrap();

        let package = collect_source_files(source, "setup.exe").unwrap();

        assert_eq!(package.file_count(), 2);
        assert!(package.files.iter().any(|f| f.is_setup_file));
    }

    #[test]
    fn test_collect_source_files_with_subdirectory() {
        let temp = TempDir::new().unwrap();
        let source = temp.path();

        // Create setup file
        File::create(source.join("setup.exe")).unwrap();

        // Create subdirectory with file
        fs::create_dir(source.join("data")).unwrap();
        File::create(source.join("data").join("config.xml")).unwrap();

        let package = collect_source_files(source, "setup.exe").unwrap();

        assert_eq!(package.file_count(), 2);
    }

    #[test]
    fn test_collect_source_files_hidden_files() {
        let temp = TempDir::new().unwrap();
        let source = temp.path();

        // Create setup file
        File::create(source.join("setup.exe")).unwrap();

        // Create hidden file (dotfile)
        File::create(source.join(".hidden")).unwrap();

        let package = collect_source_files(source, "setup.exe").unwrap();

        // Should include hidden file
        assert_eq!(package.file_count(), 2);
        assert!(
            package
                .files
                .iter()
                .any(|f| f.relative_path.to_string_lossy().contains(".hidden"))
        );
    }

    #[test]
    fn test_collect_source_files_missing_setup() {
        let temp = TempDir::new().unwrap();
        let source = temp.path();

        // Create a file but not the setup file
        File::create(source.join("other.exe")).unwrap();

        let result = collect_source_files(source, "setup.exe");

        assert!(matches!(
            result,
            Err(PackageError::SetupFileNotFound { .. })
        ));
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path(Path::new("foo/bar")), "foo/bar");
        assert_eq!(normalize_path(Path::new("foo\\bar")), "foo/bar");
        assert_eq!(normalize_path(Path::new("foo\\bar\\baz")), "foo/bar/baz");
    }

    #[test]
    fn test_is_setup_file() {
        assert!(is_setup_file(Path::new("setup.exe"), "setup.exe"));
        assert!(!is_setup_file(Path::new("other.exe"), "setup.exe"));
        assert!(!is_setup_file(Path::new("subdir/setup.exe"), "setup.exe"));
    }
}
