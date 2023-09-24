use core::arch::x86_64::_rdtsc;

#[derive(Clone, Debug, Default)]
pub struct Counter;

impl Counter {
    pub fn now(&self) -> u64 {
        unsafe { _rdtsc() }
    }
}
