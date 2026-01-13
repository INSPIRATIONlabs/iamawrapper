//! Payload assembly for macOS packages.
//!
//! Combines file collection with CPIO archive and gzip compression.

use std::fs;
use std::path::Path;

use walkdir::WalkDir;

use crate::macos::cpio::{CpioEntry, create_payload as create_cpio_payload};
use crate::models::PackageError;
use crate::models::macos::PackagePayload;

/// Collect files from a source directory.
///
/// Returns a PackagePayload containing metadata about all files.
pub fn collect_files(source_folder: &Path) -> Result<PackagePayload, PackageError> {
    let mut payload = PackagePayload::new();

    for entry in WalkDir::new(source_folder)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip the root directory itself
        if path == source_folder {
            continue;
        }

        // Get relative path
        let relative_path = path
            .strip_prefix(source_folder)
            .map_err(|e| PackageError::SourceReadError {
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?
            .to_path_buf();

        // Get metadata
        let metadata = entry
            .metadata()
            .map_err(|e| PackageError::SourceReadError {
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;

        // Get mode (default to 0o644 for files, 0o755 for directories)
        #[cfg(unix)]
        let mode = {
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode()
        };
        #[cfg(not(unix))]
        let mode = if metadata.is_dir() { 0o755 } else { 0o644 };

        if metadata.is_file() {
            payload.add_file(relative_path, metadata.len(), mode);
        } else if metadata.is_dir() {
            // Add directory with size 0
            payload.add_file(relative_path, 0, mode | 0o040000);
        }
    }

    Ok(payload)
}

/// Create a gzip-compressed CPIO payload from a source directory.
pub fn create_payload(source_folder: &Path) -> Result<Vec<u8>, PackageError> {
    let mut entries: Vec<CpioEntry> = Vec::new();

    for entry in WalkDir::new(source_folder)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip the root directory itself
        if path == source_folder {
            continue;
        }

        // Get relative path
        let relative_path =
            path.strip_prefix(source_folder)
                .map_err(|e| PackageError::SourceReadError {
                    path: path.to_path_buf(),
                    reason: e.to_string(),
                })?;

        // Get metadata
        let metadata = entry
            .metadata()
            .map_err(|e| PackageError::SourceReadError {
                path: path.to_path_buf(),
                reason: e.to_string(),
            })?;

        // Skip directories - CPIO will infer them from paths
        if metadata.is_dir() {
            continue;
        }

        // Get mode (default to 0o644 for files)
        #[cfg(unix)]
        let mode = {
            use std::os::unix::fs::PermissionsExt;
            metadata.permissions().mode() & 0o7777
        };
        #[cfg(not(unix))]
        let mode = 0o644;

        // Read file contents
        let content = fs::read(path).map_err(|e| PackageError::SourceReadError {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;

        entries.push((relative_path.to_string_lossy().to_string(), content, mode));
    }

    create_cpio_payload(&entries)
}

/// Scripts found in a scripts folder.
#[derive(Debug, Clone)]
pub struct ScriptsInfo {
    /// Whether preinstall script exists
    pub has_preinstall: bool,
    /// Whether postinstall script exists
    pub has_postinstall: bool,
}

/// Collect scripts from a scripts folder.
///
/// Looks for preinstall and postinstall scripts.
/// Returns information about which scripts were found.
pub fn collect_scripts(scripts_folder: &Path) -> Result<ScriptsInfo, PackageError> {
    if !scripts_folder.exists() {
        return Err(PackageError::ScriptsFolderNotFound {
            path: scripts_folder.to_path_buf(),
        });
    }

    let preinstall_path = scripts_folder.join("preinstall");
    let postinstall_path = scripts_folder.join("postinstall");

    let has_preinstall = preinstall_path.exists() && preinstall_path.is_file();
    let has_postinstall = postinstall_path.exists() && postinstall_path.is_file();

    Ok(ScriptsInfo {
        has_preinstall,
        has_postinstall,
    })
}

/// Create a gzip-compressed CPIO archive for scripts.
///
/// Scripts are always given mode 0755 (executable).
pub fn create_scripts_archive(scripts_folder: &Path) -> Result<Vec<u8>, PackageError> {
    let scripts_info = collect_scripts(scripts_folder)?;

    let mut entries: Vec<CpioEntry> = Vec::new();

    // Always use mode 0755 for scripts (executable)
    const SCRIPT_MODE: u32 = 0o755;

    if scripts_info.has_preinstall {
        let preinstall_path = scripts_folder.join("preinstall");
        let content = fs::read(&preinstall_path).map_err(|e| PackageError::SourceReadError {
            path: preinstall_path.clone(),
            reason: e.to_string(),
        })?;
        entries.push(("preinstall".to_string(), content, SCRIPT_MODE));
    }

    if scripts_info.has_postinstall {
        let postinstall_path = scripts_folder.join("postinstall");
        let content = fs::read(&postinstall_path).map_err(|e| PackageError::SourceReadError {
            path: postinstall_path.clone(),
            reason: e.to_string(),
        })?;
        entries.push(("postinstall".to_string(), content, SCRIPT_MODE));
    }

    if entries.is_empty() {
        return Err(PackageError::NoScriptsFound {
            path: scripts_folder.to_path_buf(),
        });
    }

    create_cpio_payload(&entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_collect_files_basic() {
        let temp_dir = TempDir::new().unwrap();

        // Create a test file
        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();

        let payload = collect_files(temp_dir.path()).unwrap();

        assert_eq!(payload.file_count(), 1);
        assert_eq!(payload.total_size, 11); // "hello world" = 11 bytes
    }

    #[test]
    fn test_collect_files_with_subdirectory() {
        let temp_dir = TempDir::new().unwrap();

        // Create nested structure
        let subdir = temp_dir.path().join("subdir");
        fs::create_dir(&subdir).unwrap();

        let file_path = subdir.join("file.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"content").unwrap();

        let payload = collect_files(temp_dir.path()).unwrap();

        // Should include directory and file
        assert!(payload.file_count() >= 1);
    }

    #[test]
    fn test_create_payload_basic() {
        let temp_dir = TempDir::new().unwrap();

        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello").unwrap();

        let payload = create_payload(temp_dir.path()).unwrap();

        // Should be gzip compressed (starts with gzip magic)
        assert_eq!(payload[0], 0x1f);
        assert_eq!(payload[1], 0x8b);
    }

    #[test]
    fn test_create_payload_multiple_files() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("file1.txt"), "content1").unwrap();
        fs::write(temp_dir.path().join("file2.txt"), "content2").unwrap();

        let payload = create_payload(temp_dir.path()).unwrap();

        // Verify it's valid gzip
        assert_eq!(payload[0], 0x1f);
        assert_eq!(payload[1], 0x8b);

        // Decompress and verify both files are present
        use flate2::read::GzDecoder;
        use std::io::Read;
        let mut decoder = GzDecoder::new(&payload[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();

        let content = String::from_utf8_lossy(&decompressed);
        assert!(content.contains("file1.txt"));
        assert!(content.contains("file2.txt"));
    }

    // T044: Unit tests for collect_scripts
    #[test]
    fn test_collect_scripts_both() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("preinstall"), "#!/bin/bash\necho pre").unwrap();
        fs::write(
            temp_dir.path().join("postinstall"),
            "#!/bin/bash\necho post",
        )
        .unwrap();

        let info = collect_scripts(temp_dir.path()).unwrap();

        assert!(info.has_preinstall, "Should detect preinstall");
        assert!(info.has_postinstall, "Should detect postinstall");
    }

    #[test]
    fn test_collect_scripts_preinstall_only() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("preinstall"), "#!/bin/bash\necho pre").unwrap();

        let info = collect_scripts(temp_dir.path()).unwrap();

        assert!(info.has_preinstall, "Should detect preinstall");
        assert!(!info.has_postinstall, "Should not detect postinstall");
    }

    #[test]
    fn test_collect_scripts_postinstall_only() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(
            temp_dir.path().join("postinstall"),
            "#!/bin/bash\necho post",
        )
        .unwrap();

        let info = collect_scripts(temp_dir.path()).unwrap();

        assert!(!info.has_preinstall, "Should not detect preinstall");
        assert!(info.has_postinstall, "Should detect postinstall");
    }

    #[test]
    fn test_collect_scripts_folder_not_found() {
        let result = collect_scripts(Path::new("/nonexistent/path"));
        assert!(matches!(
            result,
            Err(PackageError::ScriptsFolderNotFound { .. })
        ));
    }

    #[test]
    fn test_collect_scripts_empty_folder() {
        let temp_dir = TempDir::new().unwrap();

        let info = collect_scripts(temp_dir.path()).unwrap();

        assert!(
            !info.has_preinstall,
            "Should not detect preinstall in empty folder"
        );
        assert!(
            !info.has_postinstall,
            "Should not detect postinstall in empty folder"
        );
    }

    // T045: Unit tests for auto-set execute permission
    #[test]
    fn test_create_scripts_archive_executable_mode() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("preinstall"), "#!/bin/bash\necho test").unwrap();

        let archive = create_scripts_archive(temp_dir.path()).unwrap();

        // Decompress and check mode in CPIO header
        use flate2::read::GzDecoder;
        use std::io::Read;
        let mut decoder = GzDecoder::new(&archive[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();

        // CPIO odc format: mode is at offset 18-24 (6 octal digits)
        let mode_str = std::str::from_utf8(&decompressed[18..24]).unwrap();
        let mode = u32::from_str_radix(mode_str, 8).unwrap();
        let permissions = mode & 0o777;

        assert_eq!(
            permissions, 0o755,
            "Scripts should have execute permission (0755)"
        );
    }

    #[test]
    fn test_create_scripts_archive_no_scripts() {
        let temp_dir = TempDir::new().unwrap();

        // Create a file that's not preinstall or postinstall
        fs::write(temp_dir.path().join("other.sh"), "#!/bin/bash").unwrap();

        let result = create_scripts_archive(temp_dir.path());
        assert!(matches!(result, Err(PackageError::NoScriptsFound { .. })));
    }

    #[test]
    fn test_create_scripts_archive_contains_scripts() {
        let temp_dir = TempDir::new().unwrap();

        fs::write(temp_dir.path().join("preinstall"), "#!/bin/bash\necho pre").unwrap();
        fs::write(
            temp_dir.path().join("postinstall"),
            "#!/bin/bash\necho post",
        )
        .unwrap();

        let archive = create_scripts_archive(temp_dir.path()).unwrap();

        // Verify it's gzip
        assert_eq!(archive[0], 0x1f);
        assert_eq!(archive[1], 0x8b);

        // Decompress and verify both scripts are present
        use flate2::read::GzDecoder;
        use std::io::Read;
        let mut decoder = GzDecoder::new(&archive[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();

        let content = String::from_utf8_lossy(&decompressed);
        assert!(content.contains("preinstall"), "Should contain preinstall");
        assert!(
            content.contains("postinstall"),
            "Should contain postinstall"
        );
    }
}
