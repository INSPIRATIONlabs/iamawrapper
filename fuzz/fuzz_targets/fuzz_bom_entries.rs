#![no_main]

use arbitrary::{Arbitrary, Unstructured};
use libfuzzer_sys::fuzz_target;
use std::path::PathBuf;

/// Arbitrary BomEntry for fuzzing
#[derive(Debug, Clone, Arbitrary)]
struct FuzzBomEntry {
    /// Path components (will be joined with /)
    path_components: Vec<PathComponent>,
    /// File mode
    mode: FileMode,
    /// User ID
    uid: u32,
    /// Group ID
    gid: u32,
    /// File size
    size: u64,
}

/// Safe path component that avoids problematic characters
#[derive(Debug, Clone)]
struct PathComponent(String);

impl<'a> Arbitrary<'a> for PathComponent {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let len: usize = u.int_in_range(1..=50)?;
        let chars: Vec<char> = (0..len)
            .map(|_| {
                let idx: usize = u.int_in_range(0..=61).unwrap_or(0);
                match idx {
                    0..=25 => (b'a' + idx as u8) as char,
                    26..=51 => (b'A' + (idx - 26) as u8) as char,
                    52..=61 => (b'0' + (idx - 52) as u8) as char,
                    _ => 'x',
                }
            })
            .collect();
        Ok(PathComponent(chars.into_iter().collect()))
    }
}

/// Valid file modes
#[derive(Debug, Clone, Arbitrary)]
enum FileMode {
    RegularReadWrite,    // 0o100644
    RegularExecutable,   // 0o100755
    RegularReadOnly,     // 0o100444
    Directory,           // 0o040755
    DirectoryRestricted, // 0o040700
}

impl FileMode {
    fn to_mode(&self) -> u32 {
        match self {
            FileMode::RegularReadWrite => 0o100644,
            FileMode::RegularExecutable => 0o100755,
            FileMode::RegularReadOnly => 0o100444,
            FileMode::Directory => 0o040755,
            FileMode::DirectoryRestricted => 0o040700,
        }
    }

    fn is_dir(&self) -> bool {
        matches!(self, FileMode::Directory | FileMode::DirectoryRestricted)
    }
}

impl FuzzBomEntry {
    fn to_bom_entry(&self) -> Option<iamawrapper::macos::bom::BomEntry> {
        if self.path_components.is_empty() {
            return None;
        }

        let path_str = self
            .path_components
            .iter()
            .map(|c| c.0.as_str())
            .collect::<Vec<_>>()
            .join("/");

        Some(iamawrapper::macos::bom::BomEntry {
            path: PathBuf::from(path_str),
            mode: self.mode.to_mode(),
            uid: self.uid,
            gid: self.gid,
            size: if self.mode.is_dir() { 0 } else { self.size },
        })
    }
}

fuzz_target!(|entries: Vec<FuzzBomEntry>| {
    // Convert fuzz entries to BomEntries, filtering out invalid ones
    let bom_entries: Vec<_> = entries
        .iter()
        .filter_map(|e| e.to_bom_entry())
        .collect();

    if bom_entries.is_empty() {
        return;
    }

    // Deduplicate paths
    let mut seen = std::collections::HashSet::new();
    let unique_entries: Vec<_> = bom_entries
        .into_iter()
        .filter(|e| seen.insert(e.path.to_string_lossy().to_string()))
        .collect();

    if unique_entries.is_empty() {
        return;
    }

    // Try to create BOM - should not panic
    let result = iamawrapper::macos::bom::create_bom(&unique_entries);

    // If successful, verify basic structure
    if let Ok(bom) = result {
        // Magic should be "BOMStore"
        assert_eq!(&bom[0..8], b"BOMStore");

        // Version should be 1
        let version = u32::from_be_bytes([bom[8], bom[9], bom[10], bom[11]]);
        assert_eq!(version, 1);

        // Header should be at least 512 bytes
        assert!(bom.len() >= 512);
    }
});
