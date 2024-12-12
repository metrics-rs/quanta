#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
pub use self::windows::Monotonic;

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
mod wasm_browser;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub use self::wasm_browser::Monotonic;

#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
mod wasm_wasi;
#[cfg(all(target_arch = "wasm32", target_os = "wasi"))]
pub use self::wasm_wasi::Monotonic;

#[cfg(not(any(target_os = "windows", target_arch = "wasm32")))]
mod unix;
#[cfg(not(any(target_os = "windows", target_arch = "wasm32")))]
pub use self::unix::Monotonic;
