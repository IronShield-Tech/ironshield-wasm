//! # WASM Compatibility Functions and Headers.

use js_sys::{global, Reflect};
use serde::{Serialize, Deserialize};
use wasm_bindgen::prelude::*;
use web_sys::{console, window};

/// Represents the browser's WebAssembly compatibility status
/// as a JavaScript object.
///
/// * `mode`:                   The execution mode to use based 
///                             on the browser's capabilities.
///                             `mode` will be `javascript` if
///                             WebAssembly is not supported,
///                             `wasm` if WebAssembly is supported,
///                             or `wasm-mt` if WebAssembly with
///                             multithreading is supported.
/// * `supports_wasm`:          Indicates if the browser supports 
///                             WebAssembly.
/// * `supports_threads`:       Indicates if the browser supports
///                             WebAssembly (multi-)threads.
/// * `supports_shared_memory`: Indicates if the browser supports
///                             SharedArrayBuffer.
/// * `thread_count`:           The number of logical processors 
///                             available.
#[wasm_bindgen]
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmCompatibility {
    #[wasm_bindgen(skip)]
    pub mode:                   String,
    pub supports_wasm:          bool,
    pub supports_shared_memory: bool,
    pub supports_threads:       bool,
    pub thread_count:           u32,
}

#[wasm_bindgen]
impl WasmCompatibility {
    /// Creates a new compatibility object with the default values.
    pub fn new() -> Self {
        Self {
            // Predefined mode for JavaScript execution.
            // This is the worst case scenario.
            mode:                   "javascript".to_string(),
            supports_wasm:          false,
            supports_threads:       false,
            supports_shared_memory: false,
            thread_count:           1,
        }
    }
}

/// # Returns
/// * `bool`: `true` if WebAssembly is supported, `false` otherwise.
fn is_wasm_supported() -> bool {
    let obj = global();
    Reflect::has(&obj, &"WebAssembly".into()).unwrap_or(false)
}

/// # Returns
/// * `bool`: `true` if SharedArrayBuffer is supported, `false` otherwise.
fn is_shared_array_buffer_supported() -> bool {
    let obj = global();
    Reflect::has(&obj, &"SharedArrayBuffer".into()).unwrap_or(false)
}

/// # Returns
/// * `i32`: The number of logical processors available.
///          Returns 1 if the count cannot be determined.
fn get_hardware_concurrency() -> u32 {
    window()
        .and_then(|w| Some(w.navigator().hardware_concurrency()))
        .unwrap_or(1.0) as u32
}

/// # Returns
/// * `bool`: `true` if the client supports multithreading with WebAssembly,
///           `false` otherwise.
fn check_multithreading_support() -> bool {
    is_wasm_supported() && is_shared_array_buffer_supported() && get_hardware_concurrency() > 2
}

/// Checks the WebAssembly compatibility of the current browser
/// and returns an object containing the compatibility status.
///
/// # Returns
/// * `WasmCompatibility`: An object containing the WebAssembly compatibility status.
#[wasm_bindgen]
pub fn check_wasm_compatibility() -> WasmCompatibility {
    let mut compatibility = WasmCompatibility::new();

    // If WebAssembly is not supported, log a warning and return 
    // the default compatibility object provided by `WasmCompatibility::new()`.
    // Mode is already set to "javascript" by default.
    if !is_wasm_supported() {
        console::warn_1(&"WebAssembly is not supported in this browser.".into());
        return compatibility;
    }

    compatibility.supports_wasm          = true;
    compatibility.supports_shared_memory = is_shared_array_buffer_supported();
    compatibility.supports_threads       = check_multithreading_support();
    compatibility.thread_count           = get_hardware_concurrency();

    if compatibility.supports_threads {
        compatibility.mode = "wasm-mt".to_string();
    } else {
        compatibility.mode = "wasm".to_string();
    }

    compatibility
}

/// # Returns
/// * `String`: A JSON string representation of the WebAssembly compatibility status.
#[wasm_bindgen]
pub fn get_wasm_compatibility() -> String {
    let compatibility = check_wasm_compatibility();
    serde_json::to_string(&compatibility)
        .unwrap_or_else(|_| "{\"error\": \"Failed to serialize compatibility data\"}".to_string())
}