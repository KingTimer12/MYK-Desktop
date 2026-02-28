use serde::{Deserialize, Serialize};
use tauri::State;

use super::manga::mangadex::MangaDexDl;
use super::manga::mangafire::MangaFireDl;
use super::manga::{Chapter, Favorite, Language, MangaDl};

// Struct para manter o estado do downloader
pub struct DownloaderState {
    pub mangadex: MangaDexDl,
    pub mangafire: MangaFireDl,
}

impl Default for DownloaderState {
    fn default() -> Self {
        Self {
            mangadex: MangaDexDl::new(),
            mangafire: MangaFireDl::new(),
        }
    }
}

// Comandos Tauri

#[tauri::command]
pub async fn mangadex_search(
    query: String,
    state: State<'_, DownloaderState>,
) -> Result<Vec<Favorite>, String> {
    state
        .mangadex
        .search(&query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mangadex_get_favorite_languages(
    favorite_id: String,
    state: State<'_, DownloaderState>,
) -> Result<Vec<Language>, String> {
    state
        .mangadex
        .get_favorite_languages(&favorite_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mangadex_get_chapters(
    favorite_id: String,
    language: Option<String>,
    state: State<'_, DownloaderState>,
) -> Result<Vec<Chapter>, String> {
    state
        .mangadex
        .get_chapters(&favorite_id, language.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mangadex_get_chapter_images(
    chapter_id: String,
    state: State<'_, DownloaderState>,
) -> Result<Vec<String>, String> {
    state
        .mangadex
        .get_chapter_images(&chapter_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mangadex_get_manga_by_url(
    url: String,
    state: State<'_, DownloaderState>,
) -> Result<Favorite, String> {
    state
        .mangadex
        .get_manga_by_url(&url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mangadex_get_manga_by_id(
    id: String,
    state: State<'_, DownloaderState>,
) -> Result<Favorite, String> {
    state
        .mangadex
        .get_manga_by_id(&id)
        .await
        .map_err(|e| e.to_string())
}

// MangaFire commands

#[tauri::command]
pub async fn mangafire_search(
    query: String,
    state: State<'_, DownloaderState>,
) -> Result<Vec<Favorite>, String> {
    state
        .mangafire
        .search(&query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mangafire_get_favorite_languages(
    favorite_id: String,
    state: State<'_, DownloaderState>,
) -> Result<Vec<Language>, String> {
    state
        .mangafire
        .get_favorite_languages(&favorite_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mangafire_get_chapters(
    favorite_id: String,
    language: Option<String>,
    state: State<'_, DownloaderState>,
) -> Result<Vec<Chapter>, String> {
    state
        .mangafire
        .get_chapters(&favorite_id, language.as_deref())
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mangafire_get_chapter_images(
    chapter_id: String,
    state: State<'_, DownloaderState>,
) -> Result<Vec<String>, String> {
    state
        .mangafire
        .get_chapter_images(&chapter_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mangafire_get_manga_by_url(
    url: String,
    state: State<'_, DownloaderState>,
) -> Result<Favorite, String> {
    state
        .mangafire
        .get_manga_by_url(&url)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn mangafire_get_manga_by_id(
    id: String,
    state: State<'_, DownloaderState>,
) -> Result<Favorite, String> {
    state
        .mangafire
        .get_manga_by_id(&id)
        .await
        .map_err(|e| e.to_string())
}
