# Quickstart: macOS Package Development

**Feature**: 002-macos-pkg
**Date**: 2026-01-13

## Prerequisites

- Rust 1.75+ (edition 2024)
- macOS, Linux, or Windows for development
- macOS for testing packages (VM acceptable)

## Setup

### 1. Clone and Build

```bash
git clone https://github.com/INSPIRATIONLABS/iamawrapper.git
cd iamawrapper
git checkout 002-macos-pkg

# Build with macOS feature
cargo build --features macos

# Run tests
cargo test
```

### 2. Add New Dependencies

Update `Cargo.toml`:

```toml
[dependencies]
# ... existing deps ...

# New for macOS packaging
stuckliste = "0.3"      # BOM file creation
cpio-archive = "0.4"    # CPIO archive creation (verify version)
flate2 = "1.0"          # Gzip compression

[features]
default = ["intune", "macos"]
intune = []
macos = ["stuckliste", "cpio-archive", "flate2"]
```

### 3. Verify Build

```bash
cargo build --features macos
cargo clippy --features macos
cargo fmt --check
```

## Development Workflow

### Test-First Development

Per constitution, all code must follow TDD:

```bash
# 1. Write failing test
cargo test macos::bom::tests::test_bom_creation -- --nocapture

# 2. See it fail (Red)
# 3. Implement minimum code (Green)
# 4. Refactor if needed
# 5. Commit
```

### Module Development Order

1. **XAR writer** (`src/macos/xar.rs`)
   - Implement header serialization
   - TOC XML generation
   - Heap assembly

2. **CPIO wrapper** (`src/macos/cpio.rs`)
   - Wrap `cpio-archive` for our use case
   - Set uid/gid to 0/80

3. **BOM wrapper** (`src/macos/bom.rs`)
   - Wrap `stuckliste` for our use case

4. **XML generation** (`src/macos/xml.rs`)
   - PackageInfo generation
   - Distribution generation

5. **Payload assembly** (`src/macos/payload.rs`)
   - Combine CPIO + gzip
   - Scripts handling

6. **Main package function** (`src/macos/mod.rs`)
   - Orchestrate all components

7. **CLI restructure** (`src/cli/args.rs`)
   - Add subcommands

## Testing

### Unit Tests

```bash
# Run all macos tests
cargo test macos

# Run specific module tests
cargo test macos::xar
cargo test macos::bom
cargo test macos::cpio
```

### Integration Tests

```bash
# Create test package
cargo run -- macos pkg \
  -c ./test-pkg/build/root \
  -o ./output \
  --identifier com.test.app \
  --version 1.0.0

# Verify on macOS
# (must be run on actual macOS)
xar -tf output/com.test.app-1.0.0.pkg
```

### Manual Testing on macOS

```bash
# Install the package
sudo installer -pkg output/com.test.app-1.0.0.pkg -target /

# Verify installation
ls -la /tmp/iamawrapper-test.txt
cat /tmp/iamawrapper-install.log
```

## Project Structure

```
src/
├── macos/              # NEW MODULE
│   ├── mod.rs          # Module exports, package() function
│   ├── xar.rs          # XAR archive writer
│   ├── bom.rs          # BOM file wrapper
│   ├── cpio.rs         # CPIO archive wrapper
│   ├── xml.rs          # XML generation
│   └── payload.rs      # Payload assembly
├── cli/
│   ├── args.rs         # MODIFY: Add subcommands
│   └── mod.rs          # MODIFY: Route to macos
└── models/
    └── macos.rs        # NEW: macOS-specific types
```

## Key Files to Modify

| File | Change |
|------|--------|
| `Cargo.toml` | Add dependencies, features |
| `src/lib.rs` | Export macos module |
| `src/cli/args.rs` | Restructure for subcommands |
| `src/cli/mod.rs` | Add macos command routing |
| `src/cli/interactive.rs` | Add platform selection |
| `src/models/mod.rs` | Export macos types |
| `src/models/error.rs` | Add macos errors |

## Code Style

- Run `cargo fmt` before committing
- Run `cargo clippy` - fix all warnings
- Follow existing code patterns in `src/packager/`

## Reference Implementation

The manual test package we created earlier serves as reference:
- Location: `/workspaces/iamawrapper/test-pkg/build/`
- Includes: Payload, Scripts, Bom, PackageInfo, Distribution

```bash
# View structure
ls -la test-pkg/build/flat/
ls -la test-pkg/build/flat/base.pkg/
```

## Debugging Tips

### Inspect XAR archives

```bash
# List contents
xar -tf package.pkg

# Extract
xar -xf package.pkg

# View TOC
xar --dump-toc=toc.xml -f package.pkg
```

### Inspect BOM files

```bash
# On macOS
lsbom path/to/Bom

# Using stuckliste-cli
cargo install stuckliste-cli
stuckliste dump path/to/Bom
```

### Inspect CPIO payloads

```bash
# Decompress and list
gunzip -c Payload | cpio -t
```
