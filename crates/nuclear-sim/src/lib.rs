//! Simulation core: nuclides, nuclei, neutron interactions, decay, fission, energy accounting.
//! Kept free of web / WASM dependencies for tests and reuse.

mod data;
mod elements;
mod nuclide;
mod nucleus;
mod sim;

pub use data::{
    build_extracted_database, build_stub_database, DecayBranch, DecayMode, FissionProduct,
    NuclideData, NuclideDatabase, Stability,
};
pub use elements::symbol as element_symbol;
pub use nuclide::{Nuclide, NuclideError};
pub use nucleus::Nucleus;
pub use sim::{NeutronEnergy, SimError, SimEvent, Simulation};

/// Library version string (for UI / debugging).
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
