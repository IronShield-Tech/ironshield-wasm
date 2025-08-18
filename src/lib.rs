//! IronShield WebAssembly bindings

// Suppress warnings from wasm-bindgen internals during ABI transition
#![allow(wasm_c_abi)]

mod js_challenge;
mod js_response;
mod js_token;
mod wasm_compat;
mod js_request;

use wasm_bindgen::prelude::*;
use hex;
use serde_json;

/// Support for threading in WASM
#[cfg(all(feature = "parallel", not(feature = "no-parallel")))]
use wasm_bindgen_rayon::init_thread_pool;

#[cfg(all(feature = "parallel", not(feature = "no-parallel")))]
use wasm_bindgen_futures::JsFuture;

use ironshield_core;

/// JavaScript-compatible solution result for IronShield challenges
///
/// * `solution_str`:            String representation of the solution nonce
///                              to avoid JavaScript BigInt precision issues.
/// * `solution`:                Original numeric value for compatibility.
/// * `challenge_signature_hex`: Challenge signature preserved from the 
///                              original challenge.
#[derive(serde::Serialize)]
struct IronShieldSolutionResult {
    solution_str:            String,
    solution:                i64,
    challenge_signature_hex: String,
}

/// Creates a standardized IronShield solution result from core library output.
fn create_ironshield_solution_result(response: ironshield_core::IronShieldChallengeResponse) -> IronShieldSolutionResult {
    IronShieldSolutionResult {
        solution_str: response.solution.to_string(),
        solution: response.solution,
        challenge_signature_hex: hex::encode(response.solved_challenge.challenge_signature),
    }
}

/// Initializes WebAssembly thread pool for parallel proof-of-work
///
/// # Arguments
/// * `num_threads`: Number of worker threads to spawn
///
/// # Note
/// Only available when compiled with a "parallel" feature flag
#[wasm_bindgen]
#[cfg(all(feature = "parallel", not(feature = "no-parallel")))]
pub async fn init_threads(num_threads: usize) -> Result<(), JsValue> {
    // Create a shared memory thread pool for parallel processing
    let promise: js_sys::Promise = init_thread_pool(num_threads);
    JsFuture::from(promise).await.map(|_| ()).map_err(|e: JsValue| e)
}

/// Checks if parallel processing is available in the current build.
///
/// # Returns
/// `bool`: `true` if compiled with a "parallel" feature, `false` otherwise.
#[wasm_bindgen]
pub extern "C" fn are_threads_supported() -> bool {
    #[cfg(all(feature = "parallel", not(feature = "no-parallel")))]
    return true;

    #[cfg(not(all(feature = "parallel", not(feature = "no-parallel"))))]
    return false;
}

/// Outputs debug message to browser console.
///
/// # Arguments
/// * `s`: Message string to log.
///
/// # Note
/// Useful for debugging WASM execution from JavaScript.
#[wasm_bindgen]
pub fn console_log(s: &str) {
    web_sys::console::log_1(&JsValue::from_str(s));
}

/// Solves IronShield proof-of-work challenges using single-threaded computation.
///
/// # Arguments
/// * `challenge_json`: JSON string containing the IronShieldChallenge
///
/// # Returns
/// * `Result<JsValue, JsValue>`: JavaScript object with solution nonce 
///                               and challenge signature, or error message.
#[wasm_bindgen]
pub fn solve_ironshield_challenge(challenge_json: &str) -> Result<JsValue, JsValue> {
    console_log("🔍 [WASM] solve_ironshield_challenge() called - using SINGLE-THREADED algorithm");

    // Parse the challenge from JSON.
    let challenge: ironshield_core::IronShieldChallenge = serde_json::from_str(challenge_json)
        .map_err(|e: serde_json::Error| JsValue::from_str(&format!("Error parsing challenge JSON: {}", e)))?;

    console_log("🔍 [WASM] Calling ironshield_core::find_solution_single_threaded()");

    // Find valid nonce using a single-threaded algorithm.
    let response: ironshield_core::IronShieldChallengeResponse = ironshield_core::find_solution_single_threaded(&challenge, None)
        .map_err(|e: String| JsValue::from_str(&format!("Error solving IronShield challenge: {}", e)))?;

    console_log("✅ [WASM] Single-threaded solution found");

    // Package result for JavaScript consumption.
    let solution_result: IronShieldSolutionResult = create_ironshield_solution_result(response);

    // Convert Rust struct to JavaScript object
    serde_wasm_bindgen::to_value(&solution_result)
        .map_err(|err: serde_wasm_bindgen::Error| JsValue::from_str(&format!("Error serializing IronShield result: {:?}", err)))
}

/// Solves IronShield proof-of-work challenges using optimized multithreaded computation.
///
/// This function provides the fastest possible PoW solving by distributing the work
/// across the specified number of threads with optimal load balancing and early termination.
///
/// # Arguments
/// * `challenge_json`: JSON string containing the IronShieldChallenge
/// * `num_threads`:    Number of threads to use (optional, defaults to available cores)
/// * `start_offset`:   Starting nonce offset for worker coordination (optional)
/// * `stride`:         Nonce increment stride for worker coordination (optional) 
///
/// # Returns
/// JavaScript object with solution nonce and challenge signature, or error message.
///
/// # Performance
/// - **Multi-core scaling**:      Near-linear performance improvement with thread count.
/// - **Thread-stride algorithm**: Optimal load balancing without coordination overhead.
/// - **Early termination**:       Stops all threads immediately when a solution is found.
/// - **Memory efficient**:        Minimal overhead compared to a single-threaded version.
#[wasm_bindgen]
#[cfg(all(feature = "parallel", not(feature = "no-parallel")))]
pub fn solve_ironshield_challenge_multi_threaded(
    challenge_json: &str,
    start_offset: Option<u32>,
    stride: Option<u32>,
    progress_callback: &js_sys::Function,
) -> Result<JsValue, JsValue> {
    // Skip panic hook installation to avoid "unreachable executed" in workers
    // console_error_panic_hook::set_once()

    console_log("🚀 [WASM] solve_ironshield_challenge_multi_threaded() called - using WORKER COORDINATION algorithm");

    // Parse the challenge JSON
    let challenge: ironshield_core::IronShieldChallenge = serde_json::from_str(challenge_json)
        .map_err(|e: serde_json::Error| JsValue::from_str(&format!("Error parsing challenge JSON: {}", e)))?;

    let start: Option<usize> = start_offset.map(|n: u32| n as usize);
    let step: Option<usize> = stride.map(|n: u32| n as usize);
    
    if let (Some(start_val), Some(stride_val)) = (start, step) {
        console_log(&format!("🎯 [WASM] JavaScript worker coordination: start={}, stride={} (checks nonce's {}, {}, {}, ...)", start_val, stride_val, start_val, start_val + stride_val, start_val + 2*stride_val));
    } else {
        console_log("🔄 [WASM] Single-threaded fallback mode (no worker coordination)");
    }

    // Create a Rust closure that wraps the JavaScript callback function
    let callback: js_sys::Function = progress_callback.clone();
    let closure = move |progress: u64| {
        // Call the JavaScript function, passing the progress value
        let _ = callback.call1(&JsValue::NULL, &JsValue::from(progress));
    };

    // Find valid nonce using JavaScript worker coordinated algorithm.
    let response: ironshield_core::IronShieldChallengeResponse = ironshield_core::find_solution_multi_threaded(
        &challenge,
        Some(ironshield_core::PoWConfig::multi_threaded()),
        start, 
        step,
        Some(&closure)
    ).map_err(|e: String| JsValue::from_str(&format!("Error solving IronShield challenge with worker coordination: {}", e)))?;

    console_log("✅ [WASM] Worker coordination solution found");

    // Convert the response to a JavaScript object
    let result: js_sys::Object = js_sys::Object::new();
    js_sys::Reflect::set(&result, &"solution_str".into(), &response.solution.to_string().into())?;
    js_sys::Reflect::set(&result, &"solution".into(), &JsValue::from(response.solution))?;
    js_sys::Reflect::set(&result, &"challenge_signature_hex".into(), &hex::encode(response.solved_challenge.challenge_signature).into())?;

    Ok(result.into())
}

/// Verifies an IronShield proof-of-work solution without recomputing.
///
/// # Arguments
/// * `challenge_json`: JSON string containing the original IronShieldChallenge.
/// * `solution_nonce`: Proposed solution nonce as i64.
///
/// # Returns
/// `Result<bool, JsValue>`: `true` if the solution is valid, `false` otherwise.
#[wasm_bindgen]
pub fn verify_ironshield_solution(challenge_json: &str, solution_nonce: i64) -> Result<bool, JsValue> {
    // Parse the challenge from JSON
    let challenge: ironshield_core::IronShieldChallenge = serde_json::from_str(challenge_json)
        .map_err(|e: serde_json::Error| JsValue::from_str(&format!("Error parsing challenge JSON for verification: {}", e)))?;

    // Create a challenge response for verification
    let response: ironshield_core::IronShieldChallengeResponse = ironshield_core::IronShieldChallengeResponse::new(challenge, solution_nonce);

    // Verify the solution using the new API
    let is_valid: bool = ironshield_core::verify_ironshield_solution(&response);
    Ok(is_valid)
}
