# Feature Specification: Cross-Platform IntuneWin Packager

**Feature Branch**: `001-intunewin-packager`
**Created**: 2026-01-09
**Status**: Draft
**Input**: User description: "We want to write a cross platform replacement for Microsoft Win32 Content Prep Tool written in Rust. The tool should be able to create intunewin files exactly like the original. The application should be able to run under linux, macos, windows, arm64 and x64."

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Package Application for Intune (Priority: P1)

As an IT administrator, I want to package a Windows application installer into an `.intunewin` file so that I can upload it to Microsoft Intune for deployment to managed devices.

**Why this priority**: This is the core functionality of the tool - without it, the tool has no value. Every other feature depends on this capability.

**Independent Test**: Can be fully tested by providing a source folder with an MSI or EXE installer and verifying the output `.intunewin` file can be successfully uploaded to Microsoft Intune and deployed to a device.

**Acceptance Scenarios**:

1. **Given** a folder containing setup files including an MSI installer, **When** I run the tool with the source folder and setup file specified, **Then** a valid `.intunewin` file is created that contains all source files encrypted and can be uploaded to Intune.
2. **Given** a folder containing setup files including an EXE installer, **When** I run the tool specifying the EXE as the setup file, **Then** a valid `.intunewin` file is created with proper metadata reflecting the EXE as the entry point.
3. **Given** the generated `.intunewin` file, **When** I upload it to Microsoft Intune, **Then** Intune accepts the file and correctly displays the application metadata.

---

### User Story 2 - Cross-Platform Packaging (Priority: P1)

As a DevOps engineer, I want to create `.intunewin` packages from Linux or macOS build servers so that I can integrate Intune packaging into my CI/CD pipelines without requiring Windows.

**Why this priority**: Cross-platform support is the primary differentiator and motivation for this tool. Without it, users would continue using the original Microsoft tool.

**Independent Test**: Can be fully tested by running the packaging tool on Linux, macOS, and Windows systems and verifying all produce byte-compatible output files.

**Acceptance Scenarios**:

1. **Given** I am running on a Linux x64 system, **When** I package an application, **Then** the resulting `.intunewin` file is identical in structure and compatibility to one produced by the original Microsoft tool.
2. **Given** I am running on macOS ARM64 (Apple Silicon), **When** I package an application, **Then** the tool runs natively and produces a valid `.intunewin` file.
3. **Given** I am running on Windows ARM64, **When** I package an application, **Then** the tool runs natively and produces a valid `.intunewin` file.

---

### User Story 3 - Command-Line Interface Compatibility (Priority: P2)

As an automation engineer, I want to use the same command-line parameters as the original Microsoft tool so that I can replace it in existing scripts without modifications.

**Why this priority**: Enables drop-in replacement in existing automation workflows, reducing adoption friction.

**Independent Test**: Can be tested by running existing scripts that use the original Microsoft tool's CLI parameters and verifying they work without modification.

**Acceptance Scenarios**:

1. **Given** I have an existing script using `-c`, `-s`, and `-o` parameters, **When** I replace the Microsoft tool with this tool, **Then** the script executes successfully without modification.
2. **Given** I want to run in quiet mode, **When** I use the `-q` flag, **Then** no interactive prompts appear and existing output files are overwritten.
3. **Given** I want to run in silent mode, **When** I use the `-qq` flag, **Then** no console output is produced.

---

### User Story 4 - Interactive Mode (Priority: P3)

As an IT administrator new to Intune packaging, I want to run the tool without command-line parameters so that I can be guided through the packaging process step by step.

**Why this priority**: Improves usability for new users but is not essential for core functionality or automation scenarios.

**Independent Test**: Can be tested by running the tool without parameters and completing the packaging process through interactive prompts.

**Acceptance Scenarios**:

1. **Given** I run the tool without any parameters, **When** the tool starts, **Then** I am prompted for the source folder path.
2. **Given** I have entered the source folder path, **When** prompted for the setup file, **Then** I see a list of available files to choose from.
3. **Given** I have provided all required inputs interactively, **When** the packaging completes, **Then** I see a summary of the created package including file path and size.

---

### Edge Cases

- What happens when the source folder is empty? The tool displays an error message indicating no files found to package.
- What happens when the specified setup file does not exist in the source folder? The tool displays an error with the expected path and exits with a non-zero code.
- What happens when the output folder does not exist? The tool creates the output folder automatically.
- What happens when an output file already exists and `-q` is not specified? The tool prompts for confirmation before overwriting.
- What happens when the source folder contains symbolic links? The tool follows symbolic links and includes the target files.
- What happens when files are read-only? The tool reads and packages them normally.
- What happens when the total source size exceeds available memory? The tool uses streaming processing to handle large packages without loading everything into memory.

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST create `.intunewin` files that are functionally compatible with Microsoft Intune (accepted and processed correctly) and structurally match the original tool's format (XML element ordering, whitespace, field names)
- **FR-002**: System MUST compress all source files into a ZIP archive before encryption
- **FR-003**: System MUST encrypt the compressed content using AES-256 with authenticated encryption (AES-256-CBC with HMAC-SHA256)
- **FR-004**: System MUST generate a unique encryption key, initialization vector, and MAC key for each package
- **FR-005**: System MUST create a `Detection.xml` metadata file containing encryption parameters, file hashes, and setup file information
- **FR-006**: System MUST structure the output as a ZIP archive containing `IntuneWinPackage/Metadata/Detection.xml` and `IntuneWinPackage/Contents/IntunePackage.intunewin`
- **FR-007**: System MUST support the following command-line parameters: `-c` (source folder), `-s` (setup file), `-o` (output folder), `-n` (output filename, optional), `-q` (quiet mode), `-qq` (silent mode), `-h` (help), `-v` (version)
- **FR-007a**: System MUST name the output file after the setup file by default (e.g., `setup.exe` → `setup.intunewin`), unless a custom filename is specified via `-n` parameter
- **FR-008**: System MUST run natively on Linux (x64, ARM64), macOS (x64, ARM64), and Windows (x64, ARM64)
- **FR-009**: System MUST compute SHA256 file digests for integrity verification
- **FR-010**: System MUST prepend the HMAC (32 bytes) and IV (16 bytes) to the encrypted content as per the original format
- **FR-011**: System MUST support interactive mode when run without parameters, prompting users for required inputs
- **FR-012**: System MUST validate that the specified setup file exists within the source folder (any file type accepted, matching original tool behavior)
- **FR-013**: System MUST exit with appropriate non-zero exit codes on error conditions
- **FR-014**: System MUST display version information when `-v` flag is provided
- **FR-015**: System MUST display a simple progress indicator (percentage or file count) during packaging in non-silent modes (`-qq` suppresses this)
- **FR-016**: System MUST include all files from the source folder regardless of hidden status (dotfiles on Unix, hidden attribute on Windows)

### Key Entities

- **Source Package**: The collection of application files to be packaged, including the primary setup file (MSI/EXE) and any supporting files
- **IntuneWin Package**: The output artifact - a ZIP-structured file containing encrypted content and metadata
- **Detection Metadata**: XML document containing encryption keys, file hashes, setup file name, and tool version information
- **Encrypted Content**: AES-256 encrypted ZIP archive of all source files with HMAC prepended for integrity verification

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: IntuneWin files produced by this tool are accepted by Microsoft Intune without errors
- **SC-002**: Applications packaged with this tool deploy successfully to Windows devices via Intune
- **SC-003**: Tool completes packaging of a 100MB source folder in under 30 seconds on standard hardware
- **SC-004**: Tool runs without external runtime dependencies on all supported platforms (Linux, macOS, Windows on x64 and ARM64)
- **SC-005**: Existing automation scripts using Microsoft tool CLI parameters work without modification
- **SC-006**: Tool handles source folders up to 8GB without running out of memory
- **SC-007**: Users can complete packaging workflow (interactive mode) in under 2 minutes

## Clarifications

### Session 2026-01-09

- Q: Should the tool restrict setup file types to MSI/EXE or accept any file type like the original tool? → A: Accept any file type (match original tool behavior) - just verify file exists
- Q: How should the output .intunewin file be named? → A: Name after setup file by default (e.g., setup.exe → setup.intunewin), but allow user to specify custom output filename
- Q: Should the tool provide progress feedback for large packages? → A: Simple progress indicator (percentage or file count) during processing in non-silent modes
- Q: What level of compatibility is required with Microsoft's tool output? → A: Functionally compatible (Intune accepts file) AND XML structure matches original tool's formatting (element ordering, whitespace)
- Q: Should hidden files (dotfiles on Unix, hidden attribute on Windows) be included? → A: Include all files regardless of hidden status (match original tool behavior)

## Assumptions

- Users have the legal right to package and distribute the applications they are processing
- Target deployment environments are Windows devices managed by Microsoft Intune
- The tool does not need to support catalog files for Windows 10 S mode (the `-a` parameter) in the initial release
- The Detection.xml format used by Microsoft Intune is stable and will not change incompatibly
- Base64 encoding uses standard RFC 4648 encoding
- The original tool's encryption scheme (AES-256-CBC with HMAC-SHA256) remains the accepted format for Intune
