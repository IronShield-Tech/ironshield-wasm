//! IronShield WebAssembly bindings

// Suppress warnings from wasm-bindgen internals during ABI transition
#![allow(wasm_c_abi)]

mod wasm_compat;

use wasm_bindgen::prelude::*;

/// Support for threading in WASM
#[cfg(all(feature = "threading", not(feature = "no-threading")))]
use wasm_bindgen_rayon::init_thread_pool;

#[cfg(all(feature = "threading", not(feature = "no-threading")))]
use wasm_bindgen_futures::JsFuture;

/// JavaScript-compatible solution result for IronShield challenges
///
/// * `solution_str`:            String representation of the solution nonce
///                              to avoid JavaScript BigInt precision issues.
/// * `solution`:                Original numeric value for compatibility.
/// * `challenge_signature_hex`: Challenge signature preserved from the 
///                              original challenge.

/// Initializes WebAssembly thread pool for parallel proof-of-work
///
/// # Arguments
/// * `num_threads`: Number of worker threads to spawn
///
/// # Note
/// Only available when compiled with a "threading" feature flag
#[wasm_bindgen]
#[cfg(all(feature = "threading", not(feature = "no-threading")))]
pub async fn init_threads(num_threads: usize) -> Result<(), JsValue> {
    // Create a shared memory thread pool for parallel processing
    let promise: js_sys::Promise = init_thread_pool(num_threads);
    JsFuture::from(promise).await.map(|_| ()).map_err(|e: JsValue| e)
}

/// Checks if threading is available in the current build.
///
/// # Returns
/// `bool`: `true` if compiled with a "threading" feature, `false` otherwise.
#[wasm_bindgen]
pub extern "C" fn are_threads_supported() -> bool {
    #[cfg(all(feature = "threading", not(feature = "no-threading")))]
    return true;

    #[cfg(not(all(feature = "threading", not(feature = "no-threading"))))]
    return false;
}
