use thothterm_config::{RunMode, ThothConfig};
use thothterm_plugins::PluginManager;

pub fn init() {
    let mode = RunMode::detect();
    log::info!("ThothTerm run mode: {}", mode.description());
    init_config();
    init_plugins(&mode);
    start_gas_tracker();
}

fn init_config() {
    match ThothConfig::load() {
        Ok(cfg) => {
            log::info!(
                "ThothTerm config loaded: font_size={}, scheme={}",
                cfg.appearance.font_size,
                &cfg.appearance.theme
            );
        }
        Err(thothterm_config::ConfigError::NotFound { .. }) => {
            log::info!("No thothterm.toml found — using defaults. Press Ctrl+, to open settings.");
            thothterm_toast_notification::persistent_toast_notification(
                "Welcome to ThothTerm 𓆣",
                "Press Ctrl+, for settings, or create ~/.config/thothterm/thothterm.toml",
            );
        }
        Err(e) => {
            log::warn!("Failed to load thothterm.toml: {}", e);
        }
    }
}

fn init_plugins(mode: &RunMode) {
    let plugin_dir = mode.plugins_dir();

    if !plugin_dir.exists() {
        return;
    }

    let mut manager = PluginManager::new(plugin_dir);
    let results = manager.load_all();
    let ok_count = results.iter().filter(|r| r.is_ok()).count();
    let err_count = results.iter().filter(|r| r.is_err()).count();
    if ok_count > 0 {
        log::info!("ThothTerm: {} plugin(s) loaded", ok_count);
        #[cfg(feature = "wasm")]
        manager.run_hook("on_startup");
    }
    if err_count > 0 {
        log::warn!("ThothTerm: {} plugin(s) failed to load", err_count);
    }
}

/// Spawn a background thread that polls gas prices every 60 seconds.
/// Results are written to GAS_CACHE in overlay/web3.rs.
fn start_gas_tracker() {
    let rpc_url = ThothConfig::load()
        .ok()
        .filter(|c| c.web3.enabled && !c.web3.rpc_url.is_empty())
        .map(|c| c.web3.rpc_url);

    let rpc_url = match rpc_url {
        Some(url) => url,
        None => return,
    };

    std::thread::spawn(move || {
        let rt = match tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
        {
            Ok(rt) => rt,
            Err(e) => {
                log::error!("Gas tracker: failed to build tokio runtime: {}", e);
                return;
            }
        };
        loop {
            let client = thothterm_web3::Web3Client::new(&rpc_url);
            if let Ok(gas) = rt.block_on(client.gas_info()) {
                if let Ok(mut cache) = crate::overlay::web3::GAS_CACHE.lock() {
                    *cache = Some(gas.display_label());
                }
                log::debug!("Gas tracker: {}", gas.display_label());
            }
            std::thread::sleep(std::time::Duration::from_secs(60));
        }
    });
}
