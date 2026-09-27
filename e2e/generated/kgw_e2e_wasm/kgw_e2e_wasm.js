
let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;
let wasm;
const { mkdir, writeFile } = require(`node:fs/promises`);
const { dirname, join, resolve } = require(`node:path`);
const { fileURLToPath } = require(`node:url`);
const { TextDecoder, TextEncoder } = require(`util`);

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

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

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

function isLikeNone(x) {
    return x === undefined || x === null;
}

let cachedDataViewMemory0 = null;

function getDataViewMemory0() {
    if (cachedDataViewMemory0 === null || cachedDataViewMemory0.buffer.detached === true || (cachedDataViewMemory0.buffer.detached === undefined && cachedDataViewMemory0.buffer !== wasm.memory.buffer)) {
        cachedDataViewMemory0 = new DataView(wasm.memory.buffer);
    }
    return cachedDataViewMemory0;
}

let WASM_VECTOR_LEN = 0;

let cachedTextEncoder = new TextEncoder('utf-8');

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
module.exports.rawLogRejectionMarkers = function() {
    const ret = wasm.rawLogRejectionMarkers();
    return ret;
};

/**
 * @param {any} value
 * @returns {string}
 */
module.exports.sha256Hex = function(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.sha256Hex(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {any} value
 * @returns {string}
 */
module.exports.normalizeClipboardText = function(value) {
    let deferred1_0;
    let deferred1_1;
    try {
        const ret = wasm.normalizeClipboardText(value);
        deferred1_0 = ret[0];
        deferred1_1 = ret[1];
        return getStringFromWasm0(ret[0], ret[1]);
    } finally {
        wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
    }
};

/**
 * @param {any} value
 * @returns {number}
 */
module.exports.lineCount = function(value) {
    const ret = wasm.lineCount(value);
    return ret >>> 0;
};

/**
 * @param {any} value
 * @returns {boolean}
 */
module.exports.containsTransportWrapper = function(value) {
    const ret = wasm.containsTransportWrapper(value);
    return ret !== 0;
};

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_export_2.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}
/**
 * @param {any} value
 * @returns {any}
 */
module.exports.parseKeyValueLine = function(value) {
    const ret = wasm.parseKeyValueLine(value);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
};

/**
 * @param {any} value
 * @returns {any}
 */
module.exports.pidFromStatus = function(value) {
    const ret = wasm.pidFromStatus(value);
    return ret;
};

/**
 * @param {any} value
 * @returns {boolean}
 */
module.exports.isStoppedOwnerStatus = function(value) {
    const ret = wasm.isStoppedOwnerStatus(value);
    return ret !== 0;
};

/**
 * @param {any} text
 * @param {any} label
 */
module.exports.assertNoTransportWrappers = function(text, label) {
    const ret = wasm.assertNoTransportWrappers(text, label);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
};

/**
 * @param {any} options
 */
module.exports.assertDirectRawPayload = function(options) {
    const ret = wasm.assertDirectRawPayload(options);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
};

/**
 * @param {any} report
 * @param {any} options
 */
module.exports.assertRuntimeLogReport = function(report, options) {
    const ret = wasm.assertRuntimeLogReport(report, options);
    if (ret[1]) {
        throw takeFromExternrefTable0(ret[0]);
    }
};

/**
 * @param {any} env
 * @returns {any}
 */
module.exports.runtimePortProfile = function(env) {
    const ret = wasm.runtimePortProfile(env);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
};

/**
 * @param {any} url
 * @param {any} env
 * @returns {object}
 */
module.exports.artifactPathContext = function(url, env) {
    const ret = wasm.artifactPathContext(url, env);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
};

/**
 * @param {any} artifact_root
 * @param {any} slug
 * @returns {any}
 */
module.exports.artifactCaseDir = function(artifact_root, slug) {
    const ret = wasm.artifactCaseDir(artifact_root, slug);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
};

/**
 * @param {any} e2e_dir
 * @param {any} name
 * @returns {any}
 */
module.exports.artifactHelperScript = function(e2e_dir, name) {
    const ret = wasm.artifactHelperScript(e2e_dir, name);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
};

/**
 * @param {any} directory
 * @returns {Promise<any>}
 */
module.exports.artifactEnsureDir = function(directory) {
    const ret = wasm.artifactEnsureDir(directory);
    return ret;
};

/**
 * @param {any} file
 * @param {any} value
 * @returns {Promise<any>}
 */
module.exports.artifactWriteJson = function(file, value) {
    const ret = wasm.artifactWriteJson(file, value);
    return ret;
};

/**
 * @param {any} file
 * @param {any} value
 * @returns {Promise<any>}
 */
module.exports.artifactWriteText = function(file, value) {
    const ret = wasm.artifactWriteText(file, value);
    return ret;
};

function __wbg_adapter_26(arg0, arg1, arg2) {
    const ret = wasm.closure7_externref_shim_multivalue_shim(arg0, arg1, arg2);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return takeFromExternrefTable0(ret[0]);
}

module.exports.__wbg_Boolean_8afa98d5184b1132 = function(arg0) {
    const ret = Boolean(arg0);
    return ret;
};

module.exports.__wbg_Number_f7e4c6f68f48c6df = function(arg0) {
    const ret = Number(arg0);
    return ret;
};

module.exports.__wbg_String_0698c0ff1c967aa4 = function(arg0) {
    const ret = String(arg0);
    return ret;
};

module.exports.__wbg_String_209310a119167e73 = function() { return handleError(function (arg0) {
    const ret = String(arg0);
    return ret;
}, arguments) };

module.exports.__wbg_call_7cccdd69e0791ae2 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = arg0.call(arg1, arg2);
    return ret;
}, arguments) };

module.exports.__wbg_dirname_99b0ecf328866900 = function() { return handleError(function (arg0) {
    const ret = dirname(arg0);
    return ret;
}, arguments) };

module.exports.__wbg_fileURLToPath_64d71cf710015898 = function() { return handleError(function (arg0) {
    const ret = fileURLToPath(arg0);
    return ret;
}, arguments) };

module.exports.__wbg_freeze_ef6d70cf38e8d948 = function(arg0) {
    const ret = Object.freeze(arg0);
    return ret;
};

module.exports.__wbg_get_67b2ba62fc30de12 = function() { return handleError(function (arg0, arg1) {
    const ret = Reflect.get(arg0, arg1);
    return ret;
}, arguments) };

module.exports.__wbg_get_b9b93047fe3cf45b = function(arg0, arg1) {
    const ret = arg0[arg1 >>> 0];
    return ret;
};

module.exports.__wbg_instanceof_Promise_935168b8f4b49db3 = function(arg0) {
    let result;
    try {
        result = arg0 instanceof Promise;
    } catch (_) {
        result = false;
    }
    const ret = result;
    return ret;
};

module.exports.__wbg_isArray_a1eab7e0d067391b = function(arg0) {
    const ret = Array.isArray(arg0);
    return ret;
};

module.exports.__wbg_join_442dcbef917ba7a2 = function() { return handleError(function (arg0, arg1, arg2, arg3) {
    const ret = join(arg0, getStringFromWasm0(arg1, arg2), arg3);
    return ret;
}, arguments) };

module.exports.__wbg_join_fa0cf6fb2ab47ca7 = function() { return handleError(function (arg0, arg1, arg2, arg3, arg4, arg5, arg6) {
    const ret = join(arg0, getStringFromWasm0(arg1, arg2), getStringFromWasm0(arg3, arg4), getStringFromWasm0(arg5, arg6));
    return ret;
}, arguments) };

module.exports.__wbg_length_e2d2a49132c1b256 = function(arg0) {
    const ret = arg0.length;
    return ret;
};

module.exports.__wbg_mkdir_147f959b191b3c20 = function() { return handleError(function (arg0, arg1) {
    const ret = mkdir(arg0, arg1);
    return ret;
}, arguments) };

module.exports.__wbg_new_405e22f390576ce2 = function() {
    const ret = new Object();
    return ret;
};

module.exports.__wbg_new_78feb108b6472713 = function() {
    const ret = new Array();
    return ret;
};

module.exports.__wbg_new_c68d7209be747379 = function(arg0, arg1) {
    const ret = new Error(getStringFromWasm0(arg0, arg1));
    return ret;
};

module.exports.__wbg_now_807e54c39636c349 = function() {
    const ret = Date.now();
    return ret;
};

module.exports.__wbg_push_737cfc8c1432c2c6 = function(arg0, arg1) {
    const ret = arg0.push(arg1);
    return ret;
};

module.exports.__wbg_reject_b3fcf99063186ff7 = function(arg0) {
    const ret = Promise.reject(arg0);
    return ret;
};

module.exports.__wbg_resolve_412ea39f68c9e898 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = resolve(arg0, getStringFromWasm0(arg1, arg2));
    return ret;
}, arguments) };

module.exports.__wbg_set_bb8cecf6a62b9f46 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = Reflect.set(arg0, arg1, arg2);
    return ret;
}, arguments) };

module.exports.__wbg_setname_6df54b7ebf9404a9 = function(arg0, arg1, arg2) {
    arg0.name = getStringFromWasm0(arg1, arg2);
};

module.exports.__wbg_stringify_079f8cd10d739b69 = function() { return handleError(function (arg0, arg1, arg2) {
    const ret = JSON.stringify(arg0, arg1, arg2 >>> 0);
    return ret;
}, arguments) };

module.exports.__wbg_test_7f0ac7b9d67b7a48 = function(arg0, arg1, arg2) {
    const ret = arg0.test(getStringFromWasm0(arg1, arg2));
    return ret;
};

module.exports.__wbg_toString_5594a7237007a325 = function(arg0) {
    const ret = arg0.toString();
    return ret;
};

module.exports.__wbg_writeFile_9d0ce1294dd441fd = function() { return handleError(function (arg0, arg1, arg2, arg3) {
    const ret = writeFile(arg0, arg1, getStringFromWasm0(arg2, arg3));
    return ret;
}, arguments) };

module.exports.__wbindgen_boolean_get = function(arg0) {
    const v = arg0;
    const ret = typeof(v) === 'boolean' ? (v ? 1 : 0) : 2;
    return ret;
};

module.exports.__wbindgen_closure_wrapper97 = function(arg0, arg1, arg2) {
    const ret = makeMutClosure(arg0, arg1, 8, __wbg_adapter_26);
    return ret;
};

module.exports.__wbindgen_init_externref_table = function() {
    const table = wasm.__wbindgen_export_2;
    const offset = table.grow(4);
    table.set(0, undefined);
    table.set(offset + 0, undefined);
    table.set(offset + 1, null);
    table.set(offset + 2, true);
    table.set(offset + 3, false);
    ;
};

module.exports.__wbindgen_is_function = function(arg0) {
    const ret = typeof(arg0) === 'function';
    return ret;
};

module.exports.__wbindgen_is_null = function(arg0) {
    const ret = arg0 === null;
    return ret;
};

module.exports.__wbindgen_is_undefined = function(arg0) {
    const ret = arg0 === undefined;
    return ret;
};

module.exports.__wbindgen_number_get = function(arg0, arg1) {
    const obj = arg1;
    const ret = typeof(obj) === 'number' ? obj : undefined;
    getDataViewMemory0().setFloat64(arg0 + 8 * 1, isLikeNone(ret) ? 0 : ret, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, !isLikeNone(ret), true);
};

module.exports.__wbindgen_number_new = function(arg0) {
    const ret = arg0;
    return ret;
};

module.exports.__wbindgen_string_get = function(arg0, arg1) {
    const obj = arg1;
    const ret = typeof(obj) === 'string' ? obj : undefined;
    var ptr1 = isLikeNone(ret) ? 0 : passStringToWasm0(ret, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    var len1 = WASM_VECTOR_LEN;
    getDataViewMemory0().setInt32(arg0 + 4 * 1, len1, true);
    getDataViewMemory0().setInt32(arg0 + 4 * 0, ptr1, true);
};

module.exports.__wbindgen_string_new = function(arg0, arg1) {
    const ret = getStringFromWasm0(arg0, arg1);
    return ret;
};

module.exports.__wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

const path = require('path').join(__dirname, 'kgw_e2e_wasm_bg.wasm');
const bytes = require('fs').readFileSync(path);

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
wasm = wasmInstance.exports;
module.exports.__wasm = wasm;

wasm.__wbindgen_start();
