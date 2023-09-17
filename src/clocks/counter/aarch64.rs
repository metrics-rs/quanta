use core::arch::asm;

#[derive(Clone, Debug, Default)]
pub struct Counter;

impl Counter {
    pub fn now(&self) -> u64 {
		let count: u64;

        unsafe { asm!("mrs {}, cntvct_el0", out(reg) count); }

		count
    }

    pub fn freq_hz(&self) -> Option<u64> {
		let freq_hz: u64;

        unsafe { asm!("mrs {}, cntfrq_el0", out(reg) freq_hz); }

		Some(freq_hz)
	}
}
