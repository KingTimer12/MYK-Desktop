use serde::{Deserialize, Serialize};

pub mod _state;
pub mod _extension;
pub mod _extension_control;
pub mod _information;

/// Informações de uma extensão para o frontend
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub nsfw: bool,
    pub language: String,
    pub extension_type: String,
    pub base_url: String,
    pub installed: bool,
    pub loaded: bool,
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

impl From<crate::extensions::manifest::ExtensionExports> for ExtensionExports {
    fn from(exports: crate::extensions::manifest::ExtensionExports) -> Self {
        Self {
            search: exports.search,
            get_chapters: exports.get_chapters,
            get_chapter_images: exports.get_chapter_images,
            is_multi_language: exports.is_multi_language,
        }
    }
}