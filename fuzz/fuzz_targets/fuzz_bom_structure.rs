#![no_main]

use arbitrary::Arbitrary;
use libfuzzer_sys::fuzz_target;
use std::path::PathBuf;

/// Configuration for BOM structure fuzzing
#[derive(Debug, Arbitrary)]
struct BomConfig {
    /// Number of top-level files
    num_top_level: u8,
    /// Number of directories
    num_dirs: u8,
    /// Files per directory
    files_per_dir: u8,
    /// Max nested depth
    max_depth: u8,
    /// Random seeds for variety
    seeds: [u8; 8],
}

fuzz_target!(|config: BomConfig| {
    let mut entries = Vec::new();
    let mut idx = 0usize;

    // Add top-level files
    let num_top = (config.num_top_level % 20) as usize;
    for i in 0..num_top {
        let name = format!("file{}.txt", i);
        entries.push(iamawrapper::macos::bom::BomEntry {
            path: PathBuf::from(&name),
            mode: 0o100644,
            uid: config.seeds[idx % 8] as u32 * 100,
            gid: 80,
            size: (config.seeds[(idx + 1) % 8] as u64) * 1000,
        });
        idx += 1;
    }

    // Add directories with files
    let num_dirs = (config.num_dirs % 10) as usize;
    let files_per = (config.files_per_dir % 10) as usize;
    let max_depth = (config.max_depth % 5) as usize + 1;

    for d in 0..num_dirs {
        // Create nested path
        let depth = (d % max_depth) + 1;
        let dir_path: String = (0..depth)
            .map(|i| format!("level{}", i))
            .collect::<Vec<_>>()
            .join("/")
            + &format!("/dir{}", d);

        // Add directory entry
        entries.push(iamawrapper::macos::bom::BomEntry {
            path: PathBuf::from(&dir_path),
            mode: 0o040755,
            uid: 0,
            gid: 80,
            size: 0,
        });

        // Add files in directory
        for f in 0..files_per {
            let file_path = format!("{}/file{}.dat", dir_path, f);
            let mode = if config.seeds[(idx + f) % 8] % 2 == 0 {
                0o100644
            } else {
                0o100755
            };
            entries.push(iamawrapper::macos::bom::BomEntry {
                path: PathBuf::from(&file_path),
                mode,
                uid: 0,
                gid: 80,
                size: (config.seeds[(idx + f + 1) % 8] as u64) * 500,
            });
        }
        idx += files_per + 1;
    }

    if entries.is_empty() {
        return;
    }

    // Deduplicate paths
    let mut seen = std::collections::HashSet::new();
    let unique: Vec<_> = entries
        .into_iter()
        .filter(|e| seen.insert(e.path.to_string_lossy().to_string()))
        .collect();

    if unique.is_empty() {
        return;
    }

    // Create BOM
    let result = iamawrapper::macos::bom::create_bom(&unique);

    if let Ok(bom) = result {
        // Comprehensive structure validation
        assert!(bom.len() >= 512, "BOM too small");
        assert_eq!(&bom[0..8], b"BOMStore", "Bad magic");

        let version = u32::from_be_bytes([bom[8], bom[9], bom[10], bom[11]]);
        assert_eq!(version, 1, "Bad version");

        let num_blocks = u32::from_be_bytes([bom[12], bom[13], bom[14], bom[15]]);
        assert!(num_blocks > 0, "No blocks");

        let index_offset = u32::from_be_bytes([bom[16], bom[17], bom[18], bom[19]]) as usize;
        let index_length = u32::from_be_bytes([bom[20], bom[21], bom[22], bom[23]]) as usize;
        let vars_offset = u32::from_be_bytes([bom[24], bom[25], bom[26], bom[27]]) as usize;
        let vars_length = u32::from_be_bytes([bom[28], bom[29], bom[30], bom[31]]) as usize;

        // Bounds checks
        assert!(index_offset + index_length <= bom.len(), "Index OOB");
        assert!(vars_offset + vars_length <= bom.len(), "Vars OOB");
        assert_eq!(vars_offset, 512, "Vars not at 512");

        // Validate vars section has 5 entries
        if vars_length >= 4 {
            let num_vars = u32::from_be_bytes([
                bom[vars_offset],
                bom[vars_offset + 1],
                bom[vars_offset + 2],
                bom[vars_offset + 3],
            ]);
            assert_eq!(num_vars, 5, "Should have 5 vars");
        }

        // Validate block table structure
        if index_length >= 12 {
            let block_count = u32::from_be_bytes([
                bom[index_offset],
                bom[index_offset + 1],
                bom[index_offset + 2],
                bom[index_offset + 3],
            ]) as usize;

            // Null block (index 0) should be 0,0
            let null_addr = u32::from_be_bytes([
                bom[index_offset + 4],
                bom[index_offset + 5],
                bom[index_offset + 6],
                bom[index_offset + 7],
            ]);
            let null_len = u32::from_be_bytes([
                bom[index_offset + 8],
                bom[index_offset + 9],
                bom[index_offset + 10],
                bom[index_offset + 11],
            ]);
            assert_eq!(null_addr, 0, "Null block addr != 0");
            assert_eq!(null_len, 0, "Null block len != 0");

            // Verify all block entries are within bounds
            for i in 1..block_count {
                let entry_offset = index_offset + 4 + (i * 8);
                if entry_offset + 8 <= bom.len() {
                    let addr = u32::from_be_bytes([
                        bom[entry_offset],
                        bom[entry_offset + 1],
                        bom[entry_offset + 2],
                        bom[entry_offset + 3],
                    ]) as usize;
                    let len = u32::from_be_bytes([
                        bom[entry_offset + 4],
                        bom[entry_offset + 5],
                        bom[entry_offset + 6],
                        bom[entry_offset + 7],
                    ]) as usize;

                    assert!(
                        addr + len <= bom.len(),
                        "Block {} at {} len {} exceeds BOM size {}",
                        i,
                        addr,
                        len,
                        bom.len()
                    );
                }
            }
        }

        // Verify tree structures exist
        let tree_count = bom.windows(4).filter(|w| *w == b"tree").count();
        assert!(tree_count >= 4, "Should have at least 4 trees");
    }
});
