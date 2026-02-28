// Runtime WASM usando Wasmtime
// Suporta qualquer linguagem que compile para WebAssembly

use crate::extensions::{abi, ExtensionError};
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use wasmtime::*;

/// Runtime WASM que executa extensões de forma isolada e segura
pub struct WasmRuntime {
    engine: Engine,
    modules: Arc<Mutex<HashMap<String, Module>>>,
    http_client: Arc<reqwest::Client>,
}

impl WasmRuntime {
    /// Criar novo runtime WASM
    pub fn new() -> Result<Self, ExtensionError> {
        // Configurar engine com limites de segurança
        let mut config = Config::new();

        // Habilitar recursos necessários
        config.wasm_bulk_memory(true);
        config.wasm_multi_value(true);
        config.wasm_reference_types(true);

        // Limites de segurança
        config.max_wasm_stack(512 * 1024); // 512KB stack máximo
        config.consume_fuel(true); // Habilitar fuel para limitar execução

        let engine = Engine::new(&config)
            .map_err(|e| ExtensionError::RuntimeError(format!("Failed to create engine: {}", e)))?;

        // Cliente HTTP compartilhado
        let http_client = Arc::new(
            reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .user_agent("MangaYouKnow/1.0")
                .build()
                .map_err(|e| ExtensionError::HttpError(e.to_string()))?,
        );

        Ok(Self {
            engine,
            modules: Arc::new(Mutex::new(HashMap::new())),
            http_client,
        })
    }

    /// Carregar extensão WASM de um arquivo
    pub async fn load_extension(
        &mut self,
        name: &str,
        wasm_path: &Path,
    ) -> Result<(), ExtensionError> {
        if !wasm_path.exists() {
            return Err(ExtensionError::LoadError(format!(
                "WASM file not found: {:?}",
                wasm_path
            )));
        }

        // Ler bytes do arquivo WASM
        let wasm_bytes = std::fs::read(wasm_path)?;

        // Compilar módulo
        let module = Module::new(&self.engine, &wasm_bytes).map_err(|e| {
            ExtensionError::LoadError(format!("Failed to compile WASM module: {}", e))
        })?;

        // Validar que o módulo exporta as funções necessárias
        self.validate_module(&module)?;

        // Armazenar módulo compilado
        let mut modules = self.modules.lock().unwrap();
        modules.insert(name.to_string(), module);

        Ok(())
    }

    /// Validar que o módulo exporta as funções mínimas necessárias
    fn validate_module(&self, module: &Module) -> Result<(), ExtensionError> {
        let exports: Vec<String> = module
            .exports()
            .filter_map(|e| {
                if matches!(e.ty(), ExternType::Func(_)) {
                    Some(e.name().to_string())
                } else {
                    None
                }
            })
            .collect();

        // Verificar alloc e dealloc (necessários para memória)
        if !exports.contains(&abi::wasm_exports::ALLOC.to_string()) {
            return Err(ExtensionError::ValidationError(
                "Module must export 'alloc' function".into(),
            ));
        }

        if !exports.contains(&abi::wasm_exports::DEALLOC.to_string()) {
            return Err(ExtensionError::ValidationError(
                "Module must export 'dealloc' function".into(),
            ));
        }

        // Pelo menos uma função de extensão deve estar presente
        let has_any_function = exports.contains(&abi::wasm_exports::SEARCH.to_string())
            || exports.contains(&abi::wasm_exports::GET_CHAPTERS.to_string())
            || exports.contains(&abi::wasm_exports::GET_CHAPTER_IMAGES.to_string());

        if !has_any_function {
            return Err(ExtensionError::ValidationError(
                "Module must export at least one extension function (search, get_chapters, get_chapter_images)".into(),
            ));
        }

        Ok(())
    }

    /// Executar função de extensão
    pub async fn execute_function(
        &self,
        extension_name: &str,
        function_name: &str,
        params: Vec<u8>,
    ) -> Result<Vec<u8>, ExtensionError> {
        // Obter módulo
        let module = {
            let modules = self.modules.lock().unwrap();
            modules
                .get(extension_name)
                .ok_or_else(|| {
                    ExtensionError::RuntimeError(format!(
                        "Extension '{}' not loaded",
                        extension_name
                    ))
                })?
                .clone()
        };

        // Criar store com fuel limit (previne loops infinitos)
        let mut store = Store::new(&self.engine, HostState::new(self.http_client.clone()));
        store
            .set_fuel(10_000_000)
            .map_err(|e| ExtensionError::RuntimeError(format!("Failed to set fuel: {}", e)))?;

        // Criar linker e adicionar host functions
        let mut linker = Linker::new(&self.engine);
        self.add_host_functions(&mut linker)?;

        // Instanciar módulo
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| ExtensionError::RuntimeError(format!("Failed to instantiate: {}", e)))?;

        // Obter memória
        let memory = instance
            .get_memory(&mut store, "memory")
            .ok_or_else(|| ExtensionError::RuntimeError("Module must export 'memory'".into()))?;

        // Alocar espaço na memória WASM para os parâmetros
        let params_ptr = if !params.is_empty() {
            let alloc = instance
                .get_typed_func::<i32, i32>(&mut store, abi::wasm_exports::ALLOC)
                .map_err(|e| ExtensionError::RuntimeError(format!("Failed to get alloc: {}", e)))?;

            let ptr = alloc.call(&mut store, params.len() as i32).map_err(|e| {
                ExtensionError::RuntimeError(format!("Failed to allocate memory: {}", e))
            })?;

            // Escrever parâmetros na memória
            memory
                .write(&mut store, ptr as usize, &params)
                .map_err(|e| {
                    ExtensionError::RuntimeError(format!("Failed to write params: {}", e))
                })?;

            ptr
        } else {
            0
        };

        // Chamar função da extensão
        let func = instance
            .get_typed_func::<(i32, i32), i32>(&mut store, function_name)
            .map_err(|e| {
                ExtensionError::RuntimeError(format!(
                    "Function '{}' not found or has wrong signature: {}",
                    function_name, e
                ))
            })?;

        let result_ptr = func
            .call(&mut store, (params_ptr, params.len() as i32))
            .map_err(|e| {
                ExtensionError::RuntimeError(format!("Function execution failed: {}", e))
            })?;

        // Ler resultado da memória
        let result_data = if result_ptr != 0 {
            // Ler primeiros 4 bytes para obter tamanho
            let mut size_bytes = [0u8; 4];
            memory
                .read(&store, result_ptr as usize, &mut size_bytes)
                .map_err(|e| {
                    ExtensionError::RuntimeError(format!("Failed to read result size: {}", e))
                })?;

            let size = u32::from_le_bytes(size_bytes) as usize;

            // Ler dados completos (pular os 4 bytes de tamanho)
            let mut data = vec![0u8; size];
            memory
                .read(&store, (result_ptr + 4) as usize, &mut data)
                .map_err(|e| {
                    ExtensionError::RuntimeError(format!("Failed to read result: {}", e))
                })?;

            // Liberar memória no WASM
            let dealloc = instance
                .get_typed_func::<(i32, i32), ()>(&mut store, abi::wasm_exports::DEALLOC)
                .map_err(|e| {
                    ExtensionError::RuntimeError(format!("Failed to get dealloc: {}", e))
                })?;

            dealloc
                .call(&mut store, (result_ptr, (size + 4) as i32))
                .map_err(|e| {
                    ExtensionError::RuntimeError(format!("Failed to deallocate: {}", e))
                })?;

            data
        } else {
            vec![]
        };

        // Liberar parâmetros se foram alocados
        if params_ptr != 0 {
            let dealloc = instance
                .get_typed_func::<(i32, i32), ()>(&mut store, abi::wasm_exports::DEALLOC)
                .map_err(|e| {
                    ExtensionError::RuntimeError(format!("Failed to get dealloc: {}", e))
                })?;

            dealloc
                .call(&mut store, (params_ptr, params.len() as i32))
                .map_err(|e| {
                    ExtensionError::RuntimeError(format!("Failed to deallocate params: {}", e))
                })?;
        }

        Ok(result_data)
    }

    /// Adicionar host functions ao linker
    fn add_host_functions(&self, linker: &mut Linker<HostState>) -> Result<(), ExtensionError> {
        // host_log - simplificado para apenas usar println por enquanto
        linker
            .func_wrap(
                "env",
                abi::host_functions::LOG,
                |mut caller: Caller<'_, HostState>, level: i32, msg_ptr: i32, msg_len: i32| {
                    let memory = match caller.get_export("memory") {
                        Some(Extern::Memory(mem)) => mem,
                        _ => return,
                    };

                    let mut msg_bytes = vec![0u8; msg_len as usize];
                    if memory
                        .read(&caller, msg_ptr as usize, &mut msg_bytes)
                        .is_err()
                    {
                        return;
                    }

                    if let Ok(msg) = String::from_utf8(msg_bytes) {
                        match level {
                            0 => log::trace!("[WASM] {}", msg),
                            1 => log::debug!("[WASM] {}", msg),
                            2 => log::info!("[WASM] {}", msg),
                            3 => log::warn!("[WASM] {}", msg),
                            4 => log::error!("[WASM] {}", msg),
                            _ => log::info!("[WASM] {}", msg),
                        }
                    }
                },
            )
            .map_err(|e| ExtensionError::RuntimeError(format!("Failed to add host_log: {}", e)))?;

        // host_http_get - Fazer requisição HTTP GET
        linker
            .func_wrap(
                "env",
                abi::host_functions::HTTP_GET,
                |mut caller: Caller<'_, HostState>, url_ptr: i32, url_len: i32| -> i32 {
                    log::warn!(
                        "[WASM] host_http_get called with ptr={}, len={}",
                        url_ptr,
                        url_len
                    );

                    let memory = match caller.get_export("memory") {
                        Some(Extern::Memory(mem)) => mem,
                        _ => {
                            log::warn!("[WASM] host_http_get: Failed to get memory export");
                            return 0;
                        }
                    };

                    // Ler URL da memória
                    let mut url_bytes = vec![0u8; url_len as usize];
                    if memory
                        .read(&caller, url_ptr as usize, &mut url_bytes)
                        .is_err()
                    {
                        log::warn!("[WASM] host_http_get: Failed to read URL from memory");
                        return 0;
                    }

                    let url = match String::from_utf8(url_bytes) {
                        Ok(u) => u,
                        Err(_) => {
                            log::warn!("[WASM] host_http_get: Invalid UTF-8 in URL");
                            return 0;
                        }
                    };

                    log::warn!("[WASM] host_http_get: Requesting URL: {}", url);

                    // Fazer requisição HTTP usando uma thread separada para evitar
                    // "Cannot start a runtime from within a runtime"
                    let http_client = caller.data().http_client.clone();
                    let (tx, rx) = std::sync::mpsc::channel();

                    let url_clone = url.clone();
                    std::thread::spawn(move || {
                        let rt = tokio::runtime::Runtime::new().unwrap();
                        let result = rt.block_on(async {
                            let response = http_client.get(&url_clone).send().await?;
                            let response = response.error_for_status()?;
                            let bytes = response.bytes().await?;
                            Ok::<_, reqwest::Error>(bytes.to_vec())
                        });
                        let _ = tx.send(result);
                    });

                    let response_data = match rx.recv() {
                        Ok(Ok(data)) => {
                            log::warn!("[WASM] HTTP GET success for {}: {} bytes", url, data.len());
                            data
                        }
                        Ok(Err(e)) => {
                            log::warn!("[WASM] HTTP GET failed for {}: {}", url, e);
                            return 0;
                        }
                        Err(e) => {
                            log::warn!("[WASM] HTTP GET channel error for {}: {}", url, e);
                            return 0;
                        }
                    };

                    // Obter função alloc da instância
                    let alloc_func = match caller.get_export("alloc") {
                        Some(Extern::Func(f)) => f,
                        _ => return 0,
                    };

                    let alloc = match alloc_func.typed::<i32, i32>(&caller) {
                        Ok(a) => a,
                        Err(_) => return 0,
                    };

                    let result_size = 4 + response_data.len();
                    let result_ptr = match alloc.call(&mut caller, result_size as i32) {
                        Ok(ptr) => ptr,
                        Err(_) => return 0,
                    };

                    // Escrever tamanho (4 bytes) + dados
                    let size_bytes = (response_data.len() as u32).to_le_bytes();
                    if memory
                        .write(&mut caller, result_ptr as usize, &size_bytes)
                        .is_err()
                    {
                        return 0;
                    }

                    if memory
                        .write(&mut caller, (result_ptr + 4) as usize, &response_data)
                        .is_err()
                    {
                        return 0;
                    }

                    result_ptr
                },
            )
            .map_err(|e| {
                ExtensionError::RuntimeError(format!("Failed to add host_http_get: {}", e))
            })?;

        Ok(())
    }

    /// Verificar se uma extensão está carregada
    pub fn is_loaded(&self, name: &str) -> bool {
        let modules = self.modules.lock().unwrap();
        modules.contains_key(name)
    }

    /// Descarregar extensão
    pub fn unload_extension(&mut self, name: &str) -> Result<(), ExtensionError> {
        let mut modules = self.modules.lock().unwrap();
        modules.remove(name);
        Ok(())
    }

    /// Listar extensões carregadas
    pub fn list_loaded(&self) -> Vec<String> {
        let modules = self.modules.lock().unwrap();
        modules.keys().cloned().collect()
    }
}

impl Default for WasmRuntime {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// Estado do host compartilhado com WASM
struct HostState {
    http_client: Arc<reqwest::Client>,
}

impl HostState {
    fn new(http_client: Arc<reqwest::Client>) -> Self {
        Self { http_client }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_creation() {
        let runtime = WasmRuntime::new();
        assert!(runtime.is_ok());
    }
}
