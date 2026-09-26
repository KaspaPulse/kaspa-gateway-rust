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

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_2.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

export function analysisInstallBinding() {
    const ret = wasm.analysisInstallBinding();
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
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

/**
 * @returns {Array<any>}
 */
export function settingsNodeEndpoints() {
    const ret = wasm.settingsNodeEndpoints();
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

function __wbg_adapter_34(arg0, arg1) {
    wasm._dyn_core_ed718c3d60ebd546___ops__function__FnMut_____Output______as_wasm_bindgen_1ba7c375a52abd4d___closure__WasmClosure___describe__invoke______(arg0, arg1);
}

function __wbg_adapter_37(arg0, arg1, arg2) {
    wasm.closure246_externref_shim(arg0, arg1, arg2);
}

function __wbg_adapter_212(arg0, arg1, arg2, arg3) {
    wasm.closure270_externref_shim(arg0, arg1, arg2, arg3);
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
    imports.wbg.__wbg_apply_eb9e9b97497f91e4 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = Reflect.apply(arg0, arg1, arg2);
        return ret;
    }, arguments) };
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
    imports.wbg.__wbg_format_0545b83dc1d8a934 = function(arg0) {
        const ret = arg0.format;
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
                    return __wbg_adapter_212(a, state0.b, arg0, arg1);
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
    imports.wbg.__wbg_new_63847613cde5d4bc = function(arg0, arg1, arg2, arg3) {
        const ret = new RegExp(getStringFromWasm0(arg0, arg1), getStringFromWasm0(arg2, arg3));
        return ret;
    };
    imports.wbg.__wbg_new_78feb108b6472713 = function() {
        const ret = new Array();
        return ret;
    };
    imports.wbg.__wbg_new_b08a00743b8ae2f3 = function(arg0, arg1) {
        const ret = new TypeError(getStringFromWasm0(arg0, arg1));
        return ret;
    };
    imports.wbg.__wbg_newnoargs_105ed471475aaf50 = function(arg0, arg1) {
        const ret = new Function(getStringFromWasm0(arg0, arg1));
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
    imports.wbg.__wbg_resolve_4851785c9c5f573d = function(arg0) {
        const ret = Promise.resolve(arg0);
        return ret;
    };
    imports.wbg.__wbg_set_bb8cecf6a62b9f46 = function() { return handleError(function (arg0, arg1, arg2) {
        const ret = Reflect.set(arg0, arg1, arg2);
        return ret;
    }, arguments) };
    imports.wbg.__wbg_shift_9a41f00897c50537 = function(arg0) {
        const ret = arg0.shift();
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
    imports.wbg.__wbindgen_closure_wrapper885 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 192, __wbg_adapter_34);
        return ret;
    };
    imports.wbg.__wbindgen_closure_wrapper973 = function(arg0, arg1, arg2) {
        const ret = makeMutClosure(arg0, arg1, 247, __wbg_adapter_37);
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
