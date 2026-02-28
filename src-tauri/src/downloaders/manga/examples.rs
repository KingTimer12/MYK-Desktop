/// Exemplos de uso dos downloaders de manga em Rust
///
/// Este arquivo demonstra como usar os downloaders implementados
/// seguindo o paradigma de Programação Orientada a Objetos (POO)

#[cfg(test)]
mod tests {
    use crate::downloaders::{ComickDl, MangaDexDl, MangaDl};

    #[tokio::test]
    async fn example_mangadex_search() {
        // Criar uma instância do downloader MangaDex
        let mangadex = MangaDexDl::new();

        // Buscar mangás
        match mangadex.search("One Piece").await {
            Ok(results) => {
                println!("Encontrados {} mangás", results.len());
                for manga in results.iter().take(5) {
                    println!("- {}: {}", manga.name, manga.link);
                }
            }
            Err(e) => eprintln!("Erro ao buscar: {}", e),
        }
    }

    #[tokio::test]
    async fn example_mangadex_chapters() {
        let mangadex = MangaDexDl::new();

        // ID de exemplo (substituir por um ID real)
        let manga_id = "a1c7c817-4e59-43b7-9365-09675a149a6f";

        // Buscar idiomas disponíveis
        match mangadex.get_favorite_languages(manga_id).await {
            Ok(languages) => {
                println!("Idiomas disponíveis:");
                for lang in languages {
                    println!("- {}: {}", lang.id, lang.label);
                }
            }
            Err(e) => eprintln!("Erro ao buscar idiomas: {}", e),
        }

        // Buscar capítulos em inglês
        match mangadex.get_chapters(manga_id, Some("en")).await {
            Ok(chapters) => {
                println!("\nEncontrados {} capítulos", chapters.len());
                for chapter in chapters.iter().take(5) {
                    println!("- Cap. {}: {:?}", chapter.number, chapter.title);
                }
            }
            Err(e) => eprintln!("Erro ao buscar capítulos: {}", e),
        }
    }

    #[tokio::test]
    async fn example_comick_search() {
        // Criar uma instância do downloader Comick
        let comick = ComickDl::new();

        // Buscar mangás
        match comick.search("Naruto").await {
            Ok(results) => {
                println!("Encontrados {} mangás", results.len());
                for manga in results.iter().take(5) {
                    println!("- {}: {}", manga.name, manga.link);
                    if let Some(extra) = &manga.extra_name {
                        println!("  Título alternativo: {}", extra);
                    }
                }
            }
            Err(e) => eprintln!("Erro ao buscar: {}", e),
        }
    }

    #[tokio::test]
    async fn example_comick_full_flow() {
        let comick = ComickDl::new();

        // 1. Buscar um mangá
        let results = comick.search("Solo Leveling").await.unwrap();
        if let Some(manga) = results.first() {
            println!("Mangá encontrado: {}", manga.name);
            println!("Source ID: {}", manga.source_id);

            // 2. Buscar idiomas disponíveis
            if let Ok(languages) = comick.get_favorite_languages(&manga.source_id).await {
                println!("\nIdiomas disponíveis: {}", languages.len());
            }

            // 3. Buscar capítulos
            if let Ok(chapters) = comick.get_chapters(&manga.source_id, Some("en")).await {
                println!("Capítulos encontrados: {}", chapters.len());

                // 4. Buscar imagens do primeiro capítulo
                if let Some(chapter) = chapters.first() {
                    if let Ok(images) = comick.get_chapter_images(&chapter.chapter_id).await {
                        println!("\nImagens do capítulo {}: {}", chapter.number, images.len());
                        if let Some(first_image) = images.first() {
                            println!("Primeira imagem: {}", first_image);
                        }
                    }
                }
            }
        }
    }

    #[tokio::test]
    async fn example_polymorphism() {
        // Demonstração de polimorfismo usando trait objects
        let downloaders: Vec<Box<dyn MangaDl>> =
            vec![Box::new(MangaDexDl::new()), Box::new(ComickDl::new())];

        for downloader in downloaders {
            println!("\nUsando downloader: {}", downloader.base_url());
            println!(
                "Suporta múltiplos idiomas: {}",
                downloader.is_multi_language()
            );

            match downloader.search("Bleach").await {
                Ok(results) => {
                    println!("Resultados encontrados: {}", results.len());
                }
                Err(e) => {
                    eprintln!("Erro: {}", e);
                }
            }
        }
    }
}

/// Exemplo de uso em uma função Tauri command
#[cfg(feature = "tauri-command-examples")]
pub mod tauri_examples {
    use crate::downloaders::{ComickDl, Favorite, MangaDexDl, MangaDl};
    use tauri::command;

    #[command]
    pub async fn search_manga(source: String, query: String) -> Result<Vec<Favorite>, String> {
        let downloader: Box<dyn MangaDl> = match source.as_str() {
            "MangaDex" => Box::new(MangaDexDl::new()),
            "Comick" => Box::new(ComickDl::new()),
            _ => return Err(format!("Source não suportada: {}", source)),
        };

        downloader.search(&query).await.map_err(|e| e.to_string())
    }

    #[command]
    pub async fn get_chapters(
        source: String,
        manga_id: String,
        language: Option<String>,
    ) -> Result<Vec<crate::downloaders::Chapter>, String> {
        let downloader: Box<dyn MangaDl> = match source.as_str() {
            "MangaDex" => Box::new(MangaDexDl::new()),
            "Comick" => Box::new(ComickDl::new()),
            _ => return Err(format!("Source não suportada: {}", source)),
        };

        downloader
            .get_chapters(&manga_id, language.as_deref())
            .await
            .map_err(|e| e.to_string())
    }

    #[command]
    pub async fn get_chapter_images(
        source: String,
        chapter_id: String,
    ) -> Result<Vec<String>, String> {
        let downloader: Box<dyn MangaDl> = match source.as_str() {
            "MangaDex" => Box::new(MangaDexDl::new()),
            "Comick" => Box::new(ComickDl::new()),
            _ => return Err(format!("Source não suportada: {}", source)),
        };

        downloader
            .get_chapter_images(&chapter_id)
            .await
            .map_err(|e| e.to_string())
    }
}
