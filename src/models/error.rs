//! Error types for the IntuneWin packager.

use std::path::PathBuf;
use thiserror::Error;

/// Exit codes matching CLI contract
pub mod exit_codes {
    /// Success
    pub const SUCCESS: i32 = 0;
    /// General error (I/O, permissions, etc.)
    pub const ERROR: i32 = 1;
    /// Invalid or missing required arguments
    pub const INVALID_ARGS: i32 = 2;
    /// Source folder is empty
    pub const EMPTY_SOURCE: i32 = 3;
    /// Setup file not found in source folder (Intune)
    pub const SETUP_NOT_FOUND: i32 = 4;
    /// Failed to write output file
    pub const OUTPUT_ERROR: i32 = 5;
    /// Scripts folder not found (macOS)
    pub const SCRIPTS_NOT_FOUND: i32 = 6;
    /// Operation cancelled by user
    pub const CANCELLED: i32 = 7;
}

/// Result type for package operations.
pub type PackageResult<T> = Result<T, PackageError>;

/// Errors that can occur during packaging.
#[derive(Error, Debug)]
pub enum PackageError {
    /// Source folder not found or not accessible
    #[error("Source folder not found: {path}")]
    SourceFolderNotFound { path: PathBuf },

    /// Source folder is empty
    #[error("Source folder is empty: {path}")]
    SourceFolderEmpty { path: PathBuf },

    /// Setup file not found in source folder
    #[error("Setup file '{file}' not found in {folder}")]
    SetupFileNotFound { file: String, folder: PathBuf },

    /// Output folder creation failed
    #[error("Failed to create output folder '{path}': {reason}")]
    OutputFolderCreationFailed { path: PathBuf, reason: String },

    /// Output file already exists (non-quiet mode)
    #[error("Output file already exists: {path}")]
    OutputFileExists { path: PathBuf },

    /// Failed to read source file
    #[error("Failed to read source file '{path}': {reason}")]
    SourceReadError { path: PathBuf, reason: String },

    /// Encryption error
    #[error("Encryption error: {reason}")]
    EncryptionError { reason: String },

    /// Failed to write output
    #[error("Failed to write output to '{path}': {reason}")]
    OutputWriteError { path: PathBuf, reason: String },

    /// ZIP creation error
    #[error("ZIP creation error: {reason}")]
    ZipError { reason: String },

    /// XML generation error
    #[error("XML generation error: {reason}")]
    XmlError { reason: String },

    /// Invalid argument
    #[error("Invalid argument: {reason}")]
    InvalidArgument { reason: String },

    /// User cancelled operation
    #[error("Operation cancelled by user")]
    Cancelled,

    /// Invalid .intunewin file
    #[error("Invalid .intunewin file '{path}': {reason}")]
    InvalidIntunewinFile { path: PathBuf, reason: String },

    /// Decryption error
    #[error("Decryption error: {reason}")]
    DecryptionError { reason: String },

    /// HMAC verification failed
    #[error("HMAC verification failed - file may be corrupted or tampered")]
    HmacVerificationFailed,

    /// Invalid PKCS7 padding
    #[error("Invalid padding in decrypted data")]
    InvalidPadding,

    // macOS package errors
    /// Scripts folder not found
    #[error("Scripts folder not found: {path}")]
    ScriptsFolderNotFound { path: PathBuf },

    /// Scripts folder has no valid scripts
    #[error("No preinstall or postinstall scripts found in: {path}")]
    NoScriptsFound { path: PathBuf },

    /// XAR archive creation error
    #[error("XAR archive error: {reason}")]
    XarError { reason: String },

    /// CPIO archive creation error
    #[error("CPIO archive error: {reason}")]
    CpioError { reason: String },

    /// BOM file creation error
    #[error("BOM file error: {reason}")]
    BomError { reason: String },

    /// I/O error wrapper
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

impl PackageError {
    /// Get the exit code for this error.
    pub fn exit_code(&self) -> i32 {
        match self {
            PackageError::SourceFolderNotFound { .. } => exit_codes::ERROR,
            PackageError::SourceFolderEmpty { .. } => exit_codes::EMPTY_SOURCE,
            PackageError::SetupFileNotFound { .. } => exit_codes::SETUP_NOT_FOUND,
            PackageError::OutputFolderCreationFailed { .. } => exit_codes::OUTPUT_ERROR,
            PackageError::OutputFileExists { .. } => exit_codes::ERROR,
            PackageError::SourceReadError { .. } => exit_codes::ERROR,
            PackageError::EncryptionError { .. } => exit_codes::ERROR,
            PackageError::OutputWriteError { .. } => exit_codes::OUTPUT_ERROR,
            PackageError::ZipError { .. } => exit_codes::ERROR,
            PackageError::XmlError { .. } => exit_codes::ERROR,
            PackageError::InvalidArgument { .. } => exit_codes::INVALID_ARGS,
            PackageError::Cancelled => exit_codes::ERROR,
            PackageError::InvalidIntunewinFile { .. } => exit_codes::ERROR,
            PackageError::DecryptionError { .. } => exit_codes::ERROR,
            PackageError::HmacVerificationFailed => exit_codes::ERROR,
            PackageError::InvalidPadding => exit_codes::ERROR,
            // macOS errors
            PackageError::ScriptsFolderNotFound { .. } => exit_codes::SCRIPTS_NOT_FOUND,
            PackageError::NoScriptsFound { .. } => exit_codes::ERROR,
            PackageError::XarError { .. } => exit_codes::ERROR,
            PackageError::CpioError { .. } => exit_codes::ERROR,
            PackageError::BomError { .. } => exit_codes::ERROR,
            PackageError::Io(_) => exit_codes::ERROR,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_codes_mapping() {
        let err = PackageError::SourceFolderEmpty {
            path: PathBuf::from("/tmp"),
        };
        assert_eq!(err.exit_code(), exit_codes::EMPTY_SOURCE);

        let err = PackageError::SetupFileNotFound {
            file: "setup.exe".to_string(),
            folder: PathBuf::from("/tmp"),
        };
        assert_eq!(err.exit_code(), exit_codes::SETUP_NOT_FOUND);

        let err = PackageError::InvalidArgument {
            reason: "test".to_string(),
        };
        assert_eq!(err.exit_code(), exit_codes::INVALID_ARGS);

        let err = PackageError::OutputWriteError {
            path: PathBuf::from("/tmp/out"),
            reason: "test".to_string(),
        };
        assert_eq!(err.exit_code(), exit_codes::OUTPUT_ERROR);
    }
}
