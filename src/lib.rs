//! iamawrapper Packager Library
//!
//! Cross-platform packaging tool for:
//! - Microsoft Intune (.intunewin files)
//! - macOS flat packages (.pkg files)

pub mod cli;
#[cfg(feature = "macos")]
pub mod macos;
pub mod models;
pub mod packager;

pub use models::error::{PackageError, PackageResult};
#[cfg(feature = "macos")]
pub use models::macos::{MacosPkgRequest, MacosPkgResult};
pub use models::package::{IntuneWinPackage, PackageRequest, SourcePackage, Verbosity};
pub use packager::package;
