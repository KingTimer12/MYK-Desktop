use std::error::Error;
use tauri_plugin_http::reqwest::{self, header::HeaderMap, header::HeaderValue, Client, Url};

#[derive(Clone)]
pub struct ApiClient {
    base_url: Url,
    client: Client,
    headers: HeaderMap,
}

impl ApiClient {
    pub fn new(base_url: &str) -> Result<Self, Box<dyn Error>> {
        let mut headers = HeaderMap::new();
        headers.insert("User-Agent", HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36"));
        headers.insert("Accept", HeaderValue::from_static("application/json"));

        let client = reqwest::Client::builder()
            .default_headers(headers.clone())
            .build()?;

        let base_url = Url::parse(base_url)?;
        Ok(ApiClient {
            base_url,
            client,
            headers,
        })
    }

    pub fn new_with_client(base_url: &str, client: Client) -> Result<Self, Box<dyn Error>> {
        let base_url = Url::parse(base_url)?;
        Ok(ApiClient {
            base_url,
            client,
            headers: HeaderMap::new(),
        })
    }

    pub fn update_headers(&mut self, headers: HeaderMap) -> Result<(), Box<dyn Error>> {
        // Merge new headers with existing ones (new ones take precedence)
        for (key, value) in headers.iter() {
            self.headers.insert(key, value.clone());
        }

        // Rebuild the client with all headers
        let builder = Client::builder().default_headers(self.headers.clone());
        self.client = builder.build()?;
        Ok(())
    }

    // pub fn example_update_headers(&mut self, token: &str) -> Result<(), Box<dyn Error>> {
    //     let mut headers = HeaderMap::new();
    //     headers.insert("Authorization", format!("Bearer {}", token).parse()?);
    //     self.update_headers(headers)?;
    //     Ok(())
    // }

    pub async fn get<T: serde::de::DeserializeOwned>(
        &self,
        url: &str,
    ) -> Result<T, Box<dyn Error>> {
        let full_url = self.base_url.join(url)?;
        let response = self.client.get(full_url).send().await?;

        if response.status().is_success() {
            let body = response.json().await?;
            return Ok(body);
        }
        Err(format!("Request failed with status: {}", response.status()).into())
    }

    pub async fn post<T: serde::de::DeserializeOwned + std::fmt::Debug>(
        &self,
        url: &str,
        data: Option<serde_json::Value>,
    ) -> Result<T, Box<dyn Error>> {
        let url = self.base_url.join(url)?;
        let response = if data.is_some() {
            self.client.post(url).json(&data).send().await?
        } else {
            self.client.post(url).send().await?
        };

        if response.status().is_success() {
            let body = response.json().await?;
            return Ok(body);
        }
        Err(format!("Request failed with status: {}", response.status()).into())
    }
}
