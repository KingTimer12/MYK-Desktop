pub mod abi;
pub mod commands;
pub mod loader;
pub mod manifest;
pub mod repository;
pub mod runtime;
pub mod sandbox;

pub use loader::ExtensionLoader;
pub use manifest::{ExtensionExports, ExtensionManifest};
pub use runtime::WasmRuntime;
pub use sandbox::HttpSandbox;

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ExtensionError {
    LoadError(String),
    RuntimeError(String),
    ValidationError(String),
    HttpError(String),
    SerializationError(String),
}

impl fmt::Display for ExtensionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ExtensionError::LoadError(msg) => write!(f, "Load error: {}", msg),
            ExtensionError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            ExtensionError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            ExtensionError::HttpError(msg) => write!(f, "HTTP error: {}", msg),
            ExtensionError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl Error for ExtensionError {}

impl From<std::io::Error> for ExtensionError {
    fn from(err: std::io::Error) -> Self {
        ExtensionError::LoadError(err.to_string())
    }
}

impl From<serde_json::Error> for ExtensionError {
    fn from(err: serde_json::Error) -> Self {
        ExtensionError::SerializationError(err.to_string())
    }
}

impl From<rmp_serde::encode::Error> for ExtensionError {
    fn from(err: rmp_serde::encode::Error) -> Self {
        ExtensionError::SerializationError(err.to_string())
    }
}

impl From<rmp_serde::decode::Error> for ExtensionError {
    fn from(err: rmp_serde::decode::Error) -> Self {
        ExtensionError::SerializationError(err.to_string())
    }
}
