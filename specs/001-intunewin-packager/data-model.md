# Data Model: Cross-Platform IntuneWin Packager

**Date**: 2026-01-09
**Branch**: `001-intunewin-packager`

## Overview

This document defines the data structures used in the IntuneWin packager. The model is derived from the feature specification entities and the researched file format.

---

## Core Entities

### 1. PackageRequest

Represents a user's request to create an IntuneWin package.

```rust
pub struct PackageRequest {
    /// Path to the source folder containing files to package
    pub source_folder: PathBuf,

    /// Name of the setup file within the source folder
    pub setup_file: String,

    /// Path to the output folder where .intunewin will be created
    pub output_folder: PathBuf,

    /// Optional custom output filename (without extension)
    pub output_name: Option<String>,

    /// Verbosity level for output
    pub verbosity: Verbosity,
}

pub enum Verbosity {
    /// Normal mode - show progress and messages
    Normal,
    /// Quiet mode - no prompts, overwrite existing
    Quiet,
    /// Silent mode - no console output at all
    Silent,
}
```

**Validation Rules**:
- `source_folder` MUST exist and be a directory
- `source_folder` MUST contain at least one file
- `setup_file` MUST exist within `source_folder`
- `output_folder` will be created if it doesn't exist
- `output_name` if provided, MUST be a valid filename (no path separators)

---

### 2. SourcePackage

Represents the collection of files to be packaged.

```rust
pub struct SourcePackage {
    /// Root folder path
    pub root: PathBuf,

    /// Setup file path relative to root
    pub setup_file: PathBuf,

    /// All files to include (relative paths)
    pub files: Vec<SourceFile>,

    /// Total uncompressed size in bytes
    pub total_size: u64,
}

pub struct SourceFile {
    /// Path relative to source root
    pub relative_path: PathBuf,

    /// File size in bytes
    pub size: u64,

    /// Whether this is the setup file
    pub is_setup_file: bool,
}
```

**Validation Rules**:
- `files` MUST include at least the setup file
- `files` includes all files recursively (including hidden files)
- Symbolic links are followed (target content included)
- `total_size` is sum of all file sizes

**State Transitions**:
```
[Scanned] -> [Validated] -> [Packaged]
```

---

### 3. EncryptionInfo

Cryptographic parameters for the package.

```rust
pub struct EncryptionInfo {
    /// AES-256 encryption key (32 bytes)
    pub encryption_key: [u8; 32],

    /// HMAC-SHA256 key (32 bytes)
    pub mac_key: [u8; 32],

    /// AES initialization vector (16 bytes)
    pub iv: [u8; 16],

    /// HMAC-SHA256 of (IV || ciphertext)
    pub mac: [u8; 32],

    /// SHA256 hash of the encrypted file
    pub file_digest: [u8; 32],

    /// Profile identifier (always "ProfileVersion1")
    pub profile_identifier: String,

    /// Digest algorithm name (always "SHA256")
    pub file_digest_algorithm: String,
}
```

**Lifecycle**:
1. Keys generated randomly at encryption start
2. MAC computed after encryption completes
3. File digest computed over final encrypted content
4. All values serialized to Base64 for Detection.xml

---

### 4. DetectionMetadata

Metadata written to Detection.xml.

```rust
pub struct DetectionMetadata {
    /// Name of the application (setup filename)
    pub name: String,

    /// Original uncompressed content size in bytes
    pub unencrypted_content_size: u64,

    /// Encrypted file name (always "IntunePackage.intunewin")
    pub file_name: String,

    /// Setup file name
    pub setup_file: String,

    /// Encryption parameters
    pub encryption_info: EncryptionInfo,
}
```

**XML Mapping**:
| Field | XML Element |
|-------|-------------|
| name | `<Name>` |
| unencrypted_content_size | `<UnencryptedContentSize>` |
| file_name | `<FileName>` |
| setup_file | `<SetupFile>` |
| encryption_info.encryption_key | `<EncryptionKey>` (Base64) |
| encryption_info.mac_key | `<MacKey>` (Base64) |
| encryption_info.iv | `<InitializationVector>` (Base64) |
| encryption_info.mac | `<Mac>` (Base64) |
| encryption_info.profile_identifier | `<ProfileIdentifier>` |
| encryption_info.file_digest | `<FileDigest>` (Base64) |
| encryption_info.file_digest_algorithm | `<FileDigestAlgorithm>` |

---

### 5. IntuneWinPackage

The final output package structure.

```rust
pub struct IntuneWinPackage {
    /// Output file path
    pub output_path: PathBuf,

    /// Detection metadata
    pub metadata: DetectionMetadata,

    /// Size of the final .intunewin file
    pub package_size: u64,

    /// Time taken to create the package
    pub creation_time: Duration,
}
```

**File Structure**:
```
<name>.intunewin (ZIP archive)
└── IntuneWinPackage/
    ├── Metadata/
    │   └── Detection.xml
    └── Contents/
        └── IntunePackage.intunewin (encrypted)
```

---

### 6. PackageResult

Result of the packaging operation.

```rust
pub enum PackageResult {
    Success(IntuneWinPackage),
    Error(PackageError),
}

pub enum PackageError {
    /// Source folder not found or not accessible
    SourceFolderNotFound { path: PathBuf },

    /// Source folder is empty
    SourceFolderEmpty { path: PathBuf },

    /// Setup file not found in source folder
    SetupFileNotFound { file: String, folder: PathBuf },

    /// Output folder creation failed
    OutputFolderCreationFailed { path: PathBuf, reason: String },

    /// Output file already exists (non-quiet mode)
    OutputFileExists { path: PathBuf },

    /// Failed to read source file
    SourceReadError { path: PathBuf, reason: String },

    /// Encryption error
    EncryptionError { reason: String },

    /// Failed to write output
    OutputWriteError { path: PathBuf, reason: String },

    /// User cancelled operation
    Cancelled,
}
```

**Exit Code Mapping**:
| Error Variant | Exit Code |
|---------------|-----------|
| SourceFolderNotFound | 1 |
| SourceFolderEmpty | 3 |
| SetupFileNotFound | 4 |
| OutputFolderCreationFailed | 5 |
| OutputFileExists | 1 |
| SourceReadError | 1 |
| EncryptionError | 1 |
| OutputWriteError | 5 |
| Cancelled | 1 |

---

## CLI Argument Model

### CliArgs

```rust
pub struct CliArgs {
    /// Source folder containing setup files
    #[arg(short = 'c', long = "content")]
    pub content_folder: Option<PathBuf>,

    /// Setup file name (e.g., setup.exe)
    #[arg(short = 's', long = "setup")]
    pub setup_file: Option<String>,

    /// Output folder for .intunewin file
    #[arg(short = 'o', long = "output")]
    pub output_folder: Option<PathBuf>,

    /// Custom output filename (optional)
    #[arg(short = 'n', long = "name")]
    pub output_name: Option<String>,

    /// Quiet mode - no prompts, overwrite existing
    #[arg(short = 'q', long = "quiet")]
    pub quiet: bool,

    /// Silent mode - no console output
    #[arg(long = "silent", visible_alias = "qq")]
    pub silent: bool,
}
```

**Mode Detection**:
- If all required args provided: CLI mode
- If any required arg missing and not quiet/silent: Interactive mode
- If any required arg missing and quiet/silent: Error

---

## Relationships

```
PackageRequest
    │
    ├── validates to ──> SourcePackage
    │                        │
    │                        ├── files ──> SourceFile[]
    │                        │
    │                        └── packaged with ──> EncryptionInfo
    │                                                  │
    └── produces ──────────────────────────────────────┤
                                                       │
                                                       ▼
                                               DetectionMetadata
                                                       │
                                                       ▼
                                               IntuneWinPackage
                                                       │
                                                       ▼
                                               PackageResult
```

---

## Invariants

1. **Encryption keys are never reused**: Each package gets fresh random keys
2. **All files included**: No files are filtered from source (including hidden)
3. **Setup file in source**: Setup file must exist within the source folder
4. **Atomic output**: Package is fully written or not at all (no partial files)
5. **Size tracking**: UnencryptedContentSize matches actual ZIP size before encryption
6. **Format compliance**: Detection.xml structure exactly matches Microsoft format
