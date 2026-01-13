//! XAR archive format writer for macOS packages.
//!
//! XAR (eXtensible ARchive) is the container format for .pkg files.
//! Structure: 28-byte header + zlib-compressed XML TOC + heap (file data)

use std::io::Write;

use flate2::Compression;
use flate2::write::ZlibEncoder;
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use sha1::{Digest, Sha1};

use crate::models::PackageError;

/// XAR magic number "xar!" (0x78617221)
const XAR_MAGIC: &[u8; 4] = b"xar!";

/// XAR header size (always 28 bytes)
const XAR_HEADER_SIZE: u16 = 28;

/// XAR version (always 1)
const XAR_VERSION: u16 = 1;

/// Checksum algorithm: SHA1 = 1
const CKSUM_SHA1: u32 = 1;

/// XAR archive header (28 bytes, big-endian).
#[derive(Debug, Clone)]
pub struct XarHeader {
    /// TOC compressed length
    pub toc_compressed_length: u64,
    /// TOC uncompressed length
    pub toc_uncompressed_length: u64,
}

impl XarHeader {
    /// Create a new XAR header with TOC lengths.
    pub fn new(toc_compressed_length: u64, toc_uncompressed_length: u64) -> Self {
        Self {
            toc_compressed_length,
            toc_uncompressed_length,
        }
    }

    /// Serialize header to 28 bytes (big-endian).
    pub fn to_bytes(&self) -> [u8; 28] {
        let mut buf = [0u8; 28];

        // Magic: "xar!" (0-3)
        buf[0..4].copy_from_slice(XAR_MAGIC);

        // Header size: 28 (4-5)
        buf[4..6].copy_from_slice(&XAR_HEADER_SIZE.to_be_bytes());

        // Version: 1 (6-7)
        buf[6..8].copy_from_slice(&XAR_VERSION.to_be_bytes());

        // TOC compressed length (8-15)
        buf[8..16].copy_from_slice(&self.toc_compressed_length.to_be_bytes());

        // TOC uncompressed length (16-23)
        buf[16..24].copy_from_slice(&self.toc_uncompressed_length.to_be_bytes());

        // Checksum algorithm: SHA1 (24-27)
        buf[24..28].copy_from_slice(&CKSUM_SHA1.to_be_bytes());

        buf
    }
}

/// Entry type in XAR archive.
#[derive(Debug, Clone, PartialEq)]
pub enum EntryType {
    File,
    Directory,
}

/// An entry (file or directory) in the XAR archive.
#[derive(Debug, Clone)]
pub struct XarEntry {
    /// Entry name (just the filename, not full path)
    pub name: String,
    /// Full path for nested entries
    pub path: String,
    /// Entry type
    pub entry_type: EntryType,
    /// File data (empty for directories)
    pub data: Vec<u8>,
    /// Heap offset (set during finish)
    pub offset: u64,
    /// SHA1 checksum (computed during finish)
    pub checksum: String,
    /// Entry ID (assigned during add)
    pub id: u64,
    /// Parent entry ID (0 for root-level entries)
    pub parent_id: Option<u64>,
}

/// Builder for XAR archives.
#[derive(Debug)]
pub struct XarBuilder {
    entries: Vec<XarEntry>,
    next_id: u64,
}

impl XarBuilder {
    /// Create a new XAR archive builder.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            next_id: 1,
        }
    }

    /// Add a file to the archive.
    pub fn add_file(&mut self, path: &str, data: Vec<u8>) -> Result<(), PackageError> {
        let name = path.rsplit('/').next().unwrap_or(path).to_string();
        let parent_id = self.find_parent_id(path);

        self.entries.push(XarEntry {
            name,
            path: path.to_string(),
            entry_type: EntryType::File,
            data,
            offset: 0,
            checksum: String::new(),
            id: self.next_id,
            parent_id,
        });
        self.next_id += 1;
        Ok(())
    }

    /// Add a directory to the archive.
    pub fn add_directory(&mut self, path: &str) -> Result<(), PackageError> {
        let name = path.rsplit('/').next().unwrap_or(path).to_string();
        let parent_id = self.find_parent_id(path);

        self.entries.push(XarEntry {
            name,
            path: path.to_string(),
            entry_type: EntryType::Directory,
            data: Vec::new(),
            offset: 0,
            checksum: String::new(),
            id: self.next_id,
            parent_id,
        });
        self.next_id += 1;
        Ok(())
    }

    /// Find parent directory ID for a path.
    fn find_parent_id(&self, path: &str) -> Option<u64> {
        if let Some(parent_path) = path.rsplit_once('/').map(|(p, _)| p) {
            for entry in &self.entries {
                if entry.path == parent_path && entry.entry_type == EntryType::Directory {
                    return Some(entry.id);
                }
            }
        }
        None
    }

    /// SHA1 checksum size in bytes.
    const SHA1_SIZE: u64 = 20;

    /// Generate the TOC XML for the archive.
    /// The heap_start_offset is where file data begins (after the TOC checksum).
    pub fn generate_toc_xml(&self) -> Result<String, PackageError> {
        let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

        // XML declaration
        writer
            .write_event(Event::Decl(BytesDecl::new("1.0", Some("UTF-8"), None)))
            .map_err(|e| PackageError::XarError {
                reason: e.to_string(),
            })?;

        // <xar>
        let xar_start = BytesStart::new("xar");
        writer
            .write_event(Event::Start(xar_start))
            .map_err(|e| PackageError::XarError {
                reason: e.to_string(),
            })?;

        // <toc>
        let toc_start = BytesStart::new("toc");
        writer
            .write_event(Event::Start(toc_start))
            .map_err(|e| PackageError::XarError {
                reason: e.to_string(),
            })?;

        // Write TOC checksum element (points to heap offset 0)
        self.write_toc_checksum(&mut writer)?;

        // Write entries (file data starts after the TOC checksum)
        self.write_toc_entries(&mut writer, None, Self::SHA1_SIZE)?;

        // </toc>
        writer
            .write_event(Event::End(BytesEnd::new("toc")))
            .map_err(|e| PackageError::XarError {
                reason: e.to_string(),
            })?;

        // </xar>
        writer
            .write_event(Event::End(BytesEnd::new("xar")))
            .map_err(|e| PackageError::XarError {
                reason: e.to_string(),
            })?;

        let xml_bytes = writer.into_inner();
        String::from_utf8(xml_bytes).map_err(|e| PackageError::XarError {
            reason: e.to_string(),
        })
    }

    /// Write TOC entries recursively.
    fn write_toc_entries<W: Write>(
        &self,
        writer: &mut Writer<W>,
        parent_id: Option<u64>,
        heap_offset: u64,
    ) -> Result<u64, PackageError> {
        let mut current_offset = heap_offset;

        for entry in &self.entries {
            if entry.parent_id != parent_id {
                continue;
            }

            // <file id="N">
            let mut file_start = BytesStart::new("file");
            file_start.push_attribute(("id", entry.id.to_string().as_str()));
            writer
                .write_event(Event::Start(file_start))
                .map_err(|e| PackageError::XarError {
                    reason: e.to_string(),
                })?;

            // <name>
            self.write_simple_element(writer, "name", &entry.name)?;

            // <type>
            let type_str = match entry.entry_type {
                EntryType::File => "file",
                EntryType::Directory => "directory",
            };
            self.write_simple_element(writer, "type", type_str)?;

            // For files, write data section
            if entry.entry_type == EntryType::File {
                // <data>
                writer
                    .write_event(Event::Start(BytesStart::new("data")))
                    .map_err(|e| PackageError::XarError {
                        reason: e.to_string(),
                    })?;

                self.write_simple_element(writer, "offset", &current_offset.to_string())?;
                self.write_simple_element(writer, "size", &entry.data.len().to_string())?;
                self.write_simple_element(writer, "length", &entry.data.len().to_string())?;

                // <extracted-checksum style="sha1">
                let checksum = Self::compute_sha1(&entry.data);
                let mut cksum_start = BytesStart::new("extracted-checksum");
                cksum_start.push_attribute(("style", "sha1"));
                writer.write_event(Event::Start(cksum_start)).map_err(|e| {
                    PackageError::XarError {
                        reason: e.to_string(),
                    }
                })?;
                writer
                    .write_event(Event::Text(BytesText::new(&checksum)))
                    .map_err(|e| PackageError::XarError {
                        reason: e.to_string(),
                    })?;
                writer
                    .write_event(Event::End(BytesEnd::new("extracted-checksum")))
                    .map_err(|e| PackageError::XarError {
                        reason: e.to_string(),
                    })?;

                // <archived-checksum style="sha1">
                let mut arch_cksum_start = BytesStart::new("archived-checksum");
                arch_cksum_start.push_attribute(("style", "sha1"));
                writer
                    .write_event(Event::Start(arch_cksum_start))
                    .map_err(|e| PackageError::XarError {
                        reason: e.to_string(),
                    })?;
                writer
                    .write_event(Event::Text(BytesText::new(&checksum)))
                    .map_err(|e| PackageError::XarError {
                        reason: e.to_string(),
                    })?;
                writer
                    .write_event(Event::End(BytesEnd::new("archived-checksum")))
                    .map_err(|e| PackageError::XarError {
                        reason: e.to_string(),
                    })?;

                // <encoding style="application/octet-stream"/>
                let mut encoding = BytesStart::new("encoding");
                encoding.push_attribute(("style", "application/octet-stream"));
                writer
                    .write_event(Event::Empty(encoding))
                    .map_err(|e| PackageError::XarError {
                        reason: e.to_string(),
                    })?;

                // </data>
                writer
                    .write_event(Event::End(BytesEnd::new("data")))
                    .map_err(|e| PackageError::XarError {
                        reason: e.to_string(),
                    })?;

                current_offset += entry.data.len() as u64;
            }

            // Write child entries for directories
            if entry.entry_type == EntryType::Directory {
                current_offset = self.write_toc_entries(writer, Some(entry.id), current_offset)?;
            }

            // </file>
            writer
                .write_event(Event::End(BytesEnd::new("file")))
                .map_err(|e| PackageError::XarError {
                    reason: e.to_string(),
                })?;
        }

        Ok(current_offset)
    }

    /// Write the TOC checksum element.
    fn write_toc_checksum<W: Write>(&self, writer: &mut Writer<W>) -> Result<(), PackageError> {
        // <checksum style="sha1">
        let mut checksum_start = BytesStart::new("checksum");
        checksum_start.push_attribute(("style", "sha1"));
        writer
            .write_event(Event::Start(checksum_start))
            .map_err(|e| PackageError::XarError {
                reason: e.to_string(),
            })?;

        // <offset>0</offset>
        self.write_simple_element(writer, "offset", "0")?;

        // <size>20</size> (SHA1 is 20 bytes)
        self.write_simple_element(writer, "size", "20")?;

        // </checksum>
        writer
            .write_event(Event::End(BytesEnd::new("checksum")))
            .map_err(|e| PackageError::XarError {
                reason: e.to_string(),
            })?;

        Ok(())
    }

    /// Write a simple text element.
    fn write_simple_element<W: Write>(
        &self,
        writer: &mut Writer<W>,
        tag: &str,
        content: &str,
    ) -> Result<(), PackageError> {
        writer
            .write_event(Event::Start(BytesStart::new(tag)))
            .map_err(|e| PackageError::XarError {
                reason: e.to_string(),
            })?;
        writer
            .write_event(Event::Text(BytesText::new(content)))
            .map_err(|e| PackageError::XarError {
                reason: e.to_string(),
            })?;
        writer
            .write_event(Event::End(BytesEnd::new(tag)))
            .map_err(|e| PackageError::XarError {
                reason: e.to_string(),
            })?;
        Ok(())
    }

    /// Compute SHA1 hash of data.
    fn compute_sha1(data: &[u8]) -> String {
        let mut hasher = Sha1::new();
        hasher.update(data);
        let result = hasher.finalize();
        hex::encode(result)
    }

    /// Finish building and write the archive.
    pub fn finish<W: Write>(&mut self, writer: &mut W) -> Result<(), PackageError> {
        // Generate TOC XML
        let toc_xml = self.generate_toc_xml()?;
        let toc_uncompressed = toc_xml.as_bytes();

        // Compress TOC with zlib
        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
        encoder
            .write_all(toc_uncompressed)
            .map_err(|e| PackageError::XarError {
                reason: e.to_string(),
            })?;
        let toc_compressed = encoder.finish().map_err(|e| PackageError::XarError {
            reason: e.to_string(),
        })?;

        // Create header
        let header = XarHeader::new(toc_compressed.len() as u64, toc_uncompressed.len() as u64);

        // Write header
        writer
            .write_all(&header.to_bytes())
            .map_err(|e| PackageError::XarError {
                reason: e.to_string(),
            })?;

        // Write compressed TOC
        writer
            .write_all(&toc_compressed)
            .map_err(|e| PackageError::XarError {
                reason: e.to_string(),
            })?;

        // Compute SHA1 of compressed TOC for the heap checksum
        let toc_checksum = Self::compute_sha1(&toc_compressed);
        let toc_checksum_bytes = hex::decode(&toc_checksum).map_err(|e| PackageError::XarError {
            reason: format!("Failed to decode TOC checksum: {}", e),
        })?;

        // Write heap: TOC checksum first (20 bytes at offset 0)
        writer
            .write_all(&toc_checksum_bytes)
            .map_err(|e| PackageError::XarError {
                reason: e.to_string(),
            })?;

        // Write heap: file data (starting at offset 20)
        for entry in &self.entries {
            if entry.entry_type == EntryType::File {
                writer
                    .write_all(&entry.data)
                    .map_err(|e| PackageError::XarError {
                        reason: e.to_string(),
                    })?;
            }
        }

        Ok(())
    }
}

impl Default for XarBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    // T005: XAR header serialization tests
    #[test]
    fn test_xar_header_magic() {
        let header = XarHeader::new(100, 200);
        let bytes = header.to_bytes();
        assert_eq!(&bytes[0..4], b"xar!", "XAR magic must be 'xar!'");
    }

    #[test]
    fn test_xar_header_size() {
        let header = XarHeader::new(100, 200);
        let bytes = header.to_bytes();
        assert_eq!(bytes.len(), 28, "XAR header must be 28 bytes");
    }

    #[test]
    fn test_xar_header_version() {
        let header = XarHeader::new(100, 200);
        let bytes = header.to_bytes();
        let version = u16::from_be_bytes([bytes[6], bytes[7]]);
        assert_eq!(version, 1, "XAR version must be 1");
    }

    #[test]
    fn test_xar_header_toc_lengths() {
        let header = XarHeader::new(1234, 5678);
        let bytes = header.to_bytes();
        let toc_compressed = u64::from_be_bytes(bytes[8..16].try_into().unwrap());
        assert_eq!(toc_compressed, 1234);
        let toc_uncompressed = u64::from_be_bytes(bytes[16..24].try_into().unwrap());
        assert_eq!(toc_uncompressed, 5678);
    }

    #[test]
    fn test_xar_header_checksum_algorithm() {
        let header = XarHeader::new(100, 200);
        let bytes = header.to_bytes();
        let cksum_algo = u32::from_be_bytes(bytes[24..28].try_into().unwrap());
        assert_eq!(cksum_algo, 1, "Checksum algorithm should be SHA1 (1)");
    }

    // T006: XAR TOC XML generation tests
    #[test]
    fn test_xar_toc_xml_structure() {
        let mut builder = XarBuilder::new();
        builder.add_file("test.txt", b"hello".to_vec()).unwrap();
        let toc_xml = builder.generate_toc_xml().unwrap();
        assert!(toc_xml.contains("<?xml"), "TOC must have XML declaration");
        assert!(toc_xml.contains("<xar>"), "TOC must have <xar> root");
        assert!(toc_xml.contains("<toc>"), "TOC must have <toc> element");
    }

    #[test]
    fn test_xar_toc_file_entry() {
        let mut builder = XarBuilder::new();
        builder.add_file("myfile.txt", b"content".to_vec()).unwrap();
        let toc_xml = builder.generate_toc_xml().unwrap();
        assert!(toc_xml.contains("<file"), "TOC must have <file> element");
        assert!(
            toc_xml.contains("<name>myfile.txt</name>"),
            "File name must be present"
        );
    }

    #[test]
    fn test_xar_toc_directory_entry() {
        let mut builder = XarBuilder::new();
        builder.add_directory("base.pkg").unwrap();
        let toc_xml = builder.generate_toc_xml().unwrap();
        assert!(
            toc_xml.contains("<name>base.pkg</name>"),
            "Directory name must be present"
        );
        assert!(
            toc_xml.contains("<type>directory</type>"),
            "Type must be 'directory'"
        );
    }

    #[test]
    fn test_xar_toc_data_section() {
        let mut builder = XarBuilder::new();
        builder
            .add_file("test.txt", b"hello world".to_vec())
            .unwrap();
        let toc_xml = builder.generate_toc_xml().unwrap();
        assert!(toc_xml.contains("<data>"), "Must have <data> section");
        assert!(toc_xml.contains("<offset>"), "Data must have offset");
    }

    // T007: XAR archive assembly tests
    #[test]
    fn test_xar_archive_assembly() {
        let mut builder = XarBuilder::new();
        builder
            .add_file("Distribution", b"<?xml version=\"1.0\"?>".to_vec())
            .unwrap();
        let mut output = Cursor::new(Vec::new());
        builder.finish(&mut output).unwrap();
        let data = output.into_inner();
        assert_eq!(&data[0..4], b"xar!", "Archive must start with 'xar!'");
        assert!(data.len() > 28, "Archive must have content beyond header");
    }

    #[test]
    fn test_xar_archive_multiple_files() {
        let mut builder = XarBuilder::new();
        builder.add_directory("base.pkg").unwrap();
        builder
            .add_file("base.pkg/Bom", b"BOMStore data".to_vec())
            .unwrap();
        builder
            .add_file("base.pkg/Payload", b"CPIO payload".to_vec())
            .unwrap();
        builder
            .add_file("Distribution", b"<installer-script/>".to_vec())
            .unwrap();
        let mut output = Cursor::new(Vec::new());
        builder.finish(&mut output).unwrap();
        let data = output.into_inner();
        assert!(
            data.len() > 100,
            "Multi-file archive should have substantial size"
        );
    }
}
