use web_sys::{
    js_sys::Reflect,
    wasm_bindgen::{JsCast, JsValue},
    Performance,
};

const WASM_MISSING_GLOBAL_THIS_PERF: &str = "failed to find `globalThis.performance`";
const WASM_UNABLE_TO_CAST_PERF: &str =
    "Unable to cast `globalThis.performance` to Performance type";
#[derive(Clone, Copy, Debug, Default)]
pub struct Monotonic {
    _default: (),
}

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
impl Monotonic {
    pub fn now(&self) -> u64 {
        let now = Reflect::get(
            &web_sys::js_sys::global(),
            &JsValue::from_str("performance"),
        )
        .expect(WASM_MISSING_GLOBAL_THIS_PERF)
        .dyn_ref::<Performance>()
        .expect(WASM_UNABLE_TO_CAST_PERF)
        .now();
        // `performance.now()` returns the time in milliseconds.
        return f64::trunc(now * 1_000_000.0) as u64;
    }
}

#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
impl Monotonic {
    pub fn now(&self) -> u64 {
        unsafe { wasi::clock_time_get(wasi::CLOCKID_MONOTONIC, 1).expect("failed to get time") }
    }
}
