//! Detection metadata models for IntuneWin packages.

use base64::{Engine, engine::general_purpose::STANDARD as BASE64};

/// Encryption information for the package.
#[derive(Debug, Clone)]
pub struct EncryptionInfo {
    /// AES-256 encryption key (32 bytes)
    pub encryption_key: [u8; 32],
    /// HMAC-SHA256 key (32 bytes)
    pub mac_key: [u8; 32],
    /// AES initialization vector (16 bytes)
    pub iv: [u8; 16],
    /// HMAC-SHA256 of (IV || ciphertext)
    pub mac: [u8; 32],
    /// SHA256 hash of the encrypted file
    pub file_digest: [u8; 32],
    /// Profile identifier (always "ProfileVersion1")
    pub profile_identifier: String,
    /// Digest algorithm name (always "SHA256")
    pub file_digest_algorithm: String,
}

impl Default for EncryptionInfo {
    fn default() -> Self {
        Self {
            encryption_key: [0u8; 32],
            mac_key: [0u8; 32],
            iv: [0u8; 16],
            mac: [0u8; 32],
            file_digest: [0u8; 32],
            profile_identifier: "ProfileVersion1".to_string(),
            file_digest_algorithm: "SHA256".to_string(),
        }
    }
}

impl EncryptionInfo {
    /// Create new encryption info with default profile settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get Base64-encoded encryption key.
    pub fn encryption_key_base64(&self) -> String {
        BASE64.encode(self.encryption_key)
    }

    /// Get Base64-encoded MAC key.
    pub fn mac_key_base64(&self) -> String {
        BASE64.encode(self.mac_key)
    }

    /// Get Base64-encoded IV.
    pub fn iv_base64(&self) -> String {
        BASE64.encode(self.iv)
    }

    /// Get Base64-encoded MAC.
    pub fn mac_base64(&self) -> String {
        BASE64.encode(self.mac)
    }

    /// Get Base64-encoded file digest.
    pub fn file_digest_base64(&self) -> String {
        BASE64.encode(self.file_digest)
    }

    /// Set encryption key from Base64 string.
    pub fn set_encryption_key_from_base64(&mut self, b64: &str) -> Result<(), String> {
        let decoded = BASE64
            .decode(b64)
            .map_err(|e| format!("Invalid Base64 for encryption key: {}", e))?;
        if decoded.len() != 32 {
            return Err(format!(
                "Encryption key must be 32 bytes, got {}",
                decoded.len()
            ));
        }
        self.encryption_key.copy_from_slice(&decoded);
        Ok(())
    }

    /// Set MAC key from Base64 string.
    pub fn set_mac_key_from_base64(&mut self, b64: &str) -> Result<(), String> {
        let decoded = BASE64
            .decode(b64)
            .map_err(|e| format!("Invalid Base64 for MAC key: {}", e))?;
        if decoded.len() != 32 {
            return Err(format!("MAC key must be 32 bytes, got {}", decoded.len()));
        }
        self.mac_key.copy_from_slice(&decoded);
        Ok(())
    }

    /// Set IV from Base64 string.
    pub fn set_iv_from_base64(&mut self, b64: &str) -> Result<(), String> {
        let decoded = BASE64
            .decode(b64)
            .map_err(|e| format!("Invalid Base64 for IV: {}", e))?;
        if decoded.len() != 16 {
            return Err(format!("IV must be 16 bytes, got {}", decoded.len()));
        }
        self.iv.copy_from_slice(&decoded);
        Ok(())
    }

    /// Set MAC from Base64 string.
    pub fn set_mac_from_base64(&mut self, b64: &str) -> Result<(), String> {
        let decoded = BASE64
            .decode(b64)
            .map_err(|e| format!("Invalid Base64 for MAC: {}", e))?;
        if decoded.len() != 32 {
            return Err(format!("MAC must be 32 bytes, got {}", decoded.len()));
        }
        self.mac.copy_from_slice(&decoded);
        Ok(())
    }

    /// Set file digest from Base64 string.
    pub fn set_file_digest_from_base64(&mut self, b64: &str) -> Result<(), String> {
        let decoded = BASE64
            .decode(b64)
            .map_err(|e| format!("Invalid Base64 for file digest: {}", e))?;
        if decoded.len() != 32 {
            return Err(format!(
                "File digest must be 32 bytes, got {}",
                decoded.len()
            ));
        }
        self.file_digest.copy_from_slice(&decoded);
        Ok(())
    }
}

/// Metadata written to Detection.xml.
#[derive(Debug, Clone)]
pub struct DetectionMetadata {
    /// Name of the application (setup filename)
    pub name: String,
    /// Original uncompressed content size in bytes
    pub unencrypted_content_size: u64,
    /// Encrypted file name (always "IntunePackage.intunewin")
    pub file_name: String,
    /// Setup file name
    pub setup_file: String,
    /// Encryption parameters
    pub encryption_info: EncryptionInfo,
}

impl DetectionMetadata {
    /// Create new detection metadata.
    pub fn new(setup_file: String, unencrypted_content_size: u64) -> Self {
        Self {
            name: setup_file.clone(),
            unencrypted_content_size,
            file_name: "IntunePackage.intunewin".to_string(),
            setup_file,
            encryption_info: EncryptionInfo::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_info_default() {
        let info = EncryptionInfo::default();
        assert_eq!(info.profile_identifier, "ProfileVersion1");
        assert_eq!(info.file_digest_algorithm, "SHA256");
        assert_eq!(info.encryption_key.len(), 32);
        assert_eq!(info.mac_key.len(), 32);
        assert_eq!(info.iv.len(), 16);
    }

    #[test]
    fn test_base64_encoding() {
        let mut info = EncryptionInfo::new();
        info.encryption_key = [0u8; 32];
        info.iv = [0u8; 16];

        // 32 zero bytes = 44 chars Base64 (with padding)
        assert_eq!(info.encryption_key_base64().len(), 44);
        // 16 zero bytes = 24 chars Base64 (with padding)
        assert_eq!(info.iv_base64().len(), 24);
    }

    #[test]
    fn test_detection_metadata_new() {
        let meta = DetectionMetadata::new("setup.exe".to_string(), 1024);
        assert_eq!(meta.name, "setup.exe");
        assert_eq!(meta.setup_file, "setup.exe");
        assert_eq!(meta.file_name, "IntunePackage.intunewin");
        assert_eq!(meta.unencrypted_content_size, 1024);
    }
}
