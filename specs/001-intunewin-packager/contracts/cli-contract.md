# CLI Contract: IntuneWin Packager

**Date**: 2026-01-09
**Version**: 1.0.0

## Overview

This document defines the command-line interface contract for the IntuneWin packager. The interface is designed to be compatible with Microsoft's IntuneWinAppUtil while adding quality-of-life improvements.

---

## Binary Name

```
intunewin
```

Alternative names for compatibility:
- `IntuneWinAppUtil` (symlink for drop-in replacement)

---

## Usage Patterns

### Pattern 1: Full CLI Mode

```bash
intunewin -c <source_folder> -s <setup_file> -o <output_folder> [-n <name>] [-q|-qq]
```

### Pattern 2: Interactive Mode

```bash
intunewin
```
Prompts for all required inputs interactively.

### Pattern 3: Help/Version

```bash
intunewin -h
intunewin --help
intunewin -v
intunewin --version
```

---

## Arguments

### Required Arguments (CLI Mode)

| Short | Long | Value | Description |
|-------|------|-------|-------------|
| `-c` | `--content` | `<path>` | Path to source folder containing files to package |
| `-s` | `--setup` | `<filename>` | Name of the setup file within source folder |
| `-o` | `--output` | `<path>` | Path to output folder for .intunewin file |

### Optional Arguments

| Short | Long | Value | Description |
|-------|------|-------|-------------|
| `-n` | `--name` | `<filename>` | Custom output filename (without .intunewin extension) |
| `-q` | `--quiet` | flag | Quiet mode: suppress prompts, overwrite existing files |
| | `--silent` | flag | Silent mode: no console output at all |
| `-h` | `--help` | flag | Display help information |
| `-v` | `--version` | flag | Display version information |

**Note**: `-qq` is accepted as an alias for `--silent` for compatibility with common conventions.

---

## Behavior Specifications

### B1: Argument Validation

```
GIVEN invalid source folder path
WHEN tool is invoked
THEN exit with code 1 and message "Error: Source folder not found: <path>"

GIVEN source folder is empty
WHEN tool is invoked
THEN exit with code 3 and message "Error: Source folder is empty: <path>"

GIVEN setup file not in source folder
WHEN tool is invoked
THEN exit with code 4 and message "Error: Setup file '<file>' not found in <path>"

GIVEN invalid arguments
WHEN tool is invoked
THEN exit with code 2 and display usage help
```

### B2: Interactive Mode

```
GIVEN no arguments provided
AND not in quiet/silent mode
WHEN tool is invoked
THEN prompt for source folder
AND prompt for setup file (show file list)
AND prompt for output folder
AND proceed with packaging
```

### B3: Quiet Mode (-q)

```
GIVEN -q flag provided
WHEN output file exists
THEN overwrite without prompting

GIVEN -q flag provided
WHEN required argument is missing
THEN exit with code 2 (not interactive)

GIVEN -q flag provided
WHEN packaging
THEN show progress indicator (no prompts)
```

### B4: Silent Mode (--silent / -qq)

```
GIVEN --silent flag provided
WHEN tool executes
THEN produce no console output
AND write only to specified output file

GIVEN --silent flag provided
AND error occurs
THEN exit with appropriate code (no message)
```

### B5: Output Naming

```
GIVEN no -n argument
WHEN packaging setup.exe
THEN output file is named "setup.intunewin"

GIVEN -n "custom" argument
WHEN packaging
THEN output file is named "custom.intunewin"

GIVEN -n "custom.intunewin" argument
WHEN packaging
THEN output file is named "custom.intunewin" (extension not duplicated)
```

### B6: Progress Display

```
GIVEN normal mode (no -q or --silent)
WHEN packaging large folder
THEN display progress bar with percentage
AND display current file count (e.g., "Processing 45/128 files")

GIVEN quiet mode (-q)
WHEN packaging
THEN display progress bar with percentage
AND no prompts or confirmations

GIVEN silent mode (--silent)
WHEN packaging
THEN display nothing
```

---

## Exit Codes

| Code | Name | Description |
|------|------|-------------|
| 0 | SUCCESS | Package created successfully |
| 1 | ERROR | General error (I/O, permissions, etc.) |
| 2 | INVALID_ARGS | Invalid or missing required arguments |
| 3 | EMPTY_SOURCE | Source folder is empty |
| 4 | SETUP_NOT_FOUND | Setup file not found in source folder |
| 5 | OUTPUT_ERROR | Failed to write output file |

---

## Output Format

### Success (Normal Mode)

```
IntuneWin Packager v1.0.0

Source folder: /path/to/source
Setup file: setup.exe
Output folder: /path/to/output

Packaging... [████████████████████████] 100% (128/128 files)

Package created successfully:
  /path/to/output/setup.intunewin (15.2 MB)
  Creation time: 2.3s
```

### Success (Quiet Mode)

```
[████████████████████████] 100%
/path/to/output/setup.intunewin
```

### Success (Silent Mode)

```
(no output)
```

### Error (Normal/Quiet Mode)

```
Error: Setup file 'setup.exe' not found in /path/to/source
```

### Error (Silent Mode)

```
(no output, exit code indicates error)
```

---

## Help Output

```
IntuneWin Packager - Cross-platform .intunewin file creator

Usage: intunewin [OPTIONS]
       intunewin -c <folder> -s <file> -o <folder> [OPTIONS]

Options:
  -c, --content <PATH>   Source folder containing files to package
  -s, --setup <FILE>     Setup file name within source folder
  -o, --output <PATH>    Output folder for .intunewin file
  -n, --name <NAME>      Custom output filename (optional)
  -q, --quiet            Quiet mode (no prompts, overwrite existing)
      --silent           Silent mode (no console output)
  -h, --help             Print help information
  -v, --version          Print version information

Examples:
  intunewin                                    Interactive mode
  intunewin -c ./app -s setup.msi -o ./out     Package with CLI args
  intunewin -c ./app -s setup.exe -o ./out -q  Quiet mode

For more information: https://github.com/[repo]/intunewin
```

---

## Version Output

```
intunewin 1.0.0
```

---

## Compatibility Notes

### Microsoft IntuneWinAppUtil Compatibility

The following behaviors match the original Microsoft tool:
- `-c`, `-s`, `-o` parameter names
- `-q` for quiet mode
- Output file naming (setup file base name + .intunewin)
- Exit code 0 on success

The following are extensions beyond the original tool:
- `-n` for custom output naming
- `--silent` / `-qq` for completely silent operation
- Cross-platform support
- Progress bar display

### Known Differences

- Original tool uses `-a` for catalog folder (Windows 10 S mode) - not implemented
- Original tool has different internal error codes - we use simplified codes
- Original tool outputs to current directory if `-o` not specified - we require `-o`
