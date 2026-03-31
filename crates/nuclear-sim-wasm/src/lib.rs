//! WASM bindings for the nuclear simulation engine.

use nuclear_sim::{
    build_extracted_database, DecayMode, NeutronEnergy, SimEvent, Simulation as CoreSim, Stability,
    Nuclide,
};
use serde::Serialize;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn sim_version() -> String {
    nuclear_sim::version().to_string()
}

// ---------------------------------------------------------------------------
// Data types serialized to JS via serde
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct NuclideJs {
    pub z: u16,
    pub n: u16,
    pub a: u16,
    pub symbol: String,
    pub notation: String,
}

impl NuclideJs {
    fn from_nuclide(nuclide: &Nuclide) -> Self {
        Self {
            z: nuclide.z(),
            n: nuclide.n(),
            a: nuclide.mass_number(),
            symbol: nuclide.element_symbol().to_string(),
            notation: nuclide.notation(),
        }
    }
}

#[derive(Serialize)]
pub struct StepJs {
    pub index: usize,
    pub event_type: String,
    pub description: String,
    pub nuclide: NuclideJs,
    /// Extra detail depending on event type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<StepDetail>,
}

#[derive(Serialize)]
pub struct StepDetail {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<NuclideJs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_fragment: Option<NuclideJs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub heavy_fragment: Option<NuclideJs>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub neutrons_released: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decay_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<NuclideJs>,
}

fn decay_mode_name(mode: &DecayMode) -> &'static str {
    match mode {
        DecayMode::Alpha => "alpha",
        DecayMode::BetaMinus => "beta-minus",
        DecayMode::BetaPlus => "beta-plus",
        DecayMode::ElectronCapture => "electron-capture",
        DecayMode::IsomericTransition => "isomeric-transition",
    }
}

fn event_to_step(index: usize, event: &SimEvent) -> StepJs {
    match event {
        SimEvent::Start { nuclide } => StepJs {
            index,
            event_type: "start".into(),
            description: format!("Starting with {}", nuclide.notation()),
            nuclide: NuclideJs::from_nuclide(nuclide),
            detail: None,
        },
        SimEvent::NeutronAbsorbed {
            target,
            energy,
            compound,
        } => StepJs {
            index,
            event_type: "neutron-absorbed".into(),
            description: format!(
                "{} absorbs a {} neutron, forming {}",
                target.notation(),
                match energy {
                    NeutronEnergy::Slow => "slow",
                    NeutronEnergy::Fast => "fast",
                },
                compound.notation()
            ),
            nuclide: NuclideJs::from_nuclide(compound),
            detail: Some(StepDetail {
                target: Some(NuclideJs::from_nuclide(target)),
                energy: Some(
                    match energy {
                        NeutronEnergy::Slow => "slow",
                        NeutronEnergy::Fast => "fast",
                    }
                    .into(),
                ),
                light_fragment: None,
                heavy_fragment: None,
                neutrons_released: None,
                decay_mode: None,
                parent: None,
            }),
        },
        SimEvent::Fission {
            parent,
            light,
            heavy,
            neutrons_released,
        } => StepJs {
            index,
            event_type: "fission".into(),
            description: format!(
                "{} splits into {} + {} + {} neutrons",
                parent.notation(),
                light.notation(),
                heavy.notation(),
                neutrons_released
            ),
            nuclide: NuclideJs::from_nuclide(heavy),
            detail: Some(StepDetail {
                target: None,
                energy: None,
                light_fragment: Some(NuclideJs::from_nuclide(light)),
                heavy_fragment: Some(NuclideJs::from_nuclide(heavy)),
                neutrons_released: Some(*neutrons_released),
                decay_mode: None,
                parent: Some(NuclideJs::from_nuclide(parent)),
            }),
        },
        SimEvent::Decay {
            parent,
            mode,
            daughter,
        } => StepJs {
            index,
            event_type: "decay".into(),
            description: format!(
                "{} undergoes {} decay, producing {}",
                parent.notation(),
                decay_mode_name(mode),
                daughter.notation()
            ),
            nuclide: NuclideJs::from_nuclide(daughter),
            detail: Some(StepDetail {
                target: None,
                energy: None,
                light_fragment: None,
                heavy_fragment: None,
                neutrons_released: None,
                decay_mode: Some(decay_mode_name(mode).into()),
                parent: Some(NuclideJs::from_nuclide(parent)),
            }),
        },
        SimEvent::Stable { nuclide } => StepJs {
            index,
            event_type: "stable".into(),
            description: format!("{} is stable", nuclide.notation()),
            nuclide: NuclideJs::from_nuclide(nuclide),
            detail: None,
        },
    }
}

#[derive(Serialize)]
pub struct SimStateJs {
    pub cursor: usize,
    pub step_count: usize,
    pub is_complete: bool,
    pub can_decay: bool,
    pub can_fire: bool,
    pub current_step: StepJs,
    pub has_fission_branch: bool,
    pub following_heavy: bool,
}

#[derive(Serialize)]
pub struct NuclideKeyJs {
    pub z: u16,
    pub n: u16,
}

#[derive(Serialize)]
pub struct NuclideDataJs {
    pub notation: String,
    pub is_stable: bool,
    pub is_fissile: bool,
    pub half_life_s: Option<f64>,
    pub decay_modes: Vec<String>,
}

// ---------------------------------------------------------------------------
// Preset isotopes available to the UI
// ---------------------------------------------------------------------------

#[derive(Serialize)]
pub struct PresetJs {
    pub z: u16,
    pub n: u16,
    pub notation: String,
    pub label: String,
}

fn presets() -> Vec<PresetJs> {
    vec![
        PresetJs { z: 92, n: 143, notation: "U-235".into(), label: "Uranium-235 (fissile)".into() },
        PresetJs { z: 92, n: 146, notation: "U-238".into(), label: "Uranium-238".into() },
        PresetJs { z: 94, n: 145, notation: "Pu-239".into(), label: "Plutonium-239 (fissile)".into() },
        PresetJs { z: 27, n: 33, notation: "Co-60".into(), label: "Cobalt-60 (gamma source)".into() },
        PresetJs { z: 6, n: 8, notation: "C-14".into(), label: "Carbon-14 (carbon dating)".into() },
    ]
}

// ---------------------------------------------------------------------------
// WASM-exposed simulation session
// ---------------------------------------------------------------------------

#[wasm_bindgen]
pub struct SimSession {
    sim: CoreSim,
    following_heavy: bool,
}

#[wasm_bindgen]
impl SimSession {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            sim: CoreSim::new(build_extracted_database()),
            following_heavy: true,
        }
    }

    /// Get available preset isotopes.
    pub fn presets(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&presets()).unwrap()
    }

    /// Set the starting isotope by Z and N.
    pub fn set_isotope(&mut self, z: u16, n: u16) -> Result<(), JsValue> {
        self.following_heavy = true;
        self.sim
            .set_isotope(z, n)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Fire a neutron. energy: "slow" or "fast".
    pub fn fire_neutron(&mut self, energy: &str) -> Result<(), JsValue> {
        let e = match energy {
            "slow" => NeutronEnergy::Slow,
            "fast" => NeutronEnergy::Fast,
            _ => return Err(JsValue::from_str("energy must be 'slow' or 'fast'")),
        };
        self.sim
            .fire_neutron(e)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Induce a single decay step. Returns error if stable.
    pub fn induce_decay(&mut self) -> Result<(), JsValue> {
        self.sim
            .induce_decay()
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Switch to following the light or heavy fission fragment.
    /// fragment: "light" or "heavy"
    pub fn switch_branch(&mut self, fragment: &str) -> Result<(), JsValue> {
        let follow_light = match fragment {
            "light" => true,
            "heavy" => false,
            _ => return Err(JsValue::from_str("fragment must be 'light' or 'heavy'")),
        };
        self.following_heavy = !follow_light;
        self.sim
            .switch_branch(0, follow_light)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }

    /// Move cursor forward one step.
    pub fn step_forward(&mut self) -> Result<(), JsValue> {
        self.sim
            .step_forward()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    /// Move cursor backward one step.
    pub fn step_back(&mut self) -> Result<(), JsValue> {
        self.sim
            .step_back()
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    /// Jump to a specific step index.
    pub fn go_to_step(&mut self, index: usize) -> Result<(), JsValue> {
        self.sim
            .go_to_step(index)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        Ok(())
    }

    /// Get the full simulation state (current step, cursor, etc.).
    pub fn state(&self) -> JsValue {
        let event = self.sim.current_event().expect("sim should have state");
        let step = event_to_step(self.sim.cursor(), event);
        let state = SimStateJs {
            cursor: self.sim.cursor(),
            step_count: self.sim.step_count(),
            is_complete: self.sim.is_complete(),
            can_decay: self.sim.can_decay(),
            can_fire: self.sim.can_fire(),
            current_step: step,
            has_fission_branch: !self.sim.fission_branches().is_empty(),
            following_heavy: self.following_heavy,
        };
        serde_wasm_bindgen::to_value(&state).unwrap()
    }

    /// Get all steps as an array (for the chain summary).
    pub fn all_steps(&self) -> JsValue {
        let steps: Vec<StepJs> = self
            .sim
            .steps()
            .iter()
            .enumerate()
            .map(|(i, e)| event_to_step(i, e))
            .collect();
        serde_wasm_bindgen::to_value(&steps).unwrap()
    }

    /// Return all (Z, N) pairs in the database as [{z, n}].
    pub fn all_nuclide_keys(&self) -> JsValue {
        let keys: Vec<NuclideKeyJs> = self
            .sim
            .nuclide_keys()
            .into_iter()
            .map(|(z, n)| NuclideKeyJs { z, n })
            .collect();
        serde_wasm_bindgen::to_value(&keys).unwrap()
    }

    /// Look up data for a nuclide. Returns null if unknown.
    pub fn lookup(&self, z: u16, n: u16) -> JsValue {
        let nuclide = match Nuclide::try_new(z, n) {
            Ok(n) => n,
            Err(_) => return JsValue::NULL,
        };
        match self.sim.lookup(&nuclide) {
            Some(data) => {
                let js = NuclideDataJs {
                    notation: nuclide.notation(),
                    is_stable: data.stability == Stability::Stable,
                    is_fissile: data.fissile,
                    half_life_s: data.half_life_s,
                    decay_modes: data
                        .decay_modes
                        .iter()
                        .map(|b| decay_mode_name(&b.mode).to_string())
                        .collect(),
                };
                serde_wasm_bindgen::to_value(&js).unwrap()
            }
            None => JsValue::NULL,
        }
    }
}
