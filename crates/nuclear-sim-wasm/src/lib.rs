//! Thin WASM surface over `nuclear-sim`. Expand with state types as the sim grows.

use nuclear_sim::Nuclide;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn sim_version() -> String {
    nuclear_sim::version().to_string()
}

/// Snapshot of a nuclide for the UI (Z, N, A, symbol, `"U-235"`-style label).
#[wasm_bindgen]
pub struct NuclideInfo {
    z: u16,
    n: u16,
    a: u16,
    element: String,
    notation: String,
}

#[wasm_bindgen]
impl NuclideInfo {
    #[wasm_bindgen(constructor)]
    pub fn new(z: u16, n: u16) -> Result<NuclideInfo, JsValue> {
        let nuclide = Nuclide::try_new(z, n).map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(Self::from_nuclide(nuclide))
    }

    /// Preset: ²³⁵U (Z=92, N=143).
    pub fn uranium_235() -> NuclideInfo {
        Self::from_nuclide(Nuclide::uranium_235())
    }

    #[wasm_bindgen(getter)]
    pub fn z(&self) -> u16 {
        self.z
    }

    #[wasm_bindgen(getter)]
    pub fn n(&self) -> u16 {
        self.n
    }

    #[wasm_bindgen(getter)]
    pub fn a(&self) -> u16 {
        self.a
    }

    #[wasm_bindgen(getter, js_name = elementSymbol)]
    pub fn element_symbol(&self) -> String {
        self.element.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn notation(&self) -> String {
        self.notation.clone()
    }
}

impl NuclideInfo {
    fn from_nuclide(nuclide: Nuclide) -> Self {
        Self {
            z: nuclide.z(),
            n: nuclide.n(),
            a: nuclide.mass_number(),
            element: nuclide.element_symbol().to_string(),
            notation: nuclide.notation(),
        }
    }
}
