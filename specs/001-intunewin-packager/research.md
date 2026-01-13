# Research: Cross-Platform IntuneWin Packager

**Date**: 2026-01-09
**Branch**: `001-intunewin-packager`

## Executive Summary

This document captures research findings for implementing a cross-platform replacement for Microsoft's Win32 Content Prep Tool. All critical unknowns have been resolved through analysis of the original tool's output format and selection of appropriate Rust libraries.

---

## 1. IntuneWin File Format

### Decision
Implement the exact format as documented by reverse-engineering the original Microsoft tool.

### Rationale
- Microsoft does not publish an official specification
- Format was determined through reverse-engineering the original tool's output
- Format compatibility is critical for Intune acceptance

### Format Specification

**Outer Structure**: Standard ZIP archive (unencrypted)
```
<filename>.intunewin (ZIP)
├── IntuneWinPackage/
│   ├── Metadata/
│   │   └── Detection.xml    # Unencrypted metadata
│   └── Contents/
│       └── IntunePackage.intunewin  # Encrypted payload
```

**Detection.xml Schema**:
```xml
<ApplicationInfo xmlns:xsd="http://www.w3.org/2001/XMLSchema"
                 xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <Name>[setup filename]</Name>
  <UnencryptedContentSize>[original ZIP size in bytes]</UnencryptedContentSize>
  <FileName>IntunePackage.intunewin</FileName>
  <SetupFile>[setup filename]</SetupFile>
  <EncryptionInfo>
    <EncryptionKey>[Base64: 32 bytes]</EncryptionKey>
    <MacKey>[Base64: 32 bytes]</MacKey>
    <InitializationVector>[Base64: 16 bytes]</InitializationVector>
    <Mac>[Base64: 32 bytes HMAC-SHA256]</Mac>
    <ProfileIdentifier>ProfileVersion1</ProfileIdentifier>
    <FileDigest>[Base64: SHA256 of encrypted file]</FileDigest>
    <FileDigestAlgorithm>SHA256</FileDigestAlgorithm>
  </EncryptionInfo>
</ApplicationInfo>
```

**Encrypted Payload Structure** (IntunePackage.intunewin):
```
[0..31]   HMAC-SHA256 hash (32 bytes)
[32..47]  Initialization Vector (16 bytes)
[48..]    AES-256-CBC encrypted ZIP content
```

### Alternatives Considered
- **Custom format**: Rejected - would not be compatible with Intune
- **Simplified format**: Rejected - Intune validates exact structure

---

## 2. Encryption Implementation

### Decision
Use RustCrypto crates: `aes`, `cbc`, `hmac`, `sha2`

### Rationale
- RustCrypto is the standard cryptographic library ecosystem for Rust
- Well-audited and maintained by the Rust Cryptography community
- Supports all required algorithms (AES-256-CBC, HMAC-SHA256, SHA256)
- No external C dependencies (pure Rust)
- Cross-platform by default

### Implementation Details

**Key Generation**:
- EncryptionKey: 32 random bytes (AES-256)
- MacKey: 32 random bytes (HMAC-SHA256)
- IV: 16 random bytes (AES block size)
- Use `rand` crate with OS-provided entropy

**Encryption Process**:
1. Compress source files into ZIP (in-memory or streaming)
2. Generate random encryption key, MAC key, and IV
3. Encrypt ZIP content with AES-256-CBC using PKCS7 padding
4. Compute HMAC-SHA256 over (IV || ciphertext) using MAC key
5. Prepend HMAC (32 bytes) and IV (16 bytes) to ciphertext
6. Compute SHA256 digest of final encrypted file

### Alternatives Considered
- **OpenSSL bindings (rust-openssl)**: Rejected - adds C dependency, complicates cross-compilation
- **ring**: Rejected - good library but RustCrypto has better AES-CBC support
- **sodiumoxide**: Rejected - doesn't support AES-CBC (only modern ciphers)

---

## 3. ZIP Library Selection

### Decision
Use `zip` crate (v0.6+) for ZIP archive creation

### Rationale
- Pure Rust implementation
- Supports streaming writes (required for 8GB files)
- Standard deflate compression
- Well-maintained and widely used
- Cross-platform

### Streaming Strategy
For large source folders (up to 8GB):
1. Create ZIP with streaming writer
2. Add files one at a time without loading all into memory
3. Pipe compressed output through encryption
4. Write encrypted chunks to output file

### Alternatives Considered
- **async-zip**: Rejected - async not needed for CLI tool; adds complexity
- **libzip bindings**: Rejected - C dependency complicates cross-platform builds

---

## 4. CLI Framework Selection

### Decision
Use `clap` v4 with derive macros

### Rationale
- Industry standard for Rust CLI applications
- Supports both long (`--content`) and short (`-c`) flags
- Automatic help generation
- Subcommand support (though not needed here)
- Excellent error messages

### Flag Mapping
| Original Tool | Our Tool | Description |
|---------------|----------|-------------|
| `-c` | `-c, --content` | Source folder |
| `-s` | `-s, --setup` | Setup file |
| `-o` | `-o, --output` | Output folder |
| `-q` | `-q, --quiet` | Quiet mode |
| `-h` | `-h, --help` | Help |
| `-v` | `-v, --version` | Version |
| N/A | `-n, --name` | Custom output filename (extension) |
| N/A | `-qq, --silent` | Silent mode (no output) |

### Alternatives Considered
- **structopt**: Rejected - merged into clap v3+
- **argh**: Rejected - less feature-rich, smaller community
- **pico-args**: Rejected - too minimal for our needs

---

## 5. Progress Display

### Decision
Use `indicatif` crate for progress bars

### Rationale
- De facto standard for Rust progress indicators
- Supports styled progress bars and spinners
- Works well with streaming operations
- Cross-platform terminal handling

### Display Strategy
- Show percentage-based progress bar during ZIP compression
- Show file count (e.g., "Processing file 45/128")
- Suppress in quiet mode (`-q`), completely hide in silent mode (`-qq`)

### Alternatives Considered
- **pbr**: Rejected - less maintained than indicatif
- **console**: Rejected - lower-level; would need manual implementation
- **Custom implementation**: Rejected - unnecessary complexity

---

## 6. Interactive Mode

### Decision
Use `dialoguer` crate for interactive prompts

### Rationale
- Purpose-built for interactive CLI prompts
- Supports path input, selection from list, confirmation
- Works cross-platform
- Pairs well with indicatif (same author)

### Prompt Flow
1. Prompt for source folder path (with path validation)
2. List files in folder, prompt for setup file selection
3. Prompt for output folder path
4. Show confirmation and proceed

### Alternatives Considered
- **inquire**: Rejected - similar features but dialoguer more established
- **requestty**: Rejected - less mature
- **Custom stdin reading**: Rejected - poor UX, no cross-platform handling

---

## 7. XML Generation

### Decision
Use `quick-xml` crate for Detection.xml generation

### Rationale
- Fast and lightweight
- Supports writing XML with proper escaping
- No external dependencies
- Good control over formatting (needed for exact match)

### Formatting Requirements
- UTF-8 encoding with XML declaration
- Specific namespace declarations on root element
- Element ordering must match original tool exactly
- No extra whitespace between elements

### Alternatives Considered
- **xml-rs**: Rejected - more complex API than needed
- **roxmltree**: Rejected - read-only (parsing)
- **String formatting**: Rejected - error-prone, escaping issues

---

## 8. Cross-Platform Build Strategy

### Decision
Use GitHub Actions with cross-compilation via `cross` tool

### Rationale
- GitHub Actions provides Linux, macOS, and Windows runners
- `cross` tool enables ARM64 compilation from x64 hosts
- Produces static binaries with no runtime dependencies
- Standard approach for Rust cross-platform releases

### Build Targets
| Target | OS | Architecture |
|--------|-----|--------------|
| `x86_64-unknown-linux-gnu` | Linux | x64 |
| `aarch64-unknown-linux-gnu` | Linux | ARM64 |
| `x86_64-apple-darwin` | macOS | x64 |
| `aarch64-apple-darwin` | macOS | ARM64 |
| `x86_64-pc-windows-msvc` | Windows | x64 |
| `aarch64-pc-windows-msvc` | Windows | ARM64 |

### Alternatives Considered
- **Manual cross-compilation**: Rejected - complex setup, hard to maintain
- **Docker-based builds**: Partially used via `cross` tool
- **Native compilation on each platform**: Rejected - requires 6 different machines/VMs

---

## 9. Error Handling Strategy

### Decision
Use `thiserror` for error types, `anyhow` for application errors

### Rationale
- `thiserror` provides clean error type definitions
- `anyhow` simplifies error propagation in main()
- Standard pattern in Rust CLI applications
- Good error messages for users

### Exit Codes
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error (file not found, permission denied) |
| 2 | Invalid arguments |
| 3 | Source folder empty or invalid |
| 4 | Setup file not found in source |
| 5 | Output write failed |

### Alternatives Considered
- **Custom error types only**: Rejected - too verbose for CLI app
- **String errors**: Rejected - poor practice, no structure
- **eyre**: Rejected - similar to anyhow, less widely used

---

## 10. Memory Management for Large Files

### Decision
Use streaming I/O with bounded buffers

### Rationale
- 8GB files cannot fit in memory
- Streaming allows processing files of any size
- Rust's ownership model makes this safe

### Implementation Approach
1. **ZIP creation**: Use `zip::write::FileOptions` with streaming
2. **Encryption**: Process in 64KB chunks
3. **Hashing**: Update HMAC/SHA256 incrementally
4. **Output**: Write to file as chunks are processed

### Memory Budget
- Target: <100MB RAM for 8GB source
- ZIP buffer: 8MB
- Encryption buffer: 64KB
- File read buffer: 64KB per file

### Alternatives Considered
- **Memory-mapped files**: Rejected - still requires address space for full file
- **Temporary files**: Rejected - slower, requires disk space
- **Process in multiple passes**: Rejected - slower, more complex

---

## Summary of Technology Choices

| Component | Choice | Crate Version |
|-----------|--------|---------------|
| CLI parsing | clap | 4.x |
| ZIP handling | zip | 0.6+ |
| AES encryption | aes + cbc | 0.1+ |
| HMAC/SHA256 | hmac + sha2 | 0.12+ |
| Base64 | base64 | 0.21+ |
| XML writing | quick-xml | 0.31+ |
| Progress bar | indicatif | 0.17+ |
| Interactive | dialoguer | 0.11+ |
| Random | rand | 0.8+ |
| Errors | thiserror + anyhow | 1.x |

All crates are pure Rust with no external C dependencies, ensuring clean cross-platform compilation.
