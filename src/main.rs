//! IntuneWin Packager - Cross-platform replacement for Microsoft Win32 Content Prep Tool.
//!
//! Creates .intunewin files compatible with Microsoft Intune for
//! application deployment to Windows devices.

use std::process::ExitCode;

use clap::Parser;

use iamawrapper::cli::args::CliArgs;
use iamawrapper::cli::run;

fn main() -> ExitCode {
    let args = CliArgs::parse();
    run(args)
}
