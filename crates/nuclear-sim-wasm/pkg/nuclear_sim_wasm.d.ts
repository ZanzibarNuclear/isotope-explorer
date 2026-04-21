/* tslint:disable */
/* eslint-disable */

export class SimSession {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Return all (Z, N) pairs in the database as [{z, n}].
     */
    all_nuclide_keys(): any;
    /**
     * Get all steps as an array (for the chain summary).
     */
    all_steps(): any;
    /**
     * Auto-follow decay chain from a nuclide (same rules as after fission), for parallel UI legs.
     */
    decay_chain_preview(z: number, n: number): any;
    /**
     * Fire a neutron and auto-follow the full decay chain. energy: "slow" or "fast".
     */
    fire_neutron(energy: string): void;
    /**
     * Fire a neutron — adds only the immediate event, no auto-chain. energy: "slow" or "fast".
     */
    fire_neutron_step(energy: string): void;
    /**
     * Return both fragment chains for the first fission branch.
     *
     * Step indices are set to fission_step+1+offset so they match what Rust's cursor
     * would report when that chain is the active one — enabling correct card highlighting.
     * Returns null if there is no fission branch.
     */
    fission_tails(): any;
    /**
     * Jump to a specific step index.
     */
    go_to_step(index: number): void;
    /**
     * Induce a single decay step. Returns error if stable.
     */
    induce_decay(): void;
    /**
     * Induce decay and auto-follow the full chain to stability. Returns error if stable.
     */
    induce_decay_chain(): void;
    /**
     * Look up data for a nuclide. Returns null if unknown.
     */
    lookup(z: number, n: number): any;
    constructor();
    /**
     * Get available preset isotopes.
     */
    presets(): any;
    /**
     * Set the starting isotope by Z and N.
     */
    set_isotope(z: number, n: number): void;
    /**
     * Get the full simulation state (current step, cursor, etc.).
     */
    state(): any;
    /**
     * Move cursor backward one step.
     */
    step_back(): void;
    /**
     * Move cursor forward one step.
     */
    step_forward(): void;
    /**
     * Switch to following the light or heavy fission fragment, auto-chaining to stability.
     * fragment: "light" or "heavy"
     */
    switch_branch(fragment: string): void;
    /**
     * Switch fragment without auto-chaining (step-by-step mode).
     * fragment: "light" or "heavy"
     */
    switch_branch_step(fragment: string): void;
}

export function sim_version(): string;
