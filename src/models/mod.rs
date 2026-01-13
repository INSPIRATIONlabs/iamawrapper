//! Data models for the iamawrapper packager.

pub mod detection;
pub mod error;
#[cfg(feature = "macos")]
pub mod macos;
pub mod package;

pub use detection::{DetectionMetadata, EncryptionInfo};
pub use error::{PackageError, PackageResult};
#[cfg(feature = "macos")]
pub use macos::{MacosPkgRequest, MacosPkgResult, PackagePayload, PayloadFile};
pub use package::{IntuneWinPackage, PackageRequest, SourceFile, SourcePackage, Verbosity};
