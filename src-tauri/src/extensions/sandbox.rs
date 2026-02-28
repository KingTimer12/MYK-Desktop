// HTTP Sandbox - cliente HTTP com restrições de segurança
// Previne acessos a recursos perigosos e implementa rate limiting

use crate::extensions::ExtensionError;
use reqwest::{header, Client, Url};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// Cliente HTTP sandboxed com proteções de segurança
pub struct HttpSandbox {
    client: Client,
    rate_limiter: Arc<Mutex<RateLimiter>>,
    blocked_schemes: Vec<String>,
    max_response_size: usize,
}

impl HttpSandbox {
    /// Criar novo sandbox HTTP com configurações padrão
    pub fn new() -> Self {
        Self::with_config(SandboxConfig::default())
    }

    /// Criar sandbox com configuração customizada
    pub fn with_config(config: SandboxConfig) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::USER_AGENT,
            header::HeaderValue::from_static("MangaYouKnow/1.0"),
        );
        headers.insert(header::ACCEPT, header::HeaderValue::from_static("*/*"));

        let client = Client::builder()
            .default_headers(headers)
            .timeout(config.timeout)
            .redirect(reqwest::redirect::Policy::limited(config.max_redirects))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            client,
            rate_limiter: Arc::new(Mutex::new(RateLimiter::new(config.max_requests_per_second))),
            blocked_schemes: config.blocked_schemes,
            max_response_size: config.max_response_size,
        }
    }

    /// Fazer requisição GET
    pub async fn get(&self, url: &str) -> Result<Vec<u8>, ExtensionError> {
        self.validate_and_request(url, None).await
    }

    /// Fazer requisição POST
    pub async fn post(&self, url: &str, body: Vec<u8>) -> Result<Vec<u8>, ExtensionError> {
        self.validate_and_request(url, Some(body)).await
    }

    /// Validar URL e fazer requisição
    async fn validate_and_request(
        &self,
        url_str: &str,
        body: Option<Vec<u8>>,
    ) -> Result<Vec<u8>, ExtensionError> {
        // Parsear URL
        let url = Url::parse(url_str)
            .map_err(|e| ExtensionError::HttpError(format!("Invalid URL: {}", e)))?;

        // Validações de segurança
        self.validate_url(&url)?;

        // Rate limiting
        self.rate_limiter.lock().unwrap().check()?;

        // Fazer requisição
        let response = if let Some(body_data) = body {
            self.client
                .post(url.clone())
                .body(body_data)
                .send()
                .await
                .map_err(|e| ExtensionError::HttpError(format!("Request failed: {}", e)))?
        } else {
            self.client
                .get(url.clone())
                .send()
                .await
                .map_err(|e| ExtensionError::HttpError(format!("Request failed: {}", e)))?
        };

        // Verificar status
        if !response.status().is_success() {
            return Err(ExtensionError::HttpError(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        // Verificar tamanho da resposta
        if let Some(content_length) = response.content_length() {
            if content_length as usize > self.max_response_size {
                return Err(ExtensionError::HttpError(format!(
                    "Response too large: {} bytes (max: {})",
                    content_length, self.max_response_size
                )));
            }
        }

        // Ler resposta com limite de tamanho
        let bytes = response
            .bytes()
            .await
            .map_err(|e| ExtensionError::HttpError(format!("Failed to read response: {}", e)))?;

        if bytes.len() > self.max_response_size {
            return Err(ExtensionError::HttpError(format!(
                "Response too large: {} bytes (max: {})",
                bytes.len(),
                self.max_response_size
            )));
        }

        Ok(bytes.to_vec())
    }

    /// Validar URL quanto a segurança
    fn validate_url(&self, url: &Url) -> Result<(), ExtensionError> {
        // 1. Validar scheme
        let scheme = url.scheme();
        if self.blocked_schemes.contains(&scheme.to_string()) {
            return Err(ExtensionError::HttpError(format!(
                "Blocked scheme: {}",
                scheme
            )));
        }

        // Apenas HTTPS e HTTP são permitidos
        if scheme != "http" && scheme != "https" {
            return Err(ExtensionError::HttpError(format!(
                "Only HTTP and HTTPS are allowed, got: {}",
                scheme
            )));
        }

        // 2. Validar host
        if let Some(host) = url.host_str() {
            // Bloquear localhost
            if host == "localhost"
                || host == "127.0.0.1"
                || host == "::1"
                || host.starts_with("127.")
                || host.starts_with("0.")
            {
                return Err(ExtensionError::HttpError(
                    "Access to localhost is blocked".into(),
                ));
            }

            // Tentar resolver para IP e bloquear IPs privados
            if let Ok(ip) = host.parse::<IpAddr>() {
                if self.is_private_ip(&ip) {
                    return Err(ExtensionError::HttpError(
                        "Access to private IP addresses is blocked".into(),
                    ));
                }
            }

            // Bloquear meta-addresses
            if host == "0.0.0.0" || host == "::" || host.is_empty() {
                return Err(ExtensionError::HttpError("Invalid host address".into()));
            }
        } else {
            return Err(ExtensionError::HttpError("URL must have a host".into()));
        }

        // 3. Validar porta
        if let Some(port) = url.port() {
            // Bloquear portas privilegiadas suspeitas
            let blocked_ports = [
                22,    // SSH
                23,    // Telnet
                25,    // SMTP
                110,   // POP3
                143,   // IMAP
                445,   // SMB
                3306,  // MySQL
                5432,  // PostgreSQL
                6379,  // Redis
                27017, // MongoDB
            ];

            if blocked_ports.contains(&port) {
                return Err(ExtensionError::HttpError(format!(
                    "Access to port {} is blocked",
                    port
                )));
            }
        }

        Ok(())
    }

    /// Verificar se IP é privado
    fn is_private_ip(&self, ip: &IpAddr) -> bool {
        match ip {
            IpAddr::V4(ipv4) => {
                ipv4.is_private()
                    || ipv4.is_loopback()
                    || ipv4.is_link_local()
                    || ipv4.is_broadcast()
                    || ipv4.is_documentation()
                    || ipv4.is_unspecified()
                    // Carrier-grade NAT
                    || (ipv4.octets()[0] == 100 && (ipv4.octets()[1] & 0b11000000 == 64))
            }
            IpAddr::V6(ipv6) => {
                ipv6.is_loopback()
                    || ipv6.is_unspecified()
                    || ((ipv6.segments()[0] & 0xfe00) == 0xfc00) // Unique local
                    || ((ipv6.segments()[0] & 0xffc0) == 0xfe80) // Link-local
            }
        }
    }

    /// Verificar se uma URL seria permitida (sem fazer requisição)
    pub fn is_url_allowed(&self, url_str: &str) -> bool {
        if let Ok(url) = Url::parse(url_str) {
            self.validate_url(&url).is_ok()
        } else {
            false
        }
    }

    /// Resetar rate limiter (útil para testes)
    pub fn reset_rate_limiter(&self) {
        self.rate_limiter.lock().unwrap().reset();
    }
}

impl Default for HttpSandbox {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuração do sandbox
pub struct SandboxConfig {
    pub timeout: Duration,
    pub max_redirects: usize,
    pub max_requests_per_second: usize,
    pub max_response_size: usize,
    pub blocked_schemes: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            max_redirects: 5,
            max_requests_per_second: 10,
            max_response_size: 50 * 1024 * 1024, // 50 MB
            blocked_schemes: vec![
                "file".to_string(),
                "ftp".to_string(),
                "data".to_string(),
                "javascript".to_string(),
            ],
        }
    }
}

/// Rate limiter simples baseado em token bucket
struct RateLimiter {
    max_requests_per_second: usize,
    tokens: usize,
    last_refill: Instant,
}

impl RateLimiter {
    fn new(max_requests_per_second: usize) -> Self {
        Self {
            max_requests_per_second,
            tokens: max_requests_per_second,
            last_refill: Instant::now(),
        }
    }

    fn check(&mut self) -> Result<(), ExtensionError> {
        self.refill();

        if self.tokens > 0 {
            self.tokens -= 1;
            Ok(())
        } else {
            Err(ExtensionError::HttpError(
                "Rate limit exceeded. Please try again later.".into(),
            ))
        }
    }

    fn refill(&mut self) {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill);

        if elapsed >= Duration::from_secs(1) {
            let seconds_passed = elapsed.as_secs() as usize;
            self.tokens = self
                .max_requests_per_second
                .min(self.tokens + (self.max_requests_per_second * seconds_passed));
            self.last_refill = now;
        }
    }

    fn reset(&mut self) {
        self.tokens = self.max_requests_per_second;
        self.last_refill = Instant::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_creation() {
        let sandbox = HttpSandbox::new();
        assert!(sandbox.is_url_allowed("https://example.com"));
    }

    #[test]
    fn test_blocked_localhost() {
        let sandbox = HttpSandbox::new();
        assert!(!sandbox.is_url_allowed("http://localhost"));
        assert!(!sandbox.is_url_allowed("http://127.0.0.1"));
        assert!(!sandbox.is_url_allowed("http://[::1]"));
    }

    #[test]
    fn test_blocked_private_ips() {
        let sandbox = HttpSandbox::new();
        assert!(!sandbox.is_url_allowed("http://192.168.1.1"));
        assert!(!sandbox.is_url_allowed("http://10.0.0.1"));
        assert!(!sandbox.is_url_allowed("http://172.16.0.1"));
    }

    #[test]
    fn test_blocked_schemes() {
        let sandbox = HttpSandbox::new();
        assert!(!sandbox.is_url_allowed("file:///etc/passwd"));
        assert!(!sandbox.is_url_allowed("ftp://example.com"));
        assert!(!sandbox.is_url_allowed("data:text/html,<script>alert('xss')</script>"));
    }

    #[test]
    fn test_allowed_urls() {
        let sandbox = HttpSandbox::new();
        assert!(sandbox.is_url_allowed("https://example.com"));
        assert!(sandbox.is_url_allowed("http://example.com:8080"));
        assert!(sandbox.is_url_allowed("https://api.example.com/v1/data"));
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2);
        assert!(limiter.check().is_ok());
        assert!(limiter.check().is_ok());
        assert!(limiter.check().is_err()); // Terceira requisição deve falhar
    }

    #[test]
    fn test_private_ip_detection() {
        let sandbox = HttpSandbox::new();

        // IPv4 privados
        assert!(sandbox.is_private_ip(&IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1))));
        assert!(sandbox.is_private_ip(&IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1))));
        assert!(sandbox.is_private_ip(&IpAddr::V4(Ipv4Addr::new(172, 16, 0, 1))));
        assert!(sandbox.is_private_ip(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));

        // IPv4 públicos
        assert!(!sandbox.is_private_ip(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
        assert!(!sandbox.is_private_ip(&IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1))));

        // IPv6 privados
        assert!(sandbox.is_private_ip(&IpAddr::V6(Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1)))); // ::1
        assert!(sandbox.is_private_ip(&IpAddr::V6(Ipv6Addr::new(0xfe80, 0, 0, 0, 0, 0, 0, 1))));
        // Link-local
    }
}
