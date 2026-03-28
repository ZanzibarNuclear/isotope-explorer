//! Simulation core: nuclides, nuclei, neutron interactions, decay, fission, energy accounting.
//! Kept free of web / WASM dependencies for tests and reuse.

mod elements;
mod nuclide;
mod nucleus;

pub use elements::symbol as element_symbol;
pub use nuclide::{Nuclide, NuclideError};
pub use nucleus::Nucleus;

/// Library version string (for UI / debugging).
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
