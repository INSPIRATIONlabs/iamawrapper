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
    /// Named variables (e.g., "BomInfo", "Paths", etc.) - order preserved
    vars: Vec<(String, u32)>,
}

impl BomWriter {
    fn new() -> Self {
        // Block 0 is always null/empty
        Self {
            blocks: vec![vec![]],
            vars: Vec::new(),
        }
    }

    /// Add a data block and return its index
    fn add_block(&mut self, data: Vec<u8>) -> u32 {
        let index = self.blocks.len() as u32;
        self.blocks.push(data);
        index
    }

    /// Add a named variable pointing to a block (order preserved)
    fn add_var(&mut self, name: &str, block_index: u32) {
        self.vars.push((name.to_string(), block_index));
    }

    /// Write 32-bit big-endian value
    fn write_u32_be(buf: &mut Vec<u8>, val: u32) {
        buf.extend_from_slice(&val.to_be_bytes());
    }

    /// Write 16-bit big-endian value
    fn write_u16_be(buf: &mut Vec<u8>, val: u16) {
        buf.extend_from_slice(&val.to_be_bytes());
    }

    /// Build a tree structure for paths
    fn build_tree(&mut self, child_block: u32, path_count: u32) -> u32 {
        let mut data = Vec::new();
        // "tree" magic
        data.extend_from_slice(b"tree");
        // version = 1
        Self::write_u32_be(&mut data, 1);
        // child (points to BOMPaths leaf node)
        Self::write_u32_be(&mut data, child_block);
        // blockSize = 4096
        Self::write_u32_be(&mut data, 4096);
        // pathCount
        Self::write_u32_be(&mut data, path_count);
        // unknown3
        data.push(0);

        self.add_block(data)
    }

    /// Build an empty tree (for HLIndex, Size64)
    fn build_empty_tree(&mut self) -> u32 {
        // Empty leaf node - just the header, no padding needed for empty trees
        let mut leaf_data = Vec::new();
        BomWriter::write_u16_be(&mut leaf_data, 1); // isLeaf = true
        BomWriter::write_u16_be(&mut leaf_data, 0); // count = 0
        BomWriter::write_u32_be(&mut leaf_data, 0); // forward = 0
        BomWriter::write_u32_be(&mut leaf_data, 0); // backward = 0
        let empty_leaf = self.add_block(leaf_data);

        // Tree pointing to empty leaf
        self.build_tree(empty_leaf, 0)
    }

    /// Build VIndex structure (special format with 13-byte header pointing to tree)
    fn build_vindex(&mut self) -> u32 {
        // Empty leaf node for VIndex tree
        let mut leaf_data = Vec::new();
        BomWriter::write_u16_be(&mut leaf_data, 1); // isLeaf = true
        BomWriter::write_u16_be(&mut leaf_data, 0); // count = 0
        BomWriter::write_u32_be(&mut leaf_data, 0); // forward = 0
        BomWriter::write_u32_be(&mut leaf_data, 0); // backward = 0
        let empty_leaf = self.add_block(leaf_data);

        // VIndex tree with blockSize=128 (different from other trees!)
        let mut tree_data = Vec::new();
        tree_data.extend_from_slice(b"tree");
        Self::write_u32_be(&mut tree_data, 1); // version
        Self::write_u32_be(&mut tree_data, empty_leaf); // child
        Self::write_u32_be(&mut tree_data, 128); // blockSize = 128 (special for VIndex)
        Self::write_u32_be(&mut tree_data, 0); // pathCount = 0
        tree_data.push(0); // unknown3
        let tree_block = self.add_block(tree_data);

        // VIndex header structure (13 bytes)
        let mut vindex_data = Vec::new();
        Self::write_u32_be(&mut vindex_data, 1); // unknown0 = 1
        Self::write_u32_be(&mut vindex_data, tree_block); // indexToVTree
        Self::write_u32_be(&mut vindex_data, 0); // unknown2 = 0
        vindex_data.push(0); // unknown3 = 0
        self.add_block(vindex_data)
    }

    /// Build the final BOM file
    fn build(self) -> Vec<u8> {
        let mut output = Vec::new();

        // Layout: [header 512] [vars] [blocks...] [block_table]
        let header_size = 512;

        // Build vars table (order is preserved from insertion)
        let mut vars_data = Vec::new();
        Self::write_u32_be(&mut vars_data, self.vars.len() as u32);
        for (name, index) in &self.vars {
            Self::write_u32_be(&mut vars_data, *index);
            vars_data.push(name.len() as u8);
            vars_data.extend_from_slice(name.as_bytes());
        }

        // Calculate total block data size (skip block 0 which is null)
        let total_blocks_size: usize = self.blocks.iter().skip(1).map(|b| b.len()).sum();

        // Calculate offsets - vars right after header, then blocks
        let vars_offset = header_size;
        let blocks_start = vars_offset + vars_data.len();
        let index_offset = blocks_start + total_blocks_size;

        // Build block table - addresses point to blocks area
        // Format: [numberOfBlockTablePointers: u32][entries as address,length pairs...]
        let mut block_table = Vec::new();
        Self::write_u32_be(&mut block_table, self.blocks.len() as u32); // numberOfBlockTablePointers

        let mut current_offset = blocks_start as u32;
        for (i, block) in self.blocks.iter().enumerate() {
            if i == 0 {
                // Block 0 is the null block - always offset 0, size 0
                Self::write_u32_be(&mut block_table, 0); // address
                Self::write_u32_be(&mut block_table, 0); // length
            } else {
                // Block table entry format: [address][length] per entry
                Self::write_u32_be(&mut block_table, current_offset); // address
                Self::write_u32_be(&mut block_table, block.len() as u32); // length
                current_offset += block.len() as u32;
            }
        }

        // Free list - bomutils uses 0 entries for new BOMs
        // Format: [numberOfFreeListPointers: u32]
        Self::write_u32_be(&mut block_table, 0); // numberOfFreeListPointers = 0

        // Write header
        output.extend_from_slice(b"BOMStore"); // magic
        Self::write_u32_be(&mut output, 1); // version
        Self::write_u32_be(&mut output, (self.blocks.len() - 1) as u32); // numberOfBlocks (non-null)
        Self::write_u32_be(&mut output, index_offset as u32); // indexOffset
        Self::write_u32_be(&mut output, block_table.len() as u32); // indexLength
        Self::write_u32_be(&mut output, vars_offset as u32); // varsOffset
        Self::write_u32_be(&mut output, vars_data.len() as u32); // varsLength

        // Pad header to standard size
        while output.len() < header_size {
            output.push(0);
        }

        // Write vars table (right after header)
        output.extend_from_slice(&vars_data);

        // Write all blocks after vars, skip block 0 which is null
        for block in self.blocks.iter().skip(1) {
            output.extend_from_slice(block);
        }

        // Write block table (index) at the end
        output.extend_from_slice(&block_table);

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

    // Reserve block 1 for BomInfo (will fill in later) - 28 bytes (matches bomutils)
    let bom_info_placeholder = writer.add_block(vec![0u8; 28]);

    // Assign IDs to all paths (including implicit parent directories)
    // ID 0 is reserved for "no parent" in BOMFile, so IDs start from 1
    let mut path_ids: HashMap<String, u32> = HashMap::new();
    let mut all_paths: Vec<(String, Option<&BomEntry>)> = Vec::new();

    // Root directory "." gets ID 1 (not 0, since 0 means "no parent")
    path_ids.insert(".".to_string(), 1);
    all_paths.push((".".to_string(), None));

    // Collect all paths including implicit parent directories
    let mut next_id = 2u32;
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
            BomWriter::write_u16_be(&mut info2, 3); // architecture (always 3)
            // Mode includes file type bits (0o40755 for dir, 0o100644 for file)
            BomWriter::write_u16_be(&mut info2, (entry.mode & 0xFFFF) as u16);
            BomWriter::write_u32_be(&mut info2, entry.uid); // user
            BomWriter::write_u32_be(&mut info2, entry.gid); // group
            BomWriter::write_u32_be(&mut info2, 0); // modtime
            BomWriter::write_u32_be(&mut info2, entry.size as u32); // size
            info2.push(1); // unknown1 (u8, always 1)
            BomWriter::write_u32_be(&mut info2, 0); // checksum
            BomWriter::write_u32_be(&mut info2, 0); // linkNameLength
        } else {
            // Implicit directory - use full mode with file type (0o40755)
            info2.push(TYPE_DIR);
            info2.push(1); // unknown0
            BomWriter::write_u16_be(&mut info2, 3); // architecture (always 3)
            BomWriter::write_u16_be(&mut info2, 0o40755); // mode with directory type
            BomWriter::write_u32_be(&mut info2, 0); // user
            BomWriter::write_u32_be(&mut info2, 80); // group
            BomWriter::write_u32_be(&mut info2, 0); // modtime
            BomWriter::write_u32_be(&mut info2, 0); // size
            info2.push(1); // unknown1 (u8, always 1)
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

        // Find parent ID (0 = no parent, 1 = root ".")
        let parent_id = if path_str == "." {
            0 // Root has no parent
        } else if let Some(parent) = path.parent() {
            if parent.as_os_str().is_empty() {
                1 // Top-level entry, parent is root "." which has ID 1
            } else {
                let parent_str = parent.to_string_lossy().to_string();
                *path_ids.get(&parent_str).unwrap_or(&1)
            }
        } else {
            1 // Default to root
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

    let paths_leaf_block = writer.add_block(paths_data);

    // Build tree structure pointing to the leaf
    let paths_tree_block = writer.build_tree(paths_leaf_block, all_paths.len() as u32);

    // Build empty trees for HLIndex, VIndex, and Size64
    let hl_index_block = writer.build_empty_tree();
    let v_index_block = writer.build_vindex();
    let size64_block = writer.build_empty_tree();

    // Now fill in the BomInfo placeholder (block 1) with real data
    // BomInfo is 28 bytes: version, numberOfPaths, numberOfInfoEntries + 16 bytes padding
    let mut bom_info_data = Vec::new();
    BomWriter::write_u32_be(&mut bom_info_data, 1); // version
    BomWriter::write_u32_be(&mut bom_info_data, all_paths.len() as u32); // numberOfPaths
    BomWriter::write_u32_be(&mut bom_info_data, 1); // numberOfInfoEntries
    // Padding to match bomutils format (16 more bytes of zeros)
    bom_info_data.resize(28, 0);
    writer.blocks[bom_info_placeholder as usize] = bom_info_data;

    // Add named variables (matching Apple's mkbom order: BomInfo, Paths, HLIndex, VIndex, Size64)
    writer.add_var("BomInfo", bom_info_placeholder);
    writer.add_var("Paths", paths_tree_block);
    writer.add_var("HLIndex", hl_index_block);
    writer.add_var("VIndex", v_index_block);
    writer.add_var("Size64", size64_block);

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

    // ==================== Helper functions ====================

    /// Read a big-endian u32 from a byte slice at given offset
    fn read_u32_be(data: &[u8], offset: usize) -> u32 {
        u32::from_be_bytes([
            data[offset],
            data[offset + 1],
            data[offset + 2],
            data[offset + 3],
        ])
    }

    /// Find a null-terminated string in BOM data
    fn find_string_in_bom(data: &[u8], needle: &str) -> bool {
        let needle_bytes = needle.as_bytes();
        data.windows(needle_bytes.len() + 1)
            .any(|w| &w[..needle_bytes.len()] == needle_bytes && w[needle_bytes.len()] == 0)
    }

    // ==================== Error handling tests ====================

    #[test]
    fn test_bom_empty_entries_error() {
        let result = create_bom(&[]);
        assert!(result.is_err(), "Empty entries should return error");
        match result {
            Err(PackageError::BomError { reason }) => {
                assert!(
                    reason.contains("no entries"),
                    "Error should mention no entries"
                );
            }
            _ => panic!("Expected BomError"),
        }
    }

    // ==================== Header structure tests ====================

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
    fn test_bom_header_version() {
        let bom_data = create_bom(&[BomEntry {
            path: PathBuf::from("test.txt"),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 5,
        }])
        .unwrap();

        let version = read_u32_be(&bom_data, 8);
        assert_eq!(version, 1, "BOM version should be 1");
    }

    #[test]
    fn test_bom_header_offsets_valid() {
        let bom_data = create_bom(&[BomEntry {
            path: PathBuf::from("test.txt"),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 5,
        }])
        .unwrap();

        let num_blocks = read_u32_be(&bom_data, 12);
        let index_offset = read_u32_be(&bom_data, 16);
        let index_length = read_u32_be(&bom_data, 20);
        let vars_offset = read_u32_be(&bom_data, 24);
        let vars_length = read_u32_be(&bom_data, 28);

        assert!(num_blocks > 0, "Should have at least one block");
        assert!(index_offset > 0, "Index offset should be non-zero");
        assert!(index_length > 0, "Index length should be non-zero");
        assert_eq!(vars_offset, 512, "Vars should start right after header");
        assert!(vars_length > 0, "Vars length should be non-zero");

        // Validate offsets are within bounds
        assert!(
            (index_offset + index_length) as usize <= bom_data.len(),
            "Index should be within file bounds"
        );
        assert!(
            (vars_offset + vars_length) as usize <= bom_data.len(),
            "Vars should be within file bounds"
        );
    }

    #[test]
    fn test_bom_header_padding() {
        let bom_data = create_bom(&[BomEntry {
            path: PathBuf::from("test.txt"),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 5,
        }])
        .unwrap();

        // Header should be 512 bytes, with padding after the fields
        assert!(
            bom_data.len() >= 512,
            "BOM should be at least 512 bytes (header size)"
        );

        // Check that area after header fields (offset 32-512) is zero-padded
        for (i, &byte) in bom_data[32..512].iter().enumerate() {
            assert_eq!(byte, 0, "Header padding at offset {} should be 0", i + 32);
        }
    }

    // ==================== Variable table tests ====================

    #[test]
    fn test_bom_has_required_variables() {
        let bom_data = create_bom(&[BomEntry {
            path: PathBuf::from("test.txt"),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 5,
        }])
        .unwrap();

        let vars_offset = read_u32_be(&bom_data, 24) as usize;
        let vars_length = read_u32_be(&bom_data, 28) as usize;
        let vars_data = &bom_data[vars_offset..vars_offset + vars_length];

        let num_vars = read_u32_be(vars_data, 0);
        assert_eq!(num_vars, 5, "Should have exactly 5 variables");

        // Check that all required variable names are present
        let required_vars = ["BomInfo", "Paths", "HLIndex", "VIndex", "Size64"];
        for var_name in &required_vars {
            assert!(
                vars_data
                    .windows(var_name.len())
                    .any(|w| w == var_name.as_bytes()),
                "Variable '{}' should be present",
                var_name
            );
        }
    }

    #[test]
    fn test_bom_variable_order() {
        let bom_data = create_bom(&[BomEntry {
            path: PathBuf::from("test.txt"),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 5,
        }])
        .unwrap();

        let vars_offset = read_u32_be(&bom_data, 24) as usize;
        let vars_length = read_u32_be(&bom_data, 28) as usize;
        let vars_data = &bom_data[vars_offset..vars_offset + vars_length];

        // Parse variable names in order
        let mut offset = 4; // Skip count
        let mut var_names = Vec::new();
        for _ in 0..5 {
            offset += 4; // Skip block index
            let name_len = vars_data[offset] as usize;
            offset += 1;
            let name = String::from_utf8_lossy(&vars_data[offset..offset + name_len]).to_string();
            var_names.push(name);
            offset += name_len;
        }

        assert_eq!(
            var_names,
            vec!["BomInfo", "Paths", "HLIndex", "VIndex", "Size64"],
            "Variables should be in correct order"
        );
    }

    // ==================== Tree structure tests ====================

    #[test]
    fn test_bom_tree_magic() {
        let bom_data = create_bom(&[BomEntry {
            path: PathBuf::from("test.txt"),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 5,
        }])
        .unwrap();

        // BOM should contain "tree" magic somewhere in the data
        let tree_count = bom_data.windows(4).filter(|w| *w == b"tree").count();

        // Should have at least 4 trees: Paths, HLIndex, VIndex (inside), Size64
        assert!(
            tree_count >= 4,
            "BOM should contain at least 4 tree structures, found {}",
            tree_count
        );
    }

    // ==================== Path handling tests ====================

    #[test]
    fn test_bom_single_file() {
        let bom_data = create_bom(&[BomEntry {
            path: PathBuf::from("test.txt"),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 5,
        }])
        .unwrap();

        assert!(
            find_string_in_bom(&bom_data, "test.txt"),
            "BOM should contain filename"
        );
        assert!(
            find_string_in_bom(&bom_data, "."),
            "BOM should contain root directory"
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
            find_string_in_bom(&bom_data, "file1.txt"),
            "BOM should contain file1.txt"
        );
        assert!(
            find_string_in_bom(&bom_data, "file2.txt"),
            "BOM should contain file2.txt"
        );
    }

    #[test]
    fn test_bom_nested_directory_structure() {
        let bom_data = create_bom(&[
            BomEntry {
                path: PathBuf::from("Contents"),
                mode: 0o040755,
                uid: 0,
                gid: 80,
                size: 0,
            },
            BomEntry {
                path: PathBuf::from("Contents/MacOS"),
                mode: 0o040755,
                uid: 0,
                gid: 80,
                size: 0,
            },
            BomEntry {
                path: PathBuf::from("Contents/MacOS/myapp"),
                mode: 0o100755,
                uid: 0,
                gid: 80,
                size: 1000,
            },
        ])
        .unwrap();

        assert!(
            find_string_in_bom(&bom_data, "Contents"),
            "BOM should contain Contents"
        );
        assert!(
            find_string_in_bom(&bom_data, "MacOS"),
            "BOM should contain MacOS"
        );
        assert!(
            find_string_in_bom(&bom_data, "myapp"),
            "BOM should contain myapp"
        );
    }

    #[test]
    fn test_bom_implicit_parent_directories() {
        // Only provide the leaf file - parent directories should be created implicitly
        let bom_data = create_bom(&[BomEntry {
            path: PathBuf::from("a/b/c/file.txt"),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 10,
        }])
        .unwrap();

        // All parent directories should be implicitly created
        assert!(find_string_in_bom(&bom_data, "a"), "Should contain dir 'a'");
        assert!(find_string_in_bom(&bom_data, "b"), "Should contain dir 'b'");
        assert!(find_string_in_bom(&bom_data, "c"), "Should contain dir 'c'");
        assert!(
            find_string_in_bom(&bom_data, "file.txt"),
            "Should contain file"
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

        // Verify all path components are present
        assert!(find_string_in_bom(&bom_data, "Applications"));
        assert!(find_string_in_bom(&bom_data, "MyApp.app"));
        assert!(find_string_in_bom(&bom_data, "Contents"));
        assert!(find_string_in_bom(&bom_data, "MacOS"));
        assert!(find_string_in_bom(&bom_data, "myapp"));
    }

    #[test]
    fn test_bom_multiple_files_same_directory() {
        let bom_data = create_bom(&[
            BomEntry {
                path: PathBuf::from("dir/file1.txt"),
                mode: 0o100644,
                uid: 0,
                gid: 80,
                size: 10,
            },
            BomEntry {
                path: PathBuf::from("dir/file2.txt"),
                mode: 0o100644,
                uid: 0,
                gid: 80,
                size: 20,
            },
            BomEntry {
                path: PathBuf::from("dir/file3.txt"),
                mode: 0o100644,
                uid: 0,
                gid: 80,
                size: 30,
            },
        ])
        .unwrap();

        assert!(find_string_in_bom(&bom_data, "dir"));
        assert!(find_string_in_bom(&bom_data, "file1.txt"));
        assert!(find_string_in_bom(&bom_data, "file2.txt"));
        assert!(find_string_in_bom(&bom_data, "file3.txt"));
    }

    #[test]
    fn test_bom_sibling_directories() {
        let bom_data = create_bom(&[
            BomEntry {
                path: PathBuf::from("dir1/file.txt"),
                mode: 0o100644,
                uid: 0,
                gid: 80,
                size: 10,
            },
            BomEntry {
                path: PathBuf::from("dir2/file.txt"),
                mode: 0o100644,
                uid: 0,
                gid: 80,
                size: 20,
            },
        ])
        .unwrap();

        assert!(find_string_in_bom(&bom_data, "dir1"));
        assert!(find_string_in_bom(&bom_data, "dir2"));
    }

    // ==================== File type and mode tests ====================

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
    fn test_bom_file_type_detection() {
        // Test that file type is correctly determined from mode
        let bom_data = create_bom(&[
            BomEntry {
                path: PathBuf::from("regular_file"),
                mode: 0o100644, // Regular file
                uid: 0,
                gid: 80,
                size: 100,
            },
            BomEntry {
                path: PathBuf::from("directory"),
                mode: 0o040755, // Directory
                uid: 0,
                gid: 80,
                size: 0,
            },
        ])
        .unwrap();

        // Both should be in the BOM
        assert!(find_string_in_bom(&bom_data, "regular_file"));
        assert!(find_string_in_bom(&bom_data, "directory"));
    }

    #[test]
    fn test_bom_custom_uid_gid() {
        let entries = vec![BomEntry {
            path: PathBuf::from("file.txt"),
            mode: 0o100644,
            uid: 1000,
            gid: 1000,
            size: 5,
        }];
        let result = create_bom(&entries);
        assert!(result.is_ok(), "BOM should accept custom uid/gid values");
    }

    // ==================== Edge cases ====================

    #[test]
    fn test_bom_long_filename() {
        let long_name = "a".repeat(200) + ".txt";
        let bom_data = create_bom(&[BomEntry {
            path: PathBuf::from(&long_name),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 5,
        }])
        .unwrap();

        assert!(bom_data.len() > 200, "BOM should handle long filenames");
    }

    #[test]
    fn test_bom_special_characters_in_filename() {
        let result = create_bom(&[BomEntry {
            path: PathBuf::from("file with spaces.txt"),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 5,
        }]);
        assert!(result.is_ok(), "BOM should handle spaces in filenames");

        let bom_data = result.unwrap();
        assert!(find_string_in_bom(&bom_data, "file with spaces.txt"));
    }

    #[test]
    fn test_bom_unicode_filename() {
        let result = create_bom(&[BomEntry {
            path: PathBuf::from("файл.txt"), // Russian "file"
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 5,
        }]);
        assert!(result.is_ok(), "BOM should handle unicode filenames");
    }

    #[test]
    fn test_bom_many_files() {
        let entries: Vec<BomEntry> = (0..100)
            .map(|i| BomEntry {
                path: PathBuf::from(format!("file{}.txt", i)),
                mode: 0o100644,
                uid: 0,
                gid: 80,
                size: i as u64,
            })
            .collect();

        let result = create_bom(&entries);
        assert!(result.is_ok(), "BOM should handle many files");

        let bom_data = result.unwrap();
        // Verify some files are present
        assert!(find_string_in_bom(&bom_data, "file0.txt"));
        assert!(find_string_in_bom(&bom_data, "file50.txt"));
        assert!(find_string_in_bom(&bom_data, "file99.txt"));
    }

    #[test]
    fn test_bom_large_file_size() {
        let result = create_bom(&[BomEntry {
            path: PathBuf::from("large_file.bin"),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 4_294_967_295, // Max u32
        }]);
        assert!(result.is_ok(), "BOM should handle large file sizes");
    }

    #[test]
    fn test_bom_zero_size_file() {
        let result = create_bom(&[BomEntry {
            path: PathBuf::from("empty.txt"),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 0,
        }]);
        assert!(result.is_ok(), "BOM should handle zero-size files");
    }

    #[test]
    fn test_bom_executable_permissions() {
        let result = create_bom(&[
            BomEntry {
                path: PathBuf::from("script.sh"),
                mode: 0o100755,
                uid: 0,
                gid: 80,
                size: 100,
            },
            BomEntry {
                path: PathBuf::from("binary"),
                mode: 0o100755,
                uid: 0,
                gid: 80,
                size: 50000,
            },
        ]);
        assert!(result.is_ok(), "BOM should handle executable files");
    }

    // ==================== BomInfo structure tests ====================

    #[test]
    fn test_bom_info_size() {
        let bom_data = create_bom(&[BomEntry {
            path: PathBuf::from("test.txt"),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 5,
        }])
        .unwrap();

        // BomInfo should be 28 bytes
        let vars_offset = read_u32_be(&bom_data, 24) as usize;
        let vars_data = &bom_data[vars_offset..];

        // First variable is BomInfo - get its block index
        let bom_info_block = read_u32_be(vars_data, 4);

        // Get block table
        let index_offset = read_u32_be(&bom_data, 16) as usize;
        let block_table = &bom_data[index_offset..];

        // Get BomInfo block length (skip null block at index 0)
        let bom_info_length = read_u32_be(block_table, 4 + (bom_info_block as usize * 8) + 4);
        assert_eq!(bom_info_length, 28, "BomInfo should be 28 bytes");
    }

    #[test]
    fn test_bom_info_path_count() {
        // Create BOM with 3 entries (plus implicit root = 4 total paths)
        let bom_data = create_bom(&[
            BomEntry {
                path: PathBuf::from("file1.txt"),
                mode: 0o100644,
                uid: 0,
                gid: 80,
                size: 10,
            },
            BomEntry {
                path: PathBuf::from("file2.txt"),
                mode: 0o100644,
                uid: 0,
                gid: 80,
                size: 20,
            },
            BomEntry {
                path: PathBuf::from("file3.txt"),
                mode: 0o100644,
                uid: 0,
                gid: 80,
                size: 30,
            },
        ])
        .unwrap();

        // Find BomInfo block and verify path count
        let vars_offset = read_u32_be(&bom_data, 24) as usize;
        let vars_data = &bom_data[vars_offset..];
        let bom_info_block = read_u32_be(vars_data, 4);

        let index_offset = read_u32_be(&bom_data, 16) as usize;
        let block_table = &bom_data[index_offset..];

        let bom_info_addr = read_u32_be(block_table, 4 + (bom_info_block as usize * 8)) as usize;
        let bom_info_data = &bom_data[bom_info_addr..];

        let path_count = read_u32_be(bom_info_data, 4);
        // 3 files + 1 root = 4 paths
        assert_eq!(path_count, 4, "BomInfo should report correct path count");
    }

    // ==================== Block table tests ====================

    #[test]
    fn test_bom_block_table_structure() {
        let bom_data = create_bom(&[BomEntry {
            path: PathBuf::from("test.txt"),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 5,
        }])
        .unwrap();

        let index_offset = read_u32_be(&bom_data, 16) as usize;
        let block_table = &bom_data[index_offset..];

        let num_blocks = read_u32_be(block_table, 0);
        assert!(num_blocks > 0, "Should have at least one block in table");

        // First block (null block) should have offset 0, length 0
        let block0_addr = read_u32_be(block_table, 4);
        let block0_len = read_u32_be(block_table, 8);
        assert_eq!(block0_addr, 0, "Null block address should be 0");
        assert_eq!(block0_len, 0, "Null block length should be 0");
    }

    #[test]
    fn test_bom_free_list_empty() {
        let bom_data = create_bom(&[BomEntry {
            path: PathBuf::from("test.txt"),
            mode: 0o100644,
            uid: 0,
            gid: 80,
            size: 5,
        }])
        .unwrap();

        let index_offset = read_u32_be(&bom_data, 16) as usize;
        let index_length = read_u32_be(&bom_data, 20) as usize;
        let block_table = &bom_data[index_offset..index_offset + index_length];

        let num_blocks = read_u32_be(block_table, 0) as usize;
        // Free list count is after block entries: 4 + (num_blocks * 8)
        let free_list_offset = 4 + (num_blocks * 8);
        let free_list_count = read_u32_be(block_table, free_list_offset);

        assert_eq!(free_list_count, 0, "New BOMs should have empty free list");
    }

    // ==================== Directory scanning tests ====================

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

    #[test]
    fn test_create_bom_from_directory_nested() {
        use std::fs::{self, File};
        use std::io::Write;
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();

        // Create nested structure
        let nested_dir = temp_dir.path().join("subdir");
        fs::create_dir(&nested_dir).unwrap();

        let file1 = temp_dir.path().join("root.txt");
        let mut f1 = File::create(&file1).unwrap();
        f1.write_all(b"root").unwrap();

        let file2 = nested_dir.join("nested.txt");
        let mut f2 = File::create(&file2).unwrap();
        f2.write_all(b"nested").unwrap();

        let result = create_bom_from_directory(temp_dir.path());
        assert!(result.is_ok(), "Should create BOM from nested directory");

        let bom_data = result.unwrap();
        assert!(find_string_in_bom(&bom_data, "root.txt"));
        assert!(find_string_in_bom(&bom_data, "subdir"));
        assert!(find_string_in_bom(&bom_data, "nested.txt"));
    }

    #[test]
    fn test_create_bom_from_empty_directory() {
        use tempfile::TempDir;

        let temp_dir = TempDir::new().unwrap();
        // Empty directory - no files

        let result = create_bom_from_directory(temp_dir.path());
        // Should fail because no entries
        assert!(result.is_err(), "Empty directory should fail");
    }

    // ==================== Determinism test ====================

    #[test]
    fn test_bom_deterministic_output() {
        let entries = vec![
            BomEntry {
                path: PathBuf::from("file1.txt"),
                mode: 0o100644,
                uid: 0,
                gid: 80,
                size: 10,
            },
            BomEntry {
                path: PathBuf::from("file2.txt"),
                mode: 0o100644,
                uid: 0,
                gid: 80,
                size: 20,
            },
        ];

        let bom1 = create_bom(&entries).unwrap();
        let bom2 = create_bom(&entries).unwrap();

        assert_eq!(
            bom1, bom2,
            "Same inputs should produce identical BOM output"
        );
    }

    // ==================== Property-based fuzz tests ====================

    mod fuzz {
        use super::*;
        use proptest::prelude::*;

        /// Strategy to generate valid file modes
        fn file_mode_strategy() -> impl Strategy<Value = u32> {
            prop_oneof![
                Just(0o100644), // Regular file, rw-r--r--
                Just(0o100755), // Regular file, rwxr-xr-x
                Just(0o100444), // Regular file, r--r--r--
                Just(0o100600), // Regular file, rw-------
                Just(0o040755), // Directory, rwxr-xr-x
                Just(0o040700), // Directory, rwx------
            ]
        }

        /// Strategy to generate valid path components (no slashes, nulls, or empty)
        fn path_component_strategy() -> impl Strategy<Value = String> {
            "[a-zA-Z0-9_.-]{1,50}".prop_filter("no empty paths", |s| !s.is_empty())
        }

        /// Strategy to generate a valid file path (1-5 components deep)
        fn file_path_strategy() -> impl Strategy<Value = PathBuf> {
            prop::collection::vec(path_component_strategy(), 1..=5).prop_map(|components| {
                let path_str = components.join("/");
                PathBuf::from(path_str)
            })
        }

        /// Strategy to generate a single BomEntry
        fn bom_entry_strategy() -> impl Strategy<Value = BomEntry> {
            (
                file_path_strategy(),
                file_mode_strategy(),
                0u32..65535,     // uid
                0u32..65535,     // gid
                0u64..1_000_000, // size (reasonable range)
            )
                .prop_map(|(path, mode, uid, gid, size)| BomEntry {
                    path,
                    mode,
                    uid,
                    gid,
                    size,
                })
        }

        /// Strategy to generate multiple BomEntries with unique paths
        fn bom_entries_strategy(max_entries: usize) -> impl Strategy<Value = Vec<BomEntry>> {
            prop::collection::vec(bom_entry_strategy(), 1..=max_entries).prop_map(|entries| {
                // Deduplicate paths (keep first occurrence)
                let mut seen = std::collections::HashSet::new();
                entries
                    .into_iter()
                    .filter(|e| seen.insert(e.path.to_string_lossy().to_string()))
                    .collect()
            })
        }

        proptest! {
            #![proptest_config(ProptestConfig::with_cases(100))]

            /// Fuzz test: BOM creation should never panic with valid inputs
            #[test]
            fn fuzz_bom_creation_no_panic(entries in bom_entries_strategy(20)) {
                if !entries.is_empty() {
                    let _ = create_bom(&entries);
                }
            }

            /// Fuzz test: Valid entries should always produce valid BOM
            #[test]
            fn fuzz_bom_creation_success(entries in bom_entries_strategy(10)) {
                if !entries.is_empty() {
                    let result = create_bom(&entries);
                    prop_assert!(result.is_ok(), "BOM creation failed: {:?}", result.err());
                }
            }

            /// Fuzz test: BOM should always start with magic bytes
            #[test]
            fn fuzz_bom_has_magic(entries in bom_entries_strategy(10)) {
                if !entries.is_empty() {
                    let bom = create_bom(&entries).unwrap();
                    prop_assert_eq!(&bom[0..8], b"BOMStore", "Missing magic bytes");
                }
            }

            /// Fuzz test: BOM version should always be 1
            #[test]
            fn fuzz_bom_version(entries in bom_entries_strategy(10)) {
                if !entries.is_empty() {
                    let bom = create_bom(&entries).unwrap();
                    let version = read_u32_be(&bom, 8);
                    prop_assert_eq!(version, 1, "Wrong BOM version");
                }
            }

            /// Fuzz test: BOM offsets should be within bounds
            #[test]
            fn fuzz_bom_offsets_valid(entries in bom_entries_strategy(10)) {
                if !entries.is_empty() {
                    let bom = create_bom(&entries).unwrap();
                    let index_offset = read_u32_be(&bom, 16) as usize;
                    let index_length = read_u32_be(&bom, 20) as usize;
                    let vars_offset = read_u32_be(&bom, 24) as usize;
                    let vars_length = read_u32_be(&bom, 28) as usize;

                    prop_assert!(index_offset + index_length <= bom.len(),
                        "Index out of bounds: {} + {} > {}", index_offset, index_length, bom.len());
                    prop_assert!(vars_offset + vars_length <= bom.len(),
                        "Vars out of bounds: {} + {} > {}", vars_offset, vars_length, bom.len());
                }
            }

            /// Fuzz test: BOM should contain all 5 required variables
            #[test]
            fn fuzz_bom_has_all_vars(entries in bom_entries_strategy(10)) {
                if !entries.is_empty() {
                    let bom = create_bom(&entries).unwrap();
                    let vars_offset = read_u32_be(&bom, 24) as usize;
                    let vars_length = read_u32_be(&bom, 28) as usize;
                    let vars_data = &bom[vars_offset..vars_offset + vars_length];

                    let num_vars = read_u32_be(vars_data, 0);
                    prop_assert_eq!(num_vars, 5, "Should have 5 variables, got {}", num_vars);
                }
            }

            /// Fuzz test: BOM creation should be deterministic
            #[test]
            fn fuzz_bom_deterministic(entries in bom_entries_strategy(10)) {
                if !entries.is_empty() {
                    let bom1 = create_bom(&entries).unwrap();
                    let bom2 = create_bom(&entries).unwrap();
                    prop_assert_eq!(bom1, bom2, "BOM should be deterministic");
                }
            }

            /// Fuzz test: BOM header should be exactly 512 bytes
            #[test]
            fn fuzz_bom_header_size(entries in bom_entries_strategy(10)) {
                if !entries.is_empty() {
                    let bom = create_bom(&entries).unwrap();
                    let vars_offset = read_u32_be(&bom, 24) as usize;
                    prop_assert_eq!(vars_offset, 512, "Header should be 512 bytes");
                }
            }

            /// Fuzz test: Block table null entry should be zero
            #[test]
            fn fuzz_bom_null_block(entries in bom_entries_strategy(10)) {
                if !entries.is_empty() {
                    let bom = create_bom(&entries).unwrap();
                    let index_offset = read_u32_be(&bom, 16) as usize;
                    let block_table = &bom[index_offset..];

                    // First block entry (null block) should be 0,0
                    let null_addr = read_u32_be(block_table, 4);
                    let null_len = read_u32_be(block_table, 8);
                    prop_assert_eq!(null_addr, 0, "Null block addr should be 0");
                    prop_assert_eq!(null_len, 0, "Null block len should be 0");
                }
            }

            /// Fuzz test: Single entry with random valid data
            #[test]
            fn fuzz_single_entry(entry in bom_entry_strategy()) {
                let result = create_bom(&[entry]);
                prop_assert!(result.is_ok(), "Single entry BOM failed");
                let bom = result.unwrap();
                prop_assert!(bom.len() >= 512, "BOM too small");
            }

            /// Fuzz test: Random file sizes
            #[test]
            fn fuzz_file_sizes(size in 0u64..u64::MAX) {
                let entry = BomEntry {
                    path: PathBuf::from("test.bin"),
                    mode: 0o100644,
                    uid: 0,
                    gid: 80,
                    size,
                };
                let result = create_bom(&[entry]);
                prop_assert!(result.is_ok(), "BOM failed with size {}", size);
            }

            /// Fuzz test: Random UID/GID combinations
            #[test]
            fn fuzz_uid_gid(uid in 0u32..u32::MAX, gid in 0u32..u32::MAX) {
                let entry = BomEntry {
                    path: PathBuf::from("test.txt"),
                    mode: 0o100644,
                    uid,
                    gid,
                    size: 100,
                };
                let result = create_bom(&[entry]);
                prop_assert!(result.is_ok(), "BOM failed with uid={}, gid={}", uid, gid);
            }

            /// Fuzz test: Deep nested paths
            #[test]
            fn fuzz_deep_paths(depth in 1usize..20) {
                let path_str = (0..depth).map(|i| format!("dir{}", i)).collect::<Vec<_>>().join("/") + "/file.txt";
                let entry = BomEntry {
                    path: PathBuf::from(&path_str),
                    mode: 0o100644,
                    uid: 0,
                    gid: 80,
                    size: 10,
                };
                let result = create_bom(&[entry]);
                prop_assert!(result.is_ok(), "BOM failed with depth {}", depth);

                // Should have depth+2 paths (root + dirs + file)
                let bom = result.unwrap();
                let vars_offset = read_u32_be(&bom, 24) as usize;
                let vars_data = &bom[vars_offset..];
                let bom_info_block = read_u32_be(vars_data, 4);
                let index_offset = read_u32_be(&bom, 16) as usize;
                let block_table = &bom[index_offset..];
                let bom_info_addr = read_u32_be(block_table, 4 + (bom_info_block as usize * 8)) as usize;
                let path_count = read_u32_be(&bom[bom_info_addr..], 4) as usize;
                prop_assert_eq!(path_count, depth + 2, "Wrong path count for depth {}", depth);
            }

            /// Fuzz test: Many files in same directory
            #[test]
            fn fuzz_many_siblings(count in 1usize..100) {
                let entries: Vec<BomEntry> = (0..count)
                    .map(|i| BomEntry {
                        path: PathBuf::from(format!("file{}.txt", i)),
                        mode: 0o100644,
                        uid: 0,
                        gid: 80,
                        size: i as u64,
                    })
                    .collect();

                let result = create_bom(&entries);
                prop_assert!(result.is_ok(), "BOM failed with {} files", count);
            }

            /// Fuzz test: Mixed file types (files and directories)
            #[test]
            fn fuzz_mixed_types(
                num_dirs in 1usize..10,
                num_files in 1usize..10
            ) {
                let mut entries = Vec::new();

                // Add directories
                for i in 0..num_dirs {
                    entries.push(BomEntry {
                        path: PathBuf::from(format!("dir{}", i)),
                        mode: 0o040755,
                        uid: 0,
                        gid: 80,
                        size: 0,
                    });
                }

                // Add files in first directory
                for i in 0..num_files {
                    entries.push(BomEntry {
                        path: PathBuf::from(format!("dir0/file{}.txt", i)),
                        mode: 0o100644,
                        uid: 0,
                        gid: 80,
                        size: i as u64 * 100,
                    });
                }

                let result = create_bom(&entries);
                prop_assert!(result.is_ok(), "Mixed types BOM failed");
            }
        }

        /// Fuzz test: Stress test with maximum realistic entries
        #[test]
        fn fuzz_stress_test_large_bom() {
            let entries: Vec<BomEntry> = (0..500)
                .map(|i| BomEntry {
                    path: PathBuf::from(format!("dir{}/subdir{}/file{}.txt", i / 100, i / 10, i)),
                    mode: if i % 10 == 0 { 0o100755 } else { 0o100644 },
                    uid: (i % 1000) as u32,
                    gid: 80,
                    size: i as u64 * 1000,
                })
                .collect();

            let result = create_bom(&entries);
            assert!(result.is_ok(), "Large BOM creation failed");

            let bom = result.unwrap();
            assert!(bom.starts_with(b"BOMStore"));
            assert!(bom.len() > 10000, "Large BOM should be substantial");
        }

        /// Fuzz test: Boundary conditions for path lengths
        #[test]
        fn fuzz_path_length_boundaries() {
            // Test various path lengths
            for len in [1, 10, 50, 100, 200, 255] {
                let name = "x".repeat(len);
                let entry = BomEntry {
                    path: PathBuf::from(&name),
                    mode: 0o100644,
                    uid: 0,
                    gid: 80,
                    size: 5,
                };
                let result = create_bom(&[entry]);
                assert!(result.is_ok(), "Failed for path length {}", len);
            }
        }

        /// Fuzz test: All valid permission combinations
        #[test]
        fn fuzz_all_permissions() {
            let modes = [
                0o100000, 0o100400, 0o100600, 0o100644, 0o100666, 0o100700, 0o100755, 0o100777,
                0o040000, 0o040400, 0o040600, 0o040644, 0o040666, 0o040700, 0o040755, 0o040777,
            ];

            for mode in modes {
                let entry = BomEntry {
                    path: PathBuf::from("test"),
                    mode,
                    uid: 0,
                    gid: 80,
                    size: if mode & 0o170000 == 0o040000 { 0 } else { 100 },
                };
                let result = create_bom(&[entry]);
                assert!(result.is_ok(), "Failed for mode {:o}", mode);
            }
        }
    }
}
