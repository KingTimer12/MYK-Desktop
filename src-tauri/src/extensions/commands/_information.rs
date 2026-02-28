use tauri::State;

use crate::extensions::{ExtensionManifest, commands::{_extension_control::unload_extension, _state::ExtensionState, ExtensionInfo}};

/// Listar todas as extensões instaladas
#[tauri::command]
pub async fn list_installed_extensions(
    state: State<'_, ExtensionState>,
) -> Result<Vec<ExtensionInfo>, String> {
    let loader = state.loader.lock().await;
    let runtime = state.runtime.lock().await;

    let extensions = loader.discover_extensions().map_err(|e| e.to_string())?;

    let info_list = extensions
        .into_iter()
        .map(|ext| ExtensionInfo {
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
            loaded: runtime.is_loaded(&ext.id),
            exports: ext.manifest.exports.into(),
        })
        .collect();

    Ok(info_list)
}

/// Obter informações de uma extensão específica
#[tauri::command]
pub async fn get_extension_info(
    extension_id: String,
    state: State<'_, ExtensionState>,
) -> Result<ExtensionInfo, String> {
    let loader = state.loader.lock().await;
    let runtime = state.runtime.lock().await;

    let extensions = loader.discover_extensions().map_err(|e| e.to_string())?;

    let ext = extensions
        .into_iter()
        .find(|e| e.id == extension_id)
        .ok_or_else(|| format!("Extension '{}' not found", extension_id))?;

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
        loaded: runtime.is_loaded(&ext.id),
        exports: ext.manifest.exports.into(),
    })
}

/// Listar extensões carregadas no runtime
#[tauri::command]
pub async fn list_loaded_extensions(
    state: State<'_, ExtensionState>,
) -> Result<Vec<String>, String> {
    let runtime = state.runtime.lock().await;
    Ok(runtime.list_loaded())
}

/// Instalar extensão a partir de um arquivo local
#[tauri::command]
pub async fn install_extension_from_file(
    wasm_path: String,
    manifest_path: String,
    state: State<'_, ExtensionState>,
) -> Result<ExtensionInfo, String> {
    use std::fs;
    use std::path::Path;

    let loader = state.loader.lock().await;
    let extensions_dir = loader.extensions_directory();

    // Ler e validar manifest
    let manifest_content = fs::read_to_string(&manifest_path)
        .map_err(|e| format!("Failed to read manifest: {}", e))?;
    let manifest: ExtensionManifest = serde_json::from_str(&manifest_content)
        .map_err(|e| format!("Failed to parse manifest: {}", e))?;

    // Verificar checksum do WASM
    manifest
        .verify_checksum(Path::new(&wasm_path))
        .map_err(|e| format!("Checksum verification failed: {}", e))?;

    // Criar diretório da extensão
    let extension_dir = extensions_dir.join(&manifest.name);
    fs::create_dir_all(&extension_dir)
        .map_err(|e| format!("Failed to create extension directory: {}", e))?;

    // Copiar arquivos
    let dest_wasm = extension_dir.join("module.wasm");
    let dest_manifest = extension_dir.join("manifest.json");

    fs::copy(&wasm_path, &dest_wasm).map_err(|e| format!("Failed to copy WASM file: {}", e))?;
    fs::copy(&manifest_path, &dest_manifest)
        .map_err(|e| format!("Failed to copy manifest file: {}", e))?;

    // Retornar informações da extensão instalada
    Ok(ExtensionInfo {
        id: manifest.name.clone(),
        name: manifest.name.clone(),
        version: manifest.version.clone(),
        description: manifest.description.clone(),
        author: manifest.author.clone(),
        nsfw: manifest.nsfw,
        language: manifest.language.clone(),
        extension_type: manifest.extension_type.clone(),
        base_url: manifest.base_url.clone(),
        installed: true,
        loaded: false,
        exports: manifest.exports.into(),
    })
}

/// Desinstalar uma extensão
#[tauri::command]
pub async fn uninstall_extension(
    extension_id: String,
    state: State<'_, ExtensionState>,
) -> Result<(), String> {
    use std::fs;

    // Descarregar do runtime se estiver carregada
    {
        let runtime = state.runtime.lock().await;
        if runtime.is_loaded(&extension_id) {
            drop(runtime);
            unload_extension(extension_id.clone(), state.clone()).await?;
        }
    }

    // Remover diretório da extensão
    let loader = state.loader.lock().await;
    let extension_dir = loader.extensions_directory().join(&extension_id);

    if extension_dir.exists() {
        fs::remove_dir_all(&extension_dir)
            .map_err(|e| format!("Failed to remove extension directory: {}", e))?;
    }

    Ok(())
}

/// Obter caminho do diretório de extensões
#[tauri::command]
pub async fn get_extensions_directory(state: State<'_, ExtensionState>) -> Result<String, String> {
    let loader = state.loader.lock().await;
    Ok(loader
        .extensions_directory()
        .to_str()
        .unwrap_or("")
        .to_string())
}