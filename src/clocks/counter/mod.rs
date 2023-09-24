#[cfg(target_arch = "aarch64")]
mod aarch64;

#[cfg(target_arch = "aarch64")]
pub use self::aarch64::Counter;

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
mod x86_64;

#[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
pub use self::x86_64::Counter;

#[cfg(not(any(
    all(target_arch = "x86_64", target_feature = "sse2"),
    target_arch = "aarch64",
)))]
impl Counter {
    pub fn now(&self) -> u64 {
        panic!("can't use counter without TSC (x86_64) or system counter (ARM) support");
    }
}
