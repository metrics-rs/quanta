use std::cell::OnceCell;
use std::mem;
use std::time::Duration;
use web_sys::{
    js_sys::Reflect,
    wasm_bindgen::{JsCast, JsValue},
    Performance,
};

const WASM_MISSING_GLOBAL_THIS_PERF: &str = "failed to find `globalThis.performance`";
const WASM_UNABLE_TO_CAST_PERF: &str =
    "Unable to cast `globalThis.performance` to Performance type";

thread_local! {
    static GLOBAL_PERFORMANCE_INSTANCE: OnceCell<Performance> = const { OnceCell::new() };
}

#[derive(Clone, Copy, Debug, Default)]
pub struct Monotonic {
    _default: (),
}

impl Monotonic {
    pub fn now(&self) -> u64 {
        let now = GLOBAL_PERFORMANCE_INSTANCE.with(|value| {
            let performance_instance = value.get_or_init(|| {
                Reflect::get(
                    &web_sys::js_sys::global(),
                    &JsValue::from_str("performance"),
                )
                .expect(WASM_MISSING_GLOBAL_THIS_PERF)
                .dyn_into::<Performance>()
                .expect(WASM_UNABLE_TO_CAST_PERF)
            });
            performance_instance.now()
        });
        // `performance.now()` returns the time in milliseconds.
        f64::trunc(now * 1_000_000.0) as u64
    }
}


// std::time::Instant is represented as
// struct Instant(std::time::Duration);

pub(crate) fn to_std_instant(instant: u64) -> std::time::Instant {
    unsafe { mem::transmute(Duration::from_nanos(instant)) }
}

pub(crate) fn from_std_instant(instant: std::time::Instant) -> u64 {
    let dur: Duration = unsafe { mem::transmute(instant) };

    dur.as_secs() * 1_000_000_000 + dur.subsec_nanos()
}