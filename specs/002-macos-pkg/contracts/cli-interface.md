# CLI Interface Contract: iamawrapper

**Feature**: 002-macos-pkg
**Date**: 2026-01-13

## Command Structure

```
iamawrapper <COMMAND>

Commands:
  intune    Microsoft Intune packaging commands
  macos     macOS packaging commands
  help      Print help information
  --version Print version information
```

---

## Intune Subcommands

### `iamawrapper intune create`

Create a .intunewin package.

```
iamawrapper intune create [OPTIONS] -c <SOURCE> -s <SETUP> -o <OUTPUT>

Required:
  -c, --content <SOURCE>    Source folder containing files to package
  -s, --setup <SETUP>       Setup file name within source folder
  -o, --output <OUTPUT>     Output folder for .intunewin file

Optional:
  -n, --name <NAME>         Custom output filename (without extension)
  -q, --quiet               Quiet mode - no prompts, overwrite existing
      --silent              Silent mode - no console output
  -h, --help                Print help
```

**Exit Codes**:
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Invalid arguments |
| 2 | Source folder not found |
| 3 | Setup file not found |
| 4 | Output error |
| 5 | Cancelled by user |

---

### `iamawrapper intune extract`

Extract a .intunewin package.

```
iamawrapper intune extract [OPTIONS] -u <FILE> -o <OUTPUT>

Required:
  -u, --unpack <FILE>       Path to .intunewin file
  -o, --output <OUTPUT>     Output folder for extracted files

Optional:
  -q, --quiet               Quiet mode
      --silent              Silent mode
  -h, --help                Print help
```

---

## macOS Subcommands

### `iamawrapper macos pkg`

Create a macOS .pkg installer.

```
iamawrapper macos pkg [OPTIONS] -c <SOURCE> -o <OUTPUT> --identifier <ID> --version <VER>

Required:
  -c, --content <SOURCE>        Source folder containing files to package
  -o, --output <OUTPUT>         Output folder for .pkg file
      --identifier <ID>         Package identifier (e.g., com.company.app)
      --version <VER>           Package version (e.g., 1.0.0)

Optional:
      --install-location <PATH> Installation target path [default: /]
      --scripts <FOLDER>        Folder containing preinstall/postinstall scripts
  -n, --name <NAME>             Custom output filename (without extension)
  -q, --quiet                   Quiet mode - no prompts, overwrite existing
      --silent                  Silent mode - no console output
  -h, --help                    Print help
```

**Exit Codes**:
| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | Invalid arguments |
| 2 | Source folder not found |
| 3 | Source folder empty |
| 4 | Scripts folder not found |
| 5 | Output error |
| 6 | Cancelled by user |

---

## Interactive Mode

Running `iamawrapper` with no arguments enters interactive mode.

### Flow

```
$ iamawrapper

iamawrapper v0.2.0
Interactive Mode

? What would you like to do?
  > Create Microsoft Intune package (.intunewin)
    Create macOS package (.pkg)

[If macOS selected:]

? Source folder path: ./MyApp
? Package identifier: com.company.myapp
? Package version: 1.0.0
? Install location [/]: /Applications
? Scripts folder (optional, press Enter to skip):
? Output folder path: ./output

Package Summary:
  Source folder: ./MyApp
  Identifier: com.company.myapp
  Version: 1.0.0
  Install location: /Applications
  Output: ./output/com.company.myapp-1.0.0.pkg

? Proceed with packaging? [Y/n]
```

---

## Output Formats

### Normal Mode (default)

```
iamawrapper v0.2.0

Source folder: /path/to/source
Identifier: com.company.app
Version: 1.0.0
Install location: /Applications
Output folder: /path/to/output

⠋ [00:00:02] [████████████████████████████████████████] 15/15 Adding files...

Package created successfully:
  /path/to/output/com.company.app-1.0.0.pkg (2.45 MB)
  Creation time: 1.23s
```

### Quiet Mode (-q)

```
/path/to/output/com.company.app-1.0.0.pkg
```

### Silent Mode (--silent)

No output (exit code only).

---

## Error Messages

| Scenario | Message |
|----------|---------|
| Missing required arg | `error: the following required arguments were not provided: --identifier <ID>` |
| Source not found | `Error: Source folder not found: /path/to/source` |
| Empty source | `Error: Source folder is empty: /path/to/source` |
| Scripts not found | `Error: Scripts folder not found: /path/to/scripts` |
| Invalid scripts | `Warning: No preinstall or postinstall scripts found in /path/to/scripts` |
| Output exists | `Error: Output file already exists: /path/to/file.pkg` (normal mode) |
| Identifier warning | `Warning: Identifier 'myapp' does not follow reverse-DNS convention (e.g., com.company.app)` |

---

## Version Information

```
$ iamawrapper --version
iamawrapper 0.2.0
```

---

## Help Output

```
$ iamawrapper --help
Cross-platform packaging tool for Microsoft Intune and macOS

Usage: iamawrapper <COMMAND>

Commands:
  intune  Microsoft Intune packaging (.intunewin)
  macos   macOS packaging (.pkg)
  help    Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```
