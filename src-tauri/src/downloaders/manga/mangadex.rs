use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use super::{Chapter, ContentType, Favorite, Language, MangaDl};
use crate::utils::api_client::ApiClient;

const BASE_URL: &str = "https://mangadex.org";
const API_URL: &str = "https://api.mangadex.org";

pub struct MangaDexDl {
    api_client: ApiClient,
}

impl MangaDexDl {
    pub fn new() -> Self {
        Self {
            api_client: ApiClient::new(API_URL).expect("Failed to create API client"),
        }
    }

    fn get_language_label(code: &str) -> String {
        let labels: HashMap<&str, &str> = [
            ("en", "English"),
            ("pt-br", "Português (Brasil)"),
            ("pt", "Português"),
            ("es", "Español"),
            ("es-la", "Español (Latinoamérica)"),
            ("fr", "Français"),
            ("de", "Deutsch"),
            ("it", "Italiano"),
            ("ru", "Русский"),
            ("ja", "日本語"),
            ("ko", "한국어"),
            ("zh", "中文"),
            ("zh-hk", "中文 (香港)"),
        ]
        .iter()
        .cloned()
        .collect();

        labels.get(code).unwrap_or(&code).to_string()
    }
}

impl Default for MangaDexDl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MangaDl for MangaDexDl {
    fn base_url(&self) -> &str {
        BASE_URL
    }

    fn is_multi_language(&self) -> bool {
        true
    }

    async fn search(&self, query: &str) -> Result<Vec<Favorite>, Box<dyn std::error::Error>> {
        let url = format!(
            "manga?includes[]=cover_art&order[relevance]=desc&contentRating[]=safe&contentRating[]=suggestive&contentRating[]=erotica&contentRating[]=pornographic&title={}&limit=20",
            urlencoding::encode(query)
        );

        let json: Value = self.api_client.get(&url).await?;
        let mut mangas = Vec::new();

        if let Some(data) = json["data"].as_array() {
            for manga in data {
                let manga_id = manga["id"].as_str().unwrap_or_default();

                // Buscar cover art
                let mut cover_filename = String::new();
                if let Some(relationships) = manga["relationships"].as_array() {
                    for rel in relationships {
                        if rel["type"] == "cover_art" {
                            cover_filename = rel["attributes"]["fileName"]
                                .as_str()
                                .unwrap_or_default()
                                .to_string();
                            break;
                        }
                    }
                }

                // Buscar título (preferencialmente em inglês)
                let title = if let Some(en_title) = manga["attributes"]["title"]["en"].as_str() {
                    en_title.to_string()
                } else if let Some(title_obj) = manga["attributes"]["title"].as_object() {
                    title_obj
                        .values()
                        .next()
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string()
                } else {
                    String::new()
                };

                // Buscar título alternativo aleatório
                let extra_name =
                    if let Some(alt_titles) = manga["attributes"]["altTitles"].as_array() {
                        if !alt_titles.is_empty() {
                            let idx = (std::time::SystemTime::now()
                                .duration_since(std::time::UNIX_EPOCH)
                                .unwrap()
                                .as_millis() as usize)
                                % alt_titles.len();

                            alt_titles
                                .get(idx)
                                .and_then(|obj| obj.as_object())
                                .and_then(|map| map.values().next())
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                let description = manga["attributes"]["description"]["en"]
                    .as_str()
                    .map(|s| s.to_string());

                mangas.push(Favorite {
                    id: 0,
                    source_id: manga_id.to_string(),
                    name: title.clone(),
                    folder_name: urlencoding::encode(&title).to_string(),
                    link: format!("{}/title/{}", BASE_URL, manga_id),
                    extra_name,
                    description,
                    cover: format!("{}/covers/{}/{}", BASE_URL, manga_id, cover_filename),
                    source: "MangaDex".to_string(),
                    kind: Some(ContentType::Manga),
                    ..Default::default()
                });
            }
        }
        
        println!("MangaDex search for '{}' returned {} results", query, mangas.len());

        Ok(mangas)
    }

    async fn get_favorite_languages(
        &self,
        favorite_id: &str,
    ) -> Result<Vec<Language>, Box<dyn std::error::Error>> {
        let url = format!("manga/{}", favorite_id);
        let json: Value = self.api_client.get(&url).await?;
        let mut languages = Vec::new();

        if let Some(available_langs) =
            json["data"]["attributes"]["availableTranslatedLanguages"].as_array()
        {
            for lang in available_langs {
                if let Some(lang_code) = lang.as_str() {
                    languages.push(Language {
                        id: lang_code.to_string(),
                        label: Self::get_language_label(lang_code),
                    });
                }
            }
        }

        Ok(languages)
    }

    async fn get_chapters(
        &self,
        favorite_id: &str,
        language: Option<&str>,
    ) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        let lang = language.unwrap_or("en");
        let limit = 500;
        let mut offset = 0;
        let mut all_chapters = Vec::new();

        loop {
            let url = format!(
                "manga/{}/feed?limit={}&translatedLanguage[]={}&includes[]=scanlation_group&order[chapter]=desc&includeExternalUrl=0&offset={}",
                favorite_id, limit, lang, offset
            );

            let json: Value = match self.api_client.get(&url).await {
                Ok(json) => json,
                Err(_) => break,
            };

            if let Some(data) = json["data"].as_array() {
                if data.is_empty() {
                    break;
                }

                for chapter in data {
                    let chapter_id = chapter["id"].as_str().unwrap_or_default();
                    let chapter_num = chapter["attributes"]["chapter"]
                        .as_str()
                        .or_else(|| chapter["attributes"]["title"].as_str())
                        .unwrap_or(chapter_id);

                    let title = chapter["attributes"]["title"]
                        .as_str()
                        .map(|s| s.to_string());

                    // Buscar scan group
                    let scan = if let Some(relationships) = chapter["relationships"].as_array() {
                        relationships
                            .iter()
                            .find(|r| r["type"] == "scanlation_group")
                            .and_then(|r| r["attributes"]["name"].as_str())
                            .map(|s| s.to_string())
                    } else {
                        None
                    };

                    all_chapters.push(Chapter {
                        number: chapter_num.to_string(),
                        title,
                        chapter_id: chapter_id.to_string(),
                        source: "MangaDex".to_string(),
                        language: Some(lang.to_string()),
                        scan,
                        path: None,
                        thumbnail: None,
                    });
                }

                let total = json["total"].as_u64().unwrap_or(0);
                if all_chapters.len() >= total as usize {
                    break;
                }

                offset += limit;
            } else {
                break;
            }
        }

        Ok(all_chapters)
    }

    async fn get_chapter_images(
        &self,
        chapter_id: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let url = format!("at-home/server/{}?forcePort443=false", chapter_id);
        let json: Value = self.api_client.get(&url).await?;
        let mut images = Vec::new();

        if let (Some(base_url), Some(hash), Some(data)) = (
            json["baseUrl"].as_str(),
            json["chapter"]["hash"].as_str(),
            json["chapter"]["data"].as_array(),
        ) {
            for img in data {
                if let Some(img_name) = img.as_str() {
                    images.push(format!("{}/data/{}/{}", base_url, hash, img_name));
                }
            }
        }

        Ok(images)
    }

    async fn get_manga_by_url(&self, _url: &str) -> Result<Favorite, Box<dyn std::error::Error>> {
        Err("Method not implemented".into())
    }

    async fn get_manga_by_id(&self, _id: &str) -> Result<Favorite, Box<dyn std::error::Error>> {
        Err("Method not implemented".into())
    }
}
