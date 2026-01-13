# iamawrapper

A cross-platform command-line tool for creating application packages for enterprise deployment.

Supports both Microsoft Intune (`.intunewin`) and macOS flat packages (`.pkg`), written in Rust for performance and cross-platform compatibility.

## Features

### Intune Packages (.intunewin)
- **Create** `.intunewin` packages from any folder
- **Extract** existing `.intunewin` packages back to original files
- **Compatible**: Output files are fully compatible with Microsoft Intune

### macOS Packages (.pkg)
- **Create** macOS flat packages compatible with the macOS Installer
- **Scripts**: Support for preinstall and postinstall scripts
- **No dependencies**: Works on any platform (Windows, macOS, Linux)

### General
- **Cross-platform**: Build packages for any platform from any platform
- **Fast**: Native Rust implementation with minimal dependencies
- **Interactive mode**: Guided wizard for package creation

## Installation

### Download

Download the latest release for your platform from the [Releases](https://github.com/INSPIRATIONLABS/iamawrapper/releases) page.

| Platform | Architecture | Download |
|----------|--------------|----------|
| Windows | x64 | `iamawrapper-windows-x64.zip` |
| Windows | ARM64 | `iamawrapper-windows-arm64.zip` |
| macOS | x64 (Intel) | `iamawrapper-macos-x64.zip` |
| macOS | ARM64 (Apple Silicon) | `iamawrapper-macos-arm64.zip` |
| Linux | x64 | `iamawrapper-linux-x64.zip` |
| Linux | ARM64 | `iamawrapper-linux-arm64.zip` |

### Build from Source

Requires [Rust](https://rustup.rs/) 1.75 or later.

```bash
git clone https://github.com/INSPIRATIONLABS/iamawrapper.git
cd iamawrapper
cargo build --release
```

The binary will be at `target/release/iamawrapper` (or `iamawrapper.exe` on Windows).

## Usage

### Intune Packages

#### Create an Intune Package

```bash
iamawrapper intune create -c <source_folder> -s <setup_file> -o <output_folder>
```

**Arguments:**

| Flag | Description |
|------|-------------|
| `-c, --content` | Source folder containing your application files |
| `-s, --setup` | The setup file (e.g., `install.exe`, `setup.msi`, `install.ps1`) |
| `-o, --output` | Output folder where the `.intunewin` file will be created |
| `-q, --quiet` | Suppress all output |

**Example:**

```bash
# Package a PowerShell installer
iamawrapper intune create -c ./MyApp -s install.ps1 -o ./output

# Package an MSI installer
iamawrapper intune create -c ./Installer -s setup.msi -o ./packages
```

This creates a file like `output/install.intunewin` that can be uploaded to Microsoft Intune.

#### Extract an Intune Package

```bash
iamawrapper intune extract -i <intunewin_file> -o <output_folder>
```

**Example:**

```bash
# Extract an existing .intunewin file
iamawrapper intune extract -i MyApp.intunewin -o ./extracted
```

### macOS Packages

#### Create a macOS Package

```bash
iamawrapper macos pkg -c <source_folder> -o <output.pkg> --identifier <id> --version <version>
```

**Arguments:**

| Flag | Description |
|------|-------------|
| `-c, --content` | Source folder containing your application files |
| `-o, --output` | Output path for the `.pkg` file |
| `--identifier` | Package identifier in reverse-DNS format (e.g., `com.company.app`) |
| `--version` | Package version (e.g., `1.0.0`) |
| `--install-location` | Install location on target system (default: `/`) |
| `--scripts` | Folder containing preinstall/postinstall scripts |

**Examples:**

```bash
# Create a basic package
iamawrapper macos pkg -c ./MyApp.app -o ./MyApp.pkg \
  --identifier com.company.myapp --version 1.0.0

# Package with custom install location
iamawrapper macos pkg -c ./MyApp.app -o ./MyApp.pkg \
  --identifier com.company.myapp --version 1.0.0 \
  --install-location /Applications

# Package with installation scripts
iamawrapper macos pkg -c ./MyApp.app -o ./MyApp.pkg \
  --identifier com.company.myapp --version 1.0.0 \
  --scripts ./scripts
```

The scripts folder should contain `preinstall` and/or `postinstall` shell scripts.

### Interactive Mode

Run without arguments to enter interactive mode:

```bash
iamawrapper
```

You will be prompted to select the package type (Intune or macOS) and enter the required parameters.

## How It Works

### Intune Package Format

The `.intunewin` format is a ZIP archive containing:

```
IntuneWinPackage/
├── Contents/
│   └── IntunePackage.intunewin  (AES-256-CBC encrypted ZIP of source files)
└── Metadata/
    └── Detection.xml            (Encryption keys and package metadata)
```

When you upload a `.intunewin` file to Intune, the service uses the metadata to decrypt and deploy your application to managed devices.

### macOS Package Format

The `.pkg` format is a XAR archive containing:

```
Distribution           (XML installer configuration)
base.pkg/
├── Bom               (Bill of Materials - file manifest)
├── Payload           (gzip-compressed CPIO archive of files)
├── PackageInfo       (XML package metadata)
└── Scripts           (gzip-compressed CPIO archive of scripts, optional)
```

macOS packages created by iamawrapper are compatible with the standard macOS Installer application and can be installed via double-click or command line (`installer -pkg MyApp.pkg -target /`).

## Comparison

| Feature | iamawrapper | Microsoft Tool | Apple pkgbuild |
|---------|-------------|----------------|----------------|
| Intune packages | Yes | Yes | No |
| macOS packages | Yes | No | Yes |
| Extract packages | Yes | No | No |
| Windows | Yes | Yes | No |
| macOS | Yes | No | Yes |
| Linux | Yes | No | No |
| Source available | Yes | No | No |

## License

**iamawrapper** is source-available under a custom license.

### Permitted Use

- Package your own applications for deployment
- Internal business use
- Personal and educational use

### Not Permitted

- Building competing deployment/packaging products
- Offering as a managed service (SaaS)
- Patch management or MDM products

### Commercial License

Need to use iamawrapper in a commercial product? [Contact us](mailto:license@inspirationlabs.com) for a commercial license.

See [LICENSE](LICENSE) for full terms.

Copyright (c) 2026 INSPIRATIONLABS GmbH
