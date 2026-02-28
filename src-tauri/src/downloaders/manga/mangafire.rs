use async_trait::async_trait;
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::HashMap;

use super::{Chapter, ContentType, Favorite, Language, MangaDl};
use crate::utils::api_client::ApiClient;

const BASE_URL: &str = "https://mangafire.to";

pub struct MangaFireDl {
    api_client: ApiClient,
}

impl MangaFireDl {
    pub fn new() -> Self {
        Self {
            api_client: ApiClient::new(BASE_URL).expect("Failed to create API client"),
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

impl Default for MangaFireDl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl MangaDl for MangaFireDl {
    fn base_url(&self) -> &str {
        BASE_URL
    }

    fn is_multi_language(&self) -> bool {
        true
    }

    async fn search(&self, query: &str) -> Result<Vec<Favorite>, Box<dyn std::error::Error>> {
        let url = format!("ajax/manga/search?keyword={}", urlencoding::encode(query));

        #[derive(serde::Deserialize)]
        struct SearchResponse {
            result: SearchResult,
        }

        #[derive(serde::Deserialize)]
        struct SearchResult {
            count: u32,
            html: String,
        }

        let response: SearchResponse = self.api_client.get(&url).await?;

        if response.result.count == 0 {
            return Ok(vec![]);
        }

        let document = Html::parse_document(&response.result.html);
        let a_selector = Selector::parse("a").unwrap();
        let h6_selector = Selector::parse("h6").unwrap();
        let img_selector = Selector::parse("img").unwrap();

        let mut mangas = Vec::new();

        for (index, a) in document.select(&a_selector).enumerate() {
            if index >= response.result.count as usize {
                break;
            }

            let name = a
                .select(&h6_selector)
                .next()
                .map(|h6| h6.text().collect::<String>())
                .unwrap_or_default();

            let link = a
                .value()
                .attr("href")
                .unwrap_or_default()
                .replace("/manga/", "");

            let split_link: Vec<&str> = link.split('.').collect();
            let folder_name = split_link[..split_link.len().saturating_sub(1)].join(".");

            let cover = a
                .select(&img_selector)
                .next()
                .and_then(|img| img.value().attr("src"))
                .unwrap_or("/myk.png")
                .replace("@100", "");

            mangas.push(Favorite {
                id: 0,
                source_id: link.clone(),
                name,
                folder_name,
                link: format!("{}/manga/{}", BASE_URL, link),
                cover,
                source: "MangaFire".to_string(),
                kind: Some(ContentType::Manga),
                ..Default::default()
            });
        }

        Ok(mangas)
    }

    async fn get_favorite_languages(
        &self,
        favorite_id: &str,
    ) -> Result<Vec<Language>, Box<dyn std::error::Error>> {
        let url = format!("manga/{}", urlencoding::encode(favorite_id));
        let html: String = self.api_client.get(&url).await?;

        let document = Html::parse_document(&html);
        let dropdown_selector = Selector::parse("div.dropdown-menu").unwrap();
        let a_selector = Selector::parse("a").unwrap();

        let mut languages = Vec::new();

        if let Some(dropdown) = document.select(&dropdown_selector).next() {
            for a in dropdown.select(&a_selector) {
                let lang_id = a
                    .value()
                    .attr("data-code")
                    .unwrap_or_default()
                    .to_lowercase();

                let label = a
                    .value()
                    .attr("data-title")
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| Self::get_language_label(&lang_id));

                languages.push(Language { id: lang_id, label });
            }
        }

        Ok(languages)
    }

    async fn get_chapters(
        &self,
        favorite_id: &str,
        language: Option<&str>,
    ) -> Result<Vec<Chapter>, Box<dyn std::error::Error>> {
        let lang = language.unwrap_or("en").to_lowercase();
        let split: Vec<&str> = favorite_id.split('.').collect();

        if split.len() < 2 {
            return Err("Invalid favorite_id format".into());
        }

        let url = format!("ajax/read/{}/chapter/{}", split[1], lang);

        #[derive(serde::Deserialize)]
        struct ChaptersResponse {
            result: ChaptersResult,
        }

        #[derive(serde::Deserialize)]
        struct ChaptersResult {
            html: String,
        }

        let response: ChaptersResponse = self.api_client.get(&url).await?;
        let cleaned_html = response.result.html.replace("\\", "");

        let document = Html::parse_document(&cleaned_html);
        let li_selector = Selector::parse("li").unwrap();
        let a_selector = Selector::parse("a").unwrap();

        let mut chapters = Vec::new();

        for li in document.select(&li_selector) {
            if let Some(a) = li.select(&a_selector).next() {
                let number = a.value().attr("data-number").unwrap_or_default();
                let title = a.value().attr("title").map(|s| s.to_string());
                let chapter_id = a.value().attr("data-id").unwrap_or_default();

                chapters.push(Chapter {
                    number: number.to_string(),
                    title,
                    chapter_id: chapter_id.to_string(),
                    source: "MangaFire".to_string(),
                    language: Some(lang.clone()),
                    scan: None,
                    path: None,
                    thumbnail: None,
                });
            }
        }

        Ok(chapters)
    }

    async fn get_chapter_images(
        &self,
        chapter_id: &str,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let url = format!("ajax/read/chapter/{}", chapter_id);

        #[derive(serde::Deserialize)]
        struct ImagesResponse {
            result: ImagesResult,
        }

        #[derive(serde::Deserialize)]
        struct ImagesResult {
            images: Vec<Vec<String>>,
        }

        let response: ImagesResponse = self.api_client.get(&url).await?;

        let images = response
            .result
            .images
            .iter()
            .filter_map(|img_array| img_array.first())
            .map(|img| img.replace("\\", "").replace("mfcdn1", "mfcdn2"))
            .collect();

        Ok(images)
    }

    async fn get_manga_by_url(&self, _url: &str) -> Result<Favorite, Box<dyn std::error::Error>> {
        Err("Method not implemented".into())
    }

    async fn get_manga_by_id(&self, _id: &str) -> Result<Favorite, Box<dyn std::error::Error>> {
        Err("Method not implemented".into())
    }
}
