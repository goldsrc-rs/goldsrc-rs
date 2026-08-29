//! WASM Component compilation, instantiation, metadata extraction, and linker setup.

use crate::bindings::{GoldsrcPlugin, goldsrc::engine::api};
use crate::error::LoadError;
use crate::manager::state::HostState;
use crate::plugin::{LoadedPlugin, PluginMetadata, PluginStatus};
use goldsrc_api::Engine as GoldsrcEngine;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use wasmtime::Engine;
use wasmtime::component::{Component, Linker};

/// Compiles and instantiates a WASM plugin component without registering or running `on_load`.
pub fn instantiate_plugin<P: AsRef<Path>>(
    engine: &Engine,
    engine_ops: &Arc<dyn GoldsrcEngine>,
    path: P,
) -> Result<LoadedPlugin, LoadError> {
    let path = path.as_ref();
    let metadata = fs::metadata(path).map_err(|source| LoadError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    if metadata.len() > 32 * 1024 * 1024 {
        return Err(LoadError::Compile(format!(
            "Plugin size ({} bytes) exceeds maximum allowed size (32MB)",
            metadata.len()
        )));
    }
    let bytes = fs::read(path).map_err(|source| LoadError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    let is_comp = bytes.len() >= 8 && &bytes[0..8] == b"\0asm\x0d\0\x01\0";
    let component_bytes = if is_comp {
        bytes
    } else {
        let mut resolve = wit_parser::Resolve::default();
        let pkg = resolve
            .push_str(
                "goldsrc.wit",
                include_str!("../../../../core/goldsrc-api/wit/goldsrc.wit"),
            )
            .unwrap();
        let world_id = resolve
            .select_world(&[pkg], Some("goldsrc-plugin"))
            .unwrap();

        let mut wasm_bytes = bytes.to_vec();
        wit_component::embed_component_metadata(
            &mut wasm_bytes,
            &resolve,
            world_id,
            wit_component::StringEncoding::UTF8,
        )
        .map_err(|e| LoadError::Embed(e.to_string()))?;

        let mut base_encoder = wit_component::ComponentEncoder::default();
        let encoder = base_encoder.validate(true);
        let encoder = encoder
            .module(&wasm_bytes)
            .map_err(|e| LoadError::Encode(format!("{e:#?}")))?;
        encoder
            .encode()
            .map_err(|e| LoadError::Encode(format!("{e:#?}")))?
    };

    let component =
        Component::new(engine, &component_bytes).map_err(|e| LoadError::Compile(e.to_string()))?;

    let mut linker = Linker::new(engine);
    api::add_to_linker::<HostState, wasmtime::component::HasSelf<HostState>>(
        &mut linker,
        |state: &mut HostState| state,
    )
    .map_err(|e| LoadError::Link(e.to_string()))?;

    let limits = wasmtime::StoreLimitsBuilder::new()
        .memory_size(64 * 1024 * 1024) // 64MB per memory
        .table_elements(10_000)
        .memories(4)
        .tables(16)
        .instances(16)
        .build();
    let state = HostState {
        engine: engine_ops.clone(),
        limits,
        plugin_name: String::new(),
        shared_buckets: Vec::new(),
    };
    let mut store = wasmtime::Store::new(engine, state);
    store.limiter(|s| &mut s.limits);
    store.set_epoch_deadline(100);
    let bindings = GoldsrcPlugin::instantiate(&mut store, &component, &linker)
        .map_err(|e| LoadError::Instantiate(e.to_string()))?;

    let metadata = match bindings.call_get_metadata(&mut store) {
        Ok(meta_str) => match toml::from_str::<PluginMetadata>(&meta_str) {
            Ok(mut meta) => {
                if let Some(ref b) = meta.bundle {
                    if b.is_empty()
                        || b.contains("..")
                        || b.starts_with('/')
                        || b.starts_with('\\')
                        || b.contains(':')
                        || !b
                            .chars()
                            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '/')
                    {
                        crate::host_log(&format!(
                            "Warning: Rejected invalid/unsafe bundle '{b}' for plugin at {:?}",
                            path
                        ));
                        meta.bundle = None;
                    }
                }
                Some(meta)
            }
            Err(err) => {
                crate::host_log(&format!(
                    "Warning: Failed to parse metadata for plugin at {:?}: {}",
                    path, err
                ));
                None
            }
        },
        Err(_) => None,
    };

    let name = path
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let shared_buckets = metadata
        .as_ref()
        .map(|m| m.shared_buckets.clone())
        .unwrap_or_default();

    // Update HostState with validated plugin name and shared buckets allowlist
    {
        let data = store.data_mut();
        data.plugin_name = name.clone();
        data.shared_buckets = shared_buckets;
    }

    Ok(LoadedPlugin {
        name,
        path: path.to_path_buf(),
        status: PluginStatus::Loaded,
        metadata,
        store,
        bindings,
        component,
    })
}
