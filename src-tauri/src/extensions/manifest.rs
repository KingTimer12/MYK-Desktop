use crate::extensions::ExtensionError;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionManifest {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub nsfw: bool,
    pub language: String,
    #[serde(rename = "type")]
    pub extension_type: String, // "manga" | "anime" | "comic"
    pub base_url: String,
    pub checksum: String, // "sha256:..."
    pub exports: ExtensionExports,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionExports {
    pub search: bool,
    pub get_chapters: bool,
    pub get_chapter_images: bool,
    pub is_multi_language: bool,
}

impl ExtensionManifest {
    /// Carregar manifest.json de um diretório
    pub fn load_from_dir(dir: &Path) -> Result<Self, ExtensionError> {
        let manifest_path = dir.join("manifest.json");

        if !manifest_path.exists() {
            return Err(ExtensionError::LoadError(format!(
                "manifest.json not found in {:?}",
                dir
            )));
        }

        let content = fs::read_to_string(&manifest_path)?;
        let manifest: ExtensionManifest = serde_json::from_str(&content)?;

        Ok(manifest)
    }

    /// Validar manifest
    pub fn validate(&self) -> Result<(), ExtensionError> {
        // Validar nome
        if self.name.is_empty() {
            return Err(ExtensionError::ValidationError(
                "name cannot be empty".into(),
            ));
        }

        // Validar versão (formato semver básico)
        if !self.version.contains('.') {
            return Err(ExtensionError::ValidationError(
                "version must be in semver format (e.g., 1.0.0)".into(),
            ));
        }

        // Validar tipo
        if !["manga", "anime", "comic"].contains(&self.extension_type.as_str()) {
            return Err(ExtensionError::ValidationError(format!(
                "invalid type: {}",
                self.extension_type
            )));
        }

        // Validar checksum
        if !self.checksum.starts_with("sha256:") {
            return Err(ExtensionError::ValidationError(
                "checksum must start with 'sha256:'".into(),
            ));
        }

        // Validar que pelo menos uma função está exportada
        if !self.exports.search && !self.exports.get_chapters && !self.exports.get_chapter_images {
            return Err(ExtensionError::ValidationError(
                "at least one function must be exported".into(),
            ));
        }

        Ok(())
    }

    /// Verificar checksum de um arquivo WASM
    pub fn verify_checksum(&self, wasm_path: &Path) -> Result<(), ExtensionError> {
        let expected = self
            .checksum
            .strip_prefix("sha256:")
            .ok_or_else(|| ExtensionError::ValidationError("invalid checksum format".into()))?;

        let wasm_bytes = fs::read(wasm_path)?;
        let mut hasher = Sha256::new();
        hasher.update(&wasm_bytes);
        let result = hasher.finalize();
        let actual = format!("{:x}", result);

        if actual != expected {
            return Err(ExtensionError::ValidationError(format!(
                "checksum mismatch: expected {}, got {}",
                expected, actual
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_validation() {
        let manifest = ExtensionManifest {
            name: "test-extension".into(),
            version: "1.0.0".into(),
            description: "Test".into(),
            author: "Test Author".into(),
            nsfw: false,
            language: "en".into(),
            extension_type: "manga".into(),
            base_url: "https://example.com".into(),
            checksum: "sha256:abc123".into(),
            exports: ExtensionExports {
                search: true,
                get_chapters: true,
                get_chapter_images: true,
                is_multi_language: false,
            },
        };

        assert!(manifest.validate().is_ok());
    }

    #[test]
    fn test_invalid_type() {
        let mut manifest = ExtensionManifest {
            name: "test".into(),
            version: "1.0.0".into(),
            description: "Test".into(),
            author: "Test".into(),
            nsfw: false,
            language: "en".into(),
            extension_type: "invalid".into(),
            base_url: "https://example.com".into(),
            checksum: "sha256:abc".into(),
            exports: ExtensionExports {
                search: true,
                get_chapters: false,
                get_chapter_images: false,
                is_multi_language: false,
            },
        };

        assert!(manifest.validate().is_err());
    }
}
