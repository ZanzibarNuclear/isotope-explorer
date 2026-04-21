export class SimSession {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        SimSessionFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_simsession_free(ptr, 0);
    }
    /**
     * Return all (Z, N) pairs in the database as [{z, n}].
     * @returns {any}
     */
    all_nuclide_keys() {
        const ret = wasm.simsession_all_nuclide_keys(this.__wbg_ptr);
        return ret;
    }
    /**
     * Get all steps as an array (for the chain summary).
     * @returns {any}
     */
    all_steps() {
        const ret = wasm.simsession_all_steps(this.__wbg_ptr);
        return ret;
    }
    /**
     * Auto-follow decay chain from a nuclide (same rules as after fission), for parallel UI legs.
     * @param {number} z
     * @param {number} n
     * @returns {any}
     */
    decay_chain_preview(z, n) {
        const ret = wasm.simsession_decay_chain_preview(this.__wbg_ptr, z, n);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return takeFromExternrefTable0(ret[0]);
    }
    /**
     * Fire a neutron and auto-follow the full decay chain. energy: "slow" or "fast".
     * @param {string} energy
     */
    fire_neutron(energy) {
        const ptr0 = passStringToWasm0(energy, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.simsession_fire_neutron(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Fire a neutron — adds only the immediate event, no auto-chain. energy: "slow" or "fast".
     * @param {string} energy
     */
    fire_neutron_step(energy) {
        const ptr0 = passStringToWasm0(energy, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.simsession_fire_neutron_step(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Return both fragment chains for the first fission branch.
     *
     * Step indices are set to fission_step+1+offset so they match what Rust's cursor
     * would report when that chain is the active one — enabling correct card highlighting.
     * Returns null if there is no fission branch.
     * @returns {any}
     */
    fission_tails() {
        const ret = wasm.simsession_fission_tails(this.__wbg_ptr);
        return ret;
    }
    /**
     * Jump to a specific step index.
     * @param {number} index
     */
    go_to_step(index) {
        const ret = wasm.simsession_go_to_step(this.__wbg_ptr, index);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Induce a single decay step. Returns error if stable.
     */
    induce_decay() {
        const ret = wasm.simsession_induce_decay(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Induce decay and auto-follow the full chain to stability. Returns error if stable.
     */
    induce_decay_chain() {
        const ret = wasm.simsession_induce_decay_chain(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Look up data for a nuclide. Returns null if unknown.
     * @param {number} z
     * @param {number} n
     * @returns {any}
     */
    lookup(z, n) {
        const ret = wasm.simsession_lookup(this.__wbg_ptr, z, n);
        return ret;
    }
    constructor() {
        const ret = wasm.simsession_new();
        this.__wbg_ptr = ret >>> 0;
        SimSessionFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * Get available preset isotopes.
     * @returns {any}
     */
    presets() {
        const ret = wasm.simsession_presets(this.__wbg_ptr);
        return ret;
    }
    /**
     * Set the starting isotope by Z and N.
     * @param {number} z
     * @param {number} n
     */
    set_isotope(z, n) {
        const ret = wasm.simsession_set_isotope(this.__wbg_ptr, z, n);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Get the full simulation state (current step, cursor, etc.).
     * @returns {any}
     */
    state() {
        const ret = wasm.simsession_state(this.__wbg_ptr);
        return ret;
    }
    /**
     * Move cursor backward one step.
     */
    step_back() {
        const ret = wasm.simsession_step_back(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Move cursor forward one step.
     */
    step_forward() {
        const ret = wasm.simsession_step_forward(this.__wbg_ptr);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Switch to following the light or heavy fission fragment, auto-chaining to stability.
     * fragment: "light" or "heavy"
     * @param {string} fragment
     */
    switch_branch(fragment) {
        const ptr0 = passStringToWasm0(fragment, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.simsession_switch_branch(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * Switch fragment without auto-chaining (step-by-step mode).
     * fragment: "light" or "heavy"
     * @param {string} fragment
     */
    switch_branch_step(fragment) {
        const ptr0 = passStringToWasm0(fragment, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.simsession_switch_branch_step(this.__wbg_ptr, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
}
if (Symbol.dispose) SimSession.prototype[Symbol.dispose] = SimSession.prototype.free;

/**
 * @returns {string}
 */
export function sim_version() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.sim_version();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}
export function __wbg_Error_55538483de6e3abe(arg0, arg1) {
    const ret = Error(getStringFromWasm0(arg0, arg1));
    return ret;
}
export function __wbg___wbindgen_debug_string_d89627202d0155b7(arg0, arg1) {
    const ret = debugString(arg1);
    const ptr1 = passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
}
export function __wbg___wbindgen_throw_5549492daedad139(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
}
export function __wbg_new_4370be21fa2b2f80() {
    const ret = new Array();
    return ret;
}
export function __wbg_new_48e1d86cfd30c8e7() {
    const ret = new Object();
    return ret;
}
export function __wbg_set_4702dfa37c77f492(arg0, arg1, arg2) {
    arg0[arg1 >>> 0] = arg2;
}
export function __wbg_set_6be42768c690e380(arg0, arg1, arg2) {
    arg0[arg1] = arg2;
}
export function __wbindgen_cast_0000000000000001(arg0) {
    // Cast intrinsic for `F64 -> Externref`.
    const ret = arg0;
    return ret;
}
export function __wbindgen_cast_0000000000000002(arg0, arg1) {
    // Cast intrinsic for `Ref(String) -> Externref`.
    const ret = getStringFromWasm0(arg0, arg1);
    return ret;
}
export function __wbindgen_cast_0000000000000003(arg0) {
    // Cast intrinsic for `U64 -> Externref`.
    const ret = BigInt.asUintN(64, arg0);
    return ret;
}
export function __wbindgen_init_externref_table() {
    const table = wasm.__wbindgen_externrefs;
    const offset = table.grow(4);
    table.set(0, undefined);
    table.set(offset + 0, undefined);
    table.set(offset + 1, null);
    table.set(offset + 2, true);
    table.set(offset + 3, false);
}
const SimSessionFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_simsession_free(ptr >>> 0, 1));

function debugString(val) {
    // primitive types
    const type = typeof val;
    if (type == 'number' || type == 'boolean' || val == null) {
        return  `${val}`;
    }
    if (type == 'string') {
        return `"${val}"`;
    }
    if (type == 'symbol') {
        const description = val.description;
        if (description == null) {
            return 'Symbol';
        } else {
            return `Symbol(${description})`;
        }
    }
    if (type == 'function') {
        const name = val.name;
        if (typeof name == 'string' && name.length > 0) {
            return `Function(${name})`;
        } else {
            return 'Function';
        }
    }
    // objects
    if (Array.isArray(val)) {
        const length = val.length;
        let debug = '[';
        if (length > 0) {
            debug += debugString(val[0]);
        }
        for(let i = 1; i < length; i++) {
            debug += ', ' + debugString(val[i]);
        }
        debug += ']';
        return debug;
    }
    // Test for built-in
    const builtInMatches = /\[object ([^\]]+)\]/.exec(toString.call(val));
    let className;
    if (builtInMatches && builtInMatches.length > 1) {
        className = builtInMatches[1];
    } else {
        // Failed to match the standard '[object ClassName]'
        return toString.call(val);
    }
    if (className == 'Object') {
        // we're a user defined class or Object
        // JSON.stringify avoids problems with cycles, and is generally much
        // easier than looping through ownProperties of `val`.
        try {
            return 'Object(' + JSON.stringify(val) + ')';
        } catch (_) {
            return 'Object';
        }
    }
    // errors
    if (val instanceof Error) {
        return `${val.name}: ${val.message}\n${val.stack}`;
    }
    // TODO we could test for more things here, like `Set`s and `Map`s.
    return className;
}

let cachedDataViewMemory0 = null;
function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return decodeText(ptr, len);
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;


let wasm;
export function __wbg_set_wasm(val) {
    wasm = val;
}
