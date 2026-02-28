// ABI (Application Binary Interface) entre Rust host e WASM guests
// Este módulo define as funções que o host expõe para WASM e vice-versa

/// Funções que o host (Rust) expõe para os módulos WASM
pub mod host_functions {
    /// HTTP GET - recebe URL, retorna response bytes
    /// Params: url_ptr, url_len
    /// Returns: ptr para resultado MessagePack
    pub const HTTP_GET: &str = "host_http_get";

    /// HTTP POST - recebe URL e body, retorna response bytes
    /// Params: url_ptr, url_len, body_ptr, body_len
    /// Returns: ptr para resultado MessagePack
    pub const HTTP_POST: &str = "host_http_post";

    /// Logging - recebe level e mensagem
    /// Params: level (0=trace, 1=debug, 2=info, 3=warn, 4=error), msg_ptr, msg_len
    pub const LOG: &str = "host_log";
}

/// Funções que as extensões WASM devem exportar
pub mod wasm_exports {
    /// Buscar conteúdo
    /// Params: query_ptr, query_len
    /// Returns: ptr para Vec<Favorite> em MessagePack
    pub const SEARCH: &str = "extension_search";

    /// Obter capítulos/episódios
    /// Params: id_ptr, id_len, lang_ptr, lang_len (lang pode ser null)
    /// Returns: ptr para Vec<Chapter> em MessagePack
    pub const GET_CHAPTERS: &str = "extension_get_chapters";

    /// Obter imagens de um capítulo
    /// Params: id_ptr, id_len
    /// Returns: ptr para Vec<String> em MessagePack
    pub const GET_CHAPTER_IMAGES: &str = "extension_get_chapter_images";

    /// Obter idiomas disponíveis
    /// Params: id_ptr, id_len
    /// Returns: ptr para Vec<Language> em MessagePack
    pub const GET_LANGUAGES: &str = "extension_get_languages";

    /// Allocar memória no WASM (necessário para passar dados)
    /// Params: size
    /// Returns: ptr
    pub const ALLOC: &str = "alloc";

    /// Liberar memória no WASM
    /// Params: ptr, size
    pub const DEALLOC: &str = "dealloc";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_names() {
        assert_eq!(host_functions::HTTP_GET, "host_http_get");
        assert_eq!(wasm_exports::SEARCH, "extension_search");
    }
}
