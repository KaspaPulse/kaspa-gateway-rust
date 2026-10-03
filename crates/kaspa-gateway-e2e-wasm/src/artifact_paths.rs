//! Artifact paths and asynchronous evidence writes for the existing E2E host.
//! Node imports below are platform primitives; Rust owns the helper behavior.
use js_sys::{Date, Function, Object, Promise, Reflect};
use wasm_bindgen::{JsCast, closure::Closure, prelude::*};

#[wasm_bindgen(module = "node:path")]
extern "C" {
    #[wasm_bindgen(catch, js_name = dirname)]
    fn dirname(path: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = resolve)]
    fn resolve(base: &JsValue, suffix: &str) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = join)]
    fn join3(base: &JsValue, middle: &str, end: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_name = join)]
    fn join4(base: &JsValue, a: &str, b: &str, end: &str) -> Result<JsValue, JsValue>;
}
#[wasm_bindgen(module = "node:url")]
extern "C" {
    #[wasm_bindgen(catch, js_name = fileURLToPath)]
    fn file_url_to_path(url: &JsValue) -> Result<JsValue, JsValue>;
}
#[wasm_bindgen(module = "node:fs/promises")]
extern "C" {
    #[wasm_bindgen(catch, js_name = mkdir)]
    fn mkdir(path: &JsValue, options: &Object) -> Result<Promise, JsValue>;
    #[wasm_bindgen(catch, js_name = writeFile)]
    fn write_file(path: &JsValue, data: &JsValue, encoding: &str) -> Result<Promise, JsValue>;
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = Boolean)]
    fn truthy(value: &JsValue) -> bool;
    #[wasm_bindgen(catch, js_name = String)]
    fn string(value: &JsValue) -> Result<JsValue, JsValue>;
    #[wasm_bindgen(catch, js_namespace = JSON, js_name = stringify)]
    fn stringify(value: &JsValue, replacer: &JsValue, space: u32) -> Result<JsValue, JsValue>;
}
fn property(value: &JsValue, name: &str) -> Result<JsValue, JsValue> {
    Reflect::get(value, &JsValue::from_str(name))
}
fn set(value: &Object, name: &str, data: &JsValue) -> Result<(), JsValue> {
    Reflect::set(value, &JsValue::from_str(name), data).map(|_| ())
}

#[wasm_bindgen(js_name = artifactPathContext)]
pub fn artifact_path_context(url: JsValue, env: JsValue) -> Result<Object, JsValue> {
    let e2e_dir = resolve(&dirname(&file_url_to_path(&url)?)?, "..")?;
    let configured_repo = property(&env, "KGW_REPOSITORY")?;
    let repository = if truthy(&configured_repo) {
        configured_repo
    } else {
        resolve(&e2e_dir, "..")?
    };
    let configured_artifact = property(&env, "KGW_ZERO_TOUCH_ARTIFACT_DIR")?;
    let artifact_root = if truthy(&configured_artifact) {
        configured_artifact
    } else {
        join4(
            &repository,
            "artifacts",
            "zero-touch-e2e",
            &format!("wdio-{}", Date::now()),
        )?
    };
    let result = Object::new();
    set(&result, "e2eDir", &e2e_dir)?;
    set(&result, "repository", &repository)?;
    set(&result, "artifactRoot", &artifact_root)?;
    Ok(result)
}
#[wasm_bindgen(js_name = artifactCaseDir)]
pub fn artifact_case_dir(artifact_root: JsValue, slug: JsValue) -> Result<JsValue, JsValue> {
    join3(&artifact_root, "cases", &slug)
}
#[wasm_bindgen(js_name = artifactHelperScript)]
pub fn artifact_helper_script(e2e_dir: JsValue, name: JsValue) -> Result<JsValue, JsValue> {
    join3(&e2e_dir, "helpers", &name)
}
fn start_ensure_dir(directory: &JsValue) -> Result<Promise, JsValue> {
    let options = Object::new();
    set(&options, "recursive", &JsValue::TRUE)?;
    mkdir(directory, &options)
}
fn after(
    pending: Result<Promise, JsValue>,
    action: impl FnMut(JsValue) -> Result<JsValue, JsValue> + 'static,
) -> Promise {
    let result = (|| -> Result<Promise, JsValue> {
        let promise = pending?;
        let callback =
            Closure::wrap(Box::new(action) as Box<dyn FnMut(JsValue) -> Result<JsValue, JsValue>>)
                .into_js_value();
        // Supported Node hosts provide GC weak references for wasm-bindgen cleanup.
        let then = property(promise.as_ref(), "then")?.dyn_into::<Function>()?;
        then.call1(promise.as_ref(), &callback)?
            .dyn_into::<Promise>()
    })();
    result.unwrap_or_else(|failure| Promise::reject(&failure))
}
#[wasm_bindgen(js_name = artifactEnsureDir)]
pub fn artifact_ensure_dir(directory: JsValue) -> Promise {
    let pending = start_ensure_dir(&directory);
    after(pending, move |_| Ok(directory.clone()))
}
fn start_parent(file: &JsValue) -> Result<Promise, JsValue> {
    Ok(artifact_ensure_dir(dirname(file)?))
}
#[wasm_bindgen(js_name = artifactWriteJson)]
pub fn artifact_write_json(file: JsValue, value: JsValue) -> Promise {
    let parent = start_parent(&file);
    after(parent, move |_| {
        let data = stringify(&value, &JsValue::UNDEFINED, 2)?;
        write_file(&file, &data, "utf8").map(Into::into)
    })
}
#[wasm_bindgen(js_name = artifactWriteText)]
pub fn artifact_write_text(file: JsValue, value: JsValue) -> Promise {
    let parent = start_parent(&file);
    after(parent, move |_| {
        let source = if value.is_null() || value.is_undefined() {
            JsValue::from_str("")
        } else {
            value.clone()
        };
        let data = string(&source)?;
        write_file(&file, &data, "utf8").map(Into::into)
    })
}
