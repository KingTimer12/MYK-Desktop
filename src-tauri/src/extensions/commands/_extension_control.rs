use tauri::State;

use crate::extensions::commands::{_state::ExtensionState, ExtensionInfo};

/// Carregar uma extensão no runtime
#[tauri::command]
pub async fn load_extension(
    extension_id: String,
    state: State<'_, ExtensionState>,
) -> Result<ExtensionInfo, String> {
    // Verificar se já está carregada
    {
        let runtime = state.runtime.lock().await;
        if runtime.is_loaded(&extension_id) {
            return Err(format!("Extension '{}' is already loaded", extension_id));
        }
    }

    // Obter informações da extensão
    let ext = {
        let loader = state.loader.lock().await;
        let extensions = loader.discover_extensions().map_err(|e| e.to_string())?;

        extensions
            .into_iter()
            .find(|e| e.id == extension_id)
            .ok_or_else(|| format!("Extension '{}' not found", extension_id))?
    };

    // Copiar dados necessários para evitar manter lock durante await
    let ext_id = ext.id.clone();
    let wasm_path = ext.wasm_path.clone();

    // Carregar no runtime (sem lock durante await)
    {
        let mut runtime = state.runtime.lock().await;
        // Clone runtime para poder fazer await sem lock
        let result = runtime.load_extension(&ext_id, &wasm_path).await;
        drop(runtime); // Liberar lock explicitamente
        result.map_err(|e| e.to_string())?;
    }

    Ok(ExtensionInfo {
        id: ext.id.clone(),
        name: ext.manifest.name.clone(),
        version: ext.manifest.version.clone(),
        description: ext.manifest.description.clone(),
        author: ext.manifest.author.clone(),
        nsfw: ext.manifest.nsfw,
        language: ext.manifest.language.clone(),
        extension_type: ext.manifest.extension_type.clone(),
        base_url: ext.manifest.base_url.clone(),
        installed: ext.installed,
        loaded: true,
        exports: ext.manifest.exports.into(),
    })
}

/// Descarregar uma extensão do runtime
#[tauri::command]
pub async fn unload_extension(
    extension_id: String,
    state: State<'_, ExtensionState>,
) -> Result<(), String> {
    let mut runtime = state.runtime.lock().await;

    if !runtime.is_loaded(&extension_id) {
        return Err(format!("Extension '{}' is not loaded", extension_id));
    }

    runtime
        .unload_extension(&extension_id)
        .map_err(|e| e.to_string())?;

    Ok(())
}

/// Recarregar uma extensão (útil durante desenvolvimento)
#[tauri::command]
pub async fn reload_extension(
    extension_id: String,
    state: State<'_, ExtensionState>,
) -> Result<ExtensionInfo, String> {
    // Descarregar se estiver carregada
    let is_loaded = {
        let runtime = state.runtime.lock().await;
        runtime.is_loaded(&extension_id)
    };

    if is_loaded {
        unload_extension(extension_id.clone(), state.clone()).await?;
    }

    // Carregar novamente
    load_extension(extension_id, state).await
}