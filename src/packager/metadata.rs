//! Detection.xml generation and parsing.

use quick_xml::Reader;
use quick_xml::Writer;
use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};

use crate::models::detection::{DetectionMetadata, EncryptionInfo};
use crate::models::error::{PackageError, PackageResult};

/// Tool version to include in Detection.xml (matches Microsoft's format).
const TOOL_VERSION: &str = "1.8.6.0";

/// Generate Detection.xml content matching the Microsoft format.
///
/// The XML format matches the original Microsoft Win32 Content Prep Tool:
/// - No XML declaration
/// - ToolVersion attribute on root element
/// - 2-space indentation with CRLF line endings (Windows style)
pub fn generate_detection_xml(metadata: &DetectionMetadata) -> PackageResult<String> {
    let mut writer = Writer::new_with_indent(Vec::new(), b' ', 2);

    // ApplicationInfo root element with namespaces and ToolVersion
    let mut root = BytesStart::new("ApplicationInfo");
    root.push_attribute(("xmlns:xsd", "http://www.w3.org/2001/XMLSchema"));
    root.push_attribute(("xmlns:xsi", "http://www.w3.org/2001/XMLSchema-instance"));
    root.push_attribute(("ToolVersion", TOOL_VERSION));
    writer
        .write_event(Event::Start(root))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    // Name element
    write_element(&mut writer, "Name", &metadata.name)?;

    // UnencryptedContentSize element
    write_element(
        &mut writer,
        "UnencryptedContentSize",
        &metadata.unencrypted_content_size.to_string(),
    )?;

    // FileName element
    write_element(&mut writer, "FileName", &metadata.file_name)?;

    // SetupFile element
    write_element(&mut writer, "SetupFile", &metadata.setup_file)?;

    // EncryptionInfo element
    writer
        .write_event(Event::Start(BytesStart::new("EncryptionInfo")))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    let info = &metadata.encryption_info;

    write_element(&mut writer, "EncryptionKey", &info.encryption_key_base64())?;
    write_element(&mut writer, "MacKey", &info.mac_key_base64())?;
    write_element(&mut writer, "InitializationVector", &info.iv_base64())?;
    write_element(&mut writer, "Mac", &info.mac_base64())?;
    write_element(&mut writer, "ProfileIdentifier", &info.profile_identifier)?;
    write_element(&mut writer, "FileDigest", &info.file_digest_base64())?;
    write_element(
        &mut writer,
        "FileDigestAlgorithm",
        &info.file_digest_algorithm,
    )?;

    writer
        .write_event(Event::End(BytesEnd::new("EncryptionInfo")))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    // Close ApplicationInfo
    writer
        .write_event(Event::End(BytesEnd::new("ApplicationInfo")))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    let output = writer.into_inner();
    let xml = String::from_utf8(output).map_err(|e| PackageError::XmlError {
        reason: e.to_string(),
    })?;

    // Convert LF to CRLF for Windows compatibility (Microsoft tool uses CRLF)
    Ok(xml.replace('\n', "\r\n"))
}

fn write_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    name: &str,
    value: &str,
) -> PackageResult<()> {
    writer
        .write_event(Event::Start(BytesStart::new(name)))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;
    writer
        .write_event(Event::Text(BytesText::new(value)))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;
    writer
        .write_event(Event::End(BytesEnd::new(name)))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;
    Ok(())
}

/// Parse Detection.xml content into DetectionMetadata.
pub fn parse_detection_xml(xml: &str) -> PackageResult<DetectionMetadata> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);

    let mut name = String::new();
    let mut unencrypted_content_size: u64 = 0;
    let mut file_name = String::new();
    let mut setup_file = String::new();
    let mut encryption_info = EncryptionInfo::new();

    let mut current_element = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                current_element = String::from_utf8_lossy(e.name().as_ref()).to_string();
            }
            Ok(Event::Text(e)) => {
                let text = e.unescape().map_err(|err| PackageError::XmlError {
                    reason: format!("Failed to unescape text: {}", err),
                })?;

                match current_element.as_str() {
                    "Name" => name = text.to_string(),
                    "UnencryptedContentSize" => {
                        unencrypted_content_size =
                            text.parse().map_err(|e| PackageError::XmlError {
                                reason: format!("Invalid UnencryptedContentSize: {}", e),
                            })?;
                    }
                    "FileName" => file_name = text.to_string(),
                    "SetupFile" => setup_file = text.to_string(),
                    "EncryptionKey" => {
                        encryption_info
                            .set_encryption_key_from_base64(&text)
                            .map_err(|e| PackageError::XmlError { reason: e })?;
                    }
                    "MacKey" => {
                        encryption_info
                            .set_mac_key_from_base64(&text)
                            .map_err(|e| PackageError::XmlError { reason: e })?;
                    }
                    "InitializationVector" => {
                        encryption_info
                            .set_iv_from_base64(&text)
                            .map_err(|e| PackageError::XmlError { reason: e })?;
                    }
                    "Mac" => {
                        encryption_info
                            .set_mac_from_base64(&text)
                            .map_err(|e| PackageError::XmlError { reason: e })?;
                    }
                    "ProfileIdentifier" => {
                        encryption_info.profile_identifier = text.to_string();
                    }
                    "FileDigest" => {
                        encryption_info
                            .set_file_digest_from_base64(&text)
                            .map_err(|e| PackageError::XmlError { reason: e })?;
                    }
                    "FileDigestAlgorithm" => {
                        encryption_info.file_digest_algorithm = text.to_string();
                    }
                    _ => {}
                }
            }
            Ok(Event::End(_)) => {
                current_element.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                return Err(PackageError::XmlError {
                    reason: format!("XML parse error: {}", e),
                });
            }
            _ => {}
        }
        buf.clear();
    }

    // Validate required fields
    if name.is_empty() {
        return Err(PackageError::XmlError {
            reason: "Missing Name element".to_string(),
        });
    }
    if setup_file.is_empty() {
        return Err(PackageError::XmlError {
            reason: "Missing SetupFile element".to_string(),
        });
    }

    Ok(DetectionMetadata {
        name,
        unencrypted_content_size,
        file_name,
        setup_file,
        encryption_info,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::detection::EncryptionInfo;

    #[test]
    fn test_generate_detection_xml_structure() {
        let mut metadata = DetectionMetadata::new("setup.exe".to_string(), 1024);
        metadata.encryption_info = EncryptionInfo::new();

        let xml = generate_detection_xml(&metadata).unwrap();

        // Should NOT have XML declaration (matches Microsoft format)
        assert!(!xml.starts_with("<?xml"));

        // Check root element with namespaces and ToolVersion
        assert!(xml.contains("<ApplicationInfo xmlns:xsd=\"http://www.w3.org/2001/XMLSchema\""));
        assert!(xml.contains("xmlns:xsi=\"http://www.w3.org/2001/XMLSchema-instance\""));
        assert!(xml.contains("ToolVersion=\"1.8.6.0\""));

        // Check required elements
        assert!(xml.contains("<Name>setup.exe</Name>"));
        assert!(xml.contains("<UnencryptedContentSize>1024</UnencryptedContentSize>"));
        assert!(xml.contains("<FileName>IntunePackage.intunewin</FileName>"));
        assert!(xml.contains("<SetupFile>setup.exe</SetupFile>"));

        // Check encryption info
        assert!(xml.contains("<EncryptionInfo>"));
        assert!(xml.contains("<EncryptionKey>"));
        assert!(xml.contains("<MacKey>"));
        assert!(xml.contains("<InitializationVector>"));
        assert!(xml.contains("<Mac>"));
        assert!(xml.contains("<ProfileIdentifier>ProfileVersion1</ProfileIdentifier>"));
        assert!(xml.contains("<FileDigest>"));
        assert!(xml.contains("<FileDigestAlgorithm>SHA256</FileDigestAlgorithm>"));
        assert!(xml.contains("</EncryptionInfo>"));
        assert!(xml.contains("</ApplicationInfo>"));
    }

    #[test]
    fn test_generate_detection_xml_element_order() {
        let mut metadata = DetectionMetadata::new("test.msi".to_string(), 2048);
        metadata.encryption_info = EncryptionInfo::new();

        let xml = generate_detection_xml(&metadata).unwrap();

        // Verify element order (critical for Intune compatibility)
        let name_pos = xml.find("<Name>").unwrap();
        let size_pos = xml.find("<UnencryptedContentSize>").unwrap();
        let filename_pos = xml.find("<FileName>").unwrap();
        let setup_pos = xml.find("<SetupFile>").unwrap();
        let enc_info_pos = xml.find("<EncryptionInfo>").unwrap();

        assert!(name_pos < size_pos);
        assert!(size_pos < filename_pos);
        assert!(filename_pos < setup_pos);
        assert!(setup_pos < enc_info_pos);

        // Verify encryption info element order
        let enc_key_pos = xml.find("<EncryptionKey>").unwrap();
        let mac_key_pos = xml.find("<MacKey>").unwrap();
        let iv_pos = xml.find("<InitializationVector>").unwrap();
        let mac_pos = xml.find("<Mac>").unwrap();
        let profile_pos = xml.find("<ProfileIdentifier>").unwrap();
        let digest_pos = xml.find("<FileDigest>").unwrap();
        let algo_pos = xml.find("<FileDigestAlgorithm>").unwrap();

        assert!(enc_key_pos < mac_key_pos);
        assert!(mac_key_pos < iv_pos);
        assert!(iv_pos < mac_pos);
        assert!(mac_pos < profile_pos);
        assert!(profile_pos < digest_pos);
        assert!(digest_pos < algo_pos);
    }

    #[test]
    fn test_generate_detection_xml_base64_lengths() {
        let mut metadata = DetectionMetadata::new("app.exe".to_string(), 512);
        metadata.encryption_info = EncryptionInfo::new();

        let xml = generate_detection_xml(&metadata).unwrap();

        // Extract Base64 values and verify lengths
        // 32 bytes = 44 chars Base64, 16 bytes = 24 chars Base64
        let enc_key_start = xml.find("<EncryptionKey>").unwrap() + "<EncryptionKey>".len();
        let enc_key_end = xml.find("</EncryptionKey>").unwrap();
        let enc_key = &xml[enc_key_start..enc_key_end];
        assert_eq!(enc_key.len(), 44); // 32 bytes -> 44 chars

        let iv_start = xml.find("<InitializationVector>").unwrap() + "<InitializationVector>".len();
        let iv_end = xml.find("</InitializationVector>").unwrap();
        let iv = &xml[iv_start..iv_end];
        assert_eq!(iv.len(), 24); // 16 bytes -> 24 chars
    }

    #[test]
    fn test_generate_detection_xml_formatting() {
        let mut metadata = DetectionMetadata::new("setup.exe".to_string(), 1024);
        metadata.encryption_info = EncryptionInfo::new();

        let xml = generate_detection_xml(&metadata).unwrap();

        // Should have CRLF line endings (Windows style, matches Microsoft format)
        assert!(xml.contains(">\r\n"));
        // Should NOT have bare LF (Unix style)
        assert!(!xml.contains(">\n<") || xml.contains(">\r\n<"));
        // Should have 2-space indentation
        assert!(xml.contains("  <Name>"));
        assert!(xml.contains("    <EncryptionKey>")); // Nested elements have 4 spaces
    }

    #[test]
    fn test_parse_detection_xml_roundtrip() {
        // Create metadata with known values
        let mut original = DetectionMetadata::new("setup.exe".to_string(), 2048);
        original.encryption_info.encryption_key = [1u8; 32];
        original.encryption_info.mac_key = [2u8; 32];
        original.encryption_info.iv = [3u8; 16];
        original.encryption_info.mac = [4u8; 32];
        original.encryption_info.file_digest = [5u8; 32];

        // Generate XML
        let xml = generate_detection_xml(&original).unwrap();

        // Parse it back
        let parsed = parse_detection_xml(&xml).unwrap();

        // Verify all fields match
        assert_eq!(parsed.name, original.name);
        assert_eq!(
            parsed.unencrypted_content_size,
            original.unencrypted_content_size
        );
        assert_eq!(parsed.file_name, original.file_name);
        assert_eq!(parsed.setup_file, original.setup_file);
        assert_eq!(
            parsed.encryption_info.encryption_key,
            original.encryption_info.encryption_key
        );
        assert_eq!(
            parsed.encryption_info.mac_key,
            original.encryption_info.mac_key
        );
        assert_eq!(parsed.encryption_info.iv, original.encryption_info.iv);
        assert_eq!(parsed.encryption_info.mac, original.encryption_info.mac);
        assert_eq!(
            parsed.encryption_info.file_digest,
            original.encryption_info.file_digest
        );
        assert_eq!(
            parsed.encryption_info.profile_identifier,
            original.encryption_info.profile_identifier
        );
        assert_eq!(
            parsed.encryption_info.file_digest_algorithm,
            original.encryption_info.file_digest_algorithm
        );
    }

    #[test]
    fn test_parse_detection_xml_missing_name() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
            <ApplicationInfo>
                <UnencryptedContentSize>1024</UnencryptedContentSize>
            </ApplicationInfo>"#;

        let result = parse_detection_xml(xml);
        assert!(matches!(result, Err(PackageError::XmlError { .. })));
    }

    #[test]
    fn test_parse_detection_xml_invalid_base64() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
            <ApplicationInfo>
                <Name>test.exe</Name>
                <SetupFile>test.exe</SetupFile>
                <EncryptionInfo>
                    <EncryptionKey>not-valid-base64!!!</EncryptionKey>
                </EncryptionInfo>
            </ApplicationInfo>"#;

        let result = parse_detection_xml(xml);
        assert!(matches!(result, Err(PackageError::XmlError { .. })));
    }
}
