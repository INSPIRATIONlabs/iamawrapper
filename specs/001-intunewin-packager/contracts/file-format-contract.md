# File Format Contract: IntuneWin Package

**Date**: 2026-01-09
**Version**: 1.0.0

## Overview

This document defines the exact file format contract for `.intunewin` packages. Compliance with this contract ensures packages are accepted by Microsoft Intune.

---

## Package Structure

### Outer Container

The `.intunewin` file is a standard ZIP archive (unencrypted) with the following structure:

```
<filename>.intunewin
└── IntuneWinPackage/
    ├── Metadata/
    │   └── Detection.xml
    └── Contents/
        └── IntunePackage.intunewin
```

### ZIP Requirements

| Property | Requirement |
|----------|-------------|
| Format | ZIP (PKWARE 2.0+) |
| Compression | DEFLATE (method 8) |
| Encryption | None (outer archive is unencrypted) |
| Entry paths | Forward slashes, UTF-8 names |

---

## Detection.xml Format

### XML Declaration

```xml
<?xml version="1.0" encoding="utf-8"?>
```

### Root Element

```xml
<ApplicationInfo xmlns:xsd="http://www.w3.org/2001/XMLSchema"
                 xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
```

### Complete Schema

```xml
<?xml version="1.0" encoding="utf-8"?>
<ApplicationInfo xmlns:xsd="http://www.w3.org/2001/XMLSchema"
                 xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance">
  <Name>{setup_file_name}</Name>
  <UnencryptedContentSize>{size_in_bytes}</UnencryptedContentSize>
  <FileName>IntunePackage.intunewin</FileName>
  <SetupFile>{setup_file_name}</SetupFile>
  <EncryptionInfo>
    <EncryptionKey>{base64_32_bytes}</EncryptionKey>
    <MacKey>{base64_32_bytes}</MacKey>
    <InitializationVector>{base64_16_bytes}</InitializationVector>
    <Mac>{base64_32_bytes}</Mac>
    <ProfileIdentifier>ProfileVersion1</ProfileIdentifier>
    <FileDigest>{base64_32_bytes}</FileDigest>
    <FileDigestAlgorithm>SHA256</FileDigestAlgorithm>
  </EncryptionInfo>
</ApplicationInfo>
```

### Element Specifications

| Element | Type | Description |
|---------|------|-------------|
| `Name` | string | Setup file name (e.g., "setup.exe") |
| `UnencryptedContentSize` | integer | Size of inner ZIP before encryption (bytes) |
| `FileName` | string | Always "IntunePackage.intunewin" |
| `SetupFile` | string | Setup file name (same as Name) |
| `EncryptionKey` | base64 | 32 bytes (256 bits) AES key |
| `MacKey` | base64 | 32 bytes (256 bits) HMAC key |
| `InitializationVector` | base64 | 16 bytes (128 bits) AES IV |
| `Mac` | base64 | 32 bytes HMAC-SHA256 result |
| `ProfileIdentifier` | string | Always "ProfileVersion1" |
| `FileDigest` | base64 | 32 bytes SHA256 of encrypted content |
| `FileDigestAlgorithm` | string | Always "SHA256" |

### Element Order

Elements MUST appear in the exact order shown above. Intune may reject packages with different ordering.

### Whitespace Rules

- No whitespace between elements (no indentation)
- No trailing newline after closing tag
- UTF-8 encoding without BOM

### Example Detection.xml

```xml
<?xml version="1.0" encoding="utf-8"?><ApplicationInfo xmlns:xsd="http://www.w3.org/2001/XMLSchema" xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"><Name>setup.exe</Name><UnencryptedContentSize>1048576</UnencryptedContentSize><FileName>IntunePackage.intunewin</FileName><SetupFile>setup.exe</SetupFile><EncryptionInfo><EncryptionKey>AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=</EncryptionKey><MacKey>AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=</MacKey><InitializationVector>AAAAAAAAAAAAAAAAAAAAAA==</InitializationVector><Mac>AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=</Mac><ProfileIdentifier>ProfileVersion1</ProfileIdentifier><FileDigest>AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=</FileDigest><FileDigestAlgorithm>SHA256</FileDigestAlgorithm></EncryptionInfo></ApplicationInfo>
```

---

## IntunePackage.intunewin Format

### Binary Layout

```
Offset    Size      Description
──────────────────────────────────────────
0x00      32        HMAC-SHA256 (authentication tag)
0x20      16        Initialization Vector
0x30      variable  AES-256-CBC encrypted content
```

### Byte Order

All binary values are stored in their natural order (not endian-converted).

### Encryption Specification

| Property | Value |
|----------|-------|
| Algorithm | AES-256-CBC |
| Key size | 256 bits (32 bytes) |
| Block size | 128 bits (16 bytes) |
| IV size | 128 bits (16 bytes) |
| Padding | PKCS#7 |

### HMAC Specification

| Property | Value |
|----------|-------|
| Algorithm | HMAC-SHA256 |
| Key size | 256 bits (32 bytes) |
| Output size | 256 bits (32 bytes) |
| Input | IV (16 bytes) || Ciphertext |

### Encryption Process

1. Generate 32 random bytes for encryption key
2. Generate 32 random bytes for MAC key
3. Generate 16 random bytes for IV
4. Create ZIP of source files (in-memory or streaming)
5. Encrypt ZIP with AES-256-CBC(key, IV) using PKCS#7 padding
6. Compute HMAC-SHA256(mac_key, IV || ciphertext)
7. Output: HMAC || IV || ciphertext
8. Compute SHA256 of output for FileDigest

### Decryption Process (for verification)

1. Read first 32 bytes as HMAC
2. Read next 16 bytes as IV
3. Read remainder as ciphertext
4. Verify HMAC-SHA256(mac_key, IV || ciphertext) == stored HMAC
5. Decrypt ciphertext with AES-256-CBC(key, IV)
6. Remove PKCS#7 padding
7. Result is ZIP archive

---

## Inner ZIP Format

### Structure

The encrypted content is a ZIP archive containing all source files:

```
{source_folder_contents}.zip
├── {file1}
├── {file2}
├── {subfolder}/
│   └── {file3}
└── ...
```

### ZIP Requirements

| Property | Requirement |
|----------|-------------|
| Format | ZIP (PKWARE 2.0+) |
| Compression | DEFLATE (method 8) or STORE (method 0) |
| Encryption | None (encryption handled at outer layer) |
| Paths | Relative to source folder root |
| Path separators | Forward slashes |

### File Inclusion Rules

- All files in source folder MUST be included
- Hidden files (dotfiles, hidden attribute) MUST be included
- Symbolic links MUST be followed (include target content)
- Empty directories MAY be omitted
- File permissions are NOT preserved (Windows target)

### Entry Ordering

Files SHOULD be added in alphabetical order for reproducibility, but this is not strictly required.

---

## Base64 Encoding

### Specification

- Standard Base64 as per RFC 4648
- Padding with `=` characters
- Character set: A-Z, a-z, 0-9, +, /

### Expected Lengths

| Value | Bytes | Base64 Length |
|-------|-------|---------------|
| EncryptionKey | 32 | 44 |
| MacKey | 32 | 44 |
| InitializationVector | 16 | 24 |
| Mac | 32 | 44 |
| FileDigest | 32 | 44 |

---

## Size Calculations

### UnencryptedContentSize

The `UnencryptedContentSize` field contains the exact size in bytes of the inner ZIP archive before encryption.

### Encrypted Content Overhead

```
encrypted_size = unencrypted_size + padding + 32 (HMAC) + 16 (IV)
               = unencrypted_size + (16 - (unencrypted_size % 16)) + 48
```

Maximum overhead: 48 + 15 = 63 bytes

---

## Validation Rules

### Package Validation

1. Outer file MUST be valid ZIP
2. ZIP MUST contain exactly `IntuneWinPackage/Metadata/Detection.xml`
3. ZIP MUST contain exactly `IntuneWinPackage/Contents/IntunePackage.intunewin`
4. Detection.xml MUST be valid XML matching schema
5. EncryptionInfo values MUST be valid Base64
6. Encrypted file size MUST match UnencryptedContentSize + overhead

### Cryptographic Validation

1. HMAC MUST verify against (IV || ciphertext) using MacKey
2. Decrypted content MUST have valid PKCS#7 padding
3. FileDigest MUST match SHA256 of encrypted file
4. Decrypted content MUST be valid ZIP

---

## Test Vectors

### Minimal Valid Package

For testing, a minimal valid package can be created with:
- Source: single 0-byte file named "test.exe"
- Results in smallest possible valid .intunewin

### Known Values Test

Given these fixed values (for testing only - production uses random):
```
encryption_key = 0x00 * 32 (32 zero bytes)
mac_key = 0x00 * 32 (32 zero bytes)
iv = 0x00 * 16 (16 zero bytes)
plaintext = "Hello, Intune!" (14 bytes, in a minimal ZIP)
```

The encrypted output should be deterministic and can be used for contract testing.

---

## Compatibility Notes

### Microsoft Tool Version

This contract is based on analysis of IntuneWinAppUtil version 1.8.x output.

### Intune Backend

The Intune service decrypts packages using the EncryptionInfo from Detection.xml. Changes to the encryption scheme would require Intune backend updates.

### Future Compatibility

If Microsoft updates the format, this contract should be versioned (currently assumes ProfileVersion1).
