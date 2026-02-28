// Cliente do repositório/marketplace de extensões
// TODO: Implementar na Fase 4

use crate::extensions::ExtensionError;
use std::path::PathBuf;

pub struct ExtensionRepository {
    repo_url: String,
}

impl ExtensionRepository {
    pub fn new(repo_url: String) -> Self {
        Self { repo_url }
    }

    pub async fn fetch_catalog(&self) -> Result<Vec<RemoteExtensionInfo>, ExtensionError> {
        // TODO: Fazer fetch do repo.json
        Ok(vec![])
    }

    pub async fn download_extension(&self, _id: &str) -> Result<PathBuf, ExtensionError> {
        // TODO: Baixar .wasm e manifest.json
        Err(ExtensionError::LoadError("Not implemented yet".into()))
    }
}

impl Default for ExtensionRepository {
    fn default() -> Self {
        Self::new(
            "https://raw.githubusercontent.com/manga-you-know/extensions/main/repo.json".into(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct RemoteExtensionInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub download_url: String,
    pub manifest_url: String,
    pub checksum: String,
}
