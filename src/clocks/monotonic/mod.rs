#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::Monotonic;

#[cfg(target_arch = "wasm32")]
mod wasm;
#[cfg(target_arch = "wasm32")]
pub use self::wasm::Monotonic;

#[cfg(not(any(target_os = "windows", target_arch = "wasm32")))]
mod unix;
#[cfg(not(any(target_os = "windows", target_arch = "wasm32")))]
pub use self::unix::Monotonic;
