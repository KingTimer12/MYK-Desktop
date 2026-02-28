use tauri::State;

use crate::{downloaders::{Chapter, Favorite, Language}, extensions::commands::_state::ExtensionState};

/// Executar busca em uma extensão
#[tauri::command]
pub async fn extension_search(
    extension_id: String,
    query: String,
    state: State<'_, ExtensionState>,
) -> Result<Vec<Favorite>, String> {
    // Verificar se está carregada
    {
        let runtime = state.runtime.lock().await;
        if !runtime.is_loaded(&extension_id) {
            return Err(format!("Extension '{}' is not loaded", extension_id));
        }
    }

    // Serializar parâmetros para MessagePack (usando named para compatibilidade)
    let params = rmp_serde::to_vec_named(&query).map_err(|e| e.to_string())?;

    // Executar função WASM
    let result_bytes = {
        let runtime = state.runtime.lock().await;
        runtime
            .execute_function(&extension_id, "extension_search", params)
            .await
            .map_err(|e| e.to_string())?
    };

    // Deserializar resultado
    if result_bytes.is_empty() {
        return Ok(vec![]);
    }

    let favorites: Vec<Favorite> =
        rmp_serde::from_slice(&result_bytes).map_err(|e| e.to_string())?;

    Ok(favorites)
}

/// Obter capítulos de uma extensão
#[tauri::command]
pub async fn extension_get_chapters(
    extension_id: String,
    favorite_id: String,
    language: Option<String>,
    state: State<'_, ExtensionState>,
) -> Result<Vec<Chapter>, String> {
    log::warn!(
        "[extension_get_chapters] extension_id={}, favorite_id={}, language={:?}",
        extension_id,
        favorite_id,
        language
    );

    // Verificar se está carregada
    {
        let runtime = state.runtime.lock().await;
        if !runtime.is_loaded(&extension_id) {
            return Err(format!("Extension '{}' is not loaded", extension_id));
        }
    }

    // Serializar parâmetros como tupla (usando named para compatibilidade)
    let params_bytes = rmp_serde::to_vec_named(&(favorite_id.clone(), language.clone()))
        .map_err(|e| e.to_string())?;

    log::warn!(
        "[extension_get_chapters] params_bytes len={}, hex={:?}",
        params_bytes.len(),
        params_bytes
            .iter()
            .take(50)
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ")
    );

    // Executar função WASM
    let result_bytes = {
        let runtime = state.runtime.lock().await;
        runtime
            .execute_function(&extension_id, "extension_get_chapters", params_bytes)
            .await
            .map_err(|e| e.to_string())?
    };

    // Deserializar resultado
    if result_bytes.is_empty() {
        return Ok(vec![]);
    }

    log::warn!(
        "[extension_get_chapters] result_bytes len={}, hex={:?}",
        result_bytes.len(),
        result_bytes
            .iter()
            .take(50)
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ")
    );

    let chapters: Vec<Chapter> = rmp_serde::from_slice(&result_bytes).map_err(|e| {
        log::warn!("[extension_get_chapters] deserialize error: {}", e);
        e.to_string()
    })?;

    log::warn!("[extension_get_chapters] found {} chapters", chapters.len());
    Ok(chapters)
}

/// Obter imagens de um capítulo
#[tauri::command]
pub async fn extension_get_chapter_images(
    extension_id: String,
    chapter_id: String,
    state: State<'_, ExtensionState>,
) -> Result<Vec<String>, String> {
    // Verificar se está carregada
    {
        let runtime = state.runtime.lock().await;
        if !runtime.is_loaded(&extension_id) {
            return Err(format!("Extension '{}' is not loaded", extension_id));
        }
    }

    // Serializar parâmetros (usando named para compatibilidade)
    let params = rmp_serde::to_vec_named(&chapter_id).map_err(|e| e.to_string())?;

    // Executar função WASM
    let result_bytes = {
        let runtime = state.runtime.lock().await;
        runtime
            .execute_function(&extension_id, "extension_get_chapter_images", params)
            .await
            .map_err(|e| e.to_string())?
    };

    // Deserializar resultado
    if result_bytes.is_empty() {
        return Ok(vec![]);
    }

    let images: Vec<String> = rmp_serde::from_slice(&result_bytes).map_err(|e| e.to_string())?;

    Ok(images)
}

/// Obter idiomas disponíveis
#[tauri::command]
pub async fn extension_get_languages(
    extension_id: String,
    favorite_id: String,
    state: State<'_, ExtensionState>,
) -> Result<Vec<Language>, String> {
    // Verificar se está carregada
    {
        let runtime = state.runtime.lock().await;
        if !runtime.is_loaded(&extension_id) {
            return Err(format!("Extension '{}' is not loaded", extension_id));
        }
    }

    // Serializar parâmetros (usando named para compatibilidade)
    let params = rmp_serde::to_vec_named(&favorite_id).map_err(|e| e.to_string())?;

    // Executar função WASM
    let result_bytes = {
        let runtime = state.runtime.lock().await;
        runtime
            .execute_function(&extension_id, "extension_get_languages", params)
            .await
            .map_err(|e| e.to_string())?
    };

    // Deserializar resultado
    if result_bytes.is_empty() {
        return Ok(vec![]);
    }

    let languages: Vec<Language> =
        rmp_serde::from_slice(&result_bytes).map_err(|e| e.to_string())?;

    Ok(languages)
}