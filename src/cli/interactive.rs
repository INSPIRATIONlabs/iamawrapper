//! Interactive mode prompts.

use std::fs;
use std::path::PathBuf;

use dialoguer::{Confirm, Input, Select};

use crate::models::error::{PackageError, PackageResult};
#[cfg(feature = "macos")]
use crate::models::macos::MacosPkgRequest;
use crate::models::package::{PackageRequest, Verbosity};

/// Target platform for package creation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Platform {
    /// Microsoft Intune (.intunewin)
    Intune = 0,
    /// macOS flat package (.pkg)
    MacOS = 1,
}

/// Result of interactive prompts - either an Intune or macOS package request.
pub enum InteractiveResult {
    /// Intune package request
    Intune(PackageRequest),
    /// macOS package request
    #[cfg(feature = "macos")]
    MacOS(MacosPkgRequest),
}

/// Get the list of platform options for display.
pub fn platform_options() -> Vec<&'static str> {
    vec!["Microsoft Intune (.intunewin)", "macOS Flat Package (.pkg)"]
}

/// Validate a macOS package identifier (reverse-DNS format).
pub fn validate_identifier(identifier: &str) -> Result<(), String> {
    if identifier.is_empty() {
        return Err("Identifier cannot be empty".to_string());
    }

    let parts: Vec<&str> = identifier.split('.').collect();

    // Must have at least two segments (e.g., com.example)
    if parts.len() < 2 {
        return Err("Identifier must be in reverse-DNS format (e.g., com.example.app)".to_string());
    }

    // Each segment must be non-empty and start with a letter
    for part in &parts {
        if part.is_empty() {
            return Err("Identifier segments cannot be empty".to_string());
        }
        let first_char = part.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() {
            return Err("Each identifier segment must start with a letter".to_string());
        }
    }

    Ok(())
}

/// Validate a version string (must be numeric with dots).
pub fn validate_version(version: &str) -> Result<(), String> {
    if version.is_empty() {
        return Err("Version cannot be empty".to_string());
    }

    // Split by dots and verify each part is numeric
    for part in version.split('.') {
        if part.is_empty() {
            return Err("Version segments cannot be empty".to_string());
        }
        if part.parse::<u32>().is_err() {
            return Err("Version must contain only numbers and dots (e.g., 1.0.0)".to_string());
        }
    }

    Ok(())
}

/// Prompt for platform selection.
pub fn prompt_platform() -> PackageResult<Platform> {
    let options = platform_options();

    let selection = Select::new()
        .with_prompt("Select package type")
        .items(&options)
        .default(0)
        .interact()
        .map_err(|e| PackageError::Io(std::io::Error::other(e)))?;

    match selection {
        0 => Ok(Platform::Intune),
        1 => Ok(Platform::MacOS),
        _ => unreachable!(),
    }
}

/// Run interactive mode with platform selection and return the appropriate request.
pub fn run_interactive_with_platform() -> PackageResult<InteractiveResult> {
    println!("iamawrapper v{}", env!("CARGO_PKG_VERSION"));
    println!("Interactive Mode\n");

    let platform = prompt_platform()?;
    println!();

    match platform {
        Platform::Intune => {
            let request = run_interactive_intune()?;
            Ok(InteractiveResult::Intune(request))
        }
        #[cfg(feature = "macos")]
        Platform::MacOS => {
            let request = run_interactive_macos()?;
            Ok(InteractiveResult::MacOS(request))
        }
        #[cfg(not(feature = "macos"))]
        Platform::MacOS => Err(PackageError::InvalidArgument {
            reason: "macOS packaging is not enabled. Build with --features macos".to_string(),
        }),
    }
}

/// Run interactive mode for Intune packages (legacy).
pub fn run_interactive() -> PackageResult<PackageRequest> {
    run_interactive_intune()
}

/// Run interactive mode for Intune package creation.
fn run_interactive_intune() -> PackageResult<PackageRequest> {
    println!("Microsoft Intune Package (.intunewin)\n");

    // Prompt for source folder
    let source_folder = prompt_source_folder()?;

    // List files and prompt for setup file
    let setup_file = prompt_setup_file(&source_folder)?;

    // Prompt for output folder
    let output_folder = prompt_output_folder()?;

    // Check if output file exists
    let request = PackageRequest::new(
        source_folder.clone(),
        setup_file.clone(),
        output_folder.clone(),
    );
    let output_path = request.output_path();

    if output_path.exists() {
        let overwrite = Confirm::new()
            .with_prompt(format!(
                "Output file {} already exists. Overwrite?",
                output_path.display()
            ))
            .default(false)
            .interact()
            .map_err(|e| PackageError::Io(std::io::Error::other(e)))?;

        if !overwrite {
            return Err(PackageError::Cancelled);
        }
    }

    // Show summary and confirm
    println!("\nPackage Summary:");
    println!("  Source folder: {}", source_folder.display());
    println!("  Setup file: {}", setup_file);
    println!("  Output: {}", output_path.display());

    let proceed = Confirm::new()
        .with_prompt("Proceed with packaging?")
        .default(true)
        .interact()
        .map_err(|e| PackageError::Io(std::io::Error::other(e)))?;

    if !proceed {
        return Err(PackageError::Cancelled);
    }

    println!();

    Ok(
        PackageRequest::new(source_folder, setup_file, output_folder)
            .with_verbosity(Verbosity::Normal),
    )
}

/// Run interactive mode for macOS package creation.
#[cfg(feature = "macos")]
pub fn run_interactive_macos() -> PackageResult<MacosPkgRequest> {
    println!("macOS Flat Package (.pkg)\n");

    // Prompt for source folder
    let source_folder = prompt_source_folder()?;

    // Prompt for identifier
    let identifier = prompt_macos_identifier()?;

    // Prompt for version
    let version = prompt_macos_version()?;

    // Prompt for output folder
    let output_folder = prompt_output_folder()?;

    // Prompt for optional install location
    let install_location = prompt_macos_install_location()?;

    // Prompt for optional scripts folder
    let scripts_folder = prompt_macos_scripts_folder()?;

    // Create request
    let mut request = MacosPkgRequest::new(
        source_folder.clone(),
        identifier.clone(),
        version.clone(),
        output_folder.clone(),
    )
    .with_install_location(install_location.clone())
    .with_verbosity(Verbosity::Normal);

    if let Some(scripts) = &scripts_folder {
        request = request.with_scripts_folder(scripts.clone());
    }

    let output_path = request.output_path();

    // Check if output file exists
    if output_path.exists() {
        let overwrite = Confirm::new()
            .with_prompt(format!(
                "Output file {} already exists. Overwrite?",
                output_path.display()
            ))
            .default(false)
            .interact()
            .map_err(|e| PackageError::Io(std::io::Error::other(e)))?;

        if !overwrite {
            return Err(PackageError::Cancelled);
        }
    }

    // Show summary and confirm
    println!("\nPackage Summary:");
    println!("  Source folder: {}", source_folder.display());
    println!("  Identifier: {}", identifier);
    println!("  Version: {}", version);
    println!("  Install location: {}", install_location.display());
    if let Some(scripts) = &scripts_folder {
        println!("  Scripts folder: {}", scripts.display());
    }
    println!("  Output: {}", output_path.display());

    let proceed = Confirm::new()
        .with_prompt("Proceed with packaging?")
        .default(true)
        .interact()
        .map_err(|e| PackageError::Io(std::io::Error::other(e)))?;

    if !proceed {
        return Err(PackageError::Cancelled);
    }

    println!();

    Ok(request)
}

/// Prompt for macOS package identifier.
#[cfg(feature = "macos")]
fn prompt_macos_identifier() -> PackageResult<String> {
    loop {
        let input: String = Input::new()
            .with_prompt("Package identifier (e.g., com.company.app)")
            .interact_text()
            .map_err(|e| PackageError::Io(std::io::Error::other(e)))?;

        let identifier = input.trim().to_string();

        if let Err(msg) = validate_identifier(&identifier) {
            eprintln!("Error: {}", msg);
            continue;
        }

        return Ok(identifier);
    }
}

/// Prompt for macOS package version.
#[cfg(feature = "macos")]
fn prompt_macos_version() -> PackageResult<String> {
    loop {
        let input: String = Input::new()
            .with_prompt("Package version (e.g., 1.0.0)")
            .interact_text()
            .map_err(|e| PackageError::Io(std::io::Error::other(e)))?;

        let version = input.trim().to_string();

        if let Err(msg) = validate_version(&version) {
            eprintln!("Error: {}", msg);
            continue;
        }

        return Ok(version);
    }
}

/// Prompt for macOS package install location.
#[cfg(feature = "macos")]
fn prompt_macos_install_location() -> PackageResult<PathBuf> {
    let input: String = Input::new()
        .with_prompt("Install location (press Enter for /Applications)")
        .default("/Applications".to_string())
        .interact_text()
        .map_err(|e| PackageError::Io(std::io::Error::other(e)))?;

    Ok(PathBuf::from(input.trim()))
}

/// Prompt for optional scripts folder.
#[cfg(feature = "macos")]
fn prompt_macos_scripts_folder() -> PackageResult<Option<PathBuf>> {
    let use_scripts = Confirm::new()
        .with_prompt("Include installation scripts?")
        .default(false)
        .interact()
        .map_err(|e| PackageError::Io(std::io::Error::other(e)))?;

    if !use_scripts {
        return Ok(None);
    }

    loop {
        let input: String = Input::new()
            .with_prompt("Scripts folder path")
            .interact_text()
            .map_err(|e| PackageError::Io(std::io::Error::other(e)))?;

        let path = PathBuf::from(input.trim());

        if !path.exists() {
            eprintln!("Error: Folder does not exist: {}", path.display());
            continue;
        }

        if !path.is_dir() {
            eprintln!("Error: Path is not a directory: {}", path.display());
            continue;
        }

        // Check for preinstall or postinstall
        let has_preinstall = path.join("preinstall").exists();
        let has_postinstall = path.join("postinstall").exists();

        if !has_preinstall && !has_postinstall {
            eprintln!("Warning: No preinstall or postinstall scripts found in folder");
            let proceed = Confirm::new()
                .with_prompt("Continue anyway?")
                .default(false)
                .interact()
                .map_err(|e| PackageError::Io(std::io::Error::other(e)))?;

            if !proceed {
                continue;
            }
        }

        return Ok(Some(path));
    }
}

fn prompt_source_folder() -> PackageResult<PathBuf> {
    loop {
        let input: String = Input::new()
            .with_prompt("Source folder path")
            .interact_text()
            .map_err(|e| PackageError::Io(std::io::Error::other(e)))?;

        let path = PathBuf::from(input.trim());

        if !path.exists() {
            eprintln!("Error: Folder does not exist: {}", path.display());
            continue;
        }

        if !path.is_dir() {
            eprintln!("Error: Path is not a directory: {}", path.display());
            continue;
        }

        return Ok(path);
    }
}

fn prompt_setup_file(source_folder: &PathBuf) -> PackageResult<String> {
    // List files in the source folder (non-recursive, just top level)
    let mut files: Vec<String> = fs::read_dir(source_folder)
        .map_err(|e| PackageError::SourceReadError {
            path: source_folder.clone(),
            reason: e.to_string(),
        })?
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.file_name().to_string_lossy().to_string())
        .collect();

    if files.is_empty() {
        return Err(PackageError::SourceFolderEmpty {
            path: source_folder.clone(),
        });
    }

    files.sort();

    let selection = Select::new()
        .with_prompt("Select setup file")
        .items(&files)
        .default(0)
        .interact()
        .map_err(|e| PackageError::Io(std::io::Error::other(e)))?;

    Ok(files[selection].clone())
}

fn prompt_output_folder() -> PackageResult<PathBuf> {
    loop {
        let input: String = Input::new()
            .with_prompt("Output folder path")
            .interact_text()
            .map_err(|e| PackageError::Io(std::io::Error::other(e)))?;

        let path = PathBuf::from(input.trim());

        // Output folder can be created if it doesn't exist
        if path.exists() && !path.is_dir() {
            eprintln!(
                "Error: Path exists but is not a directory: {}",
                path.display()
            );
            continue;
        }

        return Ok(path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    #[test]
    fn test_list_files_in_source() {
        let temp = TempDir::new().unwrap();
        let source = temp.path();

        // Create some test files
        File::create(source.join("setup.exe")).unwrap();
        File::create(source.join("data.dll")).unwrap();
        File::create(source.join("readme.txt")).unwrap();

        let mut files: Vec<String> = fs::read_dir(source)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().is_file())
            .map(|entry| entry.file_name().to_string_lossy().to_string())
            .collect();

        files.sort();

        assert_eq!(files.len(), 3);
        assert!(files.contains(&"setup.exe".to_string()));
        assert!(files.contains(&"data.dll".to_string()));
        assert!(files.contains(&"readme.txt".to_string()));
    }

    // T055: Test for platform selection
    #[test]
    fn test_platform_enum_variants() {
        // Platform enum should have Intune and MacOS variants
        assert_eq!(Platform::Intune as u8, 0);
        assert_eq!(Platform::MacOS as u8, 1);
    }

    #[test]
    fn test_platform_display_names() {
        // Platform should have user-friendly display names
        let platforms = platform_options();
        assert_eq!(platforms.len(), 2);
        assert!(platforms.iter().any(|p| p.contains("Intune")));
        assert!(platforms.iter().any(|p| p.contains("macOS")));
    }

    // T056: Tests for macOS prompts flow
    #[test]
    fn test_validate_macos_identifier_valid() {
        assert!(validate_identifier("com.example.app").is_ok());
        assert!(validate_identifier("org.test.myapp").is_ok());
        assert!(validate_identifier("com.company.product.sub").is_ok());
    }

    #[test]
    fn test_validate_macos_identifier_invalid() {
        // Empty identifier
        assert!(validate_identifier("").is_err());
        // Single segment (no dots)
        assert!(validate_identifier("myapp").is_err());
        // Starts with number
        assert!(validate_identifier("123.example.app").is_err());
    }

    #[test]
    fn test_validate_version_valid() {
        assert!(validate_version("1.0.0").is_ok());
        assert!(validate_version("1.0").is_ok());
        assert!(validate_version("2.3.4.5").is_ok());
        assert!(validate_version("0.0.1").is_ok());
    }

    #[test]
    fn test_validate_version_invalid() {
        // Empty version
        assert!(validate_version("").is_err());
        // Non-numeric
        assert!(validate_version("abc").is_err());
        // Version with letters
        assert!(validate_version("1.0.0-beta").is_err());
    }
}
