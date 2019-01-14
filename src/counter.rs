use crate::ClockSource;

#[derive(Clone, Default)]
pub struct Counter;

impl Counter {
    #[allow(dead_code)]
    pub fn new() -> Self { Counter {} }
}

#[cfg(feature = "tsc")]
impl ClockSource for Counter {
    fn now(&self) -> u64 {
        let mut l: u32;
        let mut h: u32;
        unsafe {
            asm!("lfence; rdtsc" : "={eax}" (l), "={edx}" (h) ::: "volatile");
        }
        ((u64::from(h)) << 32) | u64::from(l)
    }

    fn start(&self) -> u64 {
        let mut l: u32;
        let mut h: u32;
        unsafe {
            asm!("lfence; rdtsc; lfence" : "={eax}" (l), "={edx}" (h) ::: "volatile");
        }
        ((u64::from(h)) << 32) | u64::from(l)
    }

    fn end(&self) -> u64 {
        let mut l: u32;
        let mut h: u32;
        unsafe {
            asm!("rdtscp; lfence" : "={eax}" (l), "={edx}" (h) ::: "volatile");
        }
        ((u64::from(h)) << 32) | u64::from(l)
    }
}

#[cfg(not(feature = "tsc"))]
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
