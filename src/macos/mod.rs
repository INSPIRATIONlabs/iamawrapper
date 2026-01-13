//! macOS flat package (.pkg) creation module.
//!
//! This module provides functionality to create macOS installer packages
//! compatible with the macOS Installer application.

#[cfg(feature = "macos")]
pub mod bom;
#[cfg(feature = "macos")]
pub mod cpio;
#[cfg(feature = "macos")]
pub mod payload;
#[cfg(feature = "macos")]
pub mod xar;
#[cfg(feature = "macos")]
pub mod xml;

#[cfg(feature = "macos")]
use std::fs;
#[cfg(feature = "macos")]
use std::time::Instant;

#[cfg(feature = "macos")]
use crate::models::PackageError;
#[cfg(feature = "macos")]
use crate::models::macos::{MacosPkgRequest, MacosPkgResult};

/// Create a macOS flat package (.pkg) from the given request.
#[cfg(feature = "macos")]
pub fn package(request: MacosPkgRequest) -> Result<MacosPkgResult, PackageError> {
    let start = Instant::now();

    // Collect files from source folder
    let payload_data = payload::collect_files(&request.source_folder)?;
    let file_count = payload_data.files.len();

    // Check for scripts
    let (has_preinstall, has_postinstall, scripts_archive) =
        if let Some(ref scripts_folder) = request.scripts_folder {
            // Validate scripts folder exists
            if !scripts_folder.exists() {
                return Err(PackageError::ScriptsFolderNotFound {
                    path: scripts_folder.clone(),
                });
            }

            let scripts_info = payload::collect_scripts(scripts_folder)?;

            // Create scripts archive if any scripts found
            let archive = if scripts_info.has_preinstall || scripts_info.has_postinstall {
                Some(payload::create_scripts_archive(scripts_folder)?)
            } else {
                None
            };

            (
                scripts_info.has_preinstall,
                scripts_info.has_postinstall,
                archive,
            )
        } else {
            (false, false, None)
        };

    // Generate XML files
    let packageinfo_xml = xml::generate_packageinfo(
        &request.identifier,
        &request.version,
        request.install_location.to_str().unwrap_or("/"),
        payload_data.total_size / 1024, // Convert to KB
        file_count,
        has_preinstall,
        has_postinstall,
    )?;

    let distribution_xml = xml::generate_distribution(
        &request.identifier,
        &request.identifier, // Use identifier as title for now
        &request.version,
        payload_data.total_size / 1024,
    )?;

    // Create CPIO payload (gzip compressed)
    let payload_bytes = payload::create_payload(&request.source_folder)?;

    // Create BOM
    let bom_bytes = bom::create_bom_from_directory(&request.source_folder)?;

    // Build outer XAR archive (flat package structure)
    let mut outer_xar = xar::XarBuilder::new();
    outer_xar.add_file("Distribution", distribution_xml.into_bytes())?;
    outer_xar.add_directory("base.pkg")?;
    outer_xar.add_file("base.pkg/Bom", bom_bytes.clone())?;
    outer_xar.add_file("base.pkg/Payload", payload_bytes.clone())?;
    outer_xar.add_file("base.pkg/PackageInfo", packageinfo_xml.into_bytes())?;

    // Add scripts archive if present
    if let Some(scripts_bytes) = scripts_archive {
        outer_xar.add_file("base.pkg/Scripts", scripts_bytes)?;
    }

    // Write to bytes
    use std::io::Cursor;
    let mut pkg_data = Cursor::new(Vec::new());
    outer_xar.finish(&mut pkg_data)?;
    let pkg_data = pkg_data.into_inner();

    // Ensure output directory exists
    if let Some(parent) = request.output_path().parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| PackageError::OutputFolderCreationFailed {
                path: parent.to_path_buf(),
                reason: e.to_string(),
            })?;
        }
    }

    // Write output file
    let output_path = request.output_path();
    fs::write(&output_path, &pkg_data).map_err(|e| PackageError::OutputWriteError {
        path: output_path.clone(),
        reason: e.to_string(),
    })?;

    let creation_time = start.elapsed();
    let package_size = pkg_data.len() as u64;

    Ok(MacosPkgResult {
        output_path,
        package_size,
        file_count,
        creation_time,
    })
}
