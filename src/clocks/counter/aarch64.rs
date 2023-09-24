use core::arch::asm;

#[derive(Clone, Debug, Default)]
pub struct Counter;

impl Counter {
    pub fn now(&self) -> u64 {
        let count: u64;

        unsafe {
            asm!("mrs {}, cntvct_el0", out(reg) count);
        }

        count
    }
}
