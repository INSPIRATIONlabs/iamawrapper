//! CPIO archive wrapper for macOS package payloads.
//!
//! macOS packages use CPIO odc (portable ASCII) format for payloads.

use std::io::Write;
use std::time::{SystemTime, UNIX_EPOCH};

use flate2::Compression;
use flate2::write::GzEncoder;

use crate::models::PackageError;

/// File entry for CPIO archive: (path, data, mode)
pub type CpioEntry = (String, Vec<u8>, u32);

/// Root UID for macOS packages
const ROOT_UID: u32 = 0;

/// Wheel GID for macOS packages (admin group)
const WHEEL_GID: u32 = 80;

/// Regular file type bits
const S_IFREG: u32 = 0o100000;

/// CPIO odc header format (76 bytes ASCII).
///
/// Format: magic(6) + dev(6) + ino(6) + mode(6) + uid(6) + gid(6) +
///         nlink(6) + rdev(6) + mtime(11) + namesize(6) + filesize(11)
struct CpioHeader {
    dev: u32,
    ino: u32,
    mode: u32,
    uid: u32,
    gid: u32,
    nlink: u32,
    rdev: u32,
    mtime: u64,
    namesize: u32,
    filesize: u64,
}

impl CpioHeader {
    /// Create a new CPIO header for a file.
    fn for_file(mode: u32, size: u64, name_len: usize, ino: u32) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        Self {
            dev: 0,
            ino,
            mode: S_IFREG | (mode & 0o7777),
            uid: ROOT_UID,
            gid: WHEEL_GID,
            nlink: 1,
            rdev: 0,
            mtime: now,
            namesize: (name_len + 1) as u32, // +1 for null terminator
            filesize: size,
        }
    }

    /// Create TRAILER!!! header.
    fn trailer() -> Self {
        Self {
            dev: 0,
            ino: 0,
            mode: 0,
            uid: 0,
            gid: 0,
            nlink: 1,
            rdev: 0,
            mtime: 0,
            namesize: 11, // "TRAILER!!!\0"
            filesize: 0,
        }
    }

    /// Serialize to 76-byte ASCII octal format.
    fn to_bytes(&self) -> Vec<u8> {
        format!(
            "{:06o}{:06o}{:06o}{:06o}{:06o}{:06o}{:06o}{:06o}{:011o}{:06o}{:011o}",
            0o70707, // magic
            self.dev,
            self.ino,
            self.mode,
            self.uid,
            self.gid,
            self.nlink,
            self.rdev,
            self.mtime,
            self.namesize,
            self.filesize,
        )
        .into_bytes()
    }
}

/// Create a CPIO archive in odc format.
///
/// Each entry is (path, data, mode). Files will have uid=0, gid=80.
pub fn create_cpio_archive(entries: &[CpioEntry]) -> Result<Vec<u8>, PackageError> {
    let mut output = Vec::new();
    let mut ino: u32 = 1;

    for (path, data, mode) in entries {
        // Write header
        let header = CpioHeader::for_file(*mode, data.len() as u64, path.len(), ino);
        output.extend_from_slice(&header.to_bytes());

        // Write filename with null terminator
        output.extend_from_slice(path.as_bytes());
        output.push(0);

        // Write file data
        output.extend_from_slice(data);

        ino += 1;
    }

    // Write trailer
    let trailer = CpioHeader::trailer();
    output.extend_from_slice(&trailer.to_bytes());
    output.extend_from_slice(b"TRAILER!!!\0");

    Ok(output)
}

/// Create a CPIO archive with directory support.
pub fn create_cpio_archive_with_dirs(
    _entries: &[(String, u32, bool)],
) -> Result<Vec<u8>, PackageError> {
    // For now, just use the regular implementation
    // Directories are typically not needed for basic payloads
    Err(PackageError::CpioError {
        reason: "Directory support not yet implemented".to_string(),
    })
}

/// Create a gzip-compressed CPIO payload.
///
/// This is the main function for creating macOS package payloads.
pub fn create_payload(entries: &[CpioEntry]) -> Result<Vec<u8>, PackageError> {
    // Create uncompressed CPIO archive
    let cpio_data = create_cpio_archive(entries)?;

    // Compress with gzip
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(&cpio_data)
        .map_err(|e| PackageError::CpioError {
            reason: e.to_string(),
        })?;

    encoder.finish().map_err(|e| PackageError::CpioError {
        reason: e.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    // T008: CPIO odc format tests
    #[test]
    fn test_cpio_odc_magic() {
        let cpio_data =
            create_cpio_archive(&[("test.txt".to_string(), b"hello".to_vec(), 0o644)]).unwrap();
        let header = std::str::from_utf8(&cpio_data[0..6]).unwrap();
        assert_eq!(header, "070707", "CPIO odc magic must be '070707'");
    }

    #[test]
    fn test_cpio_odc_header_size() {
        let cpio_data =
            create_cpio_archive(&[("test.txt".to_string(), b"hello".to_vec(), 0o644)]).unwrap();
        assert!(
            cpio_data.len() >= 76,
            "CPIO archive must have at least 76 bytes for header"
        );
    }

    #[test]
    fn test_cpio_odc_uid_gid() {
        let cpio_data =
            create_cpio_archive(&[("test.txt".to_string(), b"hello".to_vec(), 0o644)]).unwrap();
        // uid is at offset 24-30 (6 octal digits)
        let uid_str = std::str::from_utf8(&cpio_data[24..30]).unwrap();
        let uid = u32::from_str_radix(uid_str, 8).unwrap();
        assert_eq!(uid, 0, "UID must be 0 (root)");
        // gid is at offset 30-36 (6 octal digits)
        let gid_str = std::str::from_utf8(&cpio_data[30..36]).unwrap();
        let gid = u32::from_str_radix(gid_str, 8).unwrap();
        assert_eq!(gid, 80, "GID must be 80 (wheel)");
    }

    #[test]
    fn test_cpio_odc_file_mode() {
        let cpio_data =
            create_cpio_archive(&[("test.txt".to_string(), b"hello".to_vec(), 0o755)]).unwrap();
        // mode is at offset 18-24 (6 octal digits)
        let mode_str = std::str::from_utf8(&cpio_data[18..24]).unwrap();
        let mode = u32::from_str_radix(mode_str, 8).unwrap();
        let permissions = mode & 0o777;
        assert_eq!(permissions, 0o755, "File permissions must be preserved");
    }

    #[test]
    fn test_cpio_odc_filename() {
        let cpio_data =
            create_cpio_archive(&[("myfile.txt".to_string(), b"content".to_vec(), 0o644)]).unwrap();
        // namesize is at offset 59-65 (6 octal digits)
        let namesize_str = std::str::from_utf8(&cpio_data[59..65]).unwrap();
        let namesize = usize::from_str_radix(namesize_str, 8).unwrap();
        let filename = std::str::from_utf8(&cpio_data[76..76 + namesize - 1]).unwrap();
        assert_eq!(
            filename, "myfile.txt",
            "Filename must be present after header"
        );
    }

    #[test]
    fn test_cpio_odc_trailer() {
        let cpio_data =
            create_cpio_archive(&[("test.txt".to_string(), b"hello".to_vec(), 0o644)]).unwrap();
        let data_str = String::from_utf8_lossy(&cpio_data);
        assert!(
            data_str.contains("TRAILER!!!"),
            "Archive must end with TRAILER!!!"
        );
    }

    #[test]
    fn test_cpio_odc_multiple_files() {
        let cpio_data = create_cpio_archive(&[
            ("file1.txt".to_string(), b"content1".to_vec(), 0o644),
            ("file2.txt".to_string(), b"content2".to_vec(), 0o755),
        ])
        .unwrap();
        let data_str = String::from_utf8_lossy(&cpio_data);
        assert!(data_str.contains("file1.txt"), "First file must be present");
        assert!(
            data_str.contains("file2.txt"),
            "Second file must be present"
        );
    }

    // T009: Gzip-compressed payload tests
    #[test]
    fn test_payload_gzip_magic() {
        let payload =
            create_payload(&[("test.txt".to_string(), b"hello".to_vec(), 0o644)]).unwrap();
        assert_eq!(payload[0], 0x1f, "Gzip magic byte 1 must be 0x1f");
        assert_eq!(payload[1], 0x8b, "Gzip magic byte 2 must be 0x8b");
    }

    #[test]
    fn test_payload_compression() {
        let large_content = "A".repeat(10000);
        let payload = create_payload(&[(
            "test.txt".to_string(),
            large_content.as_bytes().to_vec(),
            0o644,
        )])
        .unwrap();
        assert!(payload.len() < 5000, "Payload should be compressed");
    }

    #[test]
    fn test_payload_decompression() {
        let payload =
            create_payload(&[("test.txt".to_string(), b"hello world".to_vec(), 0o644)]).unwrap();
        use flate2::read::GzDecoder;
        use std::io::Read;
        let mut decoder = GzDecoder::new(&payload[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        let magic = std::str::from_utf8(&decompressed[0..6]).unwrap();
        assert_eq!(magic, "070707", "Decompressed payload must be valid CPIO");
    }

    #[test]
    fn test_payload_with_subdirectory() {
        let payload = create_payload(&[(
            "Applications/MyApp.app/Contents/Info.plist".to_string(),
            b"<plist/>".to_vec(),
            0o644,
        )])
        .unwrap();
        use flate2::read::GzDecoder;
        use std::io::Read;
        let mut decoder = GzDecoder::new(&payload[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).unwrap();
        let data_str = String::from_utf8_lossy(&decompressed);
        assert!(
            data_str.contains("Applications/MyApp.app/Contents/Info.plist"),
            "Subdirectory path must be preserved"
        );
    }
}
