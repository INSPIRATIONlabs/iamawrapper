//! Command-line interface module.

pub mod args;
pub mod interactive;

use std::process::ExitCode;

use crate::models::error::{PackageError, PackageResult, exit_codes};
use crate::models::package::Verbosity;
use crate::packager::{package, unpack};

use self::args::{CliArgs, Commands, IntuneAction, MacosAction, MacosPkgArgs};
use self::interactive::{InteractiveResult, run_interactive_with_platform};

/// Run the CLI application.
pub fn run(args: CliArgs) -> ExitCode {
    let verbosity = args.verbosity();

    let result = match &args.command {
        Some(Commands::Intune(intune_cmd)) => run_intune_command(intune_cmd, verbosity),
        Some(Commands::Macos(macos_cmd)) => run_macos_command(macos_cmd, verbosity),
        None => {
            // No subcommand - enter interactive mode if not in quiet/silent mode
            if args.quiet || args.silent {
                Err(PackageError::InvalidArgument {
                    reason: "No command specified. Use 'intune' or 'macos' subcommand.".to_string(),
                })
            } else {
                run_interactive_mode()
            }
        }
    };

    match result {
        Ok(_) => ExitCode::from(exit_codes::SUCCESS as u8),
        Err(e) => {
            let exit_code = e.exit_code();
            if !matches!(verbosity, Verbosity::Silent) {
                eprintln!("Error: {}", e);
            }
            ExitCode::from(exit_code as u8)
        }
    }
}

fn run_intune_command(cmd: &args::IntuneCommand, verbosity: Verbosity) -> PackageResult<()> {
    match &cmd.action {
        IntuneAction::Create(create_args) => run_intune_create(create_args, verbosity),
        IntuneAction::Extract(extract_args) => run_intune_extract(extract_args, verbosity),
    }
}

fn run_intune_create(args: &args::IntuneCreateArgs, verbosity: Verbosity) -> PackageResult<()> {
    let request = args.to_package_request(verbosity);

    match verbosity {
        Verbosity::Normal => {
            println!("IntuneWin Packager v{}\n", env!("CARGO_PKG_VERSION"));
            println!("Source folder: {}", request.source_folder.display());
            println!("Setup file: {}", request.setup_file);
            println!("Output folder: {}", request.output_folder.display());
            println!();

            let result = package(&request)?;

            println!("\nPackage created successfully:");
            println!(
                "  {} ({:.2} MB)",
                result.output_path.display(),
                result.package_size as f64 / 1_048_576.0
            );
            println!(
                "  Creation time: {:.2}s",
                result.creation_time.as_secs_f64()
            );
        }
        Verbosity::Quiet => {
            let result = package(&request)?;
            println!("{}", result.output_path.display());
        }
        Verbosity::Silent => {
            let _result = package(&request)?;
        }
    }

    Ok(())
}

fn run_intune_extract(args: &args::IntuneExtractArgs, verbosity: Verbosity) -> PackageResult<()> {
    let request = args.to_unpack_request(verbosity);

    match verbosity {
        Verbosity::Normal => {
            println!("IntuneWin Unpacker v{}\n", env!("CARGO_PKG_VERSION"));
            println!("Input file: {}", request.input_file.display());
            println!("Output folder: {}", request.output_folder.display());
            println!();

            let result = unpack(&request)?;

            println!("\nPackage extracted successfully:");
            println!("  {} files extracted", result.file_count);
            println!(
                "  Total size: {:.2} MB",
                result.total_size as f64 / 1_048_576.0
            );
            println!(
                "  Extraction time: {:.2}s",
                result.unpack_time.as_secs_f64()
            );
            println!("  Setup file: {}", result.setup_file);
        }
        Verbosity::Quiet => {
            let result = unpack(&request)?;
            println!("{}", result.output_folder.display());
        }
        Verbosity::Silent => {
            let _result = unpack(&request)?;
        }
    }

    Ok(())
}

fn run_macos_command(cmd: &args::MacosCommand, verbosity: Verbosity) -> PackageResult<()> {
    match &cmd.action {
        MacosAction::Pkg(pkg_args) => run_macos_pkg(pkg_args, verbosity),
    }
}

#[cfg(feature = "macos")]
fn run_macos_pkg(args: &MacosPkgArgs, verbosity: Verbosity) -> PackageResult<()> {
    use crate::macos;
    use crate::models::macos::MacosPkgRequest;
    use std::path::PathBuf;

    // Validate source folder exists
    if !args.content_folder.exists() {
        return Err(PackageError::SourceFolderNotFound {
            path: args.content_folder.clone(),
        });
    }

    // Check source folder is not empty
    let is_empty = args
        .content_folder
        .read_dir()
        .map(|mut i| i.next().is_none())
        .unwrap_or(true);
    if is_empty {
        return Err(PackageError::SourceFolderEmpty {
            path: args.content_folder.clone(),
        });
    }

    // Determine output folder and filename from output path
    let output_folder = args
        .output
        .parent()
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));
    let output_name = args
        .output
        .file_stem()
        .and_then(|s| s.to_str())
        .map(String::from);

    // Create request
    let mut request = MacosPkgRequest::new(
        args.content_folder.clone(),
        args.identifier.clone(),
        args.version.clone(),
        output_folder,
    )
    .with_install_location(PathBuf::from(&args.install_location))
    .with_verbosity(verbosity);

    if let Some(name) = output_name {
        request = request.with_output_name(name);
    }

    if let Some(scripts) = &args.scripts_folder {
        request = request.with_scripts_folder(scripts.clone());
    }

    match verbosity {
        Verbosity::Normal => {
            println!("macOS Package Builder v{}\n", env!("CARGO_PKG_VERSION"));
            println!("Source folder: {}", request.source_folder.display());
            println!("Identifier: {}", request.identifier);
            println!("Version: {}", request.version);
            println!("Install location: {}", request.install_location.display());
            println!();

            let result = macos::package(request)?;

            println!("\nPackage created successfully:");
            println!(
                "  {} ({:.2} MB)",
                result.output_path.display(),
                result.package_size as f64 / 1_048_576.0
            );
            println!("  {} files included", result.file_count);
            println!(
                "  Creation time: {:.2}s",
                result.creation_time.as_secs_f64()
            );
        }
        Verbosity::Quiet => {
            let result = macos::package(request)?;
            println!("{}", result.output_path.display());
        }
        Verbosity::Silent => {
            let _result = macos::package(request)?;
        }
    }

    Ok(())
}

#[cfg(not(feature = "macos"))]
fn run_macos_pkg(_args: &MacosPkgArgs, _verbosity: Verbosity) -> PackageResult<()> {
    Err(PackageError::InvalidArgument {
        reason: "macOS packaging is not enabled. Build with --features macos".to_string(),
    })
}

fn run_interactive_mode() -> PackageResult<()> {
    let result = run_interactive_with_platform()?;

    match result {
        InteractiveResult::Intune(request) => {
            let result = package(&request)?;

            println!("\nPackage created successfully:");
            println!(
                "  {} ({:.2} MB)",
                result.output_path.display(),
                result.package_size as f64 / 1_048_576.0
            );
            println!(
                "  Creation time: {:.2}s",
                result.creation_time.as_secs_f64()
            );
        }
        #[cfg(feature = "macos")]
        InteractiveResult::MacOS(request) => {
            use crate::macos;

            let result = macos::package(request)?;

            println!("\nPackage created successfully:");
            println!(
                "  {} ({:.2} MB)",
                result.output_path.display(),
                result.package_size as f64 / 1_048_576.0
            );
            println!("  {} files included", result.file_count);
            println!(
                "  Creation time: {:.2}s",
                result.creation_time.as_secs_f64()
            );
        }
    }

    Ok(())
}
