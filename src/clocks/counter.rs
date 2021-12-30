#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
use core::arch::x86_64::{__rdtscp, _mm_lfence, _rdtsc};

#[derive(Clone, Debug, Default)]
pub struct Counter;

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
impl Counter {
    pub fn now(&self) -> u64 {
        unsafe {
            //_mm_lfence();
            _rdtsc()
        }
    }

    pub fn start(&self) -> u64 {
        unsafe {
            _mm_lfence();
            let result = _rdtsc();
            _mm_lfence();
            result
        }
    }

    pub fn end(&self) -> u64 {
        let mut aux: u32 = 0;
        unsafe {
            let result = __rdtscp(&mut aux as *mut _);
            _mm_lfence();
            result
        }
    }
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
impl Counter {
    pub fn now(&self) -> u64 {
        panic!("can't use counter without TSC support");
    }

    pub fn start(&self) -> u64 {
        panic!("can't use counter without TSC support");
    }

    pub fn end(&self) -> u64 {
        panic!("can't use counter without TSC support");
    }
}
