/// Since threads (WebWorkers) in browser don't shared the same memory space, so no objects would have a chance to be sended to other threads, thus it's safe to implement Send for objects here
use web_sys::{
    js_sys::Reflect,
    wasm_bindgen::{JsCast, JsValue},
    Performance,
};

const WASM_MISSING_GLOBAL_THIS_PERF: &str = "failed to find `globalThis.performance`";
const WASM_UNABLE_TO_CAST_PERF: &str =
    "Unable to cast `globalThis.performance` to Performance type";

struct PerformanceClassWrapper(Performance);

unsafe impl Send for PerformanceClassWrapper {}
unsafe impl Sync for PerformanceClassWrapper {}

static GLOBAL_PERFORMANCE_INSTANCE: std::sync::OnceLock<PerformanceClassWrapper> =
    std::sync::OnceLock::new();

#[derive(Clone, Copy, Debug)]
pub struct Monotonic {
    performance: &'static Performance,
}

unsafe impl Send for Monotonic {}
unsafe impl Sync for Monotonic {}

impl Default for Monotonic {
    fn default() -> Self {
        let performance = GLOBAL_PERFORMANCE_INSTANCE.get_or_init(|| {
            PerformanceClassWrapper(
                Reflect::get(
                    &web_sys::js_sys::global(),
                    &JsValue::from_str("performance"),
                )
                .expect(WASM_MISSING_GLOBAL_THIS_PERF)
                .dyn_into::<Performance>()
                .expect(WASM_UNABLE_TO_CAST_PERF),
            )
        });

        Self {
            performance: &performance.0,
        }
    }
}

impl Monotonic {
    pub fn now(&self) -> u64 {
        let now = self.performance.now();
        // `performance.now()` returns the time in milliseconds.
        return f64::trunc(now * 1_000_000.0) as u64;
    }
}
