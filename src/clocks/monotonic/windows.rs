use std::mem;
use std::time::Duration;
use winapi::um::profileapi;

#[derive(Clone, Copy, Debug)]
pub struct Monotonic {
    factor: u64,
}

impl Monotonic {
    pub fn now(&self) -> u64 {
        let raw = unsafe {
            // TODO: Can we do any better than the `mem::zeroed` call here?
            let mut count = mem::zeroed();
            if profileapi::QueryPerformanceCounter(&mut count) == 0 {
                unreachable!(
                    "QueryPerformanceCounter on Windows XP or later should never return zero!"
                );
            }
            *count.QuadPart() as u64
        };
        raw * self.factor
    }
}

impl Default for Monotonic {
    fn default() -> Self {
        let denom = unsafe {
            // TODO: Can we do any better than the `mem::zeroed` call here?
            let mut freq = mem::zeroed();
            if profileapi::QueryPerformanceFrequency(&mut freq) == 0 {
                unreachable!(
                    "QueryPerformanceFrequency on Windows XP or later should never return zero!"
                );
            }
            *freq.QuadPart() as u64
        };

        Self {
            factor: 1_000_000_000 / denom,
        }
    }
}

// std::time::Instant is represented as
// struct Instant {
//     t: Duration,
// }

pub(crate) fn to_std_instant(instant: u64) -> std::time::Instant {
    unsafe { mem::transmute(Duration::from_nanos(instant)) }
}

pub(crate) fn from_std_instant(instant: std::time::Instant) -> u64 {
    let dur: Duration = unsafe { mem::transmute(instant) };

    dur.as_secs() * 1_000_000_000 + dur.subsec_nanos()
}