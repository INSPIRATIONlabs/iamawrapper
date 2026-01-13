# Data Model: macOS Flat Package (.pkg) Creation

**Feature**: 002-macos-pkg
**Date**: 2026-01-13

## Entity Overview

```
┌─────────────────────┐      ┌──────────────────┐
│  MacosPkgRequest    │──────│  MacosPkgResult  │
└─────────────────────┘      └──────────────────┘
         │
         │ produces
         ▼
┌─────────────────────────────────────────────────┐
│                 Flat Package (.pkg)              │
├─────────────────────────────────────────────────┤
│  ┌─────────────┐  ┌───────────────────────────┐ │
│  │ Distribution│  │        base.pkg/          │ │
│  │    (XML)    │  ├───────────────────────────┤ │
│  └─────────────┘  │ ┌─────────┐ ┌───────────┐ │ │
│                   │ │   Bom   │ │PackageInfo│ │ │
│  ┌─────────────┐  │ │ (binary)│ │   (XML)   │ │ │
│  │  Resources/ │  │ └─────────┘ └───────────┘ │ │
│  │  (optional) │  │ ┌─────────┐ ┌───────────┐ │ │
│  └─────────────┘  │ │ Payload │ │  Scripts  │ │ │
│                   │ │(gz+cpio)│ │(gz+cpio)  │ │ │
│                   │ └─────────┘ └───────────┘ │ │
│                   └───────────────────────────┘ │
└─────────────────────────────────────────────────┘
              (XAR archive, uncompressed)
```

## Core Entities

### MacosPkgRequest

Request parameters for creating a macOS package.

| Field | Type | Required | Default | Description |
|-------|------|----------|---------|-------------|
| source_folder | PathBuf | Yes | - | Path to folder containing files to package |
| identifier | String | Yes | - | Package identifier (e.g., "com.company.app") |
| version | String | Yes | - | Package version (e.g., "1.0.0") |
| install_location | PathBuf | No | "/" | Target installation path on macOS |
| output_folder | PathBuf | Yes | - | Where to write the .pkg file |
| output_name | Option<String> | No | identifier-based | Custom output filename |
| scripts_folder | Option<PathBuf> | No | None | Folder containing preinstall/postinstall |
| verbosity | Verbosity | No | Normal | Output verbosity level |

**Validation Rules**:
- `source_folder` must exist and be a directory
- `source_folder` must not be empty
- `identifier` should follow reverse-DNS convention (warn if not)
- `version` should be semver-like (no strict validation)
- `scripts_folder`, if provided, must exist

---

### MacosPkgResult

Result of successful package creation.

| Field | Type | Description |
|-------|------|-------------|
| output_path | PathBuf | Full path to created .pkg file |
| package_size | u64 | Size of final .pkg in bytes |
| file_count | usize | Number of files in payload |
| creation_time | Duration | Time to create package |

---

### PackagePayload

Gzip-compressed CPIO archive of files to install.

| Field | Type | Description |
|-------|------|-------------|
| files | Vec<PayloadFile> | Files included in payload |
| total_size | u64 | Uncompressed total size |

**PayloadFile**:
| Field | Type | Description |
|-------|------|-------------|
| relative_path | PathBuf | Path relative to install_location |
| size | u64 | File size in bytes |
| mode | u32 | Unix permissions (preserved from source) |
| uid | u32 | Always 0 (root) |
| gid | u32 | Always 80 (wheel) |

---

### BillOfMaterials (Bom)

Binary manifest of all files in the package.

| Field | Type | Description |
|-------|------|-------------|
| entries | Vec<BomEntry> | File/directory entries |

**BomEntry**:
| Field | Type | Description |
|-------|------|-------------|
| path | String | Relative path |
| mode | u16 | Unix permissions |
| uid | u32 | Owner (0 = root) |
| gid | u32 | Group (80 = wheel) |
| size | u64 | File size |
| checksum | u32 | CRC32 checksum |

---

### PackageInfo

XML document describing the package component.

```xml
<pkg-info format-version="2"
          identifier="com.company.app"
          version="1.0.0"
          install-location="/"
          auth="root">
  <payload installKBytes="123" numberOfFiles="10"/>
  <scripts>
    <preinstall file="./preinstall"/>
    <postinstall file="./postinstall"/>
  </scripts>
</pkg-info>
```

| Element/Attribute | Description |
|-------------------|-------------|
| format-version | Always "2" |
| identifier | Package identifier |
| version | Package version |
| install-location | Target path |
| auth | Always "root" |
| payload.installKBytes | Total size in KB |
| payload.numberOfFiles | File count |
| scripts.preinstall | Present if preinstall script exists |
| scripts.postinstall | Present if postinstall script exists |

---

### Distribution

XML document for the installer UI.

```xml
<?xml version="1.0" encoding="utf-8"?>
<installer-script minSpecVersion="1.000000">
    <title>Package Title</title>
    <options customize="never" allow-external-scripts="no"/>
    <domains enable_anywhere="true"/>
    <choices-outline>
        <line choice="choice1"/>
    </choices-outline>
    <choice id="choice1" title="base">
        <pkg-ref id="com.company.app"/>
    </choice>
    <pkg-ref id="com.company.app"
             installKBytes="123"
             version="1.0.0"
             auth="Root">#base.pkg</pkg-ref>
</installer-script>
```

---

### ScriptsArchive

Optional gzip-compressed CPIO archive of install scripts.

| Script | Mode | Purpose |
|--------|------|---------|
| preinstall | 0755 | Runs before payload extraction |
| postinstall | 0755 | Runs after payload extraction |

**Script Arguments** (provided by macOS installer):
- `$1` - Package path
- `$2` - Install target
- `$3` - Target volume
- `$4` - Root path

---

## XAR Archive Structure

The final .pkg file is a XAR archive:

| Component | Compression | Description |
|-----------|-------------|-------------|
| Header | N/A | 28 bytes, magic "xar!" |
| TOC | zlib | XML table of contents |
| Heap | None | File data (already compressed where needed) |

**XAR Header** (28 bytes, big-endian):
```
Offset  Size  Field
0       4     Magic: 0x78617221 ("xar!")
4       2     Header size (28)
6       2     Version (1)
8       8     TOC compressed length
16      8     TOC uncompressed length
24      4     Checksum algorithm (0=none, 1=sha1)
```

---

## State Transitions

```
MacosPkgRequest
     │
     ▼ validate()
[Validated Request]
     │
     ├──────────────────────────────┐
     ▼                              ▼
collect_files()              collect_scripts()
     │                              │
     ▼                              ▼
[SourcePackage]              [ScriptsArchive]
     │                              │
     ├──────────────────────────────┤
     ▼                              ▼
create_payload()             create_scripts_archive()
     │                              │
     ▼                              ▼
[Payload.gz]                 [Scripts.gz]
     │                              │
     ├──────────────────────────────┤
     ▼
generate_bom()  ──────▶  [Bom]
     │
     ▼
generate_packageinfo()  ──▶  [PackageInfo.xml]
     │
     ▼
generate_distribution()  ──▶  [Distribution.xml]
     │
     ▼
create_xar_archive()
     │
     ▼
MacosPkgResult
```

---

## Shared Types (reused from Intune)

| Type | Location | Usage |
|------|----------|-------|
| Verbosity | models/package.rs | Normal/Quiet/Silent modes |
| SourceFile | models/package.rs | File metadata during collection |
| SourcePackage | models/package.rs | Collected files from source folder |
