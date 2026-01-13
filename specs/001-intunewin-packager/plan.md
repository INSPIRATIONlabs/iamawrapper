# Implementation Plan: Cross-Platform IntuneWin Packager

**Branch**: `001-intunewin-packager` | **Date**: 2026-01-09 | **Spec**: [spec.md](spec.md)
**Input**: Feature specification from `/specs/001-intunewin-packager/spec.md`

## Summary

Create a cross-platform CLI tool in Rust that packages application installers into Microsoft Intune-compatible `.intunewin` files. The tool replicates the exact format of Microsoft's Win32 Content Prep Tool, enabling IT administrators and DevOps engineers to create Intune packages from Linux, macOS, or Windows without requiring the original Windows-only tool.

## Technical Context

**Language/Version**: Rust 1.75+ (stable, with edition 2021)
**Primary Dependencies**:
- `clap` - CLI argument parsing with compatibility for original tool flags
- `zip` - ZIP archive creation/manipulation
- `aes` + `cbc` - AES-256-CBC encryption
- `hmac` + `sha2` - HMAC-SHA256 authentication and SHA256 hashing
- `base64` - Base64 encoding for Detection.xml values
- `quick-xml` - XML generation for Detection.xml
- `indicatif` - Progress bar display
- `dialoguer` - Interactive prompts

**Storage**: File system only (read source files, write .intunewin output)
**Testing**: `cargo test` with unit tests, integration tests, and contract tests for format compliance
**Target Platform**: Linux (x64, ARM64), macOS (x64, ARM64), Windows (x64, ARM64) - 6 platform targets
**Project Type**: Single CLI application
**Performance Goals**: Package 100MB source in <30 seconds; handle up to 8GB sources
**Constraints**: Streaming processing for large files; <100MB memory for 8GB sources; no external runtime dependencies
**Scale/Scope**: Single binary CLI tool with ~15 commands/flags

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Notes |
|-----------|--------|-------|
| I. Test-First Development | PASS | Tests will be written before implementation; contract tests verify Intune compatibility |
| II. Security by Design | PASS | Uses proven crypto libraries (RustCrypto); validates all inputs; no secrets stored |
| III. Code Quality & Readability | PASS | Rust's type system enforces clarity; clippy for linting; rustfmt for formatting |
| IV. Defensive Programming | PASS | All file I/O validated; errors handled explicitly; streaming prevents memory exhaustion |

**Security Requirements Check**:
- Authentication: N/A (no user authentication required)
- Authorization: N/A (local file processing only)
- Data Protection: Uses AES-256-CBC with HMAC-SHA256 (format requirement)
- Logging: Errors logged to stderr; no sensitive data logged
- Error Handling: User-friendly messages; no stack traces exposed
- Dependencies: Use audited crates from RustCrypto project
- Configuration: Secure defaults; fail on invalid input

## Project Structure

### Documentation (this feature)

```text
specs/001-intunewin-packager/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── main.rs              # Entry point, CLI setup
├── cli/
│   ├── mod.rs           # CLI module
│   ├── args.rs          # Argument parsing with clap
│   └── interactive.rs   # Interactive mode prompts
├── packager/
│   ├── mod.rs           # Packager module
│   ├── archive.rs       # ZIP creation with streaming
│   ├── encrypt.rs       # AES-256-CBC encryption with HMAC
│   └── metadata.rs      # Detection.xml generation
├── models/
│   ├── mod.rs           # Models module
│   ├── package.rs       # IntuneWin package structure
│   └── detection.rs     # Detection.xml data model
└── lib.rs               # Library exports for testing

tests/
├── unit/
│   ├── archive_test.rs  # ZIP creation tests
│   ├── encrypt_test.rs  # Encryption tests
│   └── metadata_test.rs # XML generation tests
├── integration/
│   ├── cli_test.rs      # End-to-end CLI tests
│   └── package_test.rs  # Full packaging workflow tests
└── contract/
    ├── format_test.rs   # Verify output matches Microsoft format
    └── fixtures/        # Sample packages for comparison
```

**Structure Decision**: Single project structure chosen because this is a standalone CLI tool with no web frontend, API, or mobile components. All code compiles to a single binary.

## Complexity Tracking

> No constitution violations requiring justification.

| Aspect | Complexity Level | Justification |
|--------|------------------|---------------|
| Crypto | Standard | Using well-established RustCrypto crates; format dictated by Microsoft |
| CLI | Low | Direct mapping of original tool's flags; no novel interface |
| File I/O | Medium | Streaming required for 8GB files, but standard patterns |
| Cross-platform | Low | Rust handles this natively; no platform-specific code needed |
