/* @ts-self-types="./nuclear_sim_wasm.d.ts" */

import * as wasm from "./nuclear_sim_wasm_bg.wasm";
import { __wbg_set_wasm } from "./nuclear_sim_wasm_bg.js";
__wbg_set_wasm(wasm);
wasm.__wbindgen_start();
export {
    SimSession, sim_version
} from "./nuclear_sim_wasm_bg.js";
