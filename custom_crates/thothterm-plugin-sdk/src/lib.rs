//! ThothTerm Plugin SDK
//!
//! Compile your plugin to WASM:
//!   rustup target add wasm32-wasi
//!   cargo build --release --target wasm32-wasi
//!
//! Example plugin:
//! ```ignore
//! // Compile with: cargo build --release --target wasm32-wasi
//! // See PLUGIN_SDK.md for full examples.
//! ```

// ── Host functions (implemented by ThothTerm, called by plugin) ────────────────

extern "C" {
    fn __thoth_write(ptr: *const u8, len: usize);
    fn __thoth_status_bar_set(slot: u32, ptr: *const u8, len: usize);
    fn __thoth_notify(title_ptr: *const u8, title_len: usize, body_ptr: *const u8, body_len: usize);
    fn __thoth_get_cwd(out_ptr: *mut u8, max_len: usize) -> usize;
    fn __thoth_get_time() -> u64;
    fn __thoth_log(level: u32, ptr: *const u8, len: usize);
    fn __thoth_config_set(key_ptr: *const u8, key_len: usize, val_ptr: *const u8, val_len: usize);
    fn __thoth_config_get(key_ptr: *const u8, key_len: usize, out_ptr: *mut u8, max_len: usize) -> usize;
    fn __thoth_clipboard_get(out_ptr: *mut u8, max_len: usize) -> usize;
    fn __thoth_clipboard_set(ptr: *const u8, len: usize);
}

// ── Safe Rust wrappers ────────────────────────────────────────────────────────

/// Write text to the current terminal pane.
pub fn thoth_write(text: &str) {
    unsafe { __thoth_write(text.as_ptr(), text.len()) }
}

/// Update a status bar slot (0–9) with text.
pub fn thoth_status_bar_set(slot: u32, text: &str) {
    unsafe { __thoth_status_bar_set(slot, text.as_ptr(), text.len()) }
}

/// Show a desktop notification.
pub fn thoth_notify(title: &str, body: &str) {
    unsafe {
        __thoth_notify(
            title.as_ptr(), title.len(),
            body.as_ptr(), body.len(),
        )
    }
}

/// Get the current working directory.
pub fn thoth_get_cwd() -> String {
    let mut buf = vec![0u8; 4096];
    let len = unsafe { __thoth_get_cwd(buf.as_mut_ptr(), buf.len()) };
    String::from_utf8_lossy(&buf[..len]).into_owned()
}

/// Get current Unix timestamp in seconds.
pub fn thoth_get_time() -> u64 {
    unsafe { __thoth_get_time() }
}

/// Store a config value for this plugin.
pub fn thoth_config_set(key: &str, value: &str) {
    unsafe { __thoth_config_set(key.as_ptr(), key.len(), value.as_ptr(), value.len()) }
}

/// Read a config value for this plugin. Returns None if not set.
pub fn thoth_config_get(key: &str) -> Option<String> {
    let mut buf = vec![0u8; 4096];
    let len = unsafe { __thoth_config_get(key.as_ptr(), key.len(), buf.as_mut_ptr(), buf.len()) };
    if len == 0 {
        None
    } else {
        Some(String::from_utf8_lossy(&buf[..len]).into_owned())
    }
}

/// Read from clipboard (requires "clipboard" permission).
pub fn thoth_clipboard_get() -> String {
    let mut buf = vec![0u8; 65536];
    let len = unsafe { __thoth_clipboard_get(buf.as_mut_ptr(), buf.len()) };
    String::from_utf8_lossy(&buf[..len]).into_owned()
}

/// Write to clipboard (requires "clipboard" permission).
pub fn thoth_clipboard_set(text: &str) {
    unsafe { __thoth_clipboard_set(text.as_ptr(), text.len()) }
}

/// Log a debug message (visible in ThothTerm debug log).
pub fn thoth_log_debug(msg: &str) {
    unsafe { __thoth_log(0, msg.as_ptr(), msg.len()) }
}

/// Log an info message.
pub fn thoth_log_info(msg: &str) {
    unsafe { __thoth_log(1, msg.as_ptr(), msg.len()) }
}

// ── Helper for reading strings from WASM memory ───────────────────────────────

/// Read a string from a raw pointer passed by the host.
/// # Safety
/// Only call this from within a hook function where ThothTerm passed the pointer.
pub unsafe fn read_str<'a>(ptr: *const u8, len: usize) -> &'a str {
    let slice = std::slice::from_raw_parts(ptr, len);
    std::str::from_utf8_unchecked(slice)
}

// ── Macro for easy hook registration ─────────────────────────────────────────

/// Declare a hook with automatic extern "C" wrapping.
#[macro_export]
macro_rules! thoth_hook {
    (fn $name:ident($line:ident: &str) $body:block) => {
        #[no_mangle]
        pub unsafe extern "C" fn $name(ptr: *const u8, len: usize) {
            let $line = thothterm_plugin_sdk::read_str(ptr, len);
            $body
        }
    };
}
