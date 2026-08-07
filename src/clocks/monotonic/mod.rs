#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::Monotonic;
#[cfg(target_os = "windows")]
pub(crate) use self::windows::{to_std_instant, from_std_instant};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod wasm_browser;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub use self::wasm_browser::Monotonic;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub(crate) use self::wasm_browser::{to_std_instant, from_std_instant};

#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
mod wasm_wasi;
#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
pub use self::wasm_wasi::Monotonic;
#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
pub(crate) use self::wasm_wasi::{to_std_instant, from_std_instant};

#[cfg(not(any(target_os = "windows", target_arch = "wasm32")))]
mod unix;
#[cfg(not(any(target_os = "windows", target_arch = "wasm32")))]
pub use self::unix::Monotonic;
#[cfg(not(any(target_os = "windows", target_arch = "wasm32")))]
pub(crate) use self::unix::{to_std_instant, from_std_instant};
