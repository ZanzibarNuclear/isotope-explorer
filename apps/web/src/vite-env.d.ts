/// <reference types="vite/client" />

declare module "@wasm/nuclear_sim_wasm.js" {
  export class SimSession {
    free(): void;
    [Symbol.dispose](): void;
    constructor();
    presets(): Preset[];
    set_isotope(z: number, n: number): void;
    fire_neutron(energy: "slow" | "fast"): void;
    induce_decay(): void;
    switch_branch(fragment: "light" | "heavy"): void;
    all_nuclide_keys(): { z: number; n: number }[];
    /** Auto-follow decay preview for a fragment (parallel card columns). */
    decay_chain_preview(z: number, n: number): StepInfo[];
    step_forward(): void;
    step_back(): void;
    go_to_step(index: number): void;
    state(): SimState;
    all_steps(): StepInfo[];
    lookup(z: number, n: number): NuclideData | null;
  }
  export function sim_version(): string;

  export interface Preset {
    z: number;
    n: number;
    notation: string;
    label: string;
  }

  export interface NuclideInfo {
    z: number;
    n: number;
    a: number;
    symbol: string;
    notation: string;
  }

  export interface StepDetail {
    target?: NuclideInfo;
    energy?: string;
    light_fragment?: NuclideInfo;
    heavy_fragment?: NuclideInfo;
    neutrons_released?: number;
    decay_mode?: string;
    parent?: NuclideInfo;
  }

  export interface StepInfo {
    index: number;
    event_type: string;
    description: string;
    nuclide: NuclideInfo;
    /** Stable nuclide in DB → UI shows ∞ in the chain meta column (except decay rows). */
    nuclide_is_stable: boolean;
    /** Whether this nuclide exists in the simulation database. */
    nuclide_in_database: boolean;
    /** Half-life in seconds when radioactive; omitted when stable. */
    nuclide_half_life_s?: number | null;
    detail?: StepDetail;
  }

  export interface SimState {
    cursor: number;
    step_count: number;
    is_complete: boolean;
    can_decay: boolean;
    can_fire: boolean;
    current_step: StepInfo;
    has_fission_branch: boolean;
    following_heavy: boolean;
  }

  export interface NuclideData {
    notation: string;
    is_stable: boolean;
    is_fissile: boolean;
    half_life_s: number | null;
    decay_modes: string[];
  }
}
