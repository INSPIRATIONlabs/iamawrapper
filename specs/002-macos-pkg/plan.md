# Implementation Plan: macOS Flat Package (.pkg) Creation

**Branch**: `002-macos-pkg` | **Date**: 2026-01-13 | **Spec**: [spec.md](./spec.md)
**Input**: Feature specification from `/specs/002-macos-pkg/spec.md`

## Summary

Add macOS flat package (.pkg) creation capability to iamawrapper, enabling cross-platform building of macOS installer packages. This requires:
1. Restructuring CLI to use subcommands (`iamawrapper intune create/extract`, `iamawrapper macos pkg`)
2. Implementing pure Rust creation of XAR archives, CPIO payloads, and BOM files
3. Generating valid PackageInfo and Distribution XML documents

## Technical Context

**Language/Version**: Rust 1.75+ (edition 2024 per existing Cargo.toml)
**Primary Dependencies**:
- Existing: clap 4.4, quick-xml 0.31, walkdir 2.4, indicatif 0.17, dialoguer 0.11, thiserror 1.0, anyhow 1.0
- New: flate2 (gzip compression), crc32fast (BOM checksums)
- Research needed: XAR archive creation crate or custom implementation
**Storage**: N/A (file-based I/O only)
**Testing**: cargo test with tempfile 3.8, assert_cmd 2.0, predicates 3.0
**Target Platform**: Linux, macOS, Windows (cross-platform CLI tool)
**Project Type**: Single CLI application
**Performance Goals**: < 30 seconds for packages under 100MB (per SC-001)
**Constraints**: Pure Rust implementation for cross-platform compatibility; no shell-outs to pkgbuild/xar
**Scale/Scope**: Single-user CLI tool, typical package sizes < 500MB

## Constitution Check

*GATE: Must pass before Phase 0 research. Re-check after Phase 1 design.*

| Principle | Status | Evidence/Plan |
|-----------|--------|---------------|
| I. Test-First Development | ✅ WILL COMPLY | Tests will be written before implementation for each component (BOM, CPIO, XAR, XML generation) |
| II. Security by Design | ✅ COMPLIANT | Input validation on all paths; no command injection risk (no shell-outs); path traversal protection in archive handling |
| III. Code Quality | ✅ WILL COMPLY | Following existing codebase patterns; cargo fmt/clippy enforced |
| IV. Defensive Programming | ✅ WILL COMPLY | All file I/O errors explicitly handled; fail fast on invalid input |

**Security Considerations**:
- Path traversal: Must validate all file paths stay within source folder
- File permissions: Set to safe defaults (0755 for scripts, preserve source for payload)
- No secrets handling in this feature
- No network I/O

## Project Structure

### Documentation (this feature)

```text
specs/002-macos-pkg/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/           # Phase 1 output (CLI interface contracts)
└── tasks.md             # Phase 2 output (/speckit.tasks command)
```

### Source Code (repository root)

```text
src/
├── main.rs              # Entry point (update for subcommands)
├── lib.rs               # Library exports (add macos module)
├── cli/
│   ├── mod.rs           # CLI runner (restructure for subcommands)
│   ├── args.rs          # CLI arguments (restructure for subcommands)
│   └── interactive.rs   # Interactive mode (add platform selection)
├── models/
│   ├── mod.rs           # Model exports (add macos types)
│   ├── error.rs         # Error types (add macos errors)
│   ├── package.rs       # Existing Intune types
│   ├── detection.rs     # Existing Intune detection
│   └── macos.rs         # NEW: macOS package types
├── packager/
│   ├── mod.rs           # Packager exports
│   ├── archive.rs       # Existing file collection
│   ├── encrypt.rs       # Existing Intune encryption
│   ├── metadata.rs      # Existing Intune XML
│   └── intune/          # MOVE: Reorganize Intune-specific code
│       └── mod.rs
└── macos/               # NEW: macOS packager module
    ├── mod.rs           # Module exports, package() function
    ├── bom.rs           # Bill of Materials generation
    ├── cpio.rs          # CPIO archive creation (odc format)
    ├── xar.rs           # XAR archive creation
    ├── xml.rs           # PackageInfo & Distribution XML
    └── payload.rs       # Payload assembly

tests/
├── integration/
│   ├── intune_test.rs   # Existing Intune tests
│   └── macos_test.rs    # NEW: macOS package integration tests
└── unit/
    ├── bom_test.rs      # NEW: BOM format tests
    ├── cpio_test.rs     # NEW: CPIO format tests
    └── xar_test.rs      # NEW: XAR format tests
```

**Structure Decision**: Single project with new `macos/` module alongside existing `packager/`. Intune-specific code may be reorganized under `packager/intune/` for clarity, but this is optional refactoring.

## Complexity Tracking

No constitution violations requiring justification. The implementation follows existing patterns and adds a parallel module structure.
