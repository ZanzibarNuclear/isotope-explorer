/// <reference types="vite/client" />

declare module "@wasm/nuclear_sim_wasm.js" {
  export class NuclideInfo {
    free(): void;
    [Symbol.dispose](): void;
    constructor(z: number, n: number);
    static uranium_235(): NuclideInfo;
    readonly a: number;
    readonly elementSymbol: string;
    readonly n: number;
    readonly notation: string;
    readonly z: number;
  }
  export function sim_version(): string;
}
