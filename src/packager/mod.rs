//! Package creation and extraction module.

pub mod archive;
pub mod encrypt;
pub mod metadata;

use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read as IoRead, Write};
use std::path::Path;
use std::time::Instant;

use indicatif::{ProgressBar, ProgressStyle};
use zip::ZipWriter;
use zip::read::ZipArchive;
use zip::write::FileOptions;

use crate::models::detection::DetectionMetadata;
use crate::models::error::{PackageError, PackageResult};
use crate::models::package::{
    IntuneWinPackage, PackageRequest, SourcePackage, UnpackRequest, UnpackResult, Verbosity,
};

use self::archive::collect_source_files;
use self::encrypt::{decrypt_content, encrypt_content};
use self::metadata::{generate_detection_xml, parse_detection_xml};

/// Create an IntuneWin package from the given request.
pub fn package(request: &PackageRequest) -> PackageResult<IntuneWinPackage> {
    let start_time = Instant::now();

    // Validate request
    request.validate()?;

    // Collect source files
    let source_package = collect_source_files(&request.source_folder, &request.setup_file)?;

    if source_package.files.is_empty() {
        return Err(PackageError::SourceFolderEmpty {
            path: request.source_folder.clone(),
        });
    }

    // Create output folder if needed
    if !request.output_folder.exists() {
        fs::create_dir_all(&request.output_folder).map_err(|e| {
            PackageError::OutputFolderCreationFailed {
                path: request.output_folder.clone(),
                reason: e.to_string(),
            }
        })?;
    }

    // Check if output file exists
    let output_path = request.output_path();
    if output_path.exists() && !request.verbosity.suppress_prompts() {
        return Err(PackageError::OutputFileExists { path: output_path });
    }

    // Create progress bar
    let progress = create_progress_bar(&source_package, request.verbosity);

    // Create inner ZIP (content to be encrypted)
    let inner_zip = create_inner_zip(&source_package, &progress)?;
    let unencrypted_size = inner_zip.len() as u64;

    progress.set_message("Encrypting...");

    // Encrypt the inner ZIP
    let (encrypted_content, encryption_info) = encrypt_content(&inner_zip)?;

    progress.set_message("Writing package...");

    // Create detection metadata
    let mut metadata = DetectionMetadata::new(request.setup_file.clone(), unencrypted_size);
    metadata.encryption_info = encryption_info;

    // Generate Detection.xml
    let detection_xml = generate_detection_xml(&metadata)?;

    // Create outer ZIP (final .intunewin file)
    create_outer_zip(&output_path, &detection_xml, &encrypted_content)?;

    progress.finish_with_message("Done!");

    let package_size = fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

    Ok(IntuneWinPackage {
        output_path,
        metadata,
        package_size,
        creation_time: start_time.elapsed(),
    })
}

fn create_progress_bar(source: &SourcePackage, verbosity: Verbosity) -> ProgressBar {
    if verbosity.suppress_output() {
        return ProgressBar::hidden();
    }

    let pb = ProgressBar::new(source.file_count() as u64);

    if verbosity.show_progress() {
        pb.set_style(
            ProgressStyle::default_bar()
                .template(
                    "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}",
                )
                .unwrap()
                .progress_chars("#>-"),
        );
    }

    pb
}

fn create_inner_zip(source: &SourcePackage, progress: &ProgressBar) -> PackageResult<Vec<u8>> {
    let mut buffer = Vec::new();
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buffer));
        let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);

        for file in &source.files {
            let full_path = source.root.join(&file.relative_path);

            // Use forward slashes for ZIP paths (cross-platform)
            let zip_path = file.relative_path.to_string_lossy().replace('\\', "/");

            progress.set_message(format!("Adding {}", zip_path));

            zip.start_file(&zip_path, options)
                .map_err(|e| PackageError::ZipError {
                    reason: e.to_string(),
                })?;

            let content = fs::read(&full_path).map_err(|e| PackageError::SourceReadError {
                path: full_path.clone(),
                reason: e.to_string(),
            })?;

            zip.write_all(&content)
                .map_err(|e| PackageError::ZipError {
                    reason: e.to_string(),
                })?;

            progress.inc(1);
        }

        zip.finish().map_err(|e| PackageError::ZipError {
            reason: e.to_string(),
        })?;
    }

    Ok(buffer)
}

fn create_outer_zip(
    output_path: &Path,
    detection_xml: &str,
    encrypted_content: &[u8],
) -> PackageResult<()> {
    let file = File::create(output_path).map_err(|e| PackageError::OutputWriteError {
        path: output_path.to_path_buf(),
        reason: e.to_string(),
    })?;

    let mut zip = ZipWriter::new(BufWriter::new(file));
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Stored);

    // Add encrypted content first (matches Microsoft file order)
    zip.start_file("IntuneWinPackage/Contents/IntunePackage.intunewin", options)
        .map_err(|e| PackageError::ZipError {
            reason: e.to_string(),
        })?;
    zip.write_all(encrypted_content)
        .map_err(|e| PackageError::ZipError {
            reason: e.to_string(),
        })?;

    // Add Detection.xml second
    zip.start_file("IntuneWinPackage/Metadata/Detection.xml", options)
        .map_err(|e| PackageError::ZipError {
            reason: e.to_string(),
        })?;
    zip.write_all(detection_xml.as_bytes())
        .map_err(|e| PackageError::ZipError {
            reason: e.to_string(),
        })?;

    zip.finish().map_err(|e| PackageError::ZipError {
        reason: e.to_string(),
    })?;

    Ok(())
}

/// Unpack an IntuneWin package to extract the original files.
pub fn unpack(request: &UnpackRequest) -> PackageResult<UnpackResult> {
    let start_time = Instant::now();

    // Validate request
    request.validate()?;

    // Create output folder if needed
    if !request.output_folder.exists() {
        fs::create_dir_all(&request.output_folder).map_err(|e| {
            PackageError::OutputFolderCreationFailed {
                path: request.output_folder.clone(),
                reason: e.to_string(),
            }
        })?;
    }

    // Open the outer ZIP
    let file = File::open(&request.input_file).map_err(|e| PackageError::InvalidIntunewinFile {
        path: request.input_file.clone(),
        reason: format!("Failed to open file: {}", e),
    })?;

    let mut archive =
        ZipArchive::new(BufReader::new(file)).map_err(|e| PackageError::InvalidIntunewinFile {
            path: request.input_file.clone(),
            reason: format!("Invalid ZIP archive: {}", e),
        })?;

    // Extract and parse Detection.xml
    let metadata = extract_detection_metadata(&mut archive, &request.input_file)?;

    // Extract encrypted content
    let encrypted_content = extract_encrypted_content(&mut archive, &request.input_file)?;

    // Create progress bar
    let progress = create_unpack_progress_bar(request.verbosity);
    progress.set_message("Decrypting...");

    // Decrypt the inner ZIP
    let decrypted_content = decrypt_content(&encrypted_content, &metadata.encryption_info)?;

    progress.set_message("Extracting files...");

    // Extract inner ZIP to output folder
    let (file_count, total_size) =
        extract_inner_zip(&decrypted_content, &request.output_folder, &progress)?;

    progress.finish_with_message("Done!");

    Ok(UnpackResult {
        output_folder: request.output_folder.clone(),
        file_count,
        total_size,
        unpack_time: start_time.elapsed(),
        setup_file: metadata.setup_file,
    })
}

fn create_unpack_progress_bar(verbosity: Verbosity) -> ProgressBar {
    if verbosity.suppress_output() {
        return ProgressBar::hidden();
    }

    let pb = ProgressBar::new_spinner();

    if verbosity.show_progress() {
        pb.set_style(
            ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap(),
        );
    }

    pb
}

fn extract_detection_metadata<R: IoRead + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    input_path: &Path,
) -> PackageResult<DetectionMetadata> {
    let mut detection_file = archive
        .by_name("IntuneWinPackage/Metadata/Detection.xml")
        .map_err(|e| PackageError::InvalidIntunewinFile {
            path: input_path.to_path_buf(),
            reason: format!("Missing Detection.xml: {}", e),
        })?;

    let mut xml_content = String::new();
    detection_file
        .read_to_string(&mut xml_content)
        .map_err(|e| PackageError::InvalidIntunewinFile {
            path: input_path.to_path_buf(),
            reason: format!("Failed to read Detection.xml: {}", e),
        })?;

    parse_detection_xml(&xml_content)
}

fn extract_encrypted_content<R: IoRead + std::io::Seek>(
    archive: &mut ZipArchive<R>,
    input_path: &Path,
) -> PackageResult<Vec<u8>> {
    let mut content_file = archive
        .by_name("IntuneWinPackage/Contents/IntunePackage.intunewin")
        .map_err(|e| PackageError::InvalidIntunewinFile {
            path: input_path.to_path_buf(),
            reason: format!("Missing encrypted content: {}", e),
        })?;

    let mut encrypted_content = Vec::new();
    content_file
        .read_to_end(&mut encrypted_content)
        .map_err(|e| PackageError::InvalidIntunewinFile {
            path: input_path.to_path_buf(),
            reason: format!("Failed to read encrypted content: {}", e),
        })?;

    Ok(encrypted_content)
}

fn extract_inner_zip(
    decrypted_content: &[u8],
    output_folder: &Path,
    progress: &ProgressBar,
) -> PackageResult<(usize, u64)> {
    let cursor = std::io::Cursor::new(decrypted_content);
    let mut archive = ZipArchive::new(cursor).map_err(|e| PackageError::DecryptionError {
        reason: format!("Decrypted content is not a valid ZIP: {}", e),
    })?;

    let mut file_count = 0;
    let mut total_size = 0u64;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| PackageError::ZipError {
            reason: format!("Failed to read file from archive: {}", e),
        })?;

        let file_name = file.name().to_string();

        // Skip directories
        if file_name.ends_with('/') {
            continue;
        }

        progress.set_message(format!("Extracting {}", file_name));

        let output_path = output_folder.join(&file_name);

        // Create parent directories if needed
        if let Some(parent) = output_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    PackageError::OutputFolderCreationFailed {
                        path: parent.to_path_buf(),
                        reason: e.to_string(),
                    }
                })?;
            }
        }

        // Extract file
        let mut outfile =
            File::create(&output_path).map_err(|e| PackageError::OutputWriteError {
                path: output_path.clone(),
                reason: e.to_string(),
            })?;

        let bytes_written =
            std::io::copy(&mut file, &mut outfile).map_err(|e| PackageError::OutputWriteError {
                path: output_path.clone(),
                reason: e.to_string(),
            })?;

        file_count += 1;
        total_size += bytes_written;
    }

    Ok((file_count, total_size))
}
