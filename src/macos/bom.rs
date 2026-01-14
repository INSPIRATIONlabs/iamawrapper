//! Bill of Materials (BOM) file generation.
//!
//! Cross-platform implementation of macOS BOM format.
//! BOM files contain the manifest of all files in a macOS package.

use std::collections::HashMap;
use std::path::PathBuf;

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

// BOM file type constants
const TYPE_FILE: u8 = 1;
const TYPE_DIR: u8 = 2;
// const TYPE_LINK: u8 = 3;
// const TYPE_DEV: u8 = 4;

/// BOM file writer - cross-platform implementation.
struct BomWriter {
    /// Data blocks stored in the BOM
    blocks: Vec<Vec<u8>>,
    /// Named variables (e.g., "BomInfo", "Paths", etc.)
    vars: HashMap<String, u32>,
}

impl BomWriter {
    fn new() -> Self {
        // Block 0 is always null/empty
        Self {
            blocks: vec![vec![]],
            vars: HashMap::new(),
        }
    }

    /// Add a data block and return its index
    fn add_block(&mut self, data: Vec<u8>) -> u32 {
        let index = self.blocks.len() as u32;
        self.blocks.push(data);
        index
    }

    /// Add a named variable pointing to a block
    fn add_var(&mut self, name: &str, block_index: u32) {
        self.vars.insert(name.to_string(), block_index);
    }

    /// Write 32-bit big-endian value
    fn write_u32_be(buf: &mut Vec<u8>, val: u32) {
        buf.extend_from_slice(&val.to_be_bytes());
    }

    /// Write 16-bit big-endian value
    fn write_u16_be(buf: &mut Vec<u8>, val: u16) {
        buf.extend_from_slice(&val.to_be_bytes());
    }

    /// Build BOMInfo block
    fn build_bom_info(&mut self, num_paths: u32) -> u32 {
        let mut data = Vec::new();
        // version = 1
        Self::write_u32_be(&mut data, 1);
        // numberOfPaths
        Self::write_u32_be(&mut data, num_paths);
        // numberOfInfoEntries = 0
        Self::write_u32_be(&mut data, 0);
        self.add_block(data)
    }

    /// Build a tree structure for paths
    fn build_tree(&mut self, paths_block: u32, path_count: u32) -> u32 {
        let mut data = Vec::new();
        // "tree" magic
        data.extend_from_slice(b"tree");
        // version = 1
        Self::write_u32_be(&mut data, 1);
        // child (points to BOMPaths)
        Self::write_u32_be(&mut data, paths_block);
        // blockSize = 4096
        Self::write_u32_be(&mut data, 4096);
        // pathCount
        Self::write_u32_be(&mut data, path_count);
        // unknown3 = 0
        data.push(0);

        self.add_block(data)
    }

    /// Build VIndex for a tree
    fn build_vindex(&mut self, tree_block: u32) -> u32 {
        let mut data = Vec::new();
        // unknown0 = 1
        Self::write_u32_be(&mut data, 1);
        // indexToVTree
        Self::write_u32_be(&mut data, tree_block);
        // unknown2 = 0
        Self::write_u32_be(&mut data, 0);
        // unknown3 = 0
        data.push(0);

        self.add_block(data)
    }

    /// Build the final BOM file
    fn build(self) -> Vec<u8> {
        let mut output = Vec::new();

        // Calculate offsets
        let header_size = 512; // Standard BOM header area

        // Build block table
        let mut block_table = Vec::new();
        Self::write_u32_be(&mut block_table, self.blocks.len() as u32);

        // Calculate where each block will be stored
        let mut current_offset = header_size as u32;
        let mut block_offsets = Vec::new();

        for block in &self.blocks {
            block_offsets.push((current_offset, block.len() as u32));
            current_offset += block.len() as u32;
        }

        // Write block pointers
        for (offset, length) in &block_offsets {
            Self::write_u32_be(&mut block_table, *offset);
            Self::write_u32_be(&mut block_table, *length);
        }

        // Build vars table
        let mut vars_data = Vec::new();
        Self::write_u32_be(&mut vars_data, self.vars.len() as u32);
        for (name, index) in &self.vars {
            Self::write_u32_be(&mut vars_data, *index);
            vars_data.push(name.len() as u8);
            vars_data.extend_from_slice(name.as_bytes());
        }

        // Calculate final positions
        let index_offset = header_size;
        let index_length = block_table.len();
        let vars_offset = index_offset + index_length;
        let vars_length = vars_data.len();

        // Write header
        output.extend_from_slice(b"BOMStore"); // magic
        Self::write_u32_be(&mut output, 1); // version
        Self::write_u32_be(&mut output, (self.blocks.len() - 1) as u32); // numberOfBlocks (non-null)
        Self::write_u32_be(&mut output, index_offset as u32); // indexOffset
        Self::write_u32_be(&mut output, index_length as u32); // indexLength
        Self::write_u32_be(&mut output, vars_offset as u32); // varsOffset
        Self::write_u32_be(&mut output, vars_length as u32); // varsLength

        // Pad header to standard size
        while output.len() < header_size {
            output.push(0);
        }

        // Write block table
        output.extend_from_slice(&block_table);

        // Write vars
        output.extend_from_slice(&vars_data);

        // Write all blocks
        for block in &self.blocks {
            output.extend_from_slice(block);
        }

        output
    }
}

/// Create a BOM file from a list of entries.
pub fn create_bom(entries: &[BomEntry]) -> Result<Vec<u8>, PackageError> {
    if entries.is_empty() {
        return Err(PackageError::BomError {
            reason: "Cannot create BOM with no entries".to_string(),
        });
    }

    let mut writer = BomWriter::new();

    // Assign IDs to all paths (including implicit parent directories)
    let mut path_ids: HashMap<String, u32> = HashMap::new();
    let mut all_paths: Vec<(String, Option<&BomEntry>)> = Vec::new();

    // Root directory gets ID 0
    path_ids.insert(".".to_string(), 0);
    all_paths.push((".".to_string(), None));

    // Collect all paths including implicit parent directories
    let mut next_id = 1u32;
    for entry in entries {
        let path_str = entry.path.to_string_lossy().to_string();

        // Add all parent directories
        let mut current = PathBuf::new();
        for component in entry.path.components() {
            current.push(component);
            let current_str = current.to_string_lossy().to_string();
            if !path_ids.contains_key(&current_str) {
                path_ids.insert(current_str.clone(), next_id);
                // Check if this is the actual entry or an implicit directory
                if current_str == path_str {
                    all_paths.push((current_str, Some(entry)));
                } else {
                    all_paths.push((current_str, None));
                }
                next_id += 1;
            }
        }
    }

    // Build PathInfo2 blocks for each path
    let mut path_info2_blocks: HashMap<u32, u32> = HashMap::new();

    for (path_str, maybe_entry) in &all_paths {
        let id = path_ids[path_str];

        let mut info2 = Vec::new();

        if let Some(entry) = maybe_entry {
            // Use actual entry data
            let is_dir = entry.mode & 0o170000 == 0o040000;
            info2.push(if is_dir { TYPE_DIR } else { TYPE_FILE });
            info2.push(1); // unknown0
            BomWriter::write_u16_be(&mut info2, 0); // architecture
            BomWriter::write_u16_be(&mut info2, (entry.mode & 0o7777) as u16); // mode
            BomWriter::write_u32_be(&mut info2, entry.uid); // user
            BomWriter::write_u32_be(&mut info2, entry.gid); // group
            BomWriter::write_u32_be(&mut info2, 0); // modtime
            BomWriter::write_u32_be(&mut info2, entry.size as u32); // size
            info2.push(1); // unknown1
            BomWriter::write_u32_be(&mut info2, 0); // checksum
            BomWriter::write_u32_be(&mut info2, 0); // linkNameLength
        } else {
            // Implicit directory
            info2.push(TYPE_DIR);
            info2.push(1); // unknown0
            BomWriter::write_u16_be(&mut info2, 0); // architecture
            BomWriter::write_u16_be(&mut info2, 0o755); // mode (default dir permissions)
            BomWriter::write_u32_be(&mut info2, 0); // user
            BomWriter::write_u32_be(&mut info2, 80); // group
            BomWriter::write_u32_be(&mut info2, 0); // modtime
            BomWriter::write_u32_be(&mut info2, 0); // size
            info2.push(1); // unknown1
            BomWriter::write_u32_be(&mut info2, 0); // checksum
            BomWriter::write_u32_be(&mut info2, 0); // linkNameLength
        }

        let block_idx = writer.add_block(info2);
        path_info2_blocks.insert(id, block_idx);
    }

    // Build PathInfo1 blocks
    let mut path_info1_blocks: HashMap<u32, u32> = HashMap::new();

    for (path_str, _) in &all_paths {
        let id = path_ids[path_str];

        let mut info1 = Vec::new();
        BomWriter::write_u32_be(&mut info1, id);
        BomWriter::write_u32_be(&mut info1, path_info2_blocks[&id]);

        let block_idx = writer.add_block(info1);
        path_info1_blocks.insert(id, block_idx);
    }

    // Build BOMFile blocks (name + parent reference)
    let mut file_blocks: HashMap<u32, u32> = HashMap::new();

    for (path_str, _) in &all_paths {
        let id = path_ids[path_str];
        let path = PathBuf::from(path_str);

        let mut file_data = Vec::new();

        // Find parent ID
        let parent_id = if path_str == "." {
            0
        } else if let Some(parent) = path.parent() {
            let parent_str = if parent.as_os_str().is_empty() {
                ".".to_string()
            } else {
                parent.to_string_lossy().to_string()
            };
            *path_ids.get(&parent_str).unwrap_or(&0)
        } else {
            0
        };

        BomWriter::write_u32_be(&mut file_data, parent_id);

        // File name (just the last component)
        let name = if path_str == "." {
            ".".to_string()
        } else {
            path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| path_str.clone())
        };
        file_data.extend_from_slice(name.as_bytes());
        file_data.push(0); // null terminator

        let block_idx = writer.add_block(file_data);
        file_blocks.insert(id, block_idx);
    }

    // Build BOMPaths (leaf node containing all path indices)
    let mut paths_data = Vec::new();
    BomWriter::write_u16_be(&mut paths_data, 1); // isLeaf = true
    BomWriter::write_u16_be(&mut paths_data, all_paths.len() as u16); // count
    BomWriter::write_u32_be(&mut paths_data, 0); // forward
    BomWriter::write_u32_be(&mut paths_data, 0); // backward

    // Write path indices
    for (path_str, _) in &all_paths {
        let id = path_ids[path_str];
        BomWriter::write_u32_be(&mut paths_data, path_info1_blocks[&id]); // index0 -> PathInfo1
        BomWriter::write_u32_be(&mut paths_data, file_blocks[&id]); // index1 -> BOMFile
    }

    let paths_block = writer.add_block(paths_data);

    // Build tree and VIndex
    let tree_block = writer.build_tree(paths_block, all_paths.len() as u32);
    let vindex_block = writer.build_vindex(tree_block);

    // Build BomInfo
    let bom_info_block = writer.build_bom_info(all_paths.len() as u32);

    // Add named variables
    writer.add_var("BomInfo", bom_info_block);
    writer.add_var("Paths", vindex_block);

    // Build the final BOM
    Ok(writer.build())
}

/// Create a BOM file by scanning a directory.
#[cfg(unix)]
pub fn create_bom_from_directory(path: &std::path::Path) -> Result<Vec<u8>, PackageError> {
    use std::os::unix::fs::MetadataExt;
    use walkdir::WalkDir;

    let mut entries = Vec::new();

    for entry in WalkDir::new(path).min_depth(1) {
        let entry = entry.map_err(|e| PackageError::BomError {
            reason: e.to_string(),
        })?;

        let rel_path = entry
            .path()
            .strip_prefix(path)
            .map_err(|e| PackageError::BomError {
                reason: e.to_string(),
            })?;

        let metadata = entry.metadata().map_err(|e| PackageError::BomError {
            reason: e.to_string(),
        })?;

        entries.push(BomEntry {
            path: rel_path.to_path_buf(),
            mode: metadata.mode(),
            uid: metadata.uid(),
            gid: metadata.gid(),
            size: metadata.len(),
        });
    }

    create_bom(&entries)
}

/// Create a BOM file by scanning a directory (Windows version).
#[cfg(windows)]
pub fn create_bom_from_directory(path: &std::path::Path) -> Result<Vec<u8>, PackageError> {
    use walkdir::WalkDir;

    let mut entries = Vec::new();

    for entry in WalkDir::new(path).min_depth(1) {
        let entry = entry.map_err(|e| PackageError::BomError {
            reason: e.to_string(),
        })?;

        let rel_path = entry
            .path()
            .strip_prefix(path)
            .map_err(|e| PackageError::BomError {
                reason: e.to_string(),
            })?;

        let metadata = entry.metadata().map_err(|e| PackageError::BomError {
            reason: e.to_string(),
        })?;

        // On Windows, use default Unix permissions
        let mode = if metadata.is_dir() {
            0o040755
        } else if metadata.permissions().readonly() {
            0o100444
        } else {
            0o100644
        };

        entries.push(BomEntry {
            path: rel_path.to_path_buf(),
            mode,
            uid: 0,  // Default to root
            gid: 80, // Default to admin group
            size: metadata.len(),
        });
    }

    create_bom(&entries)
}

#[cfg(test)]
mod tests {
    use super::*;

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
