pub mod mangadex;
pub mod mangafire;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// Enums e structs auxiliares
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ContentType {
    #[serde(rename = "manga")]
    Manga,
    #[serde(rename = "comic")]
    Comic,
    #[serde(rename = "anime")]
    Anime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum BoolOrString {
    Bool(bool),
    String(String),
}

// Estruturas de dados baseadas nas interfaces TypeScript
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase", default)]
pub struct Favorite {
    #[serde(default)]
    pub id: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub user_id: Option<u64>,
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub folder_name: String,
    #[serde(default)]
    pub link: String,
    #[serde(default)]
    pub cover: String,
    #[serde(default)]
    pub source: String,
    #[serde(default)]
    pub source_id: String,
    #[serde(default, rename = "type", skip_serializing_if = "Option::is_none")]
    pub kind: Option<ContentType>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extra_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub card_color: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mal_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub anilist_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grade: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub is_ultra_favorite: Option<BoolOrString>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub marks: Option<Vec<Value>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub readeds: Option<Vec<Value>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chapter {
    pub number: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub chapter_id: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scan: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thumbnail: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Language {
    pub id: String,
    pub label: String,
}

#[async_trait]
pub trait MangaDl: Send + Sync {
    fn base_url(&self) -> &str;
    fn is_multi_language(&self) -> bool;

    async fn search(&self, query: &str) -> Result<Vec<Favorite>, Box<dyn std::error::Error>>;

    async fn get_favorite_languages(
        &self,
        _favorite_id: &str,
    ) -> Result<Vec<Language>, Box<dyn std::error::Error>> {
        Ok(vec![])
    }

    async fn get_chapters(
        &self,
        favorite_id: &str,
        language: Option<&str>,
    ) -> Result<Vec<Chapter>, Box<dyn std::error::Error>>;

    async fn get_chapter_images(
        &self,
        chapter_id: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>>;

    async fn get_manga_by_url(&self, url: &str) -> Result<Favorite, Box<dyn std::error::Error>>;

    async fn get_manga_by_id(&self, id: &str) -> Result<Favorite, Box<dyn std::error::Error>>;
}
