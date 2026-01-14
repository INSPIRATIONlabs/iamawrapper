//! XML document generation for macOS packages.
//!
//! Generates PackageInfo and Distribution XML files.

use crate::models::PackageError;
use quick_xml::Writer;
use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use std::io::Cursor;

type XmlWriter = Writer<Cursor<Vec<u8>>>;

/// Convert any error to PackageError::XmlError.
fn xml_err<E: std::fmt::Display>(e: E) -> PackageError {
    PackageError::XmlError {
        reason: e.to_string(),
    }
}

/// Write an event to the XML writer.
fn write(writer: &mut XmlWriter, event: Event<'_>) -> Result<(), PackageError> {
    writer.write_event(event).map_err(xml_err)
}

/// Write a text element: <tag>content</tag>
fn write_text_element(
    writer: &mut XmlWriter,
    tag: &str,
    content: &str,
) -> Result<(), PackageError> {
    write(writer, Event::Start(BytesStart::new(tag)))?;
    write(writer, Event::Text(BytesText::new(content)))?;
    write(writer, Event::End(BytesEnd::new(tag)))
}

/// Write an empty element with a single attribute: <tag attr="value"/>
fn write_empty_element(
    writer: &mut XmlWriter,
    tag: &str,
    attr: &str,
    value: &str,
) -> Result<(), PackageError> {
    let mut elem = BytesStart::new(tag.to_owned());
    elem.push_attribute((attr, value));
    write(writer, Event::Empty(elem))
}

/// Finalize the writer and convert to String.
fn finalize(writer: XmlWriter) -> Result<String, PackageError> {
    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(xml_err)
}

/// Create a new XML writer and write the XML declaration with trailing newline.
fn create_xml_writer() -> Result<XmlWriter, PackageError> {
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);
    write(
        &mut writer,
        Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)),
    )?;
    write(&mut writer, Event::Text(BytesText::new("\n")))?;
    Ok(writer)
}

/// Generate PackageInfo XML document.
///
/// # Arguments
/// * `identifier` - Package identifier (e.g., "com.company.app")
/// * `version` - Package version (e.g., "1.0.0")
/// * `install_location` - Installation target path
/// * `install_kbytes` - Total size in KB
/// * `num_files` - Number of files
/// * `has_preinstall` - Whether preinstall script exists
/// * `has_postinstall` - Whether postinstall script exists
pub fn generate_packageinfo(
    identifier: &str,
    version: &str,
    install_location: &str,
    install_kbytes: u64,
    num_files: usize,
    has_preinstall: bool,
    has_postinstall: bool,
) -> Result<String, PackageError> {
    let mut writer = create_xml_writer()?;

    // <pkg-info> root element
    let mut pkg_info = BytesStart::new("pkg-info");
    pkg_info.push_attribute(("format-version", "2"));
    pkg_info.push_attribute(("identifier", identifier));
    pkg_info.push_attribute(("version", version));
    pkg_info.push_attribute(("install-location", install_location));
    pkg_info.push_attribute(("auth", "root"));
    write(&mut writer, Event::Start(pkg_info))?;

    // <payload> element
    let mut payload = BytesStart::new("payload");
    payload.push_attribute(("installKBytes", install_kbytes.to_string().as_str()));
    payload.push_attribute(("numberOfFiles", num_files.to_string().as_str()));
    write(&mut writer, Event::Empty(payload))?;

    // <scripts> element (if any scripts exist)
    if has_preinstall || has_postinstall {
        write(&mut writer, Event::Start(BytesStart::new("scripts")))?;

        if has_preinstall {
            write_empty_element(&mut writer, "preinstall", "file", "./preinstall")?;
        }
        if has_postinstall {
            write_empty_element(&mut writer, "postinstall", "file", "./postinstall")?;
        }

        write(&mut writer, Event::End(BytesEnd::new("scripts")))?;
    }

    write(&mut writer, Event::End(BytesEnd::new("pkg-info")))?;
    finalize(writer)
}

/// Generate Distribution XML document.
///
/// # Arguments
/// * `identifier` - Package identifier
/// * `title` - Package title for installer UI
/// * `version` - Package version
/// * `install_kbytes` - Total size in KB
pub fn generate_distribution(
    identifier: &str,
    title: &str,
    version: &str,
    install_kbytes: u64,
) -> Result<String, PackageError> {
    let mut writer = create_xml_writer()?;

    // <installer-gui-script> root element
    let mut root = BytesStart::new("installer-gui-script");
    root.push_attribute(("minSpecVersion", "1"));
    write(&mut writer, Event::Start(root))?;

    // <title>
    write_text_element(&mut writer, "title", title)?;

    // <options>
    let mut options = BytesStart::new("options");
    options.push_attribute(("customize", "never"));
    options.push_attribute(("require-scripts", "false"));
    options.push_attribute(("hostArchitectures", "x86_64,arm64"));
    write(&mut writer, Event::Empty(options))?;

    // <domains>
    let mut domains = BytesStart::new("domains");
    domains.push_attribute(("enable_anywhere", "false"));
    domains.push_attribute(("enable_currentUserHome", "false"));
    domains.push_attribute(("enable_localSystem", "true"));
    write(&mut writer, Event::Empty(domains))?;

    // <choices-outline>
    write(
        &mut writer,
        Event::Start(BytesStart::new("choices-outline")),
    )?;
    write_empty_element(&mut writer, "line", "choice", "default")?;
    write(&mut writer, Event::End(BytesEnd::new("choices-outline")))?;

    // <choice>
    let mut choice = BytesStart::new("choice");
    choice.push_attribute(("id", "default"));
    choice.push_attribute(("visible", "false"));
    choice.push_attribute(("title", title));
    write(&mut writer, Event::Start(choice))?;
    write_empty_element(&mut writer, "pkg-ref", "id", identifier)?;
    write(&mut writer, Event::End(BytesEnd::new("choice")))?;

    // <pkg-ref> with details
    let mut pkg_ref = BytesStart::new("pkg-ref");
    pkg_ref.push_attribute(("id", identifier));
    pkg_ref.push_attribute(("version", version));
    pkg_ref.push_attribute(("installKBytes", install_kbytes.to_string().as_str()));
    write(&mut writer, Event::Start(pkg_ref))?;
    write(&mut writer, Event::Text(BytesText::new("#base.pkg")))?;
    write(&mut writer, Event::End(BytesEnd::new("pkg-ref")))?;

    write(
        &mut writer,
        Event::End(BytesEnd::new("installer-gui-script")),
    )?;
    finalize(writer)
}

#[cfg(test)]
mod tests {
    use super::*;

    // T011: PackageInfo XML tests
    #[test]
    fn test_packageinfo_xml_declaration() {
        let xml =
            generate_packageinfo("com.test.app", "1.0.0", "/", 1024, 10, false, false).unwrap();
        assert!(
            xml.starts_with("<?xml"),
            "PackageInfo must start with XML declaration"
        );
    }

    #[test]
    fn test_packageinfo_root_element() {
        let xml =
            generate_packageinfo("com.test.app", "1.0.0", "/", 1024, 10, false, false).unwrap();
        assert!(
            xml.contains("<pkg-info"),
            "Must have <pkg-info> root element"
        );
        assert!(xml.contains("</pkg-info>"), "Must close </pkg-info>");
    }

    #[test]
    fn test_packageinfo_format_version() {
        let xml =
            generate_packageinfo("com.test.app", "1.0.0", "/", 1024, 10, false, false).unwrap();
        assert!(
            xml.contains("format-version=\"2\""),
            "format-version must be 2"
        );
    }

    #[test]
    fn test_packageinfo_identifier() {
        let xml = generate_packageinfo("com.example.myapp", "1.0.0", "/", 1024, 10, false, false)
            .unwrap();
        assert!(
            xml.contains("identifier=\"com.example.myapp\""),
            "identifier must be present"
        );
    }

    #[test]
    fn test_packageinfo_version() {
        let xml =
            generate_packageinfo("com.test.app", "2.5.3", "/", 1024, 10, false, false).unwrap();
        assert!(xml.contains("version=\"2.5.3\""), "version must be present");
    }

    #[test]
    fn test_packageinfo_install_location() {
        let xml = generate_packageinfo(
            "com.test.app",
            "1.0.0",
            "/Applications",
            1024,
            10,
            false,
            false,
        )
        .unwrap();
        assert!(
            xml.contains("install-location=\"/Applications\""),
            "install-location must be present"
        );
    }

    #[test]
    fn test_packageinfo_auth() {
        let xml =
            generate_packageinfo("com.test.app", "1.0.0", "/", 1024, 10, false, false).unwrap();
        assert!(xml.contains("auth=\"root\""), "auth must be 'root'");
    }

    #[test]
    fn test_packageinfo_payload() {
        let xml =
            generate_packageinfo("com.test.app", "1.0.0", "/", 2048, 25, false, false).unwrap();
        assert!(xml.contains("<payload"), "Must have <payload> element");
        assert!(
            xml.contains("installKBytes=\"2048\""),
            "installKBytes must match"
        );
        assert!(
            xml.contains("numberOfFiles=\"25\""),
            "numberOfFiles must match"
        );
    }

    #[test]
    fn test_packageinfo_without_scripts() {
        let xml =
            generate_packageinfo("com.test.app", "1.0.0", "/", 1024, 10, false, false).unwrap();
        assert!(
            !xml.contains("<scripts>"),
            "Should not have <scripts> without scripts"
        );
    }

    #[test]
    fn test_packageinfo_with_preinstall() {
        let xml =
            generate_packageinfo("com.test.app", "1.0.0", "/", 1024, 10, true, false).unwrap();
        assert!(
            xml.contains("<scripts>"),
            "Must have <scripts> with preinstall"
        );
        assert!(
            xml.contains("<preinstall"),
            "Must have <preinstall> element"
        );
    }

    #[test]
    fn test_packageinfo_with_postinstall() {
        let xml =
            generate_packageinfo("com.test.app", "1.0.0", "/", 1024, 10, false, true).unwrap();
        assert!(
            xml.contains("<scripts>"),
            "Must have <scripts> with postinstall"
        );
        assert!(
            xml.contains("<postinstall"),
            "Must have <postinstall> element"
        );
    }

    #[test]
    fn test_packageinfo_with_both_scripts() {
        let xml = generate_packageinfo("com.test.app", "1.0.0", "/", 1024, 10, true, true).unwrap();
        assert!(xml.contains("<preinstall"), "Must have <preinstall>");
        assert!(xml.contains("<postinstall"), "Must have <postinstall>");
    }

    // T012: Distribution XML tests
    #[test]
    fn test_distribution_xml_declaration() {
        let xml = generate_distribution("com.test.app", "My App", "1.0.0", 1024).unwrap();
        assert!(
            xml.starts_with("<?xml"),
            "Distribution must start with XML declaration"
        );
    }

    #[test]
    fn test_distribution_root_element() {
        let xml = generate_distribution("com.test.app", "My App", "1.0.0", 1024).unwrap();
        assert!(
            xml.contains("<installer-gui-script") || xml.contains("<installer-script"),
            "Must have installer script root element"
        );
    }

    #[test]
    fn test_distribution_min_spec_version() {
        let xml = generate_distribution("com.test.app", "My App", "1.0.0", 1024).unwrap();
        assert!(xml.contains("minSpecVersion"), "Must have minSpecVersion");
    }

    #[test]
    fn test_distribution_title() {
        let xml = generate_distribution("com.test.app", "My Amazing App", "1.0.0", 1024).unwrap();
        assert!(xml.contains("<title>"), "Must have <title> element");
        assert!(
            xml.contains("My Amazing App"),
            "Title must contain app name"
        );
    }

    #[test]
    fn test_distribution_options() {
        let xml = generate_distribution("com.test.app", "My App", "1.0.0", 1024).unwrap();
        assert!(xml.contains("<options"), "Must have <options> element");
    }

    #[test]
    fn test_distribution_domains() {
        let xml = generate_distribution("com.test.app", "My App", "1.0.0", 1024).unwrap();
        assert!(xml.contains("<domains"), "Must have <domains> element");
    }

    #[test]
    fn test_distribution_choices_outline() {
        let xml = generate_distribution("com.test.app", "My App", "1.0.0", 1024).unwrap();
        assert!(
            xml.contains("<choices-outline>"),
            "Must have <choices-outline>"
        );
        assert!(xml.contains("<line"), "Must have <line> elements");
    }

    #[test]
    fn test_distribution_choice() {
        let xml = generate_distribution("com.test.app", "My App", "1.0.0", 1024).unwrap();
        assert!(xml.contains("<choice"), "Must have <choice> element");
        assert!(xml.contains("<pkg-ref"), "Choice must reference pkg-ref");
    }

    #[test]
    fn test_distribution_pkg_ref() {
        let xml = generate_distribution("com.example.myapp", "My App", "2.0.0", 2048).unwrap();
        assert!(xml.contains("<pkg-ref"), "Must have <pkg-ref>");
        assert!(
            xml.contains("id=\"com.example.myapp\"") || xml.contains("id='com.example.myapp'"),
            "pkg-ref must have identifier"
        );
    }

    #[test]
    fn test_distribution_pkg_ref_size() {
        let xml = generate_distribution("com.test.app", "My App", "1.0.0", 5000).unwrap();
        assert!(
            xml.contains("installKBytes=\"5000\"") || xml.contains("installKBytes='5000'"),
            "pkg-ref must have installKBytes"
        );
    }

    #[test]
    fn test_distribution_base_pkg_reference() {
        let xml = generate_distribution("com.test.app", "My App", "1.0.0", 1024).unwrap();
        assert!(
            xml.contains("#base.pkg"),
            "pkg-ref must reference #base.pkg"
        );
    }
}
