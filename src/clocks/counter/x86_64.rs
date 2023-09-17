use core::arch::x86_64::_rdtsc;

#[derive(Clone, Debug, Default)]
pub struct Counter;

impl Counter {
    pub fn now(&self) -> u64 {
        unsafe { _rdtsc() }
    }

    pub fn freq_hz(&self) -> Option<u64> {
        // TODO: This is where we could potentially query the CPU or add model/arch-specific
        // overrides where the TSC frequency is known. From what I remember reading through the
        // Intel SDM, some processors _do_ have a fixed TSC frequency? Needs more investigation,
        // especially to also consider AMD, and so on.
		None
	}
}
