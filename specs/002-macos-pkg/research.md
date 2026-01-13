# Research: macOS Flat Package (.pkg) Creation

**Feature**: 002-macos-pkg
**Date**: 2026-01-13

## Research Topics

### 1. XAR Archive Creation

**Question**: What Rust crate or approach should be used for creating XAR archives?

**Research Findings**:

| Crate | Write Support | Issues |
|-------|---------------|--------|
| [apple-xar](https://crates.io/crates/apple-xar) 0.20.0 | ❌ Read-only | Despite description claiming write support, only has `reader` module |
| [zar](https://crates.io/crates/zar) 0.1.4 | ✅ Yes | Has insecure dependencies (user flagged) |
| [xar](https://crates.io/crates/xar) 0.1.1 | ❓ Unknown | Minimal documentation |

**XAR Format Analysis** (from [xarformat wiki](https://github.com/mackyle/xar/wiki/xarformat)):
- Simple structure: 28-byte header + zlib-compressed XML TOC + heap
- Header: magic "xar!" (0x78617221), size, version, TOC lengths, checksum algorithm
- For .pkg: files stored uncompressed in heap (payload already gzipped)

**Decision**: Implement XAR writer in pure Rust

**Rationale**:
- Format is well-documented and simple
- Avoids insecure dependencies from `zar`
- `apple-xar` lacks write support
- Only need uncompressed storage for .pkg (simplifies implementation)

**Alternatives Rejected**:
- `zar`: Insecure dependencies
- `libarchive2-sys`: Requires native library, breaks cross-platform goal

---

### 2. CPIO Archive Creation (odc format)

**Question**: What Rust crate should be used for creating CPIO archives in odc format?

**Research Findings**:

| Crate | Write Support | odc Format |
|-------|---------------|------------|
| [cpio-archive](https://crates.io/crates/cpio-archive) | ✅ Yes (`OdcBuilder`) | ✅ Supported |

**CPIO odc Format** (from [cpio(5) man page](https://man.archlinux.org/man/cpio.5.en)):
- 76-byte ASCII header with octal values
- Magic: "070707"
- Fields: dev, ino, mode, uid, gid, nlink, rdev, mtime, namesize, filesize
- No padding after pathname or file contents
- Archive ends with "TRAILER!!!" entry

**Decision**: Use `cpio-archive` crate with `OdcBuilder`

**Rationale**:
- Dedicated odc support
- Pure Rust
- Actively maintained

**Alternatives Rejected**:
- Custom implementation: Unnecessary when crate exists
- Binary cpio format: macOS requires odc (ASCII) format

---

### 3. Bill of Materials (BOM) Creation

**Question**: What Rust crate should be used for creating BOM files?

**Research Findings**:

| Crate | Write Support | Notes |
|-------|---------------|-------|
| [stuckliste](https://crates.io/crates/stuckliste) 0.3.8 | ✅ Yes | `ReceiptBuilder` + `Receipt::write()` |
| [apple-bom](https://crates.io/crates/apple-bom) 0.3.0 | ❓ Unknown | Part of apple-platform-rs |

**BOM Format** (from [bomutils](https://github.com/hogliux/bomutils)):
- Magic: "BOMStore" (8 bytes)
- Contains file paths, permissions, uid/gid, size, checksums
- Binary format, big-endian

**Decision**: Use `stuckliste` crate

**Rationale**:
- Explicit write support with `ReceiptBuilder` and `Receipt::write()`
- Pure Rust
- Can scan directories directly
- Well-documented API

**Alternatives Rejected**:
- `apple-bom`: Unclear if write support exists
- Custom implementation: Unnecessary complexity

---

### 4. XML Generation

**Question**: What approach for generating PackageInfo and Distribution XML?

**Decision**: Use existing `quick-xml` 0.31 (already in project)

**Rationale**:
- Already a project dependency
- Sufficient for generating simple XML structures
- Well-maintained

---

### 5. Gzip Compression

**Question**: What crate for gzip compression of CPIO payloads?

**Decision**: Use `flate2` crate

**Rationale**:
- Industry standard for Rust gzip
- Pure Rust backend available (`miniz_oxide`)
- Fast and well-tested

---

## Dependency Summary

### New Dependencies Required

```toml
[dependencies]
# Existing (no change needed)
quick-xml = "0.31"      # XML generation
walkdir = "2.4"         # File traversal

# New dependencies
stuckliste = "0.3"      # BOM file creation
cpio-archive = "*"      # CPIO odc archive creation (verify latest version)
flate2 = "1.0"          # Gzip compression

[features]
default = ["intune", "macos"]
intune = []
macos = ["stuckliste", "cpio-archive", "flate2"]
```

### Custom Implementation Required

- **XAR writer**: ~200-300 lines for basic uncompressed XAR creation
  - Header struct and serialization
  - XML TOC generation (using quick-xml)
  - Heap assembly with file data

---

## Technical Notes

### File Ownership in Packages

macOS packages require:
- uid: 0 (root)
- gid: 80 (wheel)

Both `stuckliste` and `cpio-archive` support setting custom uid/gid.

### Payload Structure

```
base.pkg/
├── Bom          # stuckliste output
├── PackageInfo  # quick-xml generated
├── Payload      # gzipped cpio-archive output
└── Scripts      # gzipped cpio-archive (optional)
```

### XAR Assembly (for final .pkg)

```
iamawrapper-test.pkg (XAR archive, no compression)
├── Distribution     # XML file
├── Resources/       # Optional resources
└── base.pkg/        # Component package directory
    └── [files above]
```
