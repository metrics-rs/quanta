use crate::ClockSource;

#[cfg(all(target_arch = "x86", target_feature = "sse2"))]
use std::arch::x86::{__rdtscp, _mm_lfence, _rdtsc};
#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
use std::arch::x86_64::{__rdtscp, _mm_lfence, _rdtsc};

#[derive(Debug, Clone, Default)]
pub struct Counter;

impl Counter {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Counter {}
    }
}

#[cfg(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2"
))]
impl ClockSource for Counter {
    fn now(&self) -> u64 {
        unsafe {
            _mm_lfence();
            _rdtsc()
        }
    }

    fn start(&self) -> u64 {
        unsafe {
            _mm_lfence();
            let result = _rdtsc();
            _mm_lfence();
            result
        }
    }

    fn end(&self) -> u64 {
        let mut _aux: u32 = 0;
        unsafe {
            let result = __rdtscp(&mut _aux as *mut _);
            _mm_lfence();
            result
        }
    }
}

#[cfg(not(all(
    any(target_arch = "x86", target_arch = "x86_64"),
    target_feature = "sse2"
)))]
impl ClockSource for Counter {
    fn now(&self) -> u64 {
        panic!("can't use counter without TSC support");
    }

    fn start(&self) -> u64 {
        panic!("can't use counter without TSC support");
    }

    fn end(&self) -> u64 {
        panic!("can't use counter without TSC support");
    }
}
