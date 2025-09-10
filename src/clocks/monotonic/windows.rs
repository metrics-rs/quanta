use std::mem;
use windows_sys::Win32::System::Performance;

#[derive(Clone, Copy, Debug)]
pub struct Monotonic {
    factor: u64,
}

impl Monotonic {
    pub fn now(&self) -> u64 {
        let raw = unsafe {
            // TODO: Can we do any better than the `mem::zeroed` call here?
            let mut count = mem::zeroed();
            if Performance::QueryPerformanceCounter(&mut count) <= 0 {
                unreachable!(
                    "QueryPerformanceCounter on Windows XP or later should never return zero!"
                );
            }
            count as u64
        };
        raw * self.factor
    }
}

impl Default for Monotonic {
    fn default() -> Self {
        let denom = unsafe {
            // TODO: Can we do any better than the `mem::zeroed` call here?
            let mut freq = mem::zeroed();
            if Performance::QueryPerformanceFrequency(&mut freq) <= 0 {
                unreachable!(
                    "QueryPerformanceFrequency on Windows XP or later should never return zero!"
                );
            }
            freq as u64
        };

        Self {
            factor: 1_000_000_000 / denom,
        }
    }
}
