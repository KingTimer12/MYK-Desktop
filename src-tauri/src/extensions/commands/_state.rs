use std::{path::PathBuf, sync::Arc};

use tokio::sync::Mutex;

use crate::extensions::{ExtensionLoader, WasmRuntime};

/// Estado global do sistema de extensões
pub struct ExtensionState {
    pub loader: Arc<Mutex<ExtensionLoader>>,
    pub runtime: Arc<Mutex<WasmRuntime>>,
}

impl ExtensionState {
    pub fn new(extensions_dir: PathBuf) -> Result<Self, String> {
        let loader = ExtensionLoader::new(extensions_dir);
        let runtime = WasmRuntime::new().map_err(|e| e.to_string())?;

        Ok(Self {
            loader: Arc::new(Mutex::new(loader)),
            runtime: Arc::new(Mutex::new(runtime)),
        })
    }
}