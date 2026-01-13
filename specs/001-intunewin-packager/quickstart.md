# Quickstart: Cross-Platform IntuneWin Packager

**Date**: 2026-01-09
**Branch**: `001-intunewin-packager`

## Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs/))
- Cargo (included with Rust)

## Build from Source

```bash
# Clone the repository
git clone https://github.com/[org]/iamawrapper.git
cd iamawrapper

# Build release binary
cargo build --release

# Binary is at target/release/intunewin
```

## Installation

### From Binary Release

Download the appropriate binary for your platform from the releases page:

| Platform | Binary |
|----------|--------|
| Linux x64 | `intunewin-linux-x64` |
| Linux ARM64 | `intunewin-linux-arm64` |
| macOS x64 | `intunewin-macos-x64` |
| macOS ARM64 | `intunewin-macos-arm64` |
| Windows x64 | `intunewin-windows-x64.exe` |
| Windows ARM64 | `intunewin-windows-arm64.exe` |

### Add to PATH

**Linux/macOS:**
```bash
chmod +x intunewin-*
sudo mv intunewin-* /usr/local/bin/intunewin
```

**Windows:**
Move `intunewin.exe` to a directory in your PATH, or add its location to PATH.

## Basic Usage

### Interactive Mode

Run without arguments to be guided through the process:

```bash
intunewin
```

You'll be prompted for:
1. Source folder path
2. Setup file selection
3. Output folder path

### Command-Line Mode

```bash
intunewin -c <source_folder> -s <setup_file> -o <output_folder>
```

**Example:**
```bash
intunewin -c ./myapp -s setup.msi -o ./output
```

This creates `./output/setup.intunewin`.

### Quiet Mode (for scripts)

```bash
intunewin -c ./myapp -s setup.exe -o ./output -q
```

- No prompts
- Overwrites existing files
- Shows progress bar only

### Silent Mode (for CI/CD)

```bash
intunewin -c ./myapp -s setup.exe -o ./output --silent
```

- No console output
- Use exit code to check success (0 = success)

## Common Examples

### Package an MSI installer

```bash
intunewin -c ./installers/myapp -s MyApp.msi -o ./packages
# Creates: ./packages/MyApp.intunewin
```

### Package with custom name

```bash
intunewin -c ./app -s setup.exe -o ./out -n "MyApp-v2.0"
# Creates: ./out/MyApp-v2.0.intunewin
```

### Package in CI/CD pipeline

```bash
#!/bin/bash
intunewin -c "$SOURCE_DIR" -s "$SETUP_FILE" -o "$OUTPUT_DIR" --silent
if [ $? -eq 0 ]; then
    echo "Package created successfully"
else
    echo "Packaging failed with exit code $?"
    exit 1
fi
```

### Drop-in replacement for Microsoft tool

If you have existing scripts using `IntuneWinAppUtil`:

```bash
# Create symlink (Linux/macOS)
sudo ln -s /usr/local/bin/intunewin /usr/local/bin/IntuneWinAppUtil

# Existing scripts work unchanged
IntuneWinAppUtil -c ./app -s setup.exe -o ./output
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | Invalid arguments |
| 3 | Source folder empty |
| 4 | Setup file not found |
| 5 | Output write failed |

## Help

```bash
intunewin --help
intunewin -h
```

## Version

```bash
intunewin --version
intunewin -v
```

## Troubleshooting

### "Source folder not found"
- Verify the path exists
- Use absolute path or check current directory
- On Windows, use forward slashes or escape backslashes

### "Setup file not found"
- File name is case-sensitive on Linux/macOS
- File must be inside the source folder (not a subfolder path)
- Example: use `setup.exe` not `./subfolder/setup.exe`

### "Permission denied"
- Check read permissions on source folder
- Check write permissions on output folder
- On Unix, ensure binary has execute permission

### Large files taking too long
- 8GB files may take several minutes
- Progress bar shows status
- Memory usage stays under 100MB regardless of file size

## Next Steps

1. Upload the `.intunewin` file to Microsoft Intune
2. Configure detection rules in Intune
3. Assign to device groups for deployment

For more information, see the [full documentation](./README.md).
