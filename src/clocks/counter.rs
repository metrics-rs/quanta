#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
use core::arch::x86_64::__rdtscp;

#[derive(Clone, Debug, Default)]
pub struct Counter;

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
impl Counter {
    pub fn now(&self) -> u64 {
        let mut aux: u32 = 0;
        unsafe { __rdtscp(&mut aux as *mut _) }
    }
}

#[cfg(not(all(target_arch = "x86_64", target_feature = "sse2")))]
impl Counter {
    pub fn now(&self) -> u64 {
        panic!("can't use counter without TSC support");
    }
}
