use crate::extensions::{ExtensionError, ExtensionManifest};
use std::fs;
use std::path::{Path, PathBuf};

pub struct ExtensionLoader {
    extensions_dir: PathBuf,
}

impl ExtensionLoader {
    /// Criar novo loader apontando para diretório de extensões
    pub fn new(extensions_dir: PathBuf) -> Self {
        Self { extensions_dir }
    }

    /// Descobrir todas as extensões no diretório
    pub fn discover_extensions(&self) -> Result<Vec<ExtensionInfo>, ExtensionError> {
        // Criar diretório se não existir
        if !self.extensions_dir.exists() {
            fs::create_dir_all(&self.extensions_dir)?;
            return Ok(vec![]);
        }

        let mut extensions = Vec::new();

        for entry in fs::read_dir(&self.extensions_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Apenas processar diretórios
            if !path.is_dir() {
                continue;
            }

            match self.load_extension_info(&path) {
                Ok(info) => extensions.push(info),
                Err(e) => {
                    eprintln!("Failed to load extension from {:?}: {}", path, e);
                    // Continuar com próxima extensão
                    continue;
                }
            }
        }

        Ok(extensions)
    }

    /// Carregar informações de uma extensão específica
    fn load_extension_info(&self, extension_dir: &Path) -> Result<ExtensionInfo, ExtensionError> {
        // Carregar manifest
        let manifest = ExtensionManifest::load_from_dir(extension_dir)?;

        // Validar manifest
        manifest.validate()?;

        // Verificar se arquivo WASM existe
        let wasm_path = extension_dir.join("module.wasm");
        if !wasm_path.exists() {
            return Err(ExtensionError::LoadError(format!(
                "module.wasm not found in {:?}",
                extension_dir
            )));
        }

        // Verificar checksum
        manifest.verify_checksum(&wasm_path)?;

        Ok(ExtensionInfo {
            id: manifest.name.clone(),
            manifest,
            wasm_path,
            installed: true,
        })
    }

    /// Obter caminho completo para uma extensão específica
    pub fn get_extension_path(&self, extension_id: &str) -> PathBuf {
        self.extensions_dir.join(extension_id)
    }

    /// Verificar se uma extensão está instalada
    pub fn is_installed(&self, extension_id: &str) -> bool {
        let extension_path = self.get_extension_path(extension_id);
        extension_path.exists() && extension_path.is_dir()
    }

    /// Obter referência ao diretório de extensões
    pub fn extensions_directory(&self) -> &Path {
        &self.extensions_dir
    }
}

#[derive(Debug, Clone)]
pub struct ExtensionInfo {
    pub id: String,
    pub manifest: ExtensionManifest,
    pub wasm_path: PathBuf,
    pub installed: bool,
}
