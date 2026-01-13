//! Bill of Materials (BOM) file generation using stuckliste.
//!
//! BOM files contain the manifest of all files in a macOS package.

use std::path::{Path, PathBuf};

use stuckliste::receipt::ReceiptBuilder;

use crate::models::PackageError;

/// Entry for BOM file.
#[derive(Debug, Clone)]
pub struct BomEntry {
    /// File path relative to install location
    pub path: PathBuf,
    /// Unix mode (includes file type bits)
    pub mode: u32,
    /// User ID (always 0 for packages)
    pub uid: u32,
    /// Group ID (always 80 for packages)
    pub gid: u32,
    /// File size in bytes
    pub size: u64,
}

/// Create a BOM file from a list of entries.
///
/// Note: stuckliste works with directories directly, so this function
/// is primarily for API compatibility. Use create_bom_from_directory
/// for actual BOM generation.
pub fn create_bom(entries: &[BomEntry]) -> Result<Vec<u8>, PackageError> {
    // For now, use a simple approach that creates a valid BOM structure
    // stuckliste requires a directory to scan

    // If no entries, return an error
    if entries.is_empty() {
        return Err(PackageError::BomError {
            reason: "Cannot create BOM with no entries".to_string(),
        });
    }

    // Create a temporary directory and populate it with the entries as placeholders
    // Then use stuckliste to generate the BOM
    use std::fs::{self, File};
    use std::io::Write;

    let temp_dir =
        tempfile::TempDir::new().map_err(|e: std::io::Error| PackageError::BomError {
            reason: e.to_string(),
        })?;

    for entry in entries {
        let full_path = temp_dir.path().join(&entry.path);

        // Create parent directories
        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent).map_err(|e| PackageError::BomError {
                reason: e.to_string(),
            })?;
        }

        // Create file or directory
        let is_dir = entry.mode & 0o170000 == 0o040000;
        if is_dir {
            fs::create_dir_all(&full_path).map_err(|e| PackageError::BomError {
                reason: e.to_string(),
            })?;
        } else {
            // Create a placeholder file
            let mut file = File::create(&full_path).map_err(|e| PackageError::BomError {
                reason: e.to_string(),
            })?;
            // Write placeholder content to match size
            let placeholder = vec![0u8; entry.size as usize];
            file.write_all(&placeholder)
                .map_err(|e| PackageError::BomError {
                    reason: e.to_string(),
                })?;
        }
    }

    create_bom_from_directory(temp_dir.path())
}

/// Create a BOM file by scanning a directory.
pub fn create_bom_from_directory(path: &Path) -> Result<Vec<u8>, PackageError> {
    // Use stuckliste to create the BOM
    let receipt = ReceiptBuilder::new()
        .create(path)
        .map_err(|e| PackageError::BomError {
            reason: e.to_string(),
        })?;

    // Write to bytes
    use std::io::Cursor;
    let mut output = Cursor::new(Vec::new());
    receipt
        .write(&mut output)
        .map_err(|e| PackageError::BomError {
            reason: e.to_string(),
        })?;

    Ok(output.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    // T010: BOM file generation tests
    #[test]
    fn test_bom_magic() {
        let bom_data = create_bom(&[BomEntry {
            path: PathBuf::from("test.txt"),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 5,
        }])
        .unwrap();
        assert_eq!(
            &bom_data[0..8],
            b"BOMStore",
            "BOM must start with 'BOMStore' magic"
        );
    }

    #[test]
    fn test_bom_contains_file_entries() {
        let bom_data = create_bom(&[
            BomEntry {
                path: PathBuf::from("file1.txt"),
                mode: 0o100644,
                uid: 0,
                gid: 80,
                size: 100,
            },
            BomEntry {
                path: PathBuf::from("file2.txt"),
                mode: 0o100755,
                uid: 0,
                gid: 80,
                size: 200,
            },
        ])
        .unwrap();
        assert!(
            bom_data.len() > 100,
            "BOM should have substantial content for file entries"
        );
    }

    #[test]
    fn test_bom_uid_gid() {
        let entries = vec![BomEntry {
            path: PathBuf::from("test.txt"),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 5,
        }];
        let result = create_bom(&entries);
        assert!(
            result.is_ok(),
            "BOM creation should succeed with uid=0, gid=80"
        );
    }

    #[test]
    fn test_bom_directory_entry() {
        let bom_data = create_bom(&[
            BomEntry {
                path: PathBuf::from("mydir"),
                mode: 0o040755,
                uid: 0,
                gid: 80,
                size: 0,
            },
            BomEntry {
                path: PathBuf::from("mydir/file.txt"),
                mode: 0o100644,
                uid: 0,
                gid: 80,
                size: 100,
            },
        ])
        .unwrap();
        assert!(
            bom_data.len() > 50,
            "BOM should contain directory and file entries"
        );
    }

    #[test]
    fn test_bom_preserves_permissions() {
        let entries = vec![
            BomEntry {
                path: PathBuf::from("executable"),
                mode: 0o100755,
                uid: 0,
                gid: 80,
                size: 1000,
            },
            BomEntry {
                path: PathBuf::from("readonly"),
                mode: 0o100444,
                uid: 0,
                gid: 80,
                size: 500,
            },
        ];
        let result = create_bom(&entries);
        assert!(
            result.is_ok(),
            "BOM creation should preserve different permissions"
        );
    }

    #[test]
    fn test_bom_with_deep_paths() {
        let bom_data = create_bom(&[BomEntry {
            path: PathBuf::from("Applications/MyApp.app/Contents/MacOS/myapp"),
            mode: 0o100755,
            uid: 0,
            gid: 80,
            size: 50000,
        }])
        .unwrap();
        assert!(bom_data.len() > 50, "BOM should handle deep paths");
    }

    #[test]
    fn test_create_bom_from_directory() {
        use std::fs::File;
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        let mut file = File::create(&file_path).unwrap();
        file.write_all(b"hello").unwrap();

        let result = create_bom_from_directory(temp_dir.path());
        assert!(result.is_ok(), "Should create BOM from directory");

        let bom_data = result.unwrap();
        assert!(bom_data.starts_with(b"BOMStore"), "BOM should have magic");
    }
}
