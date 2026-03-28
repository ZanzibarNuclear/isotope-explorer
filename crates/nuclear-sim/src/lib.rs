//! Simulation core: nuclides, neutron interactions, decay, fission, energy accounting.
//! Kept free of web / WASM dependencies for tests and reuse.

/// Library version string (for UI / debugging).
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
