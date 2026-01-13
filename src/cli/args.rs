//! CLI argument parsing.

use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::models::error::{PackageError, PackageResult};
use crate::models::package::{PackageRequest, UnpackRequest, Verbosity};

/// Cross-platform replacement for Microsoft Win32 Content Prep Tool
#[derive(Parser, Debug)]
#[command(name = "iamawrapper")]
#[command(author, version, about, long_about = None)]
#[derive(Default)]
pub struct CliArgs {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Quiet mode - no prompts, overwrite existing files
    #[arg(short = 'q', long = "quiet", global = true)]
    pub quiet: bool,

    /// Silent mode - no console output at all
    #[arg(long = "silent", visible_alias = "qq", global = true)]
    pub silent: bool,
}


/// Top-level commands
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Create or extract Microsoft Intune packages (.intunewin)
    Intune(IntuneCommand),
    /// Create macOS packages (.pkg)
    Macos(MacosCommand),
}

/// Intune subcommand options
#[derive(Parser, Debug, Clone)]
pub struct IntuneCommand {
    #[command(subcommand)]
    pub action: IntuneAction,
}

/// Intune actions
#[derive(Subcommand, Debug, Clone)]
pub enum IntuneAction {
    /// Create a new .intunewin package
    Create(IntuneCreateArgs),
    /// Extract an existing .intunewin package
    Extract(IntuneExtractArgs),
}

/// Arguments for creating Intune packages
#[derive(Parser, Debug, Clone)]
pub struct IntuneCreateArgs {
    /// Source folder containing files to package
    #[arg(short = 'c', long = "content")]
    pub content_folder: PathBuf,

    /// Setup file name within source folder
    #[arg(short = 's', long = "setup")]
    pub setup_file: String,

    /// Output folder for .intunewin file
    #[arg(short = 'o', long = "output")]
    pub output_folder: PathBuf,

    /// Custom output filename (optional, without extension)
    #[arg(short = 'n', long = "name")]
    pub output_name: Option<String>,
}

/// Arguments for extracting Intune packages
#[derive(Parser, Debug, Clone)]
pub struct IntuneExtractArgs {
    /// Input .intunewin file to extract
    #[arg(short = 'i', long = "input")]
    pub input_file: PathBuf,

    /// Output folder for extracted files
    #[arg(short = 'o', long = "output")]
    pub output_folder: PathBuf,
}

/// macOS subcommand options
#[derive(Parser, Debug, Clone)]
pub struct MacosCommand {
    #[command(subcommand)]
    pub action: MacosAction,
}

/// macOS actions
#[derive(Subcommand, Debug, Clone)]
pub enum MacosAction {
    /// Create a macOS flat package (.pkg)
    Pkg(MacosPkgArgs),
}

/// Arguments for creating macOS packages (T030)
#[derive(Parser, Debug, Clone)]
pub struct MacosPkgArgs {
    /// Source folder containing files to package
    #[arg(short = 'c', long = "content")]
    pub content_folder: PathBuf,

    /// Output path for .pkg file
    #[arg(short = 'o', long = "output")]
    pub output: PathBuf,

    /// Package identifier (reverse-DNS format, e.g., com.company.app)
    #[arg(long = "identifier")]
    pub identifier: String,

    /// Package version (e.g., 1.0.0)
    #[arg(long = "version")]
    pub version: String,

    /// Installation location (default: /)
    #[arg(long = "install-location", default_value = "/")]
    pub install_location: String,

    /// Scripts folder containing preinstall/postinstall scripts
    #[arg(long = "scripts")]
    pub scripts_folder: Option<PathBuf>,
}

// Legacy CLI support - keep existing flat structure for backwards compatibility
/// Legacy CLI arguments (for backwards compatibility)
#[derive(Parser, Debug, Clone)]
#[command(name = "iamawrapper-legacy")]
#[derive(Default)]
pub struct LegacyCliArgs {
    /// Source folder containing files to package
    #[arg(short = 'c', long = "content")]
    pub content_folder: Option<PathBuf>,

    /// Setup file name within source folder
    #[arg(short = 's', long = "setup")]
    pub setup_file: Option<String>,

    /// Output folder for .intunewin file (or extraction destination for --unpack)
    #[arg(short = 'o', long = "output")]
    pub output_folder: Option<PathBuf>,

    /// Custom output filename (optional, without extension)
    #[arg(short = 'n', long = "name")]
    pub output_name: Option<String>,

    /// Unpack an existing .intunewin file instead of creating one
    #[arg(short = 'u', long = "unpack")]
    pub unpack_file: Option<PathBuf>,

    /// Quiet mode - no prompts, overwrite existing files
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Silent mode - no console output at all
    #[arg(long = "silent", visible_alias = "qq")]
    pub silent: bool,
}


impl CliArgs {
    /// Get the verbosity level.
    pub fn verbosity(&self) -> Verbosity {
        if self.silent {
            Verbosity::Silent
        } else if self.quiet {
            Verbosity::Quiet
        } else {
            Verbosity::Normal
        }
    }
}

impl LegacyCliArgs {
    /// Check if running in unpack mode.
    pub fn is_unpack_mode(&self) -> bool {
        self.unpack_file.is_some()
    }

    /// Check if interactive mode is needed.
    pub fn needs_interactive(&self) -> bool {
        // Unpack mode doesn't support interactive
        if self.is_unpack_mode() {
            return false;
        }

        // If any required argument is missing and not in quiet/silent mode
        let missing_required = self.content_folder.is_none()
            || self.setup_file.is_none()
            || self.output_folder.is_none();

        missing_required && !self.quiet && !self.silent
    }

    /// Get the verbosity level.
    pub fn verbosity(&self) -> Verbosity {
        if self.silent {
            Verbosity::Silent
        } else if self.quiet {
            Verbosity::Quiet
        } else {
            Verbosity::Normal
        }
    }

    /// Convert CLI args to a package request.
    pub fn to_package_request(&self) -> PackageResult<PackageRequest> {
        let content_folder =
            self.content_folder
                .clone()
                .ok_or_else(|| PackageError::InvalidArgument {
                    reason: "Source folder (-c) is required".to_string(),
                })?;

        let setup_file = self
            .setup_file
            .clone()
            .ok_or_else(|| PackageError::InvalidArgument {
                reason: "Setup file (-s) is required".to_string(),
            })?;

        let output_folder =
            self.output_folder
                .clone()
                .ok_or_else(|| PackageError::InvalidArgument {
                    reason: "Output folder (-o) is required".to_string(),
                })?;

        let mut request = PackageRequest::new(content_folder, setup_file, output_folder)
            .with_verbosity(self.verbosity());

        if let Some(name) = &self.output_name {
            request = request.with_output_name(name.clone());
        }

        Ok(request)
    }

    /// Convert CLI args to an unpack request.
    pub fn to_unpack_request(&self) -> PackageResult<UnpackRequest> {
        let input_file = self
            .unpack_file
            .clone()
            .ok_or_else(|| PackageError::InvalidArgument {
                reason: "Input file (-u/--unpack) is required".to_string(),
            })?;

        let output_folder =
            self.output_folder
                .clone()
                .ok_or_else(|| PackageError::InvalidArgument {
                    reason: "Output folder (-o) is required for unpacking".to_string(),
                })?;

        Ok(UnpackRequest::new(input_file, output_folder).with_verbosity(self.verbosity()))
    }
}

impl IntuneCreateArgs {
    /// Convert to package request.
    pub fn to_package_request(&self, verbosity: Verbosity) -> PackageRequest {
        let mut request = PackageRequest::new(
            self.content_folder.clone(),
            self.setup_file.clone(),
            self.output_folder.clone(),
        )
        .with_verbosity(verbosity);

        if let Some(name) = &self.output_name {
            request = request.with_output_name(name.clone());
        }

        request
    }
}

impl IntuneExtractArgs {
    /// Convert to unpack request.
    pub fn to_unpack_request(&self, verbosity: Verbosity) -> UnpackRequest {
        UnpackRequest::new(self.input_file.clone(), self.output_folder.clone())
            .with_verbosity(verbosity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_legacy_needs_interactive_missing_all() {
        let args = LegacyCliArgs::default();
        assert!(args.needs_interactive());
    }

    #[test]
    fn test_legacy_needs_interactive_all_provided() {
        let args = LegacyCliArgs {
            content_folder: Some(PathBuf::from("/source")),
            setup_file: Some("setup.exe".to_string()),
            output_folder: Some(PathBuf::from("/output")),
            ..Default::default()
        };
        assert!(!args.needs_interactive());
    }

    #[test]
    fn test_legacy_needs_interactive_quiet_mode() {
        let args = LegacyCliArgs {
            quiet: true,
            ..Default::default()
        };
        // Missing args in quiet mode = error, not interactive
        assert!(!args.needs_interactive());
    }

    #[test]
    fn test_legacy_needs_interactive_silent_mode() {
        let args = LegacyCliArgs {
            silent: true,
            ..Default::default()
        };
        // Missing args in silent mode = error, not interactive
        assert!(!args.needs_interactive());
    }

    #[test]
    fn test_verbosity_normal() {
        let args = CliArgs::default();
        assert_eq!(args.verbosity(), Verbosity::Normal);
    }

    #[test]
    fn test_verbosity_quiet() {
        let args = CliArgs {
            quiet: true,
            ..Default::default()
        };
        assert_eq!(args.verbosity(), Verbosity::Quiet);
    }

    #[test]
    fn test_verbosity_silent() {
        let args = CliArgs {
            silent: true,
            ..Default::default()
        };
        assert_eq!(args.verbosity(), Verbosity::Silent);
    }

    #[test]
    fn test_verbosity_silent_takes_precedence() {
        let args = CliArgs {
            quiet: true,
            silent: true,
            ..Default::default()
        };
        assert_eq!(args.verbosity(), Verbosity::Silent);
    }

    #[test]
    fn test_legacy_to_package_request_success() {
        let args = LegacyCliArgs {
            content_folder: Some(PathBuf::from("/source")),
            setup_file: Some("setup.exe".to_string()),
            output_folder: Some(PathBuf::from("/output")),
            output_name: Some("MyApp".to_string()),
            unpack_file: None,
            quiet: true,
            silent: false,
        };

        let request = args.to_package_request().unwrap();

        assert_eq!(request.source_folder, PathBuf::from("/source"));
        assert_eq!(request.setup_file, "setup.exe");
        assert_eq!(request.output_folder, PathBuf::from("/output"));
        assert_eq!(request.output_name, Some("MyApp".to_string()));
        assert_eq!(request.verbosity, Verbosity::Quiet);
    }

    #[test]
    fn test_legacy_is_unpack_mode() {
        let args = LegacyCliArgs::default();
        assert!(!args.is_unpack_mode());

        let args = LegacyCliArgs {
            unpack_file: Some(PathBuf::from("/test.intunewin")),
            ..Default::default()
        };
        assert!(args.is_unpack_mode());
    }

    #[test]
    fn test_legacy_needs_interactive_unpack_mode() {
        let args = LegacyCliArgs {
            unpack_file: Some(PathBuf::from("/test.intunewin")),
            ..Default::default()
        };
        // Unpack mode never needs interactive
        assert!(!args.needs_interactive());
    }

    #[test]
    fn test_legacy_to_unpack_request_success() {
        let args = LegacyCliArgs {
            unpack_file: Some(PathBuf::from("/test.intunewin")),
            output_folder: Some(PathBuf::from("/extracted")),
            quiet: true,
            ..Default::default()
        };

        let request = args.to_unpack_request().unwrap();

        assert_eq!(request.input_file, PathBuf::from("/test.intunewin"));
        assert_eq!(request.output_folder, PathBuf::from("/extracted"));
        assert_eq!(request.verbosity, Verbosity::Quiet);
    }

    #[test]
    fn test_legacy_to_unpack_request_missing_output() {
        let args = LegacyCliArgs {
            unpack_file: Some(PathBuf::from("/test.intunewin")),
            ..Default::default()
        };

        let result = args.to_unpack_request();
        assert!(matches!(result, Err(PackageError::InvalidArgument { .. })));
    }

    #[test]
    fn test_legacy_to_package_request_missing_content() {
        let args = LegacyCliArgs {
            setup_file: Some("setup.exe".to_string()),
            output_folder: Some(PathBuf::from("/output")),
            ..Default::default()
        };

        let result = args.to_package_request();
        assert!(matches!(result, Err(PackageError::InvalidArgument { .. })));
    }

    #[test]
    fn test_legacy_to_package_request_missing_setup() {
        let args = LegacyCliArgs {
            content_folder: Some(PathBuf::from("/source")),
            output_folder: Some(PathBuf::from("/output")),
            ..Default::default()
        };

        let result = args.to_package_request();
        assert!(matches!(result, Err(PackageError::InvalidArgument { .. })));
    }

    #[test]
    fn test_legacy_to_package_request_missing_output() {
        let args = LegacyCliArgs {
            content_folder: Some(PathBuf::from("/source")),
            setup_file: Some("setup.exe".to_string()),
            ..Default::default()
        };

        let result = args.to_package_request();
        assert!(matches!(result, Err(PackageError::InvalidArgument { .. })));
    }

    #[test]
    fn test_intune_create_args_to_request() {
        let args = IntuneCreateArgs {
            content_folder: PathBuf::from("/source"),
            setup_file: "setup.exe".to_string(),
            output_folder: PathBuf::from("/output"),
            output_name: Some("MyApp".to_string()),
        };

        let request = args.to_package_request(Verbosity::Quiet);
        assert_eq!(request.source_folder, PathBuf::from("/source"));
        assert_eq!(request.setup_file, "setup.exe");
        assert_eq!(request.output_folder, PathBuf::from("/output"));
        assert_eq!(request.output_name, Some("MyApp".to_string()));
        assert_eq!(request.verbosity, Verbosity::Quiet);
    }

    #[test]
    fn test_intune_extract_args_to_request() {
        let args = IntuneExtractArgs {
            input_file: PathBuf::from("/test.intunewin"),
            output_folder: PathBuf::from("/extracted"),
        };

        let request = args.to_unpack_request(Verbosity::Normal);
        assert_eq!(request.input_file, PathBuf::from("/test.intunewin"));
        assert_eq!(request.output_folder, PathBuf::from("/extracted"));
        assert_eq!(request.verbosity, Verbosity::Normal);
    }
}
