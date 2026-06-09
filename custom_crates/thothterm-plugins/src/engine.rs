use crate::error::{PluginError, PluginResult};
use std::path::Path;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::preview1::{self, WasiP1Ctx};
use wasmtime_wasi::WasiCtxBuilder;

pub struct WasmEngine {
    engine: Engine,
}

impl WasmEngine {
    pub fn new() -> PluginResult<Self> {
        let engine = Engine::default();
        Ok(Self { engine })
    }

    /// Execute a named export hook from a WASM file.
    /// Hook signature: `fn hook_name()` — no args, no return.
    pub fn run_hook(&self, wasm_path: &Path, hook_name: &str) -> PluginResult<()> {
        if !wasm_path.exists() {
            return Err(PluginError::WasmFile { path: wasm_path.to_path_buf() });
        }

        let module = Module::from_file(&self.engine, wasm_path)
            .map_err(|e| PluginError::WasmLoad { reason: e.to_string() })?;

        let mut linker: Linker<WasiP1Ctx> = Linker::new(&self.engine);
        preview1::add_to_linker_sync(&mut linker, |t| t)
            .map_err(|e| PluginError::WasmLoad { reason: e.to_string() })?;

        let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build_p1();
        let mut store = Store::new(&self.engine, wasi_ctx);

        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| PluginError::WasmLoad { reason: e.to_string() })?;

        if let Some(func) = instance.get_func(&mut store, hook_name) {
            func.call(&mut store, &[], &mut [])
                .map_err(|e| PluginError::HookFailed {
                    hook: hook_name.to_string(),
                    reason: e.to_string(),
                })?;
        }

        Ok(())
    }
}

impl Default for WasmEngine {
    fn default() -> Self {
        Self::new().expect("Failed to create WASM engine")
    }
}
