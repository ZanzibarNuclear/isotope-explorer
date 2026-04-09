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

#[derive(Debug, Clone, PartialEq, Serialize)]
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
    /// Whether the nuclide shown in this row is stable in the database.
    pub nuclide_is_stable: bool,
    /// Whether this nuclide exists in the simulation database.
    pub nuclide_in_database: bool,
    /// Half-life in seconds when radioactive and known; omitted when stable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nuclide_half_life_s: Option<f64>,
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

fn nuclide_timing(sim: &CoreSim, nuclide: &Nuclide) -> (bool, bool, Option<f64>) {
    sim.lookup(nuclide).map_or((false, false, None), |d| {
        (
            d.stability == Stability::Stable,
            true,
            d.half_life_s,
        )
    })
}

fn event_to_step(index: usize, event: &SimEvent, sim: &CoreSim) -> StepJs {
    match event {
        SimEvent::Start { nuclide } => {
            let (nuclide_is_stable, nuclide_in_database, nuclide_half_life_s) = nuclide_timing(sim, nuclide);
            StepJs {
                index,
                event_type: "start".into(),
                description: format!("Starting with {}", nuclide.notation()),
                nuclide: NuclideJs::from_nuclide(nuclide),
                nuclide_is_stable,
                nuclide_in_database,
                nuclide_half_life_s,
                detail: None,
            }
        }
        SimEvent::NeutronAbsorbed {
            target,
            energy,
            compound,
        } => {
            let (nuclide_is_stable, nuclide_in_database, nuclide_half_life_s) = nuclide_timing(sim, compound);
            StepJs {
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
                nuclide_is_stable,
                nuclide_in_database,
                nuclide_half_life_s,
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
            }
        }
        SimEvent::Fission {
            parent,
            energy,
            light,
            heavy,
            neutrons_released,
        } => {
            let energy_label = match energy {
                NeutronEnergy::Slow => "slow",
                NeutronEnergy::Fast => "fast",
            };
            let (nuclide_is_stable, nuclide_in_database, nuclide_half_life_s) = nuclide_timing(sim, heavy);
            StepJs {
                index,
                event_type: "fission".into(),
                description: format!(
                    "{} absorbs a {} neutron and splits into {} + {} + {} neutrons",
                    parent.notation(),
                    energy_label,
                    light.notation(),
                    heavy.notation(),
                    neutrons_released
                ),
                nuclide: NuclideJs::from_nuclide(heavy),
                nuclide_is_stable,
                nuclide_in_database,
                nuclide_half_life_s,
                detail: Some(StepDetail {
                    target: None,
                    energy: Some(energy_label.into()),
                    light_fragment: Some(NuclideJs::from_nuclide(light)),
                    heavy_fragment: Some(NuclideJs::from_nuclide(heavy)),
                    neutrons_released: Some(*neutrons_released),
                    decay_mode: None,
                    parent: Some(NuclideJs::from_nuclide(parent)),
                }),
            }
        }
        SimEvent::Decay {
            parent,
            mode,
            daughter,
        } => {
            let (nuclide_is_stable, nuclide_in_database, nuclide_half_life_s) = nuclide_timing(sim, daughter);
            StepJs {
                index,
                event_type: "decay".into(),
                description: format!(
                    "{} undergoes {} decay, producing {}",
                    parent.notation(),
                    decay_mode_name(mode),
                    daughter.notation()
                ),
                nuclide: NuclideJs::from_nuclide(daughter),
                nuclide_is_stable,
                nuclide_in_database,
                nuclide_half_life_s,
                detail: Some(StepDetail {
                    target: None,
                    energy: None,
                    light_fragment: None,
                    heavy_fragment: None,
                    neutrons_released: None,
                    decay_mode: Some(decay_mode_name(mode).into()),
                    parent: Some(NuclideJs::from_nuclide(parent)),
                }),
            }
        }
        SimEvent::Stable { nuclide } => {
            let (nuclide_is_stable, nuclide_in_database, nuclide_half_life_s) = nuclide_timing(sim, nuclide);
            StepJs {
                index,
                event_type: "stable".into(),
                description: format!("{} is stable", nuclide.notation()),
                nuclide: NuclideJs::from_nuclide(nuclide),
                nuclide_is_stable,
                nuclide_in_database,
                nuclide_half_life_s,
                detail: None,
            }
        }
    }
}

fn build_sim_state(sim: &CoreSim, following_heavy: bool) -> SimStateJs {
    let event = sim.current_event().expect("sim should have state");
    let step = event_to_step(sim.cursor(), event, sim);
    SimStateJs {
        cursor: sim.cursor(),
        step_count: sim.step_count(),
        is_complete: sim.is_complete(),
        can_decay: sim.can_decay(),
        can_fire: sim.can_fire(),
        current_step: step,
        has_fission_branch: !sim.fission_branches().is_empty(),
        following_heavy,
    }
}

fn all_steps_vec(sim: &CoreSim) -> Vec<StepJs> {
    sim
        .steps()
        .iter()
        .enumerate()
        .map(|(i, e)| event_to_step(i, e, sim))
        .collect()
}

fn lookup_nuclide_data(sim: &CoreSim, z: u16, n: u16) -> Option<NuclideDataJs> {
    let nuclide = Nuclide::try_new(z, n).ok()?;
    let data = sim.lookup(&nuclide)?;
    Some(NuclideDataJs {
        notation: nuclide.notation(),
        is_stable: data.stability == Stability::Stable,
        is_fissile: data.is_fissile(),
        half_life_s: data.half_life_s,
        decay_modes: data
            .decay_modes
            .iter()
            .map(|b| decay_mode_name(&b.mode).to_string())
            .collect(),
    })
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

fn parse_neutron_energy(energy: &str) -> Result<NeutronEnergy, &'static str> {
    match energy {
        "slow" => Ok(NeutronEnergy::Slow),
        "fast" => Ok(NeutronEnergy::Fast),
        _ => Err("energy must be 'slow' or 'fast'"),
    }
}

/// Returns `Ok(true)` for light fragment, `Ok(false)` for heavy.
fn parse_fission_fragment(fragment: &str) -> Result<bool, &'static str> {
    match fragment {
        "light" => Ok(true),
        "heavy" => Ok(false),
        _ => Err("fragment must be 'light' or 'heavy'"),
    }
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
        let e = parse_neutron_energy(energy).map_err(JsValue::from_str)?;
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
        let follow_light = parse_fission_fragment(fragment).map_err(JsValue::from_str)?;
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
        serde_wasm_bindgen::to_value(&build_sim_state(&self.sim, self.following_heavy)).unwrap()
    }

    /// Get all steps as an array (for the chain summary).
    pub fn all_steps(&self) -> JsValue {
        serde_wasm_bindgen::to_value(&all_steps_vec(&self.sim)).unwrap()
    }

    /// Auto-follow decay chain from a nuclide (same rules as after fission), for parallel UI legs.
    pub fn decay_chain_preview(&self, z: u16, n: u16) -> Result<JsValue, JsValue> {
        let nuclide = Nuclide::try_new(z, n).map_err(|e| JsValue::from_str(&e.to_string()))?;
        let events = self.sim.decay_chain_events(nuclide);
        let steps: Vec<StepJs> = events
            .iter()
            .enumerate()
            .map(|(i, e)| event_to_step(i, e, &self.sim))
            .collect();
        Ok(serde_wasm_bindgen::to_value(&steps).unwrap())
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
        match lookup_nuclide_data(&self.sim, z, n) {
            Some(js) => serde_wasm_bindgen::to_value(&js).unwrap(),
            None => JsValue::NULL,
        }
    }

    #[cfg(test)]
    pub(crate) fn state_snapshot(&self) -> SimStateJs {
        build_sim_state(&self.sim, self.following_heavy)
    }

    #[cfg(test)]
    pub(crate) fn steps_snapshot(&self) -> Vec<StepJs> {
        all_steps_vec(&self.sim)
    }

    #[cfg(test)]
    pub(crate) fn lookup_data(&self, z: u16, n: u16) -> Option<NuclideDataJs> {
        lookup_nuclide_data(&self.sim, z, n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuclear_sim::{build_extracted_database, DecayMode, NeutronEnergy, Nuclide, SimEvent};

    fn test_sim() -> CoreSim {
        CoreSim::new(build_extracted_database())
    }

    #[test]
    fn sim_version_matches_core() {
        assert_eq!(sim_version(), nuclear_sim::version());
    }

    #[test]
    fn decay_mode_names_are_stable_strings() {
        assert_eq!(decay_mode_name(&DecayMode::Alpha), "alpha");
        assert_eq!(decay_mode_name(&DecayMode::BetaMinus), "beta-minus");
        assert_eq!(decay_mode_name(&DecayMode::BetaPlus), "beta-plus");
        assert_eq!(decay_mode_name(&DecayMode::ElectronCapture), "electron-capture");
        assert_eq!(
            decay_mode_name(&DecayMode::IsomericTransition),
            "isomeric-transition"
        );
    }

    #[test]
    fn nuclide_js_fields_match_nuclide() {
        let n = Nuclide::try_new(92, 143).unwrap();
        let js = NuclideJs::from_nuclide(&n);
        assert_eq!(js.z, 92);
        assert_eq!(js.n, 143);
        assert_eq!(js.a, 235);
        assert_eq!(js.symbol, "U");
        assert!(js.notation.contains("235"));
    }

    #[test]
    fn event_to_step_start() {
        let sim = test_sim();
        let n = Nuclide::try_new(6, 8).unwrap();
        let step = event_to_step(0, &SimEvent::Start { nuclide: n }, &sim);
        assert_eq!(step.index, 0);
        assert_eq!(step.event_type, "start");
        assert_eq!(step.nuclide.z, 6);
        assert_eq!(step.nuclide.n, 8);
        assert!(step.description.contains("C"));
        assert!(step.detail.is_none());
        assert!(!step.nuclide_is_stable);
        assert!(step.nuclide_in_database);
        assert!(step.nuclide_half_life_s.is_some());
    }

    #[test]
    fn event_to_step_neutron_absorbed_slow_and_fast() {
        let sim = test_sim();
        let target = Nuclide::try_new(92, 143).unwrap();
        let compound = Nuclide::try_new(92, 144).unwrap();
        for (energy, label) in [
            (NeutronEnergy::Slow, "slow"),
            (NeutronEnergy::Fast, "fast"),
        ] {
            let step = event_to_step(
                1,
                &SimEvent::NeutronAbsorbed {
                    target,
                    energy,
                    compound,
                },
                &sim,
            );
            assert_eq!(step.event_type, "neutron-absorbed");
            assert_eq!(step.nuclide, NuclideJs::from_nuclide(&compound));
            let d = step.detail.as_ref().unwrap();
            assert_eq!(d.target.as_ref().unwrap(), &NuclideJs::from_nuclide(&target));
            assert_eq!(d.energy.as_deref(), Some(label));
            assert!(d.light_fragment.is_none() && d.heavy_fragment.is_none());
        }
    }

    #[test]
    fn event_to_step_fission() {
        let sim = test_sim();
        let parent = Nuclide::try_new(92, 143).unwrap();
        let light = Nuclide::try_new(56, 85).unwrap();
        let heavy = Nuclide::try_new(36, 55).unwrap();
        let step = event_to_step(
            2,
            &SimEvent::Fission {
                parent,
                energy: NeutronEnergy::Slow,
                light,
                heavy,
                neutrons_released: 2,
            },
            &sim,
        );
        assert_eq!(step.event_type, "fission");
        assert_eq!(step.nuclide, NuclideJs::from_nuclide(&heavy));
        let d = step.detail.as_ref().unwrap();
        assert_eq!(d.parent.as_ref().unwrap(), &NuclideJs::from_nuclide(&parent));
        assert_eq!(
            d.light_fragment.as_ref().unwrap(),
            &NuclideJs::from_nuclide(&light)
        );
        assert_eq!(
            d.heavy_fragment.as_ref().unwrap(),
            &NuclideJs::from_nuclide(&heavy)
        );
        assert_eq!(d.neutrons_released, Some(2));
    }

    #[test]
    fn event_to_step_decay() {
        let sim = test_sim();
        let parent = Nuclide::try_new(6, 8).unwrap();
        let daughter = Nuclide::try_new(7, 7).unwrap();
        let step = event_to_step(
            1,
            &SimEvent::Decay {
                parent,
                mode: DecayMode::BetaMinus,
                daughter,
            },
            &sim,
        );
        assert_eq!(step.event_type, "decay");
        assert_eq!(step.nuclide, NuclideJs::from_nuclide(&daughter));
        let d = step.detail.as_ref().unwrap();
        assert_eq!(d.parent.as_ref().unwrap(), &NuclideJs::from_nuclide(&parent));
        assert_eq!(d.decay_mode.as_deref(), Some("beta-minus"));
    }

    #[test]
    fn event_to_step_stable() {
        let sim = test_sim();
        let n = Nuclide::try_new(7, 7).unwrap();
        let step = event_to_step(3, &SimEvent::Stable { nuclide: n }, &sim);
        assert_eq!(step.event_type, "stable");
        assert_eq!(step.nuclide, NuclideJs::from_nuclide(&n));
        assert!(step.detail.is_none());
        assert!(step.nuclide_is_stable);
        assert!(step.nuclide_in_database);
        assert!(step.nuclide_half_life_s.is_none());
    }

    #[test]
    fn unknown_nuclide_not_in_database() {
        let sim = test_sim();
        // H-51 won't be in the database
        let n = Nuclide::try_new(1, 50).unwrap();
        let step = event_to_step(0, &SimEvent::Start { nuclide: n }, &sim);
        assert!(!step.nuclide_in_database);
        assert!(!step.nuclide_is_stable);
        assert!(step.nuclide_half_life_s.is_none());
    }

    #[test]
    fn presets_serde_shape() {
        let v = serde_json::to_value(presets()).unwrap();
        let arr = v.as_array().unwrap();
        assert!(!arr.is_empty());
        let first = arr[0].as_object().unwrap();
        assert!(first.contains_key("z"));
        assert!(first.contains_key("n"));
        assert!(first.contains_key("notation"));
        assert!(first.contains_key("label"));
    }

    #[test]
    fn parse_neutron_energy_accepts_slow_and_fast() {
        assert_eq!(parse_neutron_energy("slow").unwrap(), NeutronEnergy::Slow);
        assert_eq!(parse_neutron_energy("fast").unwrap(), NeutronEnergy::Fast);
    }

    #[test]
    fn parse_neutron_energy_rejects_other_strings() {
        assert_eq!(
            parse_neutron_energy("thermal").unwrap_err(),
            "energy must be 'slow' or 'fast'"
        );
    }

    #[test]
    fn parse_fission_fragment_accepts_light_and_heavy() {
        assert!(parse_fission_fragment("light").unwrap());
        assert!(!parse_fission_fragment("heavy").unwrap());
    }

    #[test]
    fn parse_fission_fragment_rejects_other_strings() {
        assert_eq!(
            parse_fission_fragment("medium").unwrap_err(),
            "fragment must be 'light' or 'heavy'"
        );
    }

    #[test]
    fn sim_state_json_surface_after_set_isotope() {
        let mut s = SimSession::new();
        s.set_isotope(6, 8).unwrap();
        let v = serde_json::to_value(s.state_snapshot()).unwrap();
        assert_eq!(v["cursor"], 0);
        assert_eq!(v["step_count"], 1);
        assert_eq!(v["is_complete"], false);
        assert_eq!(v["following_heavy"], true);
        assert!(v.get("can_decay").is_some());
        assert!(v.get("can_fire").is_some());
        assert!(v.get("has_fission_branch").is_some());
        let step = v["current_step"].as_object().unwrap();
        assert_eq!(step["event_type"], "start");
        assert!(step.get("nuclide").is_some());
        assert_eq!(step["nuclide_is_stable"], false);
        assert_eq!(step["nuclide_in_database"], true);
        assert!(step.get("nuclide_half_life_s").is_some());
    }

    #[test]
    fn all_steps_matches_step_count() {
        let mut s = SimSession::new();
        s.set_isotope(6, 8).unwrap();
        let steps = s.steps_snapshot();
        let state = s.state_snapshot();
        assert_eq!(steps.len(), state.step_count);
    }

    #[test]
    fn lookup_c14_returns_decay_modes_and_notation() {
        let s = SimSession::new();
        let data = s.lookup_data(6, 8).expect("C-14 in stub db");
        assert_eq!(data.notation, "C-14");
        assert!(!data.is_stable);
        assert!(data.decay_modes.contains(&"beta-minus".to_string()));
    }

    #[test]
    fn lookup_invalid_z_returns_none() {
        let s = SimSession::new();
        assert!(s.lookup_data(0, 1).is_none());
    }
}
