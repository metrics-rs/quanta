use std::mem;
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default)]
pub struct Monotonic {
    _default: (),
}

impl Monotonic {
    pub fn now(&self) -> u64 {
        unsafe { wasi::clock_time_get(wasi::CLOCKID_MONOTONIC, 1).expect("failed to get time") }
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