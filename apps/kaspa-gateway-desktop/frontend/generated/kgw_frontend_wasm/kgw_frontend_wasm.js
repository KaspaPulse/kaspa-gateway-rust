let wasm;

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_export_2.set(idx, obj);
    return idx;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

const cachedTextDecoder = (typeof TextDecoder !== 'undefined' ? new TextDecoder('utf-8', { ignoreBOM: true, fatal: true }) : { decode: () => { throw Error('TextDecoder not available') } } );

if (typeof TextDecoder !== 'undefined') { cachedTextDecoder.decode(); };

let cachedUint8ArrayMemory0 = null;

function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

const CLOSURE_DTORS = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(state => {
    wasm.__wbindgen_export_3.get(state.dtor)(state.a, state.b)
});

function makeMutClosure(arg0, arg1, dtor, f) {
    const state = { a: arg0, b: arg1, cnt: 1, dtor };
    const real = (...args) => {
        // First up with a closure we increment the internal reference
        // count. This ensures that the Rust closure environment won't
        // be deallocated while we're invoking it.
        state.cnt++;
        const a = state.a;
        state.a = 0;
        try {
            return f(a, state.b, ...args);
        } finally {
            if (--state.cnt === 0) {
                wasm.__wbindgen_export_3.get(state.dtor)(a, state.b);
                CLOSURE_DTORS.unregister(state);
            } else {
                state.a = a;
            }
        }
    };
    real.original = state;
    CLOSURE_DTORS.register(real, state, state);
    return real;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let WASM_VECTOR_LEN = 0;

const cachedTextEncoder = (typeof TextEncoder !== 'undefined' ? new TextEncoder('utf-8') : { encode: () => { throw Error('TextEncoder not available') } } );

const encodeString = (typeof cachedTextEncoder.encodeInto === 'function'
    ? function (arg, view) {
    return cachedTextEncoder.encodeInto(arg, view);
}
    : function (arg, view) {
    const buf = cachedTextEncoder.encode(arg);
    view.set(buf);
    return {
        read: arg.length,
        written: buf.length
    };
});

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
        const ret = encodeString(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}
/**
 * @returns {any}
 */
export function explorerEnsureState() {
    const ret = wasm.explorerEnsureState();
    return ret;
}

/**
 * @param {string} value
 * @returns {string}
 */
export function explorerNormalizeTxTypeFilterValue(value) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.explorerNormalizeTxTypeFilterValue(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} value
 * @returns {string}
 */
export function explorerNormalizeDirectionFilterValue(value) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.explorerNormalizeDirectionFilterValue(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {any} section
 * @returns {any}
 */
export function explorerRepairFilterSelects(section) {
    const ret = wasm.explorerRepairFilterSelects(section);
    return ret;
}

export function explorerInstallFilterSelectRepair() {
    wasm.explorerInstallFilterSelectRepair();
}

/**
 * @param {any} section
 * @param {any} address
 * @param {any} start_ts
 * @param {any} end_ts
 * @param {any} limit
 * @returns {any}
 */
export function explorerClean2Request(section, address, start_ts, end_ts, limit) {
    const ret = wasm.explorerClean2Request(section, address, start_ts, end_ts, limit);
    return ret;
}

/**
 * @param {any} section
 * @returns {any}
 */
export function explorerEnsureFilterOptions(section) {
    const ret = wasm.explorerEnsureFilterOptions(section);
    return ret;
}

/**
 * @param {any} section
 * @returns {any}
 */
export function explorerReadFilterState(section) {
    const ret = wasm.explorerReadFilterState(section);
    return ret;
}

/**
 * @param {any} section
 * @param {any} address
 * @param {any} start_ts
 * @param {any} end_ts
 * @param {any} limit
 * @returns {any}
 */
export function explorerBuildListRequest(section, address, start_ts, end_ts, limit) {
    const ret = wasm.explorerBuildListRequest(section, address, start_ts, end_ts, limit);
    return ret;
}

/**
 * @param {string} selector
 * @param {any} section
 * @param {string} fallback
 * @returns {string}
 */
export function explorerFilterValue(selector, section, fallback) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(selector, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(fallback, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.explorerFilterValue(ptr0, len0, section, ptr1, len1);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {any} section
 * @param {any} address
 * @param {any} start_ts
 * @param {any} end_ts
 * @param {any} limit
 * @returns {any}
 */
export function explorerFilterBuildRequest(section, address, start_ts, end_ts, limit) {
    const ret = wasm.explorerFilterBuildRequest(section, address, start_ts, end_ts, limit);
    return ret;
}

export function analysisCalendarInstall() {
    wasm.analysisCalendarInstall();
}

/**
 * @returns {string}
 */
export function analysisCalendarResetToToday() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.analysisCalendarResetToToday();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} section
 * @returns {number}
 */
export function explorerSetTableFontSize(section) {
    const ret = wasm.explorerSetTableFontSize(section);
    return ret >>> 0;
}

/**
 * @param {any} section
 * @param {any} raw_value
 * @returns {number}
 */
export function explorerApplyFontSize(section, raw_value) {
    const ret = wasm.explorerApplyFontSize(section, raw_value);
    return ret >>> 0;
}

/**
 * @param {any} section
 * @returns {boolean}
 */
export function explorerBindFontSpinbox(section) {
    const ret = wasm.explorerBindFontSpinbox(section);
    return ret !== 0;
}

/**
 * @param {any} section
 * @param {boolean} busy
 * @returns {boolean}
 */
export function explorerApplyLocalBusyControls(section, busy) {
    const ret = wasm.explorerApplyLocalBusyControls(section, busy);
    return ret !== 0;
}

/**
 * @returns {boolean}
 */
export function explorerInstallFilterBusyLock() {
    const ret = wasm.explorerInstallFilterBusyLock();
    return ret !== 0;
}

/**
 * @param {string} reason
 */
export function explorerRefreshFilterAvailability(reason) {
    const ptr0 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.explorerRefreshFilterAvailability(ptr0, len0);
}

/**
 * @param {boolean} value
 * @param {string} reason
 */
export function explorerSetFilterBusy(value, reason) {
    const ptr0 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.explorerSetFilterBusy(value, ptr0, len0);
}

/**
 * @param {any} section
 * @returns {string}
 */
export function explorerManualAddressValue(section) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.explorerManualAddressValue(section);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {boolean}
 */
export function explorerIsKaspaAddress(value) {
    const ret = wasm.explorerIsKaspaAddress(value);
    return ret !== 0;
}

/**
 * @param {any} section
 * @returns {Promise<boolean>}
 */
export function explorerSaveManualAddress(section) {
    const ret = wasm.explorerSaveManualAddress(section);
    return ret;
}

/**
 * @returns {boolean}
 */
export function explorerInstallManualAddressSave() {
    const ret = wasm.explorerInstallManualAddressSave();
    return ret !== 0;
}

/**
 * @param {any} section
 * @param {string} message
 * @returns {boolean}
 */
export function explorerForceSetTableMessage(section, message) {
    const ptr0 = passStringToWasm0(message, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.explorerForceSetTableMessage(section, ptr0, len0);
    return ret !== 0;
}

/**
 * @param {any} section
 * @returns {boolean}
 */
export function explorerForceResetDisplayFiltersToAll(section) {
    const ret = wasm.explorerForceResetDisplayFiltersToAll(section);
    return ret !== 0;
}

/**
 * @param {any} section
 * @param {boolean} busy
 * @param {string} mode
 * @returns {boolean}
 */
export function explorerForceSetControlsBusy(section, busy, mode) {
    const ptr0 = passStringToWasm0(mode, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.explorerForceSetControlsBusy(section, busy, ptr0, len0);
    return ret !== 0;
}

/**
 * @returns {boolean}
 */
export function explorerInstallForceBusyBlocker() {
    const ret = wasm.explorerInstallForceBusyBlocker();
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {any} instance
 * @returns {string}
 */
export function bridgeInstancePreviewTextR8B(net, instance) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeInstancePreviewTextR8B(ptr0, len0, instance);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_2.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}
/**
 * @param {string} net
 * @param {any} bridge_instances
 * @param {any} active_instance
 */
export function bridgeSyncInstancePreviewRowsR8B(net, bridge_instances, active_instance) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeSyncInstancePreviewRowsR8B(ptr0, len0, bridge_instances, active_instance);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {string} net
 * @param {any} instance_id
 * @param {string} field_name
 * @returns {string}
 */
export function bridgeReadInstanceField(net, instance_id, field_name) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(field_name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeReadInstanceField(ptr0, len0, instance_id, ptr1, len1);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {string}
 */
export function bridgeInstancePortPlaceholderR49(net) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeInstancePortPlaceholderR49(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {string}
 */
export function bridgeInstancePromPlaceholderR49(net) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeInstancePromPlaceholderR49(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function explorerNormalizeAddress(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.explorerNormalizeAddress(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {Array<any>}
 */
export function explorerAddressLookupKeys(value) {
    const ret = wasm.explorerAddressLookupKeys(value);
    return ret;
}

/**
 * @param {any} value
 * @returns {Promise<string>}
 */
export function explorerCanonicalKaspaAddress(value) {
    const ret = wasm.explorerCanonicalKaspaAddress(value);
    return ret;
}

/**
 * @returns {Promise<any>}
 */
export function explorerLoadKnownAddressNames() {
    const ret = wasm.explorerLoadKnownAddressNames();
    return ret;
}

/**
 * @param {any} address
 * @param {any} name
 * @returns {Promise<any>}
 */
export function explorerSaveAddressToDatabase(address, name) {
    const ret = wasm.explorerSaveAddressToDatabase(address, name);
    return ret;
}

/**
 * @param {any} section
 * @param {any} address
 * @returns {Promise<string>}
 */
export function explorerRefreshAddressName(section, address) {
    const ret = wasm.explorerRefreshAddressName(section, address);
    return ret;
}

/**
 * @param {any} section
 * @returns {Promise<boolean>}
 */
export function explorerLoadSavedAddresses(section) {
    const ret = wasm.explorerLoadSavedAddresses(section);
    return ret;
}

/**
 * @param {any} section
 * @param {any} address
 * @returns {Promise<any>}
 */
export function explorerFetchBalance(section, address) {
    const ret = wasm.explorerFetchBalance(section, address);
    return ret;
}

/**
 * @param {any} address
 * @returns {any}
 */
export function explorerAddressDiagnosticsSnapshot(address) {
    const ret = wasm.explorerAddressDiagnosticsSnapshot(address);
    return ret;
}

/**
 * @returns {Array<any>}
 */
export function bridgeDifficultyPresetValuesR16C() {
    const ret = wasm.bridgeDifficultyPresetValuesR16C();
    return ret;
}

/**
 * @returns {string}
 */
export function bridgeDifficultyDatalistIdR16C() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeDifficultyDatalistIdR16C();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @returns {string}
 */
export function bridgeDifficultyDatalistR16C() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeDifficultyDatalistR16C();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} name
 * @returns {string}
 */
export function bridgeDifficultyInputAttrsR16C(name) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeDifficultyInputAttrsR16C(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} net
 * @param {string} name
 * @param {string} label
 * @param {string} value
 * @param {string} placeholder
 * @param {string} span
 * @param {string} input_attrs
 * @returns {string}
 */
export function bridgeCardInput(net, name, label, value, placeholder, span, input_attrs) {
    let deferred8_0;
    let deferred8_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len3 = WASM_VECTOR_LEN;
        const ptr4 = passStringToWasm0(placeholder, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len4 = WASM_VECTOR_LEN;
        const ptr5 = passStringToWasm0(span, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len5 = WASM_VECTOR_LEN;
        const ptr6 = passStringToWasm0(input_attrs, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len6 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeCardInput(ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3, ptr4, len4, ptr5, len5, ptr6, len6);
        deferred8_0 = ret[0];
        deferred8_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred8_0, deferred8_1, 1);
    }
}

/**
 * @param {string} net
 * @param {string} name
 * @param {string} label
 * @param {Array<any>} options
 * @param {string} value
 * @param {string} span
 * @returns {string}
 */
export function bridgeCardSelect(net, name, label, options, value, span) {
    let deferred6_0;
    let deferred6_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len3 = WASM_VECTOR_LEN;
        const ptr4 = passStringToWasm0(span, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len4 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeCardSelect(ptr0, len0, ptr1, len1, ptr2, len2, options, ptr3, len3, ptr4, len4);
        deferred6_0 = ret[0];
        deferred6_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred6_0, deferred6_1, 1);
    }
}

/**
 * @param {string} net
 * @param {string} name
 * @param {string} label
 * @param {boolean} checked
 * @param {string} span
 * @returns {string}
 */
export function bridgeCardCheck(net, name, label, checked, span) {
    let deferred5_0;
    let deferred5_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(span, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len3 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeCardCheck(ptr0, len0, ptr1, len1, ptr2, len2, checked, ptr3, len3);
        deferred5_0 = ret[0];
        deferred5_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred5_0, deferred5_1, 1);
    }
}

/**
 * @param {any} profile
 * @returns {string}
 */
export function bridgeRenderRuntime(profile) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeRenderRuntime(profile);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} profile
 * @returns {string}
 */
export function bridgeRenderDifficulty(profile) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeRenderDifficulty(profile);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} profile
 * @returns {string}
 */
export function bridgeRenderLogging(profile) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeRenderLogging(profile);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} profile
 * @returns {string}
 */
export function bridgeRenderPorts(profile) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeRenderPorts(profile);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} profile
 * @returns {string}
 */
export function bridgeRenderCpuMiner(profile) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeRenderCpuMiner(profile);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} net
 * @param {string} role
 * @param {string} _instance_id
 * @returns {boolean}
 */
export function bridgeRenderRawLogBuffer(net, role, _instance_id) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(role, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passStringToWasm0(_instance_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len2 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeRenderRawLogBuffer(ptr0, len0, ptr1, len1, ptr2, len2);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {string} role
 * @param {any} report
 * @param {string} _instance_id
 * @returns {number}
 */
export function bridgeApplyRuntimeLogReport(net, role, report, _instance_id) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(role, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passStringToWasm0(_instance_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len2 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeApplyRuntimeLogReport(ptr0, len0, ptr1, len1, report, ptr2, len2);
    return ret >>> 0;
}

/**
 * @param {string} net
 * @param {string} role
 * @param {string} _instance_id
 */
export function bridgeClearRawLogBuffer(net, role, _instance_id) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(role, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passStringToWasm0(_instance_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len2 = WASM_VECTOR_LEN;
    wasm.bridgeClearRawLogBuffer(ptr0, len0, ptr1, len1, ptr2, len2);
}

/**
 * @param {any} paths
 * @param {boolean} force
 */
export function settingsPathsApply(paths, force) {
    wasm.settingsPathsApply(paths, force);
}

/**
 * @param {string} reason
 * @returns {Promise<any>}
 */
export function settingsPathsLoadDefaults(reason) {
    const ptr0 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.settingsPathsLoadDefaults(ptr0, len0);
    return ret;
}

/**
 * @returns {Promise<void>}
 */
export function settingsPathsRepairBeforeSave() {
    const ret = wasm.settingsPathsRepairBeforeSave();
    return ret;
}

/**
 * @param {string} target_id
 * @returns {Promise<boolean>}
 */
export function settingsPathsBrowse(target_id) {
    const ptr0 = passStringToWasm0(target_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.settingsPathsBrowse(ptr0, len0);
    return ret;
}

/**
 * @param {any} result
 * @returns {any}
 */
export function explorerNormalizeUnifiedResult(result) {
    const ret = wasm.explorerNormalizeUnifiedResult(result);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} result
 * @returns {any}
 */
export function explorerDaySummaryRowsFromResult(result) {
    const ret = wasm.explorerDaySummaryRowsFromResult(result);
    return ret;
}

/**
 * @returns {Array<any>}
 */
export function settingsNodeEndpoints() {
    const ret = wasm.settingsNodeEndpoints();
    return ret;
}

/**
 * @param {any} value
 * @returns {boolean}
 */
export function settingsIsKaspaAddress(value) {
    const ret = wasm.settingsIsKaspaAddress(value);
    return ret !== 0;
}

/**
 * @param {any} record
 * @returns {any}
 */
export function settingsAddressNormalize(record) {
    const ret = wasm.settingsAddressNormalize(record);
    return ret;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function settingsDbKindFromFileName(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.settingsDbKindFromFileName(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function settingsExplorerAddress(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.settingsExplorerAddress(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function settingsExplorerUrl(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.settingsExplorerUrl(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function settingsToWesternDigits(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.settingsToWesternDigits(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} checks
 * @param {Array<any>} known_entries
 * @returns {any}
 */
export function settingsDisplayChecksWithDefaults(checks, known_entries) {
    const ret = wasm.settingsDisplayChecksWithDefaults(checks, known_entries);
    return ret;
}

/**
 * @param {any} checks
 * @param {string} prefix
 * @returns {Array<any>}
 */
export function settingsSelectedDisplayKeys(checks, prefix) {
    const ptr0 = passStringToWasm0(prefix, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.settingsSelectedDisplayKeys(checks, ptr0, len0);
    return ret;
}

/**
 * @param {any} checks
 * @returns {boolean}
 */
export function settingsDisplayStateMissingContract(checks) {
    const ret = wasm.settingsDisplayStateMissingContract(checks);
    return ret !== 0;
}

/**
 * @param {any} checks
 * @returns {any}
 */
export function settingsDisplayPreferences(checks) {
    const ret = wasm.settingsDisplayPreferences(checks);
    return ret;
}

/**
 * @returns {object}
 */
export function settingsNodeManaged() {
    const ret = wasm.settingsNodeManaged();
    return ret;
}

/**
 * @returns {object}
 */
export function settingsNodeRequired() {
    const ret = wasm.settingsNodeRequired();
    return ret;
}

/**
 * @returns {Array<any>}
 */
export function settingsNodeOptional() {
    const ret = wasm.settingsNodeOptional();
    return ret;
}

/**
 * @param {any} host
 * @returns {boolean}
 */
export function settingsIsLoopback(host) {
    const ret = wasm.settingsIsLoopback(host);
    return ret !== 0;
}

/**
 * @param {any} a
 * @param {any} b
 * @returns {boolean}
 */
export function settingsListenersOverlap(a, b) {
    const ret = wasm.settingsListenersOverlap(a, b);
    return ret !== 0;
}

/**
 * @param {string} name
 * @param {any} values
 * @param {any} options
 * @returns {boolean}
 */
export function settingsNodeFieldEnabled(name, values, options) {
    const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.settingsNodeFieldEnabled(ptr0, len0, values, options);
    return ret !== 0;
}

/**
 * @param {any} values
 * @param {any} options
 * @param {string} network
 * @returns {any}
 */
export function settingsValidateNodeForm(values, options, network) {
    const ptr0 = passStringToWasm0(network, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.settingsValidateNodeForm(values, options, ptr0, len0);
    return ret;
}

/**
 * @param {string} name
 * @param {any} values
 * @param {any} options
 * @returns {boolean}
 */
export function settingsBridgeFieldEnabled(name, values, options) {
    const ptr0 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.settingsBridgeFieldEnabled(ptr0, len0, values, options);
    return ret !== 0;
}

/**
 * @param {any} values
 * @param {any} options
 * @param {string} network
 * @returns {any}
 */
export function settingsValidateBridgeForm(values, options, network) {
    const ptr0 = passStringToWasm0(network, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.settingsValidateBridgeForm(values, options, ptr0, len0);
    return ret;
}

/**
 * @param {any} root
 * @param {string} prefix
 * @param {any} errors
 */
export function settingsRenderFieldErrors(root, prefix, errors) {
    const ptr0 = passStringToWasm0(prefix, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.settingsRenderFieldErrors(root, ptr0, len0, errors);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {any} message
 * @returns {Promise<boolean>}
 */
export function settingsConfirmUserAction(message) {
    const ret = wasm.settingsConfirmUserAction(message);
    return ret;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function settingsAddressShort(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.settingsAddressShort(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @returns {object}
 */
export function settingsNodeDangerous() {
    const ret = wasm.settingsNodeDangerous();
    return ret;
}

/**
 * @returns {object}
 */
export function settingsBridgeManaged() {
    const ret = wasm.settingsBridgeManaged();
    return ret;
}

/**
 * @returns {Array<any>}
 */
export function settingsBridgeRequired() {
    const ret = wasm.settingsBridgeRequired();
    return ret;
}

/**
 * @returns {Array<any>}
 */
export function settingsBridgeOptional() {
    const ret = wasm.settingsBridgeOptional();
    return ret;
}

/**
 * @param {any} value
 * @returns {boolean}
 */
export function settingsIsHost(value) {
    const ret = wasm.settingsIsHost(value);
    return ret !== 0;
}

/**
 * @param {any} value
 * @returns {boolean}
 */
export function settingsIsPort(value) {
    const ret = wasm.settingsIsPort(value);
    return ret !== 0;
}

/**
 * @param {any} host
 * @param {any} port
 * @returns {string}
 */
export function settingsEndpoint(host, port) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.settingsEndpoint(host, port);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {any}
 */
export function settingsSplitEndpoint(value) {
    const ret = wasm.settingsSplitEndpoint(value);
    return ret;
}

/**
 * @param {string} address
 */
export function explorerLiveCoreReset(address) {
    const ptr0 = passStringToWasm0(address, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.explorerLiveCoreReset(ptr0, len0);
}

/**
 * @returns {string}
 */
export function explorerLiveCoreAddress() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.explorerLiveCoreAddress();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} address
 * @param {any} rows
 */
export function explorerLiveCoreSeedRows(address, rows) {
    const ptr0 = passStringToWasm0(address, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.explorerLiveCoreSeedRows(ptr0, len0, rows);
}

/**
 * @param {any} records
 */
export function explorerLiveCoreMergeRecords(records) {
    wasm.explorerLiveCoreMergeRecords(records);
}

/**
 * @param {any} days
 */
export function explorerLiveCoreMergeDays(days) {
    wasm.explorerLiveCoreMergeDays(days);
}

/**
 * @returns {any}
 */
export function explorerLiveCoreRows() {
    const ret = wasm.explorerLiveCoreRows();
    return ret;
}

/**
 * @param {any} payload
 * @returns {boolean}
 */
export function explorerLiveCoreShouldRender(payload) {
    const ret = wasm.explorerLiveCoreShouldRender(payload);
    return ret !== 0;
}

/**
 * @param {string} action
 * @returns {string}
 */
export function bridgeRuntimeCommandForAction(action) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(action, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeRuntimeCommandForAction(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} action
 * @param {any} value
 * @returns {any}
 */
export function bridgeRuntimeActionOutcome(action, value) {
    const ptr0 = passStringToWasm0(action, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeRuntimeActionOutcome(ptr0, len0, value);
    return ret;
}

/**
 * @param {any} fields
 * @param {string} ui_mode
 * @param {string} preview
 * @returns {boolean}
 */
export function bridgeStartWasInprocessR65F(fields, ui_mode, preview) {
    const ptr0 = passStringToWasm0(ui_mode, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(preview, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeStartWasInprocessR65F(fields, ptr0, len0, ptr1, len1);
    return ret !== 0;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function bridgeStringifyRuntimeResult(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeStringifyRuntimeResult(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} error
 * @returns {string}
 */
export function bridgeNormalizeRuntimeError(error) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeNormalizeRuntimeError(error);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {any}
 */
export function bridgeParseRuntimeKeyValueResponse(value) {
    const ret = wasm.bridgeParseRuntimeKeyValueResponse(value);
    return ret;
}

/**
 * @param {any} value
 * @returns {boolean}
 */
export function bridgeV7RuntimeRunningFromText(value) {
    const ret = wasm.bridgeV7RuntimeRunningFromText(value);
    return ret !== 0;
}

/**
 * @param {any} value
 * @returns {boolean}
 */
export function bridgeR51IsRunning(value) {
    const ret = wasm.bridgeR51IsRunning(value);
    return ret !== 0;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function bridgeRuntimeErrorFromStatus(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeRuntimeErrorFromStatus(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} value
 * @returns {string}
 */
export function bridgeNormalizeNodeModeR65F(value) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeNormalizeNodeModeR65F(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} value
 * @returns {boolean}
 */
export function bridgePreviewDeclaresInprocessR65F(value) {
    const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgePreviewDeclaresInprocessR65F(ptr0, len0);
    return ret !== 0;
}

/**
 * @returns {any}
 */
export function settingsStateCollect() {
    const ret = wasm.settingsStateCollect();
    return ret;
}

/**
 * @param {any} state
 */
export function settingsStateApply(state) {
    wasm.settingsStateApply(state);
}

/**
 * @param {string} tab
 */
export function settingsStateActivateOuter(tab) {
    const ptr0 = passStringToWasm0(tab, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.settingsStateActivateOuter(ptr0, len0);
}

/**
 * @param {string} tab
 */
export function settingsStateActivateInner(tab) {
    const ptr0 = passStringToWasm0(tab, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.settingsStateActivateInner(ptr0, len0);
}

/**
 * @param {any} checks
 * @param {string} prefix
 * @returns {Array<any>}
 */
export function settingsDisplaySelectedKeys(checks, prefix) {
    const ptr0 = passStringToWasm0(prefix, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.settingsDisplaySelectedKeys(checks, ptr0, len0);
    return ret;
}

export function settingsStateCombineUrl() {
    wasm.settingsStateCombineUrl();
}

/**
 * @param {string} _reason
 * @returns {Array<any>}
 */
export function settingsDisplayEnsureDefaults(_reason) {
    const ptr0 = passStringToWasm0(_reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.settingsDisplayEnsureDefaults(ptr0, len0);
    return ret;
}

/**
 * @returns {boolean}
 */
export function settingsDisplayValidateForSave() {
    const ret = wasm.settingsDisplayValidateForSave();
    return ret !== 0;
}

export function settingsDisplayBindMinimumGuards() {
    wasm.settingsDisplayBindMinimumGuards();
}

/**
 * @param {string} reason
 * @param {boolean} persist
 * @returns {any}
 */
export function settingsDisplayBuildCanonicalDefaultState(reason, persist) {
    const ptr0 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.settingsDisplayBuildCanonicalDefaultState(ptr0, len0, persist);
    return ret;
}

/**
 * @param {any} state
 * @param {string} reason
 * @returns {any}
 */
export function settingsDisplayReapplyState(state, reason) {
    const ptr0 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.settingsDisplayReapplyState(state, ptr0, len0);
    return ret;
}

/**
 * @param {any} state
 * @returns {boolean}
 */
export function settingsDisplayStateLooksLegacyAllSelected(state) {
    const ret = wasm.settingsDisplayStateLooksLegacyAllSelected(state);
    return ret !== 0;
}

/**
 * @param {any} state
 * @param {string} reason
 * @returns {any}
 */
export function settingsDisplayApplyShellFromState(state, reason) {
    const ptr0 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.settingsDisplayApplyShellFromState(state, ptr0, len0);
    return ret;
}

/**
 * @param {string} net
 * @param {Array<any>} instances
 * @returns {Array<any>}
 */
export function bridgeBuildCommandLines(net, instances) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeBuildCommandLines(ptr0, len0, instances);
    return ret;
}

/**
 * @param {string} key
 * @returns {string}
 */
export function bridgeAutoFixTextUiR54D3(key) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeAutoFixTextUiR54D3(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {any} root
 */
export function bridgeAutofixButtonInitialLabelUiR111G(root) {
    wasm.bridgeAutofixButtonInitialLabelUiR111G(root);
}

/**
 * @param {string} net
 * @param {string} reason
 * @param {any} bridge_instances
 * @param {any} active_instance
 * @returns {any}
 */
export function bridgeValidateAndApplyPortConflictStateUiR33(net, reason, bridge_instances, active_instance) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeValidateAndApplyPortConflictStateUiR33(ptr0, len0, ptr1, len1, bridge_instances, active_instance);
    return ret;
}

/**
 * @param {string} reason
 * @param {any} bridge_instances
 * @param {any} active_instance
 * @returns {any}
 */
export function bridgeValidateAllPortConflictStatesUiR33(reason, bridge_instances, active_instance) {
    const ptr0 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeValidateAllPortConflictStatesUiR33(ptr0, len0, bridge_instances, active_instance);
    return ret;
}

/**
 * @param {string} net
 * @param {string} reason
 * @param {any} bridge_instances
 * @param {any} active_instance
 */
export function bridgeSchedulePortConflictValidationUiR33(net, reason, bridge_instances, active_instance) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeSchedulePortConflictValidationUiR33(ptr0, len0, ptr1, len1, bridge_instances, active_instance);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {string} net
 * @param {string} phase
 * @param {any} details
 */
export function bridgeTracePortAutofixUiR37(net, phase, details) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(phase, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    wasm.bridgeTracePortAutofixUiR37(ptr0, len0, ptr1, len1, details);
}

/**
 * @returns {Array<any>}
 */
export function bridgeAutofixButtonsUiR37() {
    const ret = wasm.bridgeAutofixButtonsUiR37();
    return ret;
}

/**
 * @param {string} _reason
 * @param {any} bridge_instances
 * @param {any} active_instance
 */
export function bridgeRefreshPortAutofixButtonsUiR37(_reason, bridge_instances, active_instance) {
    const ptr0 = passStringToWasm0(_reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.bridgeRefreshPortAutofixButtonsUiR37(ptr0, len0, bridge_instances, active_instance);
}

/**
 * @param {string} net
 * @param {string} reason
 * @param {any} bridge_instances
 * @param {any} active_instance
 */
export function bridgeSchedulePortAutofixRefreshUiR37(net, reason, bridge_instances, active_instance) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    wasm.bridgeSchedulePortAutofixRefreshUiR37(ptr0, len0, ptr1, len1, bridge_instances, active_instance);
}

/**
 * @param {string} active_net
 * @param {any} bridge_instances
 * @param {any} active_instance
 * @param {any} refresh_instances
 * @param {any} update_command
 * @param {any} runtime_activity
 * @returns {any}
 */
export function bridgeApplyPortAutofixUiR37(active_net, bridge_instances, active_instance, refresh_instances, update_command, runtime_activity) {
    const ptr0 = passStringToWasm0(active_net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeApplyPortAutofixUiR37(ptr0, len0, bridge_instances, active_instance, refresh_instances, update_command, runtime_activity);
    return ret;
}

/**
 * @param {any} root
 * @param {any} bridge_instances
 * @param {any} active_instance
 */
export function bridgeInstallPortAutofixButtonUiR37(root, bridge_instances, active_instance) {
    wasm.bridgeInstallPortAutofixButtonUiR37(root, bridge_instances, active_instance);
}

export function settingsProfilesInstall() {
    wasm.settingsProfilesInstall();
}

/**
 * @param {any} row
 */
export function settingsProfilesSelectEndpoint(row) {
    wasm.settingsProfilesSelectEndpoint(row);
}

/**
 * @returns {Promise<any>}
 */
export function settingsProfilesRefresh() {
    const ret = wasm.settingsProfilesRefresh();
    return ret;
}

/**
 * @returns {Promise<boolean>}
 */
export function settingsPersistenceSave() {
    const ret = wasm.settingsPersistenceSave();
    return ret;
}

/**
 * @param {any} options
 * @returns {Promise<void>}
 */
export function settingsPersistenceResetDefaults(options) {
    const ret = wasm.settingsPersistenceResetDefaults(options);
    return ret;
}

export function settingsPersistenceLoadSaved() {
    wasm.settingsPersistenceLoadSaved();
}

/**
 * @param {string} net
 * @param {boolean} locked
 * @param {any} details
 */
export function bridgeSetOwnedNodeLockR65E(net, locked, details) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.bridgeSetOwnedNodeLockR65E(ptr0, len0, locked, details);
}

/**
 * @param {any} root
 * @returns {boolean}
 */
export function nodeInitKaspaNodeTab(root) {
    const ret = wasm.nodeInitKaspaNodeTab(root);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {boolean} running
 * @param {boolean} bridge_inprocess_locked
 * @param {string} runtime_error
 * @param {string} status_text
 */
export function nodeSetRuntimeButtons(net, running, bridge_inprocess_locked, runtime_error, status_text) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(runtime_error, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passStringToWasm0(status_text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len2 = WASM_VECTOR_LEN;
    wasm.nodeSetRuntimeButtons(ptr0, len0, running, bridge_inprocess_locked, ptr1, len1, ptr2, len2);
}

/**
 * @param {string} net
 * @param {string} _reason
 * @returns {Promise<void>}
 */
export function nodeRefreshOne(net, _reason) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(_reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodeRefreshOne(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * @param {string} action
 * @param {string} net
 * @returns {Promise<boolean>}
 */
export function nodeRunIntegratedAction(action, net) {
    const ptr0 = passStringToWasm0(action, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodeRunIntegratedAction(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * @param {string} net
 * @param {string} value
 */
export function nodeSetRuntimeTransition(net, value) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    wasm.nodeSetRuntimeTransition(ptr0, len0, ptr1, len1);
}

/**
 * @param {string} net
 * @returns {boolean}
 */
export function nodeBridgeOwnedLocked(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeBridgeOwnedLocked(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {boolean} locked
 * @param {string} reason
 */
export function nodeApplyBridgeDisplayOnly(net, locked, reason) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    wasm.nodeApplyBridgeDisplayOnly(ptr0, len0, locked, ptr1, len1);
}

/**
 * @param {any} value
 * @returns {string}
 */
export function toEnglishDigits(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.toEnglishDigits(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {Array<any>} values
 * @returns {any}
 */
export function pick(values) {
    const ret = wasm.pick(values);
    return ret;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function formatUsd(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.formatUsd(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function kgwSummaryFormatKas(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.kgwSummaryFormatKas(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function kgwSummaryFormatUsd(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.kgwSummaryFormatUsd(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function kgwClean2Kas(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.kgwClean2Kas(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function kgwClean2Usd(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.kgwClean2Usd(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} state
 * @returns {string}
 */
export function statusTone(state) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.statusTone(state);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} element
 * @param {any} state
 */
export function applyStatusTone(element, state) {
    const ret = wasm.applyStatusTone(element, state);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {any} element
 * @param {any} text
 */
export function renderStatusSummary(element, text) {
    const ret = wasm.renderStatusSummary(element, text);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @returns {number}
 */
export function parseHeaderUsdPrice() {
    const ret = wasm.parseHeaderUsdPrice();
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ret[0];
}

/**
 * @param {any} value
 * @param {number} fallback
 * @returns {number}
 */
export function toNumber(value, fallback) {
    const ret = wasm.toNumber(value, fallback);
    return ret;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function kgwClean2SafeText(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.kgwClean2SafeText(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function normalizeDateInputValue(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.normalizeDateInputValue(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @param {boolean} end_of_day
 * @returns {any}
 */
export function parseDateSeconds(value, end_of_day) {
    const ret = wasm.parseDateSeconds(value, end_of_day);
    return ret;
}

/**
 * @param {any} value
 * @param {boolean} end_of_day
 * @returns {any}
 */
export function kgwDayToEpochSeconds(value, end_of_day) {
    const ret = wasm.kgwDayToEpochSeconds(value, end_of_day);
    return ret;
}

/**
 * @param {any} value
 * @param {boolean} end_of_day
 * @returns {any}
 */
export function kgwTxDayToEpochSeconds(value, end_of_day) {
    const ret = wasm.kgwTxDayToEpochSeconds(value, end_of_day);
    return ret;
}

/**
 * @param {any} value
 * @param {boolean} end_of_day
 * @returns {any}
 */
export function kgwClean2DayToSeconds(value, end_of_day) {
    const ret = wasm.kgwClean2DayToSeconds(value, end_of_day);
    return ret;
}

/**
 * @param {any} row
 * @returns {string}
 */
export function kgwTransactionDateKey(row) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.kgwTransactionDateKey(row);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function formatKas(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.formatKas(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

export function shellI18nInstall() {
    wasm.shellI18nInstall();
}

/**
 * @returns {Promise<any>}
 */
export function analysisLoadSavedAddresses() {
    const ret = wasm.analysisLoadSavedAddresses();
    return ret;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function analysisNormalizeTimeRange(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.analysisNormalizeTimeRange(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

export function analysisClear() {
    wasm.analysisClear();
}

/**
 * @returns {Promise<any>}
 */
export function analysisRun() {
    const ret = wasm.analysisRun();
    return ret;
}

export function analysisInitTab() {
    const ret = wasm.analysisInitTab();
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

export function analysisInstallBinding() {
    const ret = wasm.analysisInstallBinding();
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {string} net
 * @param {any} structured_reader
 * @param {any} bridge_instances
 * @param {any} active_instance
 * @returns {any}
 */
export function bridgeAssertNoPortConflictsR5(net, structured_reader, bridge_instances, active_instance) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeAssertNoPortConflictsR5(ptr0, len0, structured_reader, bridge_instances, active_instance);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} bridge_instances
 * @param {any} active_instance
 * @param {string} net
 * @param {string} reason
 * @returns {any}
 */
export function bridgeValidateAndApplyPortConflictStateR33(bridge_instances, active_instance, net, reason) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeValidateAndApplyPortConflictStateR33(bridge_instances, active_instance, ptr0, len0, ptr1, len1);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} bridge_instances
 * @param {any} active_instance
 * @param {string} reason
 * @returns {any}
 */
export function bridgeValidateAllPortConflictStatesR33(bridge_instances, active_instance, reason) {
    const ptr0 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeValidateAllPortConflictStatesR33(bridge_instances, active_instance, ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} bridge_instances
 * @param {any} active_instance
 * @param {string} net
 * @param {string} reason
 */
export function bridgeSchedulePortConflictValidationR33(bridge_instances, active_instance, net, reason) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeSchedulePortConflictValidationR33(bridge_instances, active_instance, ptr0, len0, ptr1, len1);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @returns {number}
 */
export function explorerSummaryCurrentUsdPrice() {
    const ret = wasm.explorerSummaryCurrentUsdPrice();
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ret[0];
}

/**
 * @param {any} value
 * @returns {number}
 */
export function explorerSummaryUsdForKas(value) {
    const ret = wasm.explorerSummaryUsdForKas(value);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ret[0];
}

/**
 * @param {any} result
 * @returns {any}
 */
export function explorerNormalizeDaySummaries(result) {
    const ret = wasm.explorerNormalizeDaySummaries(result);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} summary
 * @returns {number}
 */
export function explorerSummaryUsdForSummary(summary) {
    const ret = wasm.explorerSummaryUsdForSummary(summary);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ret[0];
}

export function settingsLayoutInstallSettingsI18nBindings() {
    const ret = wasm.settingsLayoutInstallSettingsI18nBindings();
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

export function settingsLayoutInstallManageAddressesClean() {
    const ret = wasm.settingsLayoutInstallManageAddressesClean();
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @returns {any}
 */
export function settingsLayoutHelpI18nKeys() {
    const ret = wasm.settingsLayoutHelpI18nKeys();
    return ret;
}

/**
 * @returns {Array<any>}
 */
export function settingsLayoutGlobalHelpIds() {
    const ret = wasm.settingsLayoutGlobalHelpIds();
    return ret;
}

/**
 * @param {any} field
 * @returns {string}
 */
export function settingsLayoutFieldKind(field) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.settingsLayoutFieldKind(field);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} scope
 * @param {string} net
 * @param {any} groups
 * @returns {string}
 */
export function settingsLayoutRenderTabs(scope, net, groups) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(scope, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.settingsLayoutRenderTabs(ptr0, len0, ptr1, len1, groups);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {any} root
 */
export function settingsLayoutDecorateFields(root) {
    const ret = wasm.settingsLayoutDecorateFields(root);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {any} field
 * @param {string} message
 */
export function settingsLayoutSetFieldState(field, message) {
    const ptr0 = passStringToWasm0(message, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.settingsLayoutSetFieldState(field, ptr0, len0);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {any} field
 */
export function settingsLayoutRevealField(field) {
    wasm.settingsLayoutRevealField(field);
}

/**
 * @param {any} root
 */
export function settingsLayoutInstall(root) {
    const ret = wasm.settingsLayoutInstall(root);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {any} root
 */
export function settingsLayoutDecorateGlobal(root) {
    const ret = wasm.settingsLayoutDecorateGlobal(root);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {any} root
 */
export function settingsLayoutInstallGlobal(root) {
    const ret = wasm.settingsLayoutInstallGlobal(root);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

export function settingsAddressesInstallManage() {
    const ret = wasm.settingsAddressesInstallManage();
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @returns {Promise<any>}
 */
export function settingsAddressesRefresh() {
    const ret = wasm.settingsAddressesRefresh();
    return ret;
}

/**
 * @param {any} records
 * @param {any} options
 * @returns {Promise<boolean>}
 */
export function settingsAddressesRenderRows(records, options) {
    const ret = wasm.settingsAddressesRenderRows(records, options);
    return ret;
}

export function settingsAddressesInstallExplorerOpen() {
    const ret = wasm.settingsAddressesInstallExplorerOpen();
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

export function settingsAddressesInstallIo() {
    const ret = wasm.settingsAddressesInstallIo();
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

export function settingsAddressesInstallDeleteTransactions() {
    const ret = wasm.settingsAddressesInstallDeleteTransactions();
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

export function settingsAddressesClearFields() {
    wasm.settingsAddressesClearFields();
}

/**
 * @param {string} message
 * @param {string} state
 */
export function settingsAddressesSetStatus(message, state) {
    const ptr0 = passStringToWasm0(message, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(state, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    wasm.settingsAddressesSetStatus(ptr0, len0, ptr1, len1);
}

/**
 * @returns {string}
 */
export function settingsAddressesNow() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.settingsAddressesNow();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @returns {bigint}
 */
export function settingsAddressesIncrementRestoreEpoch() {
    const ret = wasm.settingsAddressesIncrementRestoreEpoch();
    return BigInt.asUintN(64, ret);
}

/**
 * @returns {bigint}
 */
export function settingsAddressesRestoreEpoch() {
    const ret = wasm.settingsAddressesRestoreEpoch();
    return BigInt.asUintN(64, ret);
}

/**
 * @param {string} address
 * @returns {boolean}
 */
export function settingsAddressesOpenExplorer(address) {
    const ptr0 = passStringToWasm0(address, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.settingsAddressesOpenExplorer(ptr0, len0);
    return ret !== 0;
}

export function settingsAddressesInstallAll() {
    const ret = wasm.settingsAddressesInstallAll();
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {any} line
 * @returns {string}
 */
export function kgwLogParseLevel(line) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.kgwLogParseLevel(line);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} text
 * @param {any} env
 * @returns {Promise<boolean>}
 */
export function kgwLogCopyTextToClipboard(text, env) {
    const ret = wasm.kgwLogCopyTextToClipboard(text, env);
    return ret;
}

export function kgwLogInitTab() {
    const ret = wasm.kgwLogInitTab();
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {any} payload
 */
export function analysisViewSetData(payload) {
    wasm.analysisViewSetData(payload);
}

export function analysisViewRenderRows() {
    wasm.analysisViewRenderRows();
}

export function analysisViewApplyFilter() {
    wasm.analysisViewApplyFilter();
}

/**
 * @returns {Array<any>}
 */
export function analysisViewFilteredRows() {
    const ret = wasm.analysisViewFilteredRows();
    return ret;
}

export function analysisViewResetExpansion() {
    wasm.analysisViewResetExpansion();
}

/**
 * @param {any} value
 * @returns {boolean}
 */
export function nodeRawLogTextHasTransportWrapper(value) {
    const ret = wasm.nodeRawLogTextHasTransportWrapper(value);
    return ret !== 0;
}

/**
 * @param {any} root
 * @returns {boolean}
 */
export function nodeTraceRenderedStartControls(root) {
    const ret = wasm.nodeTraceRenderedStartControls(root);
    return ret !== 0;
}

/**
 * @param {string} command
 * @param {string} net
 * @returns {Promise<any>}
 */
export function nodeInvokeIntegratedRuntime(command, net) {
    const ptr0 = passStringToWasm0(command, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodeInvokeIntegratedRuntime(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * @param {string} command
 * @returns {string}
 */
export function nodeRuntimeActionForCommand(command) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(command, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.nodeRuntimeActionForCommand(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {any} net
 * @param {any} action
 * @param {any} phase
 * @param {any} details
 * @returns {boolean}
 */
export function nodeSmallOwnerTrace(net, action, phase, details) {
    const ret = wasm.nodeSmallOwnerTrace(net, action, phase, details);
    return ret !== 0;
}

/**
 * @param {any} net
 * @param {any} action
 * @param {any} phase
 * @param {any} details
 * @returns {boolean}
 */
export function nodeExplicitTrace(net, action, phase, details) {
    const ret = wasm.nodeExplicitTrace(net, action, phase, details);
    return ret !== 0;
}

/**
 * @param {any} net
 * @param {any} action
 * @param {any} phase
 * @param {any} details
 * @returns {boolean}
 */
export function nodeExplicitOwnerTrace(net, action, phase, details) {
    const ret = wasm.nodeExplicitOwnerTrace(net, action, phase, details);
    return ret !== 0;
}

/**
 * @param {string} adapter
 * @returns {any}
 */
export function nodeStartTraceTauriShape(adapter) {
    const ptr0 = passStringToWasm0(adapter, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeStartTraceTauriShape(ptr0, len0);
    return ret;
}

/**
 * @returns {any}
 */
export function nodeResolvePublicTauriInvoke() {
    const ret = wasm.nodeResolvePublicTauriInvoke();
    return ret;
}

/**
 * @param {any} stage
 * @param {any} options
 * @returns {boolean}
 */
export function nodeStartTraceFrontend(stage, options) {
    const ret = wasm.nodeStartTraceFrontend(stage, options);
    return ret !== 0;
}

/**
 * @param {any} report
 * @returns {string}
 */
export function nodeLegacyTransportReportText(report) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.nodeLegacyTransportReportText(report);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} entry
 * @param {string} expected_net
 * @param {string} expected_role
 * @returns {any}
 */
export function nodeNormalizeRawLogEntry(entry, expected_net, expected_role) {
    const ptr0 = passStringToWasm0(expected_net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(expected_role, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodeNormalizeRawLogEntry(entry, ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * @param {string} net
 * @param {string} role
 * @returns {string}
 */
export function nodeVisibleRawLogText(net, role) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(role, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.nodeVisibleRawLogText(ptr0, len0, ptr1, len1);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {any} text
 * @returns {number}
 */
export function nodeClipboardCharacterCount(text) {
    const ret = wasm.nodeClipboardCharacterCount(text);
    return ret >>> 0;
}

/**
 * @param {any} text
 * @returns {number}
 */
export function nodeClipboardLineCount(text) {
    const ret = wasm.nodeClipboardLineCount(text);
    return ret >>> 0;
}

/**
 * @param {any} text
 * @returns {string}
 */
export function nodeNormalizeClipboardLineEndings(text) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.nodeNormalizeClipboardLineEndings(text);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} error
 * @returns {string}
 */
export function nodeClipboardSafeError(error) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.nodeClipboardSafeError(error);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {string}
 */
export function nodeClipboardPlaceholderText(net) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.nodeClipboardPlaceholderText(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {any} element
 * @param {any} root
 * @returns {string}
 */
export function nodeTraceNetworkFromElement(element, root) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.nodeTraceNetworkFromElement(element, root);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} root
 * @returns {string}
 */
export function nodeTraceActiveNetwork(root) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.nodeTraceActiveNetwork(root);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {any}
 */
export function nodeTraceStartButtonState(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeTraceStartButtonState(ptr0, len0);
    return ret;
}

/**
 * @param {any} root
 * @returns {boolean}
 */
export function nodeInstallStartTraceDocumentClickObserver(root) {
    const ret = wasm.nodeInstallStartTraceDocumentClickObserver(root);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {string} role
 * @returns {boolean}
 */
export function nodeRenderRawLogBuffer(net, role) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(role, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodeRenderRawLogBuffer(ptr0, len0, ptr1, len1);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {string} role
 * @param {any} report
 * @returns {number}
 */
export function nodeApplyRuntimeLogReport(net, role, report) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(role, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodeApplyRuntimeLogReport(ptr0, len0, ptr1, len1, report);
    return ret >>> 0;
}

/**
 * @param {string} net
 * @param {string} role
 */
export function nodeClearRawLogBuffer(net, role) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(role, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    wasm.nodeClearRawLogBuffer(ptr0, len0, ptr1, len1);
}

/**
 * @param {string} net
 * @param {string} role
 * @returns {Promise<any>}
 */
export function nodeDispatchRuntimeLogClear(net, role) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(role, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodeDispatchRuntimeLogClear(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * @param {string} net
 * @param {string} text
 * @param {any} metadata
 * @returns {Promise<any>}
 */
export function nodeDispatchClipboardWrite(net, text, metadata) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodeDispatchClipboardWrite(ptr0, len0, ptr1, len1, metadata);
    return ret;
}

/**
 * @param {string} net
 * @param {any} button
 * @param {any} error
 * @param {any} details
 * @returns {boolean}
 */
export function nodeCopyLogFailure(net, button, error, details) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeCopyLogFailure(ptr0, len0, button, error, details);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {any} button
 * @returns {Promise<boolean>}
 */
export function nodeHandleCopyLog(net, button) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeHandleCopyLog(ptr0, len0, button);
    return ret;
}

/**
 * @param {string} action
 * @param {string} net
 * @param {any} button
 * @returns {Promise<boolean>}
 */
export function nodeHandleLogAction(action, net, button) {
    const ptr0 = passStringToWasm0(action, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodeHandleLogAction(ptr0, len0, ptr1, len1, button);
    return ret;
}

/**
 * @param {any} state
 * @returns {any}
 */
export function runtimePresentation(state) {
    const ret = wasm.runtimePresentation(state);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} fields
 * @param {any} running
 * @param {any} cpu_only
 * @returns {string}
 */
export function runtimeObservationSummary(fields, running, cpu_only) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ret = wasm.runtimeObservationSummary(fields, running, cpu_only);
        var ptr1 = ret[0];
        var len1 = ret[1];
        if (ret[3]) {
            ptr1 = 0; len1 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred2_0 = ptr1;
        deferred2_1 = len1;
        return getStringFromWasm0(ptr1, len1);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} stage
 * @param {any} details
 */
export function explorerMicroscopeLog(stage, details) {
    const ptr0 = passStringToWasm0(stage, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.explorerMicroscopeLog(ptr0, len0, details);
}

/**
 * @param {string} stage
 * @param {any} details
 */
export function explorerMicroscopeWarn(stage, details) {
    const ptr0 = passStringToWasm0(stage, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.explorerMicroscopeWarn(ptr0, len0, details);
}

/**
 * @param {string} stage
 * @param {any} error
 * @param {any} details
 */
export function explorerMicroscopeError(stage, error, details) {
    const ptr0 = passStringToWasm0(stage, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.explorerMicroscopeError(ptr0, len0, error, details);
}

/**
 * @param {any} section
 * @returns {any}
 */
export function explorerMicroscopeElementReport(section) {
    const ret = wasm.explorerMicroscopeElementReport(section);
    return ret;
}

/**
 * @param {any} section
 * @returns {any}
 */
export function explorerMicroscopeLayoutReport(section) {
    const ret = wasm.explorerMicroscopeLayoutReport(section);
    return ret;
}

/**
 * @param {string} label
 * @param {any} snapshot
 */
export function explorerMicroscopeStateReport(label, snapshot) {
    const ptr0 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.explorerMicroscopeStateReport(ptr0, len0, snapshot);
}

/**
 * @param {string} label
 * @returns {any}
 */
export function explorerMicroscopeCurrentStateReport(label) {
    const ptr0 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.explorerMicroscopeCurrentStateReport(ptr0, len0);
    return ret;
}

/**
 * @param {string} label
 * @param {any} value
 */
export function explorerMicroscopeApiShape(label, value) {
    const ptr0 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.explorerMicroscopeApiShape(ptr0, len0, value);
}

/**
 * @param {any} root
 * @param {any} callbacks
 * @returns {boolean}
 */
export function settingsOwnerInstall(root, callbacks) {
    const ret = wasm.settingsOwnerInstall(root, callbacks);
    return ret !== 0;
}

/**
 * @param {any} root
 * @param {any} callbacks
 * @returns {boolean}
 */
export function nodeInstallSettingsOwner(root, callbacks) {
    const ret = wasm.nodeInstallSettingsOwner(root, callbacks);
    return ret !== 0;
}

/**
 * @param {any} root
 * @param {string} network
 * @param {boolean} _disabled
 * @param {string} reason
 * @param {any} callbacks
 */
export function settingsOwnerSetDisabled(root, network, _disabled, reason, callbacks) {
    const ptr0 = passStringToWasm0(network, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    wasm.settingsOwnerSetDisabled(root, ptr0, len0, _disabled, ptr1, len1, callbacks);
}

/**
 * @param {any} root
 * @param {string} network
 * @param {boolean} disabled
 * @param {string} reason
 * @param {any} callbacks
 */
export function nodeSettingsOwnerSetDisabled(root, network, disabled, reason, callbacks) {
    const ptr0 = passStringToWasm0(network, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    wasm.nodeSettingsOwnerSetDisabled(root, ptr0, len0, disabled, ptr1, len1, callbacks);
}

/**
 * @param {any} root
 * @param {string} network
 * @returns {Array<any>}
 */
export function settingsOwnerButtons(root, network) {
    const ptr0 = passStringToWasm0(network, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.settingsOwnerButtons(root, ptr0, len0);
    return ret;
}

/**
 * @param {any} root
 * @param {string} network
 * @returns {Array<any>}
 */
export function nodeSettingsOwnerButtons(root, network) {
    const ptr0 = passStringToWasm0(network, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeSettingsOwnerButtons(root, ptr0, len0);
    return ret;
}

/**
 * @param {any} level
 * @param {any} message
 * @param {any} details
 * @param {any} source
 * @returns {any}
 */
export function shellLoggerLog(level, message, details, source) {
    const ret = wasm.shellLoggerLog(level, message, details, source);
    return ret;
}

/**
 * @param {any} error
 * @param {any} source
 */
export function shellLoggerFatal(error, source) {
    wasm.shellLoggerFatal(error, source);
}

/**
 * @returns {any}
 */
export function shellLoggerGetBufferedLogs() {
    const ret = wasm.shellLoggerGetBufferedLogs();
    return ret;
}

export function shellLoggerClearBufferedLogs() {
    wasm.shellLoggerClearBufferedLogs();
}

/**
 * @returns {boolean}
 */
export function shellLoggerInstall() {
    const ret = wasm.shellLoggerInstall();
    return ret !== 0;
}

/**
 * @param {any} section
 */
export function explorerSyncCurrentActionState(section) {
    const ret = wasm.explorerSyncCurrentActionState(section);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @returns {boolean}
 */
export function explorerInstallPriceRerender() {
    const ret = wasm.explorerInstallPriceRerender();
    return ret !== 0;
}

/**
 * @param {any} section
 * @param {any} rows
 * @param {string} status_text
 */
export function explorerRenderSummaries(section, rows, status_text) {
    const ptr0 = passStringToWasm0(status_text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.explorerRenderSummaries(section, rows, ptr0, len0);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {any} section
 */
export function explorerRenderTable(section) {
    const ret = wasm.explorerRenderTable(section);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {any} section
 * @param {string} reason
 * @param {any} options
 */
export function explorerClearTransactionTable(section, reason, options) {
    const ptr0 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.explorerClearTransactionTable(section, ptr0, len0, options);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {any} section
 */
export function explorerResetFilters(section) {
    const ret = wasm.explorerResetFilters(section);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {any} section
 * @param {any} address
 * @param {any} start_ts
 * @param {any} end_ts
 * @param {string} status_text
 * @returns {Promise<any>}
 */
export function explorerLoadAndRenderDaySummaries(section, address, start_ts, end_ts, status_text) {
    const ptr0 = passStringToWasm0(status_text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.explorerLoadAndRenderDaySummaries(section, address, start_ts, end_ts, ptr0, len0);
    return ret;
}

/**
 * @param {string} root
 * @param {string} child
 * @returns {string}
 */
export function nodeJoinPath(root, child) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(root, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(child, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.nodeJoinPath(ptr0, len0, ptr1, len1);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {any} paths
 * @returns {string}
 */
export function nodeExtractUserLocalAppData(paths) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.nodeExtractUserLocalAppData(paths);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} paths
 * @param {string} net
 * @returns {string}
 */
export function nodeRustyKaspaLocalAppDataRoot(paths, net) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.nodeRustyKaspaLocalAppDataRoot(paths, ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} value
 * @returns {boolean}
 */
export function nodeIsEmptyOrGeneratedPath(value) {
    const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeIsEmptyOrGeneratedPath(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {any} rows
 */
export function settingsDatabaseRenderRows(rows) {
    wasm.settingsDatabaseRenderRows(rows);
}

/**
 * @returns {Promise<void>}
 */
export function settingsDatabaseRefresh() {
    const ret = wasm.settingsDatabaseRefresh();
    return ret;
}

export function settingsDatabaseInstall() {
    const ret = wasm.settingsDatabaseInstall();
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

export function settingsDatabaseInstallMaintenance() {
    const ret = wasm.settingsDatabaseInstallMaintenance();
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {any} reason
 * @returns {Promise<boolean>}
 */
export function headerLiveMetricsRefreshSelectedCurrencyPrice(reason) {
    const ret = wasm.headerLiveMetricsRefreshSelectedCurrencyPrice(reason);
    return ret;
}

export function headerLiveMetricsInit() {
    wasm.headerLiveMetricsInit();
}

export function headerLiveMetricsInstallModule() {
    wasm.headerLiveMetricsInstallModule();
}

/**
 * @param {any} value
 * @returns {number}
 */
export function headerLiveMetricsExtractUsdPrice(value) {
    const ret = wasm.headerLiveMetricsExtractUsdPrice(value);
    return ret;
}

/**
 * @param {string} label
 * @param {string} value
 * @param {string} updated_at
 * @param {string} source
 * @returns {string}
 */
export function headerLiveMetricsBuildEnglishTooltip(label, value, updated_at, source) {
    let deferred5_0;
    let deferred5_1;
    try {
        const ptr0 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(updated_at, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(source, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len3 = WASM_VECTOR_LEN;
        const ret = wasm.headerLiveMetricsBuildEnglishTooltip(ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
        deferred5_0 = ret[0];
        deferred5_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred5_0, deferred5_1, 1);
    }
}

/**
 * @param {number} epoch_ms
 * @returns {string}
 */
export function headerLiveMetricsFormatLocalDateTime(epoch_ms) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.headerLiveMetricsFormatLocalDateTime(epoch_ms);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {number} epoch_ms
 * @returns {string}
 */
export function headerLiveMetricsFormatEnglishTime(epoch_ms) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.headerLiveMetricsFormatEnglishTime(epoch_ms);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {number} epoch_ms
 * @returns {string}
 */
export function headerLiveMetricsFormatEnglishDate(epoch_ms) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.headerLiveMetricsFormatEnglishDate(epoch_ms);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {number} epoch_ms
 * @returns {string}
 */
export function headerLiveMetricsFormatEnglishStamp(epoch_ms) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.headerLiveMetricsFormatEnglishStamp(epoch_ms);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} kind
 * @returns {any}
 */
export function headerLiveMetricsMetricIds(kind) {
    const ptr0 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.headerLiveMetricsMetricIds(ptr0, len0);
    return ret;
}

/**
 * @param {any} prices
 * @param {string} currency
 * @returns {any}
 */
export function headerLiveMetricsNumericPrice(prices, currency) {
    const ptr0 = passStringToWasm0(currency, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.headerLiveMetricsNumericPrice(prices, ptr0, len0);
    return ret;
}

/**
 * @param {string} currency
 * @param {number} value
 * @returns {string}
 */
export function headerLiveMetricsFormatCurrency(currency, value) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(currency, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.headerLiveMetricsFormatCurrency(ptr0, len0, value);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} metric_value
 * @param {string} current_text
 * @returns {string}
 */
export function headerLiveMetricsSelectedCurrencyValue(metric_value, current_text) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(metric_value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(current_text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.headerLiveMetricsSelectedCurrencyValue(ptr0, len0, ptr1, len1);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {string} key
 * @param {string} fallback
 * @returns {any}
 */
export function nodeI18nText(key, fallback) {
    const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(fallback, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodeI18nText(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function nodeEscapeHtml(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.nodeEscapeHtml(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} net
 * @param {string} name
 * @returns {string}
 */
export function nodeElementId(net, name) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.nodeElementId(ptr0, len0, ptr1, len1);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {string} net
 * @param {string} name
 * @returns {string}
 */
export function nodeValue(net, name) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.nodeValue(ptr0, len0, ptr1, len1);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {any}
 */
export function nodeReadCommandOptions(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeReadCommandOptions(ptr0, len0);
    return ret;
}

/**
 * @param {string} net
 * @param {any} values
 * @returns {any}
 */
export function nodeApplyCommandOptions(net, values) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeApplyCommandOptions(ptr0, len0, values);
    return ret;
}

/**
 * @param {string} net
 * @param {string} message
 * @param {boolean} error
 * @returns {boolean}
 */
export function nodePreviewMessage(net, message, error) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(message, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodePreviewMessage(ptr0, len0, ptr1, len1, error);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {any} state
 * @param {any} evidence
 * @param {any} error_text
 * @param {any} error_source
 * @returns {boolean}
 */
export function nodeSetRuntimeNotice(net, state, evidence, error_text, error_source) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeSetRuntimeNotice(ptr0, len0, state, evidence, error_text, error_source);
    return ret !== 0;
}

/**
 * @param {string} net
 * @returns {boolean}
 */
export function nodeMarkRestartRequired(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeMarkRestartRequired(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {boolean} locked
 * @returns {boolean}
 */
export function nodeSyncDependencies(net, locked) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeSyncDependencies(ptr0, len0, locked);
    return ret !== 0;
}

/**
 * @param {string} net
 * @returns {boolean}
 */
export function nodePanelStartFromMonitor(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodePanelStartFromMonitor(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {boolean} focus
 * @returns {any}
 */
export function nodeValidateForm(net, focus) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeValidateForm(ptr0, len0, focus);
    return ret;
}

/**
 * @param {string} net
 */
export function nodeRequireValidSettings(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeRequireValidSettings(ptr0, len0);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {string} net
 * @param {string} name
 * @returns {boolean}
 */
export function nodeChecked(net, name) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodeChecked(ptr0, len0, ptr1, len1);
    return ret !== 0;
}

/**
 * @param {string} net
 * @returns {string}
 */
export function nodeCommandInlineStateKey(net) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.nodeCommandInlineStateKey(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {any}
 */
export function nodeCommandInlineState(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeCommandInlineState(ptr0, len0);
    return ret;
}

/**
 * @param {string} net
 * @param {string} name
 * @returns {boolean}
 */
export function nodeCommandOptionEnabled(net, name) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodeCommandOptionEnabled(ptr0, len0, ptr1, len1);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {string} name
 * @returns {boolean}
 */
export function nodeCommandShouldInclude(net, name) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodeCommandOptionEnabled(ptr0, len0, ptr1, len1);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {string} name
 * @returns {string}
 */
export function nodeCommandInlineToggle(net, name) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.nodeCommandInlineToggle(ptr0, len0, ptr1, len1);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {string} net
 */
export function nodeRefreshInlineCommandToggles(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.nodeRefreshInlineCommandToggles(ptr0, len0);
}

/**
 * @param {string} net
 * @param {string} name
 * @returns {boolean}
 */
export function nodeToggleCommandOption(net, name) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodeToggleCommandOption(ptr0, len0, ptr1, len1);
    return ret !== 0;
}

/**
 * @returns {string}
 */
export function nodeCommandOptionsKey() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.nodeCommandOptionsKey();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {any}
 */
export function nodeEffectiveNodeSettings(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeEffectiveNodeSettings(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {string} net
 * @param {string} command
 * @returns {any}
 */
export function nodeRuntimeArgs(net, command) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(command, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodeRuntimeArgs(ptr0, len0, ptr1, len1);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {string} net
 * @returns {any}
 */
export function nodeR51ReadSettingsTracked(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeR51ReadSettingsTracked(ptr0, len0);
    return ret;
}

export function nodeR51CaptureFactoryDefaults() {
    const ret = wasm.nodeR51CaptureFactoryDefaults();
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @returns {Array<any>}
 */
export function nodeR51LoadSavedSettings() {
    const ret = wasm.nodeR51LoadSavedSettings();
    return ret;
}

/**
 * @param {string} net
 * @returns {any}
 */
export function nodeR51SaveSettingsAction(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeR51SaveSettingsAction(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {string} net
 * @returns {any}
 */
export function nodeR51SetDefaultsAction(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeR51SetDefaultsAction(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {string} net
 * @returns {any}
 */
export function nodeR51RestoreDefaultsAction(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeR51RestoreDefaultsAction(ptr0, len0);
    return ret;
}

/**
 * @returns {string}
 */
export function nodeRenderNetworkPanelsHtml() {
    let deferred2_0;
    let deferred2_1;
    try {
        const ret = wasm.nodeRenderNetworkPanelsHtml();
        var ptr1 = ret[0];
        var len1 = ret[1];
        if (ret[3]) {
            ptr1 = 0; len1 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred2_0 = ptr1;
        deferred2_1 = len1;
        return getStringFromWasm0(ptr1, len1);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} net
 * @param {string} name
 * @param {string} label
 * @param {string} value
 * @param {string} placeholder
 * @param {boolean} span2
 * @param {string} toggle
 * @returns {string}
 */
export function nodeCardInput(net, name, label, value, placeholder, span2, toggle) {
    let deferred7_0;
    let deferred7_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len3 = WASM_VECTOR_LEN;
        const ptr4 = passStringToWasm0(placeholder, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len4 = WASM_VECTOR_LEN;
        const ptr5 = passStringToWasm0(toggle, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len5 = WASM_VECTOR_LEN;
        const ret = wasm.nodeCardInput(ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3, ptr4, len4, span2, ptr5, len5);
        deferred7_0 = ret[0];
        deferred7_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred7_0, deferred7_1, 1);
    }
}

/**
 * @param {string} net
 * @param {string} name
 * @param {string} label
 * @param {Array<any>} options
 * @param {string} value
 * @param {boolean} span2
 * @param {string} toggle
 * @returns {string}
 */
export function nodeCardSelect(net, name, label, options, value, span2, toggle) {
    let deferred6_0;
    let deferred6_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ptr3 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len3 = WASM_VECTOR_LEN;
        const ptr4 = passStringToWasm0(toggle, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len4 = WASM_VECTOR_LEN;
        const ret = wasm.nodeCardSelect(ptr0, len0, ptr1, len1, ptr2, len2, options, ptr3, len3, span2, ptr4, len4);
        deferred6_0 = ret[0];
        deferred6_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred6_0, deferred6_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {any}
 */
export function nodeR51SaveSettings(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeR51SaveSettings(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {string} net
 * @returns {any}
 */
export function nodeR51SetAsDefaults(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeR51SetAsDefaults(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @returns {Array<any>}
 */
export function nodeR51Keys() {
    const ret = wasm.nodeR51Keys();
    return ret;
}

/**
 * @param {string} net
 * @returns {any}
 */
export function nodeR51Panel(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeR51Panel(ptr0, len0);
    return ret;
}

/**
 * @param {string} net
 * @returns {Array<any>}
 */
export function nodeR51Fields(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeR51Fields(ptr0, len0);
    return ret;
}

/**
 * @param {string} net
 * @returns {any}
 */
export function nodeR51ReadSettings(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeR51ReadSettings(ptr0, len0);
    return ret;
}

/**
 * @param {string} net
 * @param {any} values
 * @returns {any}
 */
export function nodeR51WriteSettings(net, values) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeR51WriteSettings(ptr0, len0, values);
    return ret;
}

/**
 * @param {string} key
 * @param {any} value
 */
export function nodeR51Store(key, value) {
    const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeR51Store(ptr0, len0, value);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {string} key
 * @returns {any}
 */
export function nodeR51Load(key) {
    const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeR51Load(ptr0, len0);
    return ret;
}

/**
 * @param {string} net
 * @param {string} name
 * @param {string} label
 * @param {boolean} checked
 * @param {boolean} span2
 * @returns {string}
 */
export function nodeCardCheck(net, name, label, checked, span2) {
    let deferred4_0;
    let deferred4_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.nodeCardCheck(ptr0, len0, ptr1, len1, ptr2, len2, checked, span2);
        deferred4_0 = ret[0];
        deferred4_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
    }
}

/**
 * @param {string} net
 * @param {boolean} locked
 * @returns {bigint}
 */
export function nodeUpdateCommand(net, locked) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeUpdateCommand(ptr0, len0, locked);
    return BigInt.asUintN(64, ret);
}

/**
 * @param {string} net
 * @returns {bigint}
 */
export function nodePreviewSequence(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodePreviewSequence(ptr0, len0);
    return BigInt.asUintN(64, ret);
}

/**
 * @param {string} net
 * @param {any} effective
 * @returns {Promise<any>}
 */
export function nodePreparePreview(net, effective) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodePreparePreview(ptr0, len0, effective);
    return ret;
}

/**
 * @param {any} root
 * @param {any} callbacks
 * @returns {boolean}
 */
export function nodeInstallNetworkTabs(root, callbacks) {
    const ret = wasm.nodeInstallNetworkTabs(root, callbacks);
    return ret !== 0;
}

/**
 * @param {any} root
 * @returns {boolean}
 */
export function nodeInstallDelegatedTabs(root) {
    const ret = wasm.nodeInstallDelegatedTabs(root);
    return ret !== 0;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function nodeStringifyRuntimeResult(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.nodeStringifyRuntimeResult(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} error
 * @returns {string}
 */
export function nodeNormalizeRuntimeError(error) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.nodeNormalizeRuntimeError(error);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {any}
 */
export function nodeParseRuntimeFields(value) {
    const ret = wasm.nodeParseRuntimeFields(value);
    return ret;
}

/**
 * @param {any} value
 * @returns {any}
 */
export function nodeRuntimeEvidence(value) {
    const ret = wasm.nodeRuntimeEvidence(value);
    return ret;
}

/**
 * @param {string} net
 * @param {any} value
 * @returns {any}
 */
export function nodeAssertStartEvidence(net, value) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeAssertStartEvidence(ptr0, len0, value);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} value
 * @returns {boolean}
 */
export function nodeRuntimeIsRunning(value) {
    const ret = wasm.nodeRuntimeIsRunning(value);
    return ret !== 0;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function nodeRuntimeErrorFromStatus(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.nodeRuntimeErrorFromStatus(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} command
 * @param {any} payload
 * @returns {Promise<any>}
 */
export function nodeBackendInvoke(command, payload) {
    const ptr0 = passStringToWasm0(command, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeBackendInvoke(ptr0, len0, payload);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {string} net
 * @param {boolean} locked
 * @returns {Promise<any>}
 */
export function nodeApplyRootDefaultPath(net, locked) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeApplyRootDefaultPath(ptr0, len0, locked);
    return ret;
}

/**
 * @param {string} net
 * @param {string} name
 * @param {boolean} locked
 * @returns {boolean}
 */
export function nodeToggleCommandOptionAndUpdate(net, name, locked) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.nodeToggleCommandOptionAndUpdate(ptr0, len0, ptr1, len1, locked);
    return ret !== 0;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function nodeNormalizeInnerTab(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.nodeNormalizeInnerTab(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {string}
 */
export function nodeResolveInnerTab(net) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.nodeResolveInnerTab(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} net
 * @param {any} selected
 * @returns {string}
 */
export function nodeSaveInnerTab(net, selected) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.nodeSaveInnerTab(ptr0, len0, selected);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function nodeNormalizeNetwork(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.nodeNormalizeNetwork(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @returns {string}
 */
export function nodeReadLastNetwork() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.nodeReadLastNetwork();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} net
 * @returns {string}
 */
export function nodeSaveLastNetwork(net) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.nodeSaveLastNetwork(net);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {boolean}
 */
export function nodeLogAutoScrollEnabled(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeLogAutoScrollEnabled(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {boolean} enabled
 */
export function nodeSetLogAutoScroll(net, enabled) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.nodeSetLogAutoScroll(ptr0, len0, enabled);
}

export function nodeInstallLogAutoScrollControls() {
    wasm.nodeInstallLogAutoScrollControls();
}

/**
 * @param {any} network
 * @returns {Promise<any>}
 */
export function nodeSuperMegaIsolatedAdapterStatusPreviewV1(network) {
    const ret = wasm.nodeSuperMegaIsolatedAdapterStatusPreviewV1(network);
    return ret;
}

/**
 * @param {any} network
 * @param {any} app_dir_name
 * @returns {Promise<any>}
 */
export function nodeFinalIsolatedAdapterStartV1(network, app_dir_name) {
    const ret = wasm.nodeFinalIsolatedAdapterStartV1(network, app_dir_name);
    return ret;
}

/**
 * @param {any} network
 * @returns {Promise<any>}
 */
export function nodeV67StopRuntime(network) {
    const ret = wasm.nodeV67StopRuntime(network);
    return ret;
}

/**
 * @param {any} network
 * @returns {Promise<any>}
 */
export function nodeV67RuntimeFeaturePolicy(network) {
    const ret = wasm.nodeV67RuntimeFeaturePolicy(network);
    return ret;
}

/**
 * @returns {Array<any>}
 */
export function nodeNetworkProfiles() {
    const ret = wasm.nodeNetworkProfiles();
    return ret;
}

/**
 * @param {string} net
 * @returns {string}
 */
export function nodeNetworkPolicyKey(net) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.nodeNetworkPolicyKey(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {any}
 */
export function nodeNetworkProfile(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeNetworkProfile(ptr0, len0);
    return ret;
}

/**
 * @param {string} net
 * @returns {boolean}
 */
export function nodeNetworkEnabled(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeNetworkEnabled(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {boolean} enabled
 */
export function nodeSetNetworkEnabled(net, enabled) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.nodeSetNetworkEnabled(ptr0, len0, enabled);
}

/**
 * @param {string} net
 * @returns {string}
 */
export function nodeNetworkPolicyMessage(net) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.nodeNetworkPolicyMessage(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} id
 * @returns {any}
 */
export function nodeById(id) {
    const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.nodeById(ptr0, len0);
    return ret;
}

/**
 * @param {any} network
 * @returns {Promise<any>}
 */
export function nodeFinalIsolatedAdapterStatusV1(network) {
    const ret = wasm.nodeFinalIsolatedAdapterStatusV1(network);
    return ret;
}

/**
 * @param {any} network
 * @returns {Promise<any>}
 */
export function nodeFinalIsolatedAdapterStopV1(network) {
    const ret = wasm.nodeFinalIsolatedAdapterStopV1(network);
    return ret;
}

/**
 * @param {any} network
 * @returns {Promise<any>}
 */
export function nodeV66RuntimeFeaturePolicyV1(network) {
    const ret = wasm.nodeV66RuntimeFeaturePolicyV1(network);
    return ret;
}

/**
 * @param {any} network
 * @param {any} app_dir_name
 * @returns {Promise<any>}
 */
export function nodeV66IsolatedAdapterStartV1(network, app_dir_name) {
    const ret = wasm.nodeV66IsolatedAdapterStartV1(network, app_dir_name);
    return ret;
}

/**
 * @param {any} network
 * @returns {Promise<any>}
 */
export function nodeV66IsolatedAdapterStatusV1(network) {
    const ret = wasm.nodeV66IsolatedAdapterStatusV1(network);
    return ret;
}

/**
 * @param {any} network
 * @returns {Promise<any>}
 */
export function nodeV66IsolatedAdapterStopV1(network) {
    const ret = wasm.nodeV66IsolatedAdapterStopV1(network);
    return ret;
}

/**
 * @param {any} network
 * @param {any} app_dir_name
 * @returns {Promise<any>}
 */
export function nodeV67StartRuntime(network, app_dir_name) {
    const ret = wasm.nodeV67StartRuntime(network, app_dir_name);
    return ret;
}

/**
 * @param {any} network
 * @returns {Promise<any>}
 */
export function nodeV67StatusRuntime(network) {
    const ret = wasm.nodeV67StatusRuntime(network);
    return ret;
}

export function topAddressesInitTab() {
    wasm.topAddressesInitTab();
}

export function topAddressesRefresh() {
    wasm.topAddressesRefresh();
}

/**
 * @param {any} row
 * @param {number} index
 * @returns {any}
 */
export function topAddressesNormalizeRow(row, index) {
    const ret = wasm.topAddressesNormalizeRow(row, index);
    return ret;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function topAddressesEscapeCsv(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.topAddressesEscapeCsv(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {Array<any>} value
 */
export function topAddressesContractSetFilteredRows(value) {
    wasm.topAddressesContractSetFilteredRows(value);
}

/**
 * @param {string} value
 */
export function topAddressesContractSetLastUpdatedText(value) {
    const ptr0 = passStringToWasm0(value, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.topAddressesContractSetLastUpdatedText(ptr0, len0);
}

/**
 * @param {any} value
 * @returns {string}
 */
export function topAddressesEscapeHtml(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.topAddressesEscapeHtml(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function topAddressesAddressUrl(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.topAddressesAddressUrl(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @returns {string}
 */
export function topAddressesSelectedCurrency() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.topAddressesSelectedCurrency();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} row
 * @param {string} currency
 * @returns {string}
 */
export function topAddressesValueForCurrency(row, currency) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(currency, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.topAddressesValueForCurrency(row, ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @returns {any}
 */
export function topAddressesClientTable() {
    const ret = wasm.topAddressesClientTable();
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {string} format
 * @returns {any}
 */
export function topAddressesNativeDialogFilter(format) {
    const ptr0 = passStringToWasm0(format, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.topAddressesNativeDialogFilter(ptr0, len0);
    return ret;
}

/**
 * @param {string} locale
 * @returns {any}
 */
export function topAddressesPromptText(locale) {
    const ptr0 = passStringToWasm0(locale, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.topAddressesPromptText(ptr0, len0);
    return ret;
}

/**
 * @param {any} value
 */
export function topAddressesContractSetPrices(value) {
    wasm.topAddressesContractSetPrices(value);
}

/**
 * @param {any} section
 * @param {any} address
 * @param {any} start_ts
 * @param {any} end_ts
 * @returns {any}
 */
export function explorerLegacyListRequest(section, address, start_ts, end_ts) {
    const ret = wasm.explorerLegacyListRequest(section, address, start_ts, end_ts);
    return ret;
}

/**
 * @param {any} section
 * @param {any} address
 * @param {any} start_ts
 * @param {any} end_ts
 * @returns {Promise<any>}
 */
export function explorerLoadTransactionDaySummariesFromDb(section, address, start_ts, end_ts) {
    const ret = wasm.explorerLoadTransactionDaySummariesFromDb(section, address, start_ts, end_ts);
    return ret;
}

/**
 * @param {any} section
 * @param {any} address
 * @param {string} day
 * @returns {Promise<any>}
 */
export function explorerLoadTransactionsForSingleDayFromDb(section, address, day) {
    const ptr0 = passStringToWasm0(day, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.explorerLoadTransactionsForSingleDayFromDb(section, address, ptr0, len0);
    return ret;
}

/**
 * @param {any} section
 * @param {any} address
 * @param {string} day
 * @returns {Promise<any>}
 */
export function explorerLoadTransactionsForDay(section, address, day) {
    const ptr0 = passStringToWasm0(day, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.explorerLoadTransactionsForDay(section, address, ptr0, len0);
    return ret;
}

/**
 * @param {any} request
 * @returns {Promise<any>}
 */
export function explorerInvokeDaySummaries(request) {
    const ret = wasm.explorerInvokeDaySummaries(request);
    return ret;
}

/**
 * @param {any} section
 * @param {any} message
 * @param {any} state
 * @returns {boolean}
 */
export function explorerSetStatus(section, message, state) {
    const ret = wasm.explorerSetStatus(section, message, state);
    return ret !== 0;
}

/**
 * @param {any} section
 * @param {boolean} busy
 * @param {number} rows_count
 * @param {number} filtered_rows_count
 */
export function explorerSyncActionState(section, busy, rows_count, filtered_rows_count) {
    const ret = wasm.explorerSyncActionState(section, busy, rows_count, filtered_rows_count);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {any} result
 * @returns {any}
 */
export function explorerExtractRowsFromUnifiedResult(result) {
    const ret = wasm.explorerExtractRowsFromUnifiedResult(result);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} action
 * @param {any} phase
 * @param {any} details
 * @returns {Promise<any>}
 */
export function explorerUiTrace(action, phase, details) {
    const ret = wasm.explorerUiTrace(action, phase, details);
    return ret;
}

/**
 * @param {any} label
 * @param {any} payload
 */
export function explorerFilterTrace(label, payload) {
    wasm.explorerFilterTrace(label, payload);
}

/**
 * @param {string} label
 * @param {any} payload
 */
export function explorerClean2Log(label, payload) {
    const ptr0 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.explorerClean2Log(ptr0, len0, payload);
}

/**
 * @param {any} section
 * @returns {any}
 */
export function explorerClean2Section(section) {
    const ret = wasm.explorerClean2Section(section);
    return ret;
}

/**
 * @param {any} section
 * @returns {any}
 */
export function explorerClean2Body(section) {
    const ret = wasm.explorerClean2Body(section);
    return ret;
}

/**
 * @param {any} section
 * @param {any} address
 * @param {any} start_ts
 * @param {any} end_ts
 * @returns {Promise<any>}
 */
export function explorerClean2LoadSummaries(section, address, start_ts, end_ts) {
    const ret = wasm.explorerClean2LoadSummaries(section, address, start_ts, end_ts);
    return ret;
}

/**
 * @param {any} section
 * @param {any} address
 * @param {string} day
 * @returns {Promise<any>}
 */
export function explorerClean2LoadDayTransactions(section, address, day) {
    const ptr0 = passStringToWasm0(day, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.explorerClean2LoadDayTransactions(section, address, ptr0, len0);
    return ret;
}

/**
 * @param {any} request
 * @returns {Promise<any>}
 */
export function explorerInvokeUnifiedFetch(request) {
    const ret = wasm.explorerInvokeUnifiedFetch(request);
    return ret;
}

/**
 * @param {any} request_id
 * @returns {Promise<any>}
 */
export function explorerInvokeCancelTransactions(request_id) {
    const ret = wasm.explorerInvokeCancelTransactions(request_id);
    return ret;
}

/**
 * @param {any} request
 * @returns {Promise<any>}
 */
export function explorerInvokeGroupedTransactions(request) {
    const ret = wasm.explorerInvokeGroupedTransactions(request);
    return ret;
}

/**
 * @param {any} section
 * @returns {boolean}
 */
export function explorerDefaultDates(section) {
    const ret = wasm.explorerDefaultDates(section);
    return ret !== 0;
}

/**
 * @param {string} action
 * @param {string} net
 * @param {any} button
 * @param {any} deps
 * @returns {Promise<void>}
 */
export function bridgeHandleLogAction(action, net, button, deps) {
    const ptr0 = passStringToWasm0(action, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeHandleLogAction(ptr0, len0, ptr1, len1, button, deps);
    return ret;
}

/**
 * @param {any} button
 */
export function bridgeRestoreLogActionLabel(button) {
    wasm.bridgeRestoreLogActionLabel(button);
}

/**
 * @param {any} button
 * @param {string} done_label
 */
export function bridgeFlashLogActionButton(button, done_label) {
    const ptr0 = passStringToWasm0(done_label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.bridgeFlashLogActionButton(button, ptr0, len0);
}

/**
 * @returns {boolean}
 */
export function bridgeRuntimeInvokeAvailable() {
    const ret = wasm.bridgeRuntimeInvokeAvailable();
    return ret !== 0;
}

/**
 * @param {any} text
 * @returns {string}
 */
export function bridgeNormalizeClipboardLineEndings(text) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeNormalizeClipboardLineEndings(text);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} error
 * @returns {string}
 */
export function bridgeClipboardSafeError(error) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeClipboardSafeError(error);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} text
 * @returns {Promise<string>}
 */
export function bridgeSha256Hex(text) {
    const ret = wasm.bridgeSha256Hex(text);
    return ret;
}

/**
 * @param {any} stage
 * @param {any} options
 * @returns {boolean}
 */
export function bridgeStartTraceFrontend(stage, options) {
    const ret = wasm.bridgeStartTraceFrontend(stage, options);
    return ret !== 0;
}

/**
 * @param {string} command
 * @param {any} payload
 * @returns {Promise<any>}
 */
export function bridgeInvokeRuntimeCommand(command, payload) {
    const ptr0 = passStringToWasm0(command, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeInvokeRuntimeCommand(ptr0, len0, payload);
    return ret;
}

/**
 * @param {string} _net
 * @param {any} payload
 * @returns {Promise<any>}
 */
export function bridgePreparePreview(_net, payload) {
    const ptr0 = passStringToWasm0(_net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgePreparePreview(ptr0, len0, payload);
    return ret;
}

/**
 * @param {string} net
 * @param {string} text
 * @param {any} metadata
 * @returns {Promise<any>}
 */
export function bridgeDispatchClipboardWrite(net, text, metadata) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(text, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeDispatchClipboardWrite(ptr0, len0, ptr1, len1, metadata);
    return ret;
}

/**
 * @param {string} net
 * @returns {any}
 */
export function bridgeClipboardStatusElement(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeClipboardStatusElement(ptr0, len0);
    return ret;
}

/**
 * @param {string} net
 * @param {string} message
 * @param {string} state
 * @returns {boolean}
 */
export function bridgeSetClipboardStatus(net, message, state) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(message, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passStringToWasm0(state, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len2 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeSetClipboardStatus(ptr0, len0, ptr1, len1, ptr2, len2);
    return ret !== 0;
}

/**
 * @param {string} net
 * @returns {any}
 */
export function bridgeReadClipboardRawLogBuffer(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeReadClipboardRawLogBuffer(ptr0, len0);
    return ret;
}

/**
 * @param {any} text
 * @returns {number}
 */
export function bridgeClipboardCharacterCount(text) {
    const ret = wasm.bridgeClipboardCharacterCount(text);
    return ret >>> 0;
}

/**
 * @param {any} text
 * @returns {number}
 */
export function bridgeClipboardLineCount(text) {
    const ret = wasm.bridgeClipboardLineCount(text);
    return ret >>> 0;
}

export function shellAuxInstall() {
    wasm.shellAuxInstall();
}

export function explorerTabInstall() {
    wasm.explorerTabInstall();
}

/**
 * @returns {Promise<void>}
 */
export function explorerInitTab() {
    const ret = wasm.explorerInitTab();
    return ret;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function explorerRawExportStringV2(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.explorerRawExportStringV2(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @param {number} digits
 * @returns {string}
 */
export function explorerRawExportNumberV2(value, digits) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.explorerRawExportNumberV2(value, digits);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function explorerRawExportTxUrlV2(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.explorerRawExportTxUrlV2(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function explorerRawExportAddressUrlV2(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.explorerRawExportAddressUrlV2(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {Array<any>} values
 * @returns {string}
 */
export function explorerRawExportJoinAddressesV2(values) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.explorerRawExportJoinAddressesV2(values);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} row
 * @returns {any}
 */
export function explorerRawExportNormalizeRawTxV2(row) {
    const ret = wasm.explorerRawExportNormalizeRawTxV2(row);
    return ret;
}

/**
 * @param {any} section
 * @returns {Promise<any>}
 */
export function explorerBuildRawExportTableV2(section) {
    const ret = wasm.explorerBuildRawExportTableV2(section);
    return ret;
}

export function explorerExportInstall() {
    wasm.explorerExportInstall();
}

/**
 * @param {any} section
 */
export function explorerOpenBlockExplorer(section) {
    wasm.explorerOpenBlockExplorer(section);
}

/**
 * @param {any} section
 */
export function explorerExportCsv(section) {
    wasm.explorerExportCsv(section);
}

/**
 * @param {any} section
 */
export function explorerExportHtml(section) {
    wasm.explorerExportHtml(section);
}

/**
 * @param {any} section
 */
export function explorerExportPdf(section) {
    wasm.explorerExportPdf(section);
}

/**
 * @param {string} net
 * @returns {string}
 */
export function bridgeCommandInlineStateKeyR7(net) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeCommandInlineStateKeyR7(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {any}
 */
export function bridgeCommandInlineStateR7(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeCommandInlineStateR7(ptr0, len0);
    return ret;
}

/**
 * @param {string} net
 * @param {string} name
 * @returns {boolean}
 */
export function bridgeCommandOptionEnabledR7(net, name) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeCommandOptionEnabledR7(ptr0, len0, ptr1, len1);
    return ret !== 0;
}

/**
 * @param {string} net
 * @returns {boolean}
 */
export function bridgeHasConfig(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeHasConfig(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {any} instance_id
 * @param {string} name
 * @param {boolean} enabled
 * @returns {string}
 */
export function bridgeInstanceCommandSetOptionR13B(net, instance_id, name, enabled) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeInstanceCommandSetOptionR13B(ptr0, len0, instance_id, ptr1, len1, enabled);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {string} net
 * @param {any} instance_id
 * @param {string} name
 * @param {any} record
 * @returns {string}
 */
export function bridgeInstanceCommandCheckboxR13B(net, instance_id, name, record) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeInstanceCommandCheckboxR13B(ptr0, len0, instance_id, ptr1, len1, record);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {any} bridge_instances
 * @param {string} net
 * @param {any} instance_id
 * @param {string} name
 * @returns {string}
 */
export function bridgeInstanceCommandCheckboxFromInstancesR13B(bridge_instances, net, instance_id, name) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeInstanceCommandCheckboxFromInstancesR13B(bridge_instances, ptr0, len0, instance_id, ptr1, len1);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {string} net
 * @param {string} name
 * @returns {boolean}
 */
export function bridgeCommandShouldIncludeR7(net, name) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeCommandOptionEnabledR7(ptr0, len0, ptr1, len1);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {string} name
 * @param {boolean} enabled
 * @returns {boolean}
 */
export function bridgeCommandSetOptionR7(net, name, enabled) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeCommandSetOptionR7(ptr0, len0, ptr1, len1, enabled);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {string} name
 * @returns {boolean}
 */
export function bridgeCommandToggleOptionR7(net, name) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeCommandToggleOptionR7(ptr0, len0, ptr1, len1);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {string} name
 * @returns {string}
 */
export function bridgeCommandInlineToggleR7(net, name) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeCommandInlineToggleR7(ptr0, len0, ptr1, len1);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {string} net
 * @param {any} instance_id
 * @param {string} name
 * @returns {string}
 */
export function bridgeInstanceCommandStateKeyR13B(net, instance_id, name) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeInstanceCommandStateKeyR13B(ptr0, len0, instance_id, ptr1, len1);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {string} net
 * @param {any} instance_id
 * @param {string} name
 * @param {any} record
 * @returns {boolean}
 */
export function bridgeInstanceCommandOptionEnabledR13B(net, instance_id, name, record) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeInstanceCommandOptionEnabledR13B(ptr0, len0, instance_id, ptr1, len1, record);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {any} instance_id
 * @param {string} name
 * @param {any} record
 * @returns {boolean}
 */
export function bridgeInstanceCommandShouldIncludeR13B(net, instance_id, name, record) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeInstanceCommandShouldIncludeR13B(ptr0, len0, instance_id, ptr1, len1, record);
    return ret !== 0;
}

/**
 * @param {any} bridge_instances
 * @param {string} net
 * @param {any} instance_id
 * @param {string} name
 * @returns {boolean}
 */
export function bridgeInstanceCommandShouldIncludeFromInstancesR13B(bridge_instances, net, instance_id, name) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeInstanceCommandShouldIncludeFromInstancesR13B(bridge_instances, ptr0, len0, instance_id, ptr1, len1);
    return ret !== 0;
}

/**
 * @param {string} active_net
 * @param {any} validation
 * @param {any} bridge_instances
 * @returns {any}
 */
export function bridgePlanPortAutofixR37(active_net, validation, bridge_instances) {
    const ptr0 = passStringToWasm0(active_net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgePlanPortAutofixR37(ptr0, len0, validation, bridge_instances);
    return ret;
}

/**
 * @param {string} net
 * @param {string} kind
 * @param {any} port
 * @returns {boolean}
 */
export function bridgePortIsInsideAnyKnownRangeR91(net, kind, port) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgePortIsInsideAnyKnownRangeR91(ptr0, len0, ptr1, len1, port);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {string} kind
 * @param {any} value
 * @returns {boolean}
 */
export function bridgeInstancePortShouldFollowExternalRangeR91(net, kind, value) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeInstancePortShouldFollowExternalRangeR91(ptr0, len0, ptr1, len1, value);
    return ret !== 0;
}

/**
 * @param {any} used
 * @param {any} value
 */
export function bridgeAddUsedPortR91(used, value) {
    wasm.bridgeAddUsedPortR91(used, value);
}

/**
 * @param {any} validation
 * @returns {string}
 */
export function bridgePortConflictMessageR33(validation) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgePortConflictMessageR33(validation);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} net
 * @param {string} phase
 * @param {any} validation
 * @param {any} details
 * @returns {boolean}
 */
export function bridgeTracePortConflictR33(net, phase, validation, details) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(phase, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeTracePortConflictR33(ptr0, len0, ptr1, len1, validation, details);
    return ret !== 0;
}

/**
 * @param {string} net
 * @returns {Array<any>}
 */
export function bridgeStartButtonsForNetR33(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeStartButtonsForNetR33(ptr0, len0);
    return ret;
}

/**
 * @param {string} net
 * @param {any} validation
 * @param {string} reason
 * @returns {any}
 */
export function bridgeApplyPortConflictStartStateR33(net, validation, reason) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeApplyPortConflictStartStateR33(ptr0, len0, validation, ptr1, len1);
    return ret;
}

/**
 * @param {any} owner
 * @returns {string}
 */
export function bridgeInstanceIdFromOwnerR37(owner) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeInstanceIdFromOwnerR37(owner);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function bridgeNormalizePortR37(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeNormalizePortR37(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} instance
 * @param {any} conflict_port
 * @returns {string}
 */
export function bridgeInstancePortKindForConflictR37(instance, conflict_port) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeInstancePortKindForConflictR37(instance, conflict_port);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} change
 * @returns {string}
 */
export function bridgeAutofixChangeKeyR37(change) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeAutofixChangeKeyR37(change);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} owner
 * @returns {string}
 */
export function bridgeOwnerKeyR45(owner) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeOwnerKeyR45(owner);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} owners
 * @returns {Array<any>}
 */
export function bridgeUniqueConflictOwnersR45(owners) {
    const ret = wasm.bridgeUniqueConflictOwnersR45(owners);
    return ret;
}

/**
 * @param {string} active_net
 * @param {any} owners
 * @returns {Array<any>}
 */
export function bridgeOwnersToAutofixR45(active_net, owners) {
    const ptr0 = passStringToWasm0(active_net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeOwnersToAutofixR45(ptr0, len0, owners);
    return ret;
}

/**
 * @param {Array<any>} items
 * @param {string} active_net
 * @returns {any}
 */
export function bridgeValidatePortConflictsR5(items, active_net) {
    const ptr0 = passStringToWasm0(active_net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeValidatePortConflictsR5(items, ptr0, len0);
    return ret;
}

/**
 * @param {any} validation
 * @returns {Array<any>}
 */
export function bridgePortConflictCompactSummaryR33(validation) {
    const ret = wasm.bridgePortConflictCompactSummaryR33(validation);
    return ret;
}

/**
 * @param {any} change
 * @param {any} planned_used
 * @param {any} bridge_instances
 * @param {any} collected_records
 * @returns {string}
 */
export function bridgeChooseReplacementPortR37(change, planned_used, bridge_instances, collected_records) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ret = wasm.bridgeChooseReplacementPortR37(change, planned_used, bridge_instances, collected_records);
        var ptr1 = ret[0];
        var len1 = ret[1];
        if (ret[3]) {
            ptr1 = 0; len1 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred2_0 = ptr1;
        deferred2_1 = len1;
        return getStringFromWasm0(ptr1, len1);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {any} bridge_instances
 * @param {string} net
 * @param {string} instance_id
 * @param {string} kind
 * @param {string} new_port
 * @returns {boolean}
 */
export function bridgeWriteInstancePortR37(bridge_instances, net, instance_id, kind, new_port) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(instance_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ptr2 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len2 = WASM_VECTOR_LEN;
    const ptr3 = passStringToWasm0(new_port, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len3 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeWriteInstancePortR37(bridge_instances, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3);
    return ret !== 0;
}

/**
 * @param {string} active_net
 * @param {any} validation
 * @param {any} bridge_instances
 * @param {any} collected_records
 * @param {number} max_passes
 * @returns {any}
 */
export function bridgeApplyPortAutofixR37(active_net, validation, bridge_instances, collected_records, max_passes) {
    const ptr0 = passStringToWasm0(active_net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeApplyPortAutofixR37(ptr0, len0, validation, bridge_instances, collected_records, max_passes);
    return ret;
}

/**
 * @param {any} start_port
 * @param {any} used_ports
 * @returns {string}
 */
export function bridgeFindNearestUnusedPortR9(start_port, used_ports) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ret = wasm.bridgeFindNearestUnusedPortR9(start_port, used_ports);
        var ptr1 = ret[0];
        var len1 = ret[1];
        if (ret[3]) {
            ptr1 = 0; len1 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred2_0 = ptr1;
        deferred2_1 = len1;
        return getStringFromWasm0(ptr1, len1);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function bridgeNormalizePortLiteralR91(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeNormalizePortLiteralR91(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} net
 * @param {string} kind
 * @param {any} fallback_range
 * @returns {string}
 */
export function bridgeExternalBasePortR91(net, kind, fallback_range) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeExternalBasePortR91(ptr0, len0, ptr1, len1, fallback_range);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {any} base_port
 * @param {any} fallback_range
 * @returns {any}
 */
export function bridgeRangeFromExternalBaseR91(base_port, fallback_range) {
    const ret = wasm.bridgeRangeFromExternalBaseR91(base_port, fallback_range);
    return ret;
}

/**
 * @param {string} net
 * @returns {any}
 */
export function bridgePortProfileR35B(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgePortProfileR35B(ptr0, len0);
    return ret;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function bridgeNormalizePortSoftR35B(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeNormalizePortSoftR35B(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} port
 * @param {any} range
 * @returns {boolean}
 */
export function bridgePortInRangeR35B(port, range) {
    const ret = wasm.bridgePortInRangeR35B(port, range);
    return ret !== 0;
}

/**
 * @param {any} range
 * @param {any} used_ports
 * @param {any} fallback_start
 * @returns {string}
 */
export function bridgeFindUnusedPortInRangeR35B(range, used_ports, fallback_start) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeFindUnusedPortInRangeR35B(range, used_ports, fallback_start);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} net
 * @param {string} kind
 * @param {any} used_ports
 * @param {any} fallback_start
 * @returns {string}
 */
export function bridgeFindRecommendedOrNearestUnusedPortR35B(net, kind, used_ports, fallback_start) {
    let deferred4_0;
    let deferred4_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(kind, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeFindRecommendedOrNearestUnusedPortR35B(ptr0, len0, ptr1, len1, used_ports, fallback_start);
        var ptr3 = ret[0];
        var len3 = ret[1];
        if (ret[3]) {
            ptr3 = 0; len3 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred4_0 = ptr3;
        deferred4_1 = len3;
        return getStringFromWasm0(ptr3, len3);
    } finally {
        wasm.__wbindgen_free(deferred4_0, deferred4_1, 1);
    }
}

/**
 * @returns {any}
 */
export function bridgePortProfilesR35B() {
    const ret = wasm.bridgePortProfilesR35B();
    return ret;
}

/**
 * @param {string} net
 * @returns {any}
 */
export function bridgeStaticPortProfileR91(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeStaticPortProfileR91(ptr0, len0);
    return ret;
}

/**
 * @param {any} value
 * @returns {Array<any>}
 */
export function bridgeExtractPortsFromTextR5(value) {
    const ret = wasm.bridgeExtractPortsFromTextR5(value);
    return ret;
}

/**
 * @param {Array<any>} items
 * @param {any} port
 * @param {any} role
 * @param {any} owner
 * @param {any} net
 */
export function bridgePushPortR5(items, port, role, owner, net) {
    wasm.bridgePushPortR5(items, port, role, owner, net);
}

/**
 * @param {any} item
 * @returns {string}
 */
export function bridgePortConflictLogicalKeyR64F(item) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgePortConflictLogicalKeyR64F(item);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} owners
 * @returns {boolean}
 */
export function bridgePortOwnersRepresentSameLogicalEndpointR64F(owners) {
    const ret = wasm.bridgePortOwnersRepresentSameLogicalEndpointR64F(owners);
    return ret !== 0;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function bridgeNormalizePortR9(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeNormalizePortR9(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {boolean}
 */
export function bridgePortIsValidR9(value) {
    const ret = wasm.bridgePortIsValidR9(value);
    return ret !== 0;
}

/**
 * @param {string} net
 * @returns {string}
 */
export function bridgeCurrentNodeModeFromUiR65F(net) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeCurrentNodeModeFromUiR65F(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {any} value
 * @param {any} fallback
 * @returns {string}
 */
export function bridgeInstanceNetworkKeyR15(value, fallback) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeInstanceNetworkKeyR15(value, fallback);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {boolean}
 */
export function bridgeNetworkEnabled(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeNetworkEnabled(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {boolean} enabled
 */
export function bridgeSetNetworkEnabled(net, enabled) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.bridgeSetNetworkEnabled(ptr0, len0, enabled);
}

/**
 * @param {string} net
 * @param {Function} update_command
 * @returns {Promise<any>}
 */
export function bridgeApplyRustyKaspaRootOnlyDefaultPathsR5(net, update_command) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeApplyRustyKaspaRootOnlyDefaultPathsR5(ptr0, len0, update_command);
    return ret;
}

/**
 * @param {string} net
 * @param {Function} update_command
 */
export function bridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(net, update_command) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.bridgeApplyRustyKaspaRootOnlyDefaultPathsSoonR5(ptr0, len0, update_command);
}

/**
 * @param {string} net
 * @returns {string}
 */
export function bridgeNetworkPolicyMessage(net) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeNetworkPolicyMessage(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} id
 * @returns {any}
 */
export function bridgeById(id) {
    const ptr0 = passStringToWasm0(id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeById(ptr0, len0);
    return ret;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function bridgeEscapeHtml(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeEscapeHtml(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} net
 * @param {string} name
 * @returns {string}
 */
export function bridgeElementId(net, name) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeElementId(ptr0, len0, ptr1, len1);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {string} net
 * @param {any} instance_id
 * @param {string} name
 * @returns {string}
 */
export function bridgeInstanceElementId(net, instance_id, name) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeInstanceElementId(ptr0, len0, instance_id, ptr1, len1);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {string} net
 * @param {string} name
 * @returns {string}
 */
export function bridgeValue(net, name) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len1 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeValue(ptr0, len0, ptr1, len1);
        deferred3_0 = ret[0];
        deferred3_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {string}
 */
export function bridgeNodeMode(net) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeNodeMode(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} net
 * @param {string} name
 * @returns {boolean}
 */
export function bridgeChecked(net, name) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(name, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeChecked(ptr0, len0, ptr1, len1);
    return ret !== 0;
}

/**
 * @param {any} net
 * @param {any} action
 * @param {any} phase
 * @param {any} details
 * @returns {boolean}
 */
export function bridgeSmallOwnerTraceR44D(net, action, phase, details) {
    const ret = wasm.bridgeSmallOwnerTraceR44D(net, action, phase, details);
    return ret !== 0;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function bridgePlainPortOnlyValueR98(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgePlainPortOnlyValueR98(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} left
 * @param {any} right
 * @returns {boolean}
 */
export function bridgeSamePortValueR98(left, right) {
    const ret = wasm.bridgeSamePortValueR98(left, right);
    return ret !== 0;
}

/**
 * @param {string} key
 * @param {any} value
 */
export function bridgeR51Store(key, value) {
    const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeR51Store(ptr0, len0, value);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @returns {string}
 */
export function bridgeReadLastNetwork() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeReadLastNetwork();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} net
 * @returns {string}
 */
export function bridgeSaveLastNetwork(net) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeSaveLastNetwork(net);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} key
 * @param {string} fallback
 * @returns {any}
 */
export function bridgeTranslateRuntimeFeedback(key, fallback) {
    const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(fallback, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeTranslateRuntimeFeedback(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * @param {string} key
 * @param {string} fallback
 * @returns {any}
 */
export function bridgeI18nTextR41(key, fallback) {
    const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(fallback, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeI18nTextR41(ptr0, len0, ptr1, len1);
    return ret;
}

/**
 * @param {string} command
 * @param {any} payload
 * @returns {Promise<any>}
 */
export function bridgeBackendInvokeR5(command, payload) {
    const ptr0 = passStringToWasm0(command, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeBackendInvokeR5(ptr0, len0, payload);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @returns {Array<any>}
 */
export function bridgeNetworkProfiles() {
    const ret = wasm.bridgeNetworkProfiles();
    return ret;
}

/**
 * @returns {Array<any>}
 */
export function bridgeR51Keys() {
    const ret = wasm.bridgeR51Keys();
    return ret;
}

/**
 * @param {string} net
 * @returns {string}
 */
export function bridgeNetworkPolicyKey(net) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeNetworkPolicyKey(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {any}
 */
export function bridgeNetworkProfile(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeNetworkProfile(ptr0, len0);
    return ret;
}

/**
 * @param {string} key
 * @returns {any}
 */
export function bridgeR51Load(key) {
    const ptr0 = passStringToWasm0(key, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeR51Load(ptr0, len0);
    return ret;
}

/**
 * @param {string} net
 * @returns {boolean}
 */
export function bridgeLogAutoScrollEnabled(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeLogAutoScrollEnabled(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {string} net
 * @param {boolean} enabled
 */
export function bridgeSetLogAutoScroll(net, enabled) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    wasm.bridgeSetLogAutoScroll(ptr0, len0, enabled);
}

export function bridgeInstallLogAutoScrollControls() {
    wasm.bridgeInstallLogAutoScrollControls();
}

/**
 * @param {any} value
 * @returns {string}
 */
export function bridgeNormalizeInnerTab(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeNormalizeInnerTab(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {string}
 */
export function bridgeResolveInnerTab(net) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeResolveInnerTab(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} net
 * @param {any} selected
 * @returns {string}
 */
export function bridgeSaveInnerTab(net, selected) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeSaveInnerTab(ptr0, len0, selected);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function bridgeNormalizeNetwork(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeNormalizeNetwork(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} net
 * @returns {any}
 */
export function bridgeR51Panel(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeR51Panel(ptr0, len0);
    return ret;
}

/**
 * @param {string} net
 * @returns {Array<any>}
 */
export function bridgeR51Fields(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeR51Fields(ptr0, len0);
    return ret;
}

/**
 * @param {string} net
 * @param {any} callbacks
 * @returns {any}
 */
export function bridgeR51ReadSettings(net, callbacks) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeR51ReadSettings(ptr0, len0, callbacks);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {string} net
 * @param {any} values
 * @param {any} bridge_instances
 * @param {any} active_instance
 * @param {any} callbacks
 */
export function bridgeR51WriteSettings(net, values, bridge_instances, active_instance, callbacks) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeR51WriteSettings(ptr0, len0, values, bridge_instances, active_instance, callbacks);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

/**
 * @param {any} id_value
 * @returns {any}
 */
export function bridgeDefaultInstanceRecord(id_value) {
    const ret = wasm.bridgeDefaultInstanceRecord(id_value);
    return ret;
}

/**
 * @param {any} raw
 * @param {any} fallback_id
 * @returns {any}
 */
export function bridgeNormalizeInstanceRecord(raw, fallback_id) {
    const ret = wasm.bridgeNormalizeInstanceRecord(raw, fallback_id);
    return ret;
}

/**
 * @param {string} net
 * @param {any} instance
 * @returns {string}
 */
export function bridgeBuildUpstreamInstanceArg(net, instance) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeBuildUpstreamInstanceArg(ptr0, len0, instance);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} net
 * @param {any} structured_instances
 * @returns {any}
 */
export function bridgeEffectiveSettingsV1(net, structured_instances) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeEffectiveSettingsV1(ptr0, len0, structured_instances);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {string} net
 * @returns {any}
 */
export function bridgeEffectiveInprocessNodeSettings(net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeEffectiveInprocessNodeSettings(ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} value
 * @returns {any}
 */
export function bridgeInstanceParseStructured(value) {
    const ret = wasm.bridgeInstanceParseStructured(value);
    return ret;
}

/**
 * @param {any} value
 * @returns {string}
 */
export function bridgeInstancePortValue(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeInstancePortValue(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {string}
 */
export function bridgeInstancePlainValue(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.bridgeInstancePlainValue(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {any} value
 * @returns {any}
 */
export function bridgeOptionalTextV1(value) {
    const ret = wasm.bridgeOptionalTextV1(value);
    return ret;
}

/**
 * @param {any} value
 * @param {any} fallback
 * @returns {any}
 */
export function bridgePortListenV1(value, fallback) {
    const ret = wasm.bridgePortListenV1(value, fallback);
    return ret;
}

/**
 * @param {string} label
 * @param {any} value
 * @param {any} fallback
 * @param {number} max
 * @returns {any}
 */
export function bridgeParseUnsignedV1(label, value, fallback, max) {
    const ptr0 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeParseUnsignedV1(ptr0, len0, value, fallback, max);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {string} label
 * @param {any} value
 * @param {any} fallback
 * @returns {any}
 */
export function bridgeParseDurationMsV1(label, value, fallback) {
    const ptr0 = passStringToWasm0(label, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeParseDurationMsV1(ptr0, len0, value, fallback);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} value
 * @param {any} fallback
 * @returns {any}
 */
export function bridgeBoolValueV1(value, fallback) {
    const ret = wasm.bridgeBoolValueV1(value, fallback);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

export function settingsUiInitTab() {
    wasm.settingsUiInitTab();
}

/**
 * @param {any} prefs
 * @param {string} reason
 * @returns {any}
 */
export function shellDisplayApplyDirect(prefs, reason) {
    const ptr0 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.shellDisplayApplyDirect(prefs, ptr0, len0);
    return ret;
}

export function shellDisplayInstall() {
    wasm.shellDisplayInstall();
}

/**
 * @param {any} tabs
 */
export function shellRuntimeInstall(tabs) {
    wasm.shellRuntimeInstall(tabs);
}

/**
 * @param {string} tab_id
 * @param {any} options
 * @returns {Promise<boolean>}
 */
export function shellRuntimeOpenTab(tab_id, options) {
    const ptr0 = passStringToWasm0(tab_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.shellRuntimeOpenTab(ptr0, len0, options);
    return ret;
}

/**
 * @param {string} tab_id
 * @returns {string}
 */
export function shellRuntimeActivateTab(tab_id) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(tab_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.shellRuntimeActivateTab(ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {string} theme
 * @returns {string}
 */
export function shellRuntimeApplyTheme(theme) {
    let deferred2_0;
    let deferred2_1;
    try {
        const ptr0 = passStringToWasm0(theme, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.shellRuntimeApplyTheme(ptr0, len0);
        deferred2_0 = ret[0];
        deferred2_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred2_0, deferred2_1, 1);
    }
}

/**
 * @param {string} reason
 * @returns {boolean}
 */
export function shellRuntimeScheduleSavedRestore(reason) {
    const ptr0 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.shellRuntimeScheduleSavedRestore(ptr0, len0);
    return ret !== 0;
}

/**
 * @param {string} tab_id
 * @returns {number}
 */
export function shellRuntimeRecordExplicitNavigation(tab_id) {
    const ptr0 = passStringToWasm0(tab_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.shellRuntimeRecordExplicitNavigation(ptr0, len0);
    return ret >>> 0;
}

/**
 * @returns {string}
 */
export function shellRuntimeSavedMainTab() {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.shellRuntimeSavedMainTab();
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
}

/**
 * @param {string} tab_id
 * @param {string} phase
 * @param {any} details
 */
export function shellRuntimeTraceTab(tab_id, phase, details) {
    const ptr0 = passStringToWasm0(tab_id, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(phase, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    wasm.shellRuntimeTraceTab(ptr0, len0, ptr1, len1, details);
}

export function settingsDiagnosticsInstall() {
    wasm.settingsDiagnosticsInstall();
}

/**
 * @param {any} bridge_instances
 * @param {any} active_instance
 * @param {string} net
 * @returns {string}
 */
export function bridgeActiveRawLogInstanceId(bridge_instances, active_instance, net) {
    let deferred3_0;
    let deferred3_1;
    try {
        const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.bridgeActiveRawLogInstanceId(bridge_instances, active_instance, ptr0, len0);
        var ptr2 = ret[0];
        var len2 = ret[1];
        if (ret[3]) {
            ptr2 = 0; len2 = 0;
            throw takeFromExternrefTable0(ret[2]);
        }
        deferred3_0 = ptr2;
        deferred3_1 = len2;
        return getStringFromWasm0(ptr2, len2);
    } finally {
        wasm.__wbindgen_free(deferred3_0, deferred3_1, 1);
    }
}

/**
 * @param {any} bridge_instances
 * @param {any} active_instance
 * @returns {Array<any>}
 */
export function bridgeCollectConfiguredPortsR5(bridge_instances, active_instance) {
    const ret = wasm.bridgeCollectConfiguredPortsR5(bridge_instances, active_instance);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} bridge_instances
 * @param {string} skip_net
 * @param {any} skip_instance_id
 * @returns {any}
 */
export function bridgeUsedPortSetR9(bridge_instances, skip_net, skip_instance_id) {
    const ptr0 = passStringToWasm0(skip_net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeUsedPortSetR9(bridge_instances, ptr0, len0, skip_instance_id);
    return ret;
}

/**
 * @param {any} bridge_instances
 * @param {string} active_net
 * @returns {any}
 */
export function bridgeUsedPortSetExcludingNetworkInstancesR91(bridge_instances, active_net) {
    const ptr0 = passStringToWasm0(active_net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeUsedPortSetExcludingNetworkInstancesR91(bridge_instances, ptr0, len0);
    return ret;
}

/**
 * @param {any} bridge_instances
 * @param {string} net
 * @param {any} instance
 * @returns {any}
 */
export function bridgeAssignMissingInstancePortsR9(bridge_instances, net, instance) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeAssignMissingInstancePortsR9(bridge_instances, ptr0, len0, instance);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} bridge_instances
 * @param {string} net
 * @param {string} reason
 * @returns {boolean}
 */
export function bridgeReassignInstancePortsFromExternalRangeR91(bridge_instances, net, reason) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(reason, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeReassignInstancePortsFromExternalRangeR91(bridge_instances, ptr0, len0, ptr1, len1);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return ret[0] !== 0;
}

/**
 * @param {any} bridge_instances
 * @param {string} net
 * @returns {any}
 */
export function bridgeCreateInstanceRecordR9(bridge_instances, net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeCreateInstanceRecordR9(bridge_instances, ptr0, len0);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

/**
 * @param {any} bridge_instances
 * @param {any} active_instance
 * @param {string} net
 */
export function bridgeEnsureInstanceState(bridge_instances, active_instance, net) {
    const ptr0 = passStringToWasm0(net, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.bridgeEnsureInstanceState(bridge_instances, active_instance, ptr0, len0);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
}

function __wbg_adapter_40(arg0, arg1) {
    wasm._dyn_core_ed718c3d60ebd546___ops__function__FnMut_____Output______as_wasm_bindgen_1ba7c375a52abd4d___closure__WasmClosure___describe__invoke______(arg0, arg1);
}

function __wbg_adapter_43(arg0, arg1, arg2, arg3, arg4) {
    wasm.closure701_externref_shim(arg0, arg1, arg2, arg3, arg4);
}

function __wbg_adapter_46(arg0, arg1, arg2, arg3) {
    const ret = wasm.closure705_externref_shim(arg0, arg1, arg2, arg3);
    return ret;
}

function __wbg_adapter_49(arg0, arg1, arg2, arg3) {
    wasm.closure708_externref_shim(arg0, arg1, arg2, arg3);
}

function __wbg_adapter_52(arg0, arg1, arg2) {
    const ret = wasm.closure711_externref_shim(arg0, arg1, arg2);
    return ret;
}

function __wbg_adapter_57(arg0, arg1, arg2) {
    const ret = wasm.closure715_externref_shim(arg0, arg1, arg2);
    return ret !== 0;
}

function __wbg_adapter_60(arg0, arg1) {
    const ret = wasm.closure718_externref_shim(arg0, arg1);
    return ret;
}

function __wbg_adapter_63(arg0, arg1) {
    const ret = wasm._dyn_core_ed718c3d60ebd546___ops__function__FnMut_____Output______as_wasm_bindgen_1ba7c375a52abd4d___closure__WasmClosure___describe__invoke___bool_(arg0, arg1);
    return ret !== 0;
}

function __wbg_adapter_66(arg0, arg1, arg2) {
    wasm.closure1135_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_741(arg0, arg1, arg2, arg3) {
    wasm.closure1159_externref_shim(arg0, arg1, arg2, arg3);
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);

            } catch (e) {
                if (module.headers.get('Content-Type') != 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else {
                    throw e;
                }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);

    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };

        } else {
            return instance;
        }
    }
}

function __wbg_get_imports() {
    const imports = {};
    imports.wbg = {};
    imports.wbg.__wbg_Boolean_156dadba361eb4de = function(arg0) {
        const ret = Boolean(arg0);
        return ret;
    };
    imports.wbg.__wbg_Number_a11ec3febff3cf8a = function() { return handleError(function (arg0) {
        const ret = Number(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_Number_a8498279eca758ed = function(arg0) {
        const ret = Number(arg0);
        return ret;
    };
    imports.wbg.__wbg_Object_03066f860601d582 = function(arg0) {
        const ret = Object(arg0);
        return ret;
    };
    imports.wbg.__wbg_String_0688f1288e3f182b = function() { return handleError(function (arg0) {
        const ret = String(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_String_919110ca02bcc15b = function(arg0) {
        const ret = String(arg0);
        return ret;
    };
    imports.wbg.__wbg_add_883d9432f9188ef2 = function(arg0, arg1) {
        const ret = arg0.add(arg1);
        return ret;
    };
    imports.wbg.__wbg_apply_36be6a55257c99bf = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.apply(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_apply_eb9e9b97497f91e4 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = Reflect.apply(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_buffer_609cc3eee51ed158 = function(arg0) {
        const ret = arg0.buffer;
        return ret;
    };
    imports.wbg.__wbg_call_672a4d21634d4a24 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.call(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_call_7cccdd69e0791ae2 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = arg0.call(arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_call_833bed5770ea2041 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
        const ret = arg0.call(arg1, arg2, arg3);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_call_b8adc8b1d0a0d8eb = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4) {
        const ret = arg0.call(arg1, arg2, arg3, arg4);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_catch_a6e601879b2610e9 = function(arg0, arg1) {
        const ret = arg0.catch(arg1);
        return ret;
    };
    imports.wbg.__wbg_construct_b91ff0e53b60c0c3 = function() { return handleError(function (arg0, arg1) {
        const ret = Reflect.construct(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_deleteProperty_96363d4a1d977c97 = function() { return handleError(function (arg0, arg1) {
        const ret = Reflect.deleteProperty(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_delete_d6860deb47204f3b = function(arg0, arg1) {
        const ret = arg0.delete(arg1);
        return ret;
    };
    imports.wbg.__wbg_entries_3265d4158b33e5dc = function(arg0) {
        const ret = Object.entries(arg0);
        return ret;
    };
    imports.wbg.__wbg_exec_3e2d2d0644c927df = function(arg0, arg1, arg2) {
        const ret = arg0.exec(getStringFromWasm0(arg1, arg2));
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_format_0545b83dc1d8a934 = function(arg0) {
        const ret = arg0.format;
        return ret;
    };
    imports.wbg.__wbg_freeze_ef6d70cf38e8d948 = function(arg0) {
        const ret = Object.freeze(arg0);
        return ret;
    };
    imports.wbg.__wbg_from_2a5d3e218e67aa85 = function(arg0) {
        const ret = Array.from(arg0);
        return ret;
    };
    imports.wbg.__wbg_getDate_ef336e14594b35ce = function(arg0) {
        const ret = arg0.getDate();
        return ret;
    };
    imports.wbg.__wbg_getDay_3da98b461c969439 = function(arg0) {
        const ret = arg0.getDay();
        return ret;
    };
    imports.wbg.__wbg_getFullYear_17d3c9e4db748eb7 = function(arg0) {
        const ret = arg0.getFullYear();
        return ret;
    };
    imports.wbg.__wbg_getHours_70451b8de3ce8638 = function(arg0) {
        const ret = arg0.getHours();
        return ret;
    };
    imports.wbg.__wbg_getMinutes_e793d718371e18f7 = function(arg0) {
        const ret = arg0.getMinutes();
        return ret;
    };
    imports.wbg.__wbg_getMonth_d37edcd23642c97d = function(arg0) {
        const ret = arg0.getMonth();
        return ret;
    };
    imports.wbg.__wbg_getSeconds_755197b634cca692 = function(arg0) {
        const ret = arg0.getSeconds();
        return ret;
    };
    imports.wbg.__wbg_getTime_46267b1c24877e30 = function(arg0) {
        const ret = arg0.getTime();
        return ret;
    };
    imports.wbg.__wbg_get_67b2ba62fc30de12 = function() { return handleError(function (arg0, arg1) {
        const ret = Reflect.get(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_get_b9b93047fe3cf45b = function(arg0, arg1) {
        const ret = arg0[arg1 >>> 0];
        return ret;
    };
    imports.wbg.__wbg_hasOwnProperty_eb9a168e9990a716 = function(arg0, arg1) {
        const ret = arg0.hasOwnProperty(arg1);
        return ret;
    };
    imports.wbg.__wbg_has_a5ea9117f258a0ec = function() { return handleError(function (arg0, arg1) {
        const ret = Reflect.has(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_instanceof_Error_4d54113b22d20306 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Error;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Object_7f2dcef8f78644a4 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Object;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Promise_935168b8f4b49db3 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Promise;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_instanceof_Set_f48781e4bf8ffb09 = function(arg0) {
        let result;
        try {
            result = arg0 instanceof Set;
        } catch (_) {
            result = false;
        }
        const ret = result;
        return ret;
    };
    imports.wbg.__wbg_isArray_a1eab7e0d067391b = function(arg0) {
        const ret = Array.isArray(arg0);
        return ret;
    };
    imports.wbg.__wbg_is_c7481c65e7e5df9e = function(arg0, arg1) {
        const ret = Object.is(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbg_keys_5c77a08ddc2fb8a6 = function(arg0) {
        const ret = Object.keys(arg0);
        return ret;
    };
    imports.wbg.__wbg_length_a446193dc22c12f8 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_length_e2d2a49132c1b256 = function(arg0) {
        const ret = arg0.length;
        return ret;
    };
    imports.wbg.__wbg_new0_f788a2397c7ca929 = function() {
        const ret = new Date();
        return ret;
    };
    imports.wbg.__wbg_new_08dc65a1d6785f11 = function(arg0, arg1) {
        const ret = new Intl.NumberFormat(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbg_new_23a2665fac83c611 = function(arg0, arg1) {
        try {
            var state0 = {a: arg0, b: arg1};
            var cb0 = (arg0, arg1) => {
                const a = state0.a;
                state0.a = 0;
                try {
                    return __wbg_adapter_741(a, state0.b, arg0, arg1);
                } finally {
                    state0.a = a;
                }
            };
            const ret = new Promise(cb0);
            return ret;
        } finally {
            state0.a = state0.b = 0;
        }
    };
    imports.wbg.__wbg_new_31a97dac4f10fab7 = function(arg0) {
        const ret = new Date(arg0);
        return ret;
    };
    imports.wbg.__wbg_new_405e22f390576ce2 = function() {
        const ret = new Object();
        return ret;
    };
    imports.wbg.__wbg_new_5e0be73521bc8c17 = function() {
        const ret = new Map();
        return ret;
    };
    imports.wbg.__wbg_new_63847613cde5d4bc = function(arg0, arg1, arg2, arg3) {
        const ret = new RegExp(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
        return ret;
    };
    imports.wbg.__wbg_new_78feb108b6472713 = function() {
        const ret = new Array();
        return ret;
    };
    imports.wbg.__wbg_new_a12002a7f91c75be = function(arg0) {
        const ret = new Uint8Array(arg0);
        return ret;
    };
    imports.wbg.__wbg_new_a239edaa1dc2968f = function(arg0) {
        const ret = new Set(arg0);
        return ret;
    };
    imports.wbg.__wbg_new_b08a00743b8ae2f3 = function(arg0, arg1) {
        const ret = new TypeError(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_new_c68d7209be747379 = function(arg0, arg1) {
        const ret = new Error(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_newnoargs_105ed471475aaf50 = function(arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_newwithbyteoffsetandlength_d97e637ebe145a9a = function(arg0, arg1, arg2) {
        const ret = new Uint8Array(arg0, arg1 >>> 0, arg2 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_newwithyearmonthday_03748851282a850d = function(arg0, arg1, arg2) {
        const ret = new Date(arg0 >>> 0, arg1, arg2);
        return ret;
    };
    imports.wbg.__wbg_now_807e54c39636c349 = function() {
        const ret = Date.now();
        return ret;
    };
    imports.wbg.__wbg_parse_def2e24ef1252aff = function() { return handleError(function (arg0, arg1) {
        const ret = JSON.parse(getStringFromWasm0(arg0, arg1));
        return ret;
    }, arguments) };
    imports.wbg.__wbg_push_737cfc8c1432c2c6 = function(arg0, arg1) {
        const ret = arg0.push(arg1);
        return ret;
    };
    imports.wbg.__wbg_queueMicrotask_97d92b4fcc8a61c5 = function(arg0) {
        queueMicrotask(arg0);
    };
    imports.wbg.__wbg_queueMicrotask_d3219def82552485 = function(arg0) {
        const ret = arg0.queueMicrotask;
        return ret;
    };
    imports.wbg.__wbg_race_ace53f9902587e09 = function(arg0) {
        const ret = Promise.race(arg0);
        return ret;
    };
    imports.wbg.__wbg_random_3ad904d98382defe = function() {
        const ret = Math.random();
        return ret;
    };
    imports.wbg.__wbg_resolve_4851785c9c5f573d = function(arg0) {
        const ret = Promise.resolve(arg0);
        return ret;
    };
    imports.wbg.__wbg_round_d3c6f5f0c2d66c40 = function(arg0) {
        const ret = Math.round(arg0);
        return ret;
    };
    imports.wbg.__wbg_setDate_90491bc93d09cadd = function(arg0, arg1) {
        const ret = arg0.setDate(arg1 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_setTime_8afa2faa26e7eb59 = function(arg0, arg1) {
        const ret = arg0.setTime(arg1);
        return ret;
    };
    imports.wbg.__wbg_set_65595bdd868b3009 = function(arg0, arg1, arg2) {
        arg0.set(arg1, arg2 >>> 0);
    };
    imports.wbg.__wbg_set_bb8cecf6a62b9f46 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = Reflect.set(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_shift_9a41f00897c50537 = function(arg0) {
        const ret = arg0.shift();
        return ret;
    };
    imports.wbg.__wbg_slice_a18aced6f26168b6 = function(arg0, arg1, arg2) {
        const ret = arg0.slice(arg1 >>> 0, arg2 >>> 0);
        return ret;
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_88a902d13a557d07 = function() {
        const ret = typeof global === 'undefined' ? null : global;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_GLOBAL_THIS_56578be7e9f832b0 = function() {
        const ret = typeof globalThis === 'undefined' ? null : globalThis;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_SELF_37c5d418e4bf5819 = function() {
        const ret = typeof self === 'undefined' ? null : self;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_static_accessor_WINDOW_5de37043a91a9c40 = function() {
        const ret = typeof window === 'undefined' ? null : window;
        return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
    };
    imports.wbg.__wbg_stringify_b1b3844ae02664a1 = function() { return handleError(function (arg0, arg1) {
        const ret = JSON.stringify(arg0, arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_stringify_f7ed6987935b4a24 = function() { return handleError(function (arg0) {
        const ret = JSON.stringify(arg0);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_test_7f0ac7b9d67b7a48 = function(arg0, arg1, arg2) {
        const ret = arg0.test(getStringFromWasm0(arg1, arg2));
        return ret;
    };
    imports.wbg.__wbg_then_44b73946d2fb3e7d = function(arg0, arg1) {
        const ret = arg0.then(arg1);
        return ret;
    };
    imports.wbg.__wbg_then_48b406749878a531 = function(arg0, arg1, arg2) {
        const ret = arg0.then(arg1, arg2);
        return ret;
    };
    imports.wbg.__wbg_toFixed_43af7895cf202c17 = function() { return handleError(function (arg0, arg1) {
        const ret = arg0.toFixed(arg1);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_toISOString_b015155a5a6fe219 = function(arg0) {
        const ret = arg0.toISOString();
        return ret;
    };
    imports.wbg.__wbg_toPrimitive_693467a3eb50bdea = function() {
        const ret = Symbol.toPrimitive;
        return ret;
    };
    imports.wbg.__wbg_values_fcb8ba8c0aad8b58 = function(arg0) {
        const ret = Object.values(arg0);
        return ret;
    };
    imports.wbg.__wbindgen_boolean_get = function(arg0) {
        const v = arg0;
        const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
        return ret;
    };
    imports.wbg.__wbindgen_cb_drop = function(arg0) {
        const obj = arg0.original;
        if (obj.cnt-- == 1) {
            obj.a = 0;
            return true;
        }
        const ret = false;
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper2411 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 702, __wbg_adapter_40);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper2457 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 702, __wbg_adapter_43);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper2459 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 702, __wbg_adapter_46);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper2461 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 702, __wbg_adapter_49);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper2463 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 702, __wbg_adapter_52);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper2465 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 702, __wbg_adapter_52);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper2467 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 702, __wbg_adapter_57);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper2469 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 702, __wbg_adapter_60);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper2471 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 702, __wbg_adapter_63);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper4132 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 1136, __wbg_adapter_66);
        return ret;
    };
    imports.wbg.__wbindgen_init_externref_table = function() {
        const table = wasm.__wbindgen_export_2;
        const offset = table.grow(4);
        table.set(0, undefined);
        table.set(offset + 0, undefined);
        table.set(offset + 1, null);
        table.set(offset + 2, true);
        table.set(offset + 3, false);
        ;
    };
    imports.wbg.__wbindgen_is_function = function(arg0) {
        const ret = typeof(arg0) === 'function';
        return ret;
    };
    imports.wbg.__wbindgen_is_null = function(arg0) {
        const ret = arg0 === null;
        return ret;
    };
    imports.wbg.__wbindgen_is_object = function(arg0) {
        const val = arg0;
        const ret = typeof(val) === 'object' && val !== null;
        return ret;
    };
    imports.wbg.__wbindgen_is_symbol = function(arg0) {
        const ret = typeof(arg0) === 'symbol';
        return ret;
    };
    imports.wbg.__wbindgen_is_undefined = function(arg0) {
        const ret = arg0 === undefined;
        return ret;
    };
    imports.wbg.__wbindgen_jsval_eq = function(arg0, arg1) {
        const ret = arg0 === arg1;
        return ret;
    };
    imports.wbg.__wbindgen_memory = function() {
        const ret = wasm.memory;
        return ret;
    };
    imports.wbg.__wbindgen_number_get = function(arg0, arg1) {
        const obj = arg1;
        const ret = typeof(obj) === 'number' ? obj : undefined;
        getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
    };
    imports.wbg.__wbindgen_number_new = function(arg0) {
        const ret = arg0;
        return ret;
    };
    imports.wbg.__wbindgen_rethrow = function(arg0) {
        throw arg0;
    };
    imports.wbg.__wbindgen_string_get = function(arg0, arg1) {
        const obj = arg1;
        const ret = typeof(obj) === 'string' ? obj : undefined;
        var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        var len1 = WASM_VECTOR_LEN;
        getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
        getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
    };
    imports.wbg.__wbindgen_string_new = function(arg0, arg1) {
        const ret = getStringFromWasm0(arg0, arg1);
        return ret;
    };
    imports.wbg.__wbindgen_throw = function(arg0, arg1) {
        throw new Error(getStringFromWasm0(arg0, arg1));
    };
    imports.wbg.__wbindgen_typeof = function(arg0) {
        const ret = typeof arg0;
        return ret;
    };

    return imports;
}

function __wbg_init_memory(imports, memory) {

}

function __wbg_finalize_init(instance, module) {
    wasm = instance.exports;
    __wbg_init.__wbindgen_wasm_module = module;
    cachedDataViewMemory0 = null;
    cachedUint8ArrayMemory0 = null;


    wasm.__wbindgen_start();
    return wasm;
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (typeof module !== 'undefined') {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();

    __wbg_init_memory(imports);

    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }

    const instance = new WebAssembly.Instance(module, imports);

    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (typeof module_or_path !== 'undefined') {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (typeof module_or_path === 'undefined') {
        module_or_path = new URL('kgw_frontend_wasm_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    __wbg_init_memory(imports);

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync };
export default __wbg_init;
