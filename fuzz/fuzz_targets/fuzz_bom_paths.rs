#![no_main]

use libfuzzer_sys::fuzz_target;
use std::path::PathBuf;

// Fuzz test focused on path handling edge cases
fuzz_target!(|data: &[u8]| {
    // Use fuzz data to generate path configurations
    if data.len() < 4 {
        return;
    }

    let depth = (data[0] % 20) as usize + 1;
    let num_files = (data[1] % 50) as usize + 1;
    let mode_selector = data[2];
    let size_multiplier = data[3] as u64;

    let mut entries = Vec::new();

    // Generate nested directory structure
    for i in 0..num_files {
        let path_components: Vec<String> = (0..depth.min(10))
            .map(|d| format!("dir{}_{}", d, i % 5))
            .collect();

        let path_str = if path_components.is_empty() {
            format!("file{}.txt", i)
        } else {
            format!("{}/file{}.txt", path_components.join("/"), i)
        };

        let mode = match mode_selector % 4 {
            0 => 0o100644,
            1 => 0o100755,
            2 => 0o040755,
            _ => 0o100644,
        };

        entries.push(iamawrapper::macos::bom::BomEntry {
            path: PathBuf::from(path_str),
            mode,
            uid: 0,
            gid: 80,
            size: (i as u64).wrapping_mul(size_multiplier),
        });
    }

    if entries.is_empty() {
        return;
    }

    // Deduplicate
    let mut seen = std::collections::HashSet::new();
    let unique: Vec<_> = entries
        .into_iter()
        .filter(|e| seen.insert(e.path.to_string_lossy().to_string()))
        .collect();

    if unique.is_empty() {
        return;
    }

    // Create BOM - should not panic
    let result = iamawrapper::macos::bom::create_bom(&unique);

    if let Ok(bom) = result {
        // Validate structure
        assert!(bom.starts_with(b"BOMStore"));

        // Check offsets are within bounds
        let index_offset = u32::from_be_bytes([bom[16], bom[17], bom[18], bom[19]]) as usize;
        let index_length = u32::from_be_bytes([bom[20], bom[21], bom[22], bom[23]]) as usize;
        let vars_offset = u32::from_be_bytes([bom[24], bom[25], bom[26], bom[27]]) as usize;
        let vars_length = u32::from_be_bytes([bom[28], bom[29], bom[30], bom[31]]) as usize;

        assert!(index_offset + index_length <= bom.len());
        assert!(vars_offset + vars_length <= bom.len());
    }
});
