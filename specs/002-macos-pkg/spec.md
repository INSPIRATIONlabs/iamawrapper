# Feature Specification: macOS Flat Package (.pkg) Creation

**Feature Branch**: `002-macos-pkg`
**Created**: 2026-01-13
**Status**: Draft
**Input**: User description: "create the rust version"

## Clarifications

### Session 2026-01-13

- Q: What CLI structure should be used for multi-platform support? → A: Subcommands (`iamawrapper intune create/extract`, `iamawrapper macos pkg`)
- Q: How should scripts without execute permissions be handled? → A: Auto-fix by setting mode 0755 on scripts in the package

## User Scenarios & Testing *(mandatory)*

### User Story 1 - Create Basic macOS Package (Priority: P1)

As a developer or IT administrator, I want to create a macOS .pkg installer from a folder of files so that I can distribute and install software on macOS devices through MDM solutions like Jamf, Mosyle, or manual installation.

**Why this priority**: This is the core functionality that enables the entire feature. Without package creation, no other capabilities matter. This mirrors the existing Intune packaging workflow.

**Independent Test**: Can be fully tested by running `iamawrapper macos pkg -c ./MyApp -o ./output --identifier com.company.app --version 1.0.0` and verifying the resulting .pkg file installs correctly on macOS.

**Acceptance Scenarios**:

1. **Given** a folder containing application files, **When** the user runs `iamawrapper macos pkg` with required parameters, **Then** a valid .pkg file is created that can be installed on macOS 10.13+
2. **Given** a source folder with nested subdirectories, **When** the package is created, **Then** all files and directory structure are preserved in the package payload
3. **Given** valid input parameters, **When** package creation completes, **Then** the user sees a success message with output path and package size

---

### User Story 2 - Include Installation Scripts (Priority: P2)

As a developer, I want to include preinstall and postinstall scripts in my .pkg so that I can perform setup or cleanup tasks during installation.

**Why this priority**: Installation scripts are essential for many real-world packages (setting permissions, registering services, displaying notifications) but the tool is still useful without them.

**Independent Test**: Can be tested by creating a package with `--scripts ./scripts-folder` containing preinstall/postinstall scripts, installing the package, and verifying the scripts executed (e.g., checking log files created by the scripts).

**Acceptance Scenarios**:

1. **Given** a scripts folder containing a `preinstall` script, **When** the package is created and installed, **Then** the preinstall script runs before files are copied
2. **Given** a scripts folder containing a `postinstall` script, **When** the package is created and installed, **Then** the postinstall script runs after files are copied
3. **Given** scripts without execute permissions, **When** the package is created, **Then** the tool automatically sets execute permissions (mode 0755) on the scripts

---

### User Story 3 - Interactive Mode for macOS Packages (Priority: P3)

As a user unfamiliar with command-line options, I want an interactive mode that prompts me for package details so that I can create packages without memorizing all parameters.

**Why this priority**: Improves usability for occasional users, but power users and automation workflows will use CLI flags directly.

**Independent Test**: Can be tested by running `iamawrapper` without arguments, selecting "macOS package" when prompted, and following the interactive prompts to create a valid package.

**Acceptance Scenarios**:

1. **Given** the user runs `iamawrapper` with no arguments, **When** they select macOS package creation, **Then** they are prompted for source folder, identifier, version, and output location
2. **Given** the user is in interactive mode, **When** they enter all required information, **Then** a valid .pkg file is created

---

### Edge Cases

- What happens when the source folder is empty?
  - The tool displays an error message and exits without creating a package
- What happens when the output file already exists?
  - In normal mode: prompt user for confirmation to overwrite
  - In quiet mode: overwrite without prompting
- What happens when the identifier format is invalid (e.g., missing dots)?
  - Display a warning about convention but allow creation (identifiers like "myapp" are technically valid)
- What happens when scripts folder is specified but contains no valid scripts?
  - Display a warning that no preinstall or postinstall scripts were found, continue with package creation
- How does the system handle files with special characters or Unicode names?
  - All valid filesystem characters are preserved correctly in the package payload
- What happens when a file in the source folder cannot be read?
  - Display an error with the specific file path and exit without creating a partial package

## Requirements *(mandatory)*

### Functional Requirements

- **FR-001**: System MUST create valid macOS flat package (.pkg) files that install correctly on macOS 10.13 and later
- **FR-002**: System MUST support specifying a package identifier (e.g., `com.company.appname`) via `--identifier` flag
- **FR-003**: System MUST support specifying a package version (e.g., `1.0.0`) via `--version` flag
- **FR-004**: System MUST support specifying an install location (e.g., `/Applications`) via `--install-location` flag, defaulting to `/` if not specified
- **FR-005**: System MUST preserve file permissions and directory structure from the source folder
- **FR-006**: System MUST set file ownership to root:wheel (uid 0, gid 80) in the package payload as required by macOS installer
- **FR-007**: System MUST support optional preinstall and postinstall scripts via `--scripts` flag pointing to a folder
- **FR-008**: System MUST generate valid PackageInfo XML with correct file count and install size
- **FR-009**: System MUST generate valid Distribution XML for the installer
- **FR-010**: System MUST generate a valid Bill of Materials (Bom) file listing all payload files
- **FR-011**: System MUST create the payload as a gzip-compressed cpio archive in odc format
- **FR-012**: System MUST bundle all components into a XAR archive without additional compression
- **FR-013**: System MUST display progress during package creation when not in quiet/silent mode
- **FR-014**: System MUST support `-q/--quiet` and `--silent` flags consistent with existing Intune functionality
- **FR-015**: System MUST work on Linux, macOS, and Windows (cross-platform package creation)
- **FR-016**: CLI MUST use subcommand structure: `iamawrapper intune create/extract` for Intune packages, `iamawrapper macos pkg` for macOS packages
- **FR-017**: System MUST automatically set execute permissions (mode 0755) on preinstall/postinstall scripts in the package, regardless of source file permissions

### Key Entities

- **Package Request**: Source folder path, package identifier, version, install location, output folder, optional scripts folder, verbosity settings
- **Package Payload**: Gzip-compressed cpio archive containing files to install with correct ownership (root:wheel)
- **Bill of Materials (Bom)**: Binary file listing all files, directories, and their metadata (permissions, size, checksum)
- **PackageInfo**: XML document describing the package (identifier, version, install location, file count, scripts)
- **Distribution**: XML document describing the installer requirements and component packages
- **Scripts Archive**: Optional gzip-compressed cpio archive containing preinstall/postinstall scripts

## Success Criteria *(mandatory)*

### Measurable Outcomes

- **SC-001**: Users can create a macOS .pkg installer from a source folder in under 30 seconds for typical application sizes (< 100MB)
- **SC-002**: Created packages install correctly on macOS without errors or warnings
- **SC-003**: Packages created on Linux or Windows install correctly on macOS without modification
- **SC-004**: 100% of files from source folder are correctly included in the package payload with preserved structure
- **SC-005**: Installation scripts execute with correct permissions and receive standard installer arguments
- **SC-006**: Tool provides clear error messages for invalid input, enabling users to correct issues on first retry

## Assumptions

- Users understand macOS package identifier conventions (reverse-DNS format recommended but not enforced)
- The tool targets component packages (single .pkg), not product archives (multiple components with custom installer UI)
- Scripts provided by users are valid shell scripts compatible with macOS
- Source folders contain files that are legally distributable by the user

## Scope Boundaries

**In Scope**:
- Creating component flat packages (.pkg)
- Optional preinstall/postinstall scripts
- Cross-platform package creation (build macOS packages from Linux/Windows)
- CLI and interactive mode
- Progress display during package creation

**Out of Scope**:
- Product archives with multiple component packages
- Code signing (may be added in future version)
- Notarization
- Custom installer backgrounds, license files, or readme files
- Package extraction/unpacking (may be added in future version)
