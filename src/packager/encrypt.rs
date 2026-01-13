//! Encryption module for IntuneWin packages.
//!
//! Implements AES-256-CBC encryption with HMAC-SHA256 authentication.

use aes::cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit, block_padding::Pkcs7};
use hmac::{Hmac, Mac};
use rand::RngCore;
use sha2::{Digest, Sha256};

use crate::models::detection::EncryptionInfo;
use crate::models::error::{PackageError, PackageResult};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;
type HmacSha256 = Hmac<Sha256>;

/// Encrypt content using AES-256-CBC with HMAC-SHA256 authentication.
///
/// Returns the encrypted content (HMAC || IV || ciphertext) and encryption info.
pub fn encrypt_content(plaintext: &[u8]) -> PackageResult<(Vec<u8>, EncryptionInfo)> {
    let mut info = EncryptionInfo::new();

    // Generate random keys and IV
    generate_keys(&mut info)?;

    // Encrypt the content
    let ciphertext = aes_encrypt(plaintext, &info.encryption_key, &info.iv)?;

    // Compute HMAC over (IV || ciphertext)
    let mut hmac_input = Vec::with_capacity(info.iv.len() + ciphertext.len());
    hmac_input.extend_from_slice(&info.iv);
    hmac_input.extend_from_slice(&ciphertext);

    info.mac = compute_hmac(&info.mac_key, &hmac_input)?;

    // Assemble output: HMAC (32 bytes) || IV (16 bytes) || ciphertext
    let mut output = Vec::with_capacity(32 + 16 + ciphertext.len());
    output.extend_from_slice(&info.mac);
    output.extend_from_slice(&info.iv);
    output.extend_from_slice(&ciphertext);

    // Compute file digest (SHA256 of the unencrypted content, per Microsoft spec)
    info.file_digest = compute_sha256(plaintext);

    Ok((output, info))
}

/// Decrypt content that was encrypted with AES-256-CBC and HMAC-SHA256 authentication.
///
/// Expects encrypted data in format: HMAC (32 bytes) || IV (16 bytes) || ciphertext
pub fn decrypt_content(
    encrypted_data: &[u8],
    encryption_info: &EncryptionInfo,
) -> PackageResult<Vec<u8>> {
    // Minimum size: HMAC (32) + IV (16) + at least one block (16)
    if encrypted_data.len() < 64 {
        return Err(PackageError::DecryptionError {
            reason: "Encrypted data too short".to_string(),
        });
    }

    // Extract components
    let stored_hmac = &encrypted_data[0..32];
    let iv = &encrypted_data[32..48];
    let ciphertext = &encrypted_data[48..];

    // Verify HMAC over (IV || ciphertext)
    let mut hmac_input = Vec::with_capacity(iv.len() + ciphertext.len());
    hmac_input.extend_from_slice(iv);
    hmac_input.extend_from_slice(ciphertext);

    let computed_hmac = compute_hmac(&encryption_info.mac_key, &hmac_input)?;

    // Constant-time comparison to prevent timing attacks
    if !constant_time_compare(&computed_hmac, stored_hmac) {
        return Err(PackageError::HmacVerificationFailed);
    }

    // Decrypt
    let mut iv_array = [0u8; 16];
    iv_array.copy_from_slice(iv);

    aes_decrypt(ciphertext, &encryption_info.encryption_key, &iv_array)
}

/// Constant-time comparison to prevent timing attacks.
fn constant_time_compare(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let mut result = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        result |= x ^ y;
    }
    result == 0
}

/// Decrypt data with AES-256-CBC and remove PKCS7 padding.
fn aes_decrypt(ciphertext: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> PackageResult<Vec<u8>> {
    if ciphertext.is_empty() || ciphertext.len() % 16 != 0 {
        return Err(PackageError::DecryptionError {
            reason: "Invalid ciphertext length (must be multiple of 16)".to_string(),
        });
    }

    let mut buffer = ciphertext.to_vec();
    let decryptor = Aes256CbcDec::new(key.into(), iv.into());

    let plaintext = decryptor
        .decrypt_padded_mut::<Pkcs7>(&mut buffer)
        .map_err(|_| PackageError::InvalidPadding)?;

    Ok(plaintext.to_vec())
}

/// Generate random encryption key, MAC key, and IV.
fn generate_keys(info: &mut EncryptionInfo) -> PackageResult<()> {
    let mut rng = rand::thread_rng();

    rng.fill_bytes(&mut info.encryption_key);
    rng.fill_bytes(&mut info.mac_key);
    rng.fill_bytes(&mut info.iv);

    Ok(())
}

/// Encrypt data with AES-256-CBC using PKCS7 padding.
fn aes_encrypt(plaintext: &[u8], key: &[u8; 32], iv: &[u8; 16]) -> PackageResult<Vec<u8>> {
    // Calculate padded size (PKCS7 padding to 16-byte boundary)
    let block_size = 16;
    let padding = block_size - (plaintext.len() % block_size);
    let padded_len = plaintext.len() + padding;

    let mut buffer = vec![0u8; padded_len];
    buffer[..plaintext.len()].copy_from_slice(plaintext);

    let encryptor = Aes256CbcEnc::new(key.into(), iv.into());

    let ciphertext = encryptor
        .encrypt_padded_mut::<Pkcs7>(&mut buffer, plaintext.len())
        .map_err(|e| PackageError::EncryptionError {
            reason: format!("AES encryption failed: {}", e),
        })?;

    Ok(ciphertext.to_vec())
}

/// Compute HMAC-SHA256.
fn compute_hmac(key: &[u8; 32], data: &[u8]) -> PackageResult<[u8; 32]> {
    let mut mac = HmacSha256::new_from_slice(key).map_err(|e| PackageError::EncryptionError {
        reason: format!("HMAC initialization failed: {}", e),
    })?;

    mac.update(data);

    let result = mac.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result.into_bytes());

    Ok(output)
}

/// Compute SHA256 hash.
fn compute_sha256(data: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(data);

    let result = hasher.finalize();
    let mut output = [0u8; 32];
    output.copy_from_slice(&result);

    output
}

/// Verify HMAC for decryption (used for testing).
#[allow(dead_code)]
pub fn verify_hmac(key: &[u8; 32], data: &[u8], expected: &[u8; 32]) -> bool {
    match compute_hmac(key, data) {
        Ok(computed) => computed == *expected,
        Err(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_content_structure() {
        let plaintext = b"Hello, Intune!";

        let (encrypted, info) = encrypt_content(plaintext).unwrap();

        // Verify structure: HMAC (32) + IV (16) + ciphertext
        assert!(encrypted.len() >= 48);
        assert_eq!(&encrypted[0..32], &info.mac);
        assert_eq!(&encrypted[32..48], &info.iv);
    }

    #[test]
    fn test_encrypt_content_unique_keys() {
        let plaintext = b"Test data";

        let (_, info1) = encrypt_content(plaintext).unwrap();
        let (_, info2) = encrypt_content(plaintext).unwrap();

        // Each encryption should generate unique keys
        assert_ne!(info1.encryption_key, info2.encryption_key);
        assert_ne!(info1.mac_key, info2.mac_key);
        assert_ne!(info1.iv, info2.iv);
    }

    #[test]
    fn test_encrypt_content_hmac_verification() {
        let plaintext = b"Test data for HMAC";

        let (encrypted, info) = encrypt_content(plaintext).unwrap();

        // Extract IV and ciphertext
        let iv = &encrypted[32..48];
        let ciphertext = &encrypted[48..];

        // Verify HMAC
        let mut hmac_input = Vec::new();
        hmac_input.extend_from_slice(iv);
        hmac_input.extend_from_slice(ciphertext);

        assert!(verify_hmac(&info.mac_key, &hmac_input, &info.mac));
    }

    #[test]
    fn test_encrypt_content_file_digest() {
        let plaintext = b"Test data";

        let (_encrypted, info) = encrypt_content(plaintext).unwrap();

        // File digest should be SHA256 of the unencrypted content (per Microsoft spec)
        let expected_digest = compute_sha256(plaintext);
        assert_eq!(info.file_digest, expected_digest);
    }

    #[test]
    fn test_aes_encrypt_padding() {
        let key = [0u8; 32];
        let iv = [0u8; 16];

        // Test with data not aligned to block size
        let plaintext = b"Hello"; // 5 bytes

        let ciphertext = aes_encrypt(plaintext, &key, &iv).unwrap();

        // Should be padded to 16 bytes (one block)
        assert_eq!(ciphertext.len(), 16);
    }

    #[test]
    fn test_aes_encrypt_multiple_blocks() {
        let key = [0u8; 32];
        let iv = [0u8; 16];

        // Test with data spanning multiple blocks (17 bytes = 2 blocks after padding)
        let plaintext = b"12345678901234567"; // 17 bytes

        let ciphertext = aes_encrypt(plaintext, &key, &iv).unwrap();

        // Should be padded to 32 bytes (two blocks)
        assert_eq!(ciphertext.len(), 32);
    }

    #[test]
    fn test_compute_sha256() {
        let data = b"test";
        let hash = compute_sha256(data);

        // Known SHA256 hash of "test"
        let expected = [
            0x9f, 0x86, 0xd0, 0x81, 0x88, 0x4c, 0x7d, 0x65, 0x9a, 0x2f, 0xea, 0xa0, 0xc5, 0x5a,
            0xd0, 0x15, 0xa3, 0xbf, 0x4f, 0x1b, 0x2b, 0x0b, 0x82, 0x2c, 0xd1, 0x5d, 0x6c, 0x15,
            0xb0, 0xf0, 0x0a, 0x08,
        ];

        assert_eq!(hash, expected);
    }

    #[test]
    fn test_encryption_info_profile() {
        let plaintext = b"test";
        let (_, info) = encrypt_content(plaintext).unwrap();

        assert_eq!(info.profile_identifier, "ProfileVersion1");
        assert_eq!(info.file_digest_algorithm, "SHA256");
    }

    #[test]
    fn test_decrypt_content_roundtrip() {
        let plaintext = b"Hello, Intune! This is a test message for encryption round-trip.";

        let (encrypted, info) = encrypt_content(plaintext).unwrap();
        let decrypted = decrypt_content(&encrypted, &info).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_content_short_message() {
        let plaintext = b"Hi";

        let (encrypted, info) = encrypt_content(plaintext).unwrap();
        let decrypted = decrypt_content(&encrypted, &info).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_content_exact_block_size() {
        // Exactly 16 bytes (one block)
        let plaintext = b"0123456789ABCDEF";

        let (encrypted, info) = encrypt_content(plaintext).unwrap();
        let decrypted = decrypt_content(&encrypted, &info).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_content_large_data() {
        // Test with larger data
        let plaintext: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();

        let (encrypted, info) = encrypt_content(&plaintext).unwrap();
        let decrypted = decrypt_content(&encrypted, &info).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_decrypt_content_invalid_hmac() {
        let plaintext = b"Test data";

        let (mut encrypted, info) = encrypt_content(plaintext).unwrap();

        // Tamper with the HMAC
        encrypted[0] ^= 0xFF;

        let result = decrypt_content(&encrypted, &info);
        assert!(matches!(
            result,
            Err(crate::models::error::PackageError::HmacVerificationFailed)
        ));
    }

    #[test]
    fn test_decrypt_content_too_short() {
        let info = EncryptionInfo::new();

        // Less than minimum 64 bytes
        let short_data = vec![0u8; 63];

        let result = decrypt_content(&short_data, &info);
        assert!(matches!(
            result,
            Err(crate::models::error::PackageError::DecryptionError { .. })
        ));
    }

    #[test]
    fn test_aes_decrypt_roundtrip() {
        let key = [1u8; 32];
        let iv = [2u8; 16];
        let plaintext = b"Test decryption";

        let ciphertext = aes_encrypt(plaintext, &key, &iv).unwrap();
        let decrypted = aes_decrypt(&ciphertext, &key, &iv).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_constant_time_compare_equal() {
        let a = [1, 2, 3, 4, 5];
        let b = [1, 2, 3, 4, 5];
        assert!(constant_time_compare(&a, &b));
    }

    #[test]
    fn test_constant_time_compare_not_equal() {
        let a = [1, 2, 3, 4, 5];
        let b = [1, 2, 3, 4, 6];
        assert!(!constant_time_compare(&a, &b));
    }

    #[test]
    fn test_constant_time_compare_different_lengths() {
        let a = [1, 2, 3];
        let b = [1, 2, 3, 4];
        assert!(!constant_time_compare(&a, &b));
    }
}
