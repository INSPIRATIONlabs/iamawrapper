//! XML document generation for macOS packages.
//!
//! Generates PackageInfo and Distribution XML files.

use crate::models::PackageError;

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
    use quick_xml::Writer;
    use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
    use std::io::Cursor;

    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    // XML declaration
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    // Write newline after declaration
    writer
        .write_event(Event::Text(BytesText::new("\n")))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    // <pkg-info> root element
    let mut pkg_info = BytesStart::new("pkg-info");
    pkg_info.push_attribute(("format-version", "2"));
    pkg_info.push_attribute(("identifier", identifier));
    pkg_info.push_attribute(("version", version));
    pkg_info.push_attribute(("install-location", install_location));
    pkg_info.push_attribute(("auth", "root"));

    writer
        .write_event(Event::Start(pkg_info))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    // <payload> element
    let mut payload = BytesStart::new("payload");
    payload.push_attribute(("installKBytes", install_kbytes.to_string().as_str()));
    payload.push_attribute(("numberOfFiles", num_files.to_string().as_str()));

    writer
        .write_event(Event::Empty(payload))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    // <scripts> element (if any scripts exist)
    if has_preinstall || has_postinstall {
        writer
            .write_event(Event::Start(BytesStart::new("scripts")))
            .map_err(|e| PackageError::XmlError {
                reason: e.to_string(),
            })?;

        if has_preinstall {
            let mut preinstall = BytesStart::new("preinstall");
            preinstall.push_attribute(("file", "./preinstall"));
            writer
                .write_event(Event::Empty(preinstall))
                .map_err(|e| PackageError::XmlError {
                    reason: e.to_string(),
                })?;
        }

        if has_postinstall {
            let mut postinstall = BytesStart::new("postinstall");
            postinstall.push_attribute(("file", "./postinstall"));
            writer
                .write_event(Event::Empty(postinstall))
                .map_err(|e| PackageError::XmlError {
                    reason: e.to_string(),
                })?;
        }

        writer
            .write_event(Event::End(BytesEnd::new("scripts")))
            .map_err(|e| PackageError::XmlError {
                reason: e.to_string(),
            })?;
    }

    // Close </pkg-info>
    writer
        .write_event(Event::End(BytesEnd::new("pkg-info")))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| PackageError::XmlError {
        reason: e.to_string(),
    })
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
    use quick_xml::Writer;
    use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
    use std::io::Cursor;

    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    // XML declaration
    writer
        .write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    // Write newline after declaration
    writer
        .write_event(Event::Text(BytesText::new("\n")))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    // <installer-gui-script> root element
    let mut root = BytesStart::new("installer-gui-script");
    root.push_attribute(("minSpecVersion", "1"));

    writer
        .write_event(Event::Start(root))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    // <title>
    writer
        .write_event(Event::Start(BytesStart::new("title")))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;
    writer
        .write_event(Event::Text(BytesText::new(title)))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;
    writer
        .write_event(Event::End(BytesEnd::new("title")))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    // <options>
    let mut options = BytesStart::new("options");
    options.push_attribute(("customize", "never"));
    options.push_attribute(("require-scripts", "false"));
    options.push_attribute(("hostArchitectures", "x86_64,arm64"));

    writer
        .write_event(Event::Empty(options))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    // <domains>
    let mut domains = BytesStart::new("domains");
    domains.push_attribute(("enable_anywhere", "false"));
    domains.push_attribute(("enable_currentUserHome", "false"));
    domains.push_attribute(("enable_localSystem", "true"));

    writer
        .write_event(Event::Empty(domains))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    // <choices-outline>
    writer
        .write_event(Event::Start(BytesStart::new("choices-outline")))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    let mut line = BytesStart::new("line");
    line.push_attribute(("choice", "default"));
    writer
        .write_event(Event::Empty(line))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    writer
        .write_event(Event::End(BytesEnd::new("choices-outline")))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    // <choice>
    let mut choice = BytesStart::new("choice");
    choice.push_attribute(("id", "default"));
    choice.push_attribute(("visible", "false"));
    choice.push_attribute(("title", title));

    writer
        .write_event(Event::Start(choice))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    // <pkg-ref> inside choice
    let mut pkg_ref_inner = BytesStart::new("pkg-ref");
    pkg_ref_inner.push_attribute(("id", identifier));
    writer
        .write_event(Event::Empty(pkg_ref_inner))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    writer
        .write_event(Event::End(BytesEnd::new("choice")))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    // <pkg-ref> with details
    let mut pkg_ref = BytesStart::new("pkg-ref");
    pkg_ref.push_attribute(("id", identifier));
    pkg_ref.push_attribute(("version", version));
    pkg_ref.push_attribute(("installKBytes", install_kbytes.to_string().as_str()));

    writer
        .write_event(Event::Start(pkg_ref))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    writer
        .write_event(Event::Text(BytesText::new("#base.pkg")))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    writer
        .write_event(Event::End(BytesEnd::new("pkg-ref")))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    // Close </installer-gui-script>
    writer
        .write_event(Event::End(BytesEnd::new("installer-gui-script")))
        .map_err(|e| PackageError::XmlError {
            reason: e.to_string(),
        })?;

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).map_err(|e| PackageError::XmlError {
        reason: e.to_string(),
    })
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
