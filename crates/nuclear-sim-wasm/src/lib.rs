//! Thin WASM surface over `nuclear-sim`. Expand with state types as the sim grows.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn sim_version() -> String {
    nuclear_sim::version().to_string()
}
