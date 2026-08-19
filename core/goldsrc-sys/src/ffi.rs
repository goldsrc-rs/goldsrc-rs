/// Utilities for safe FFI boundary enforcement.
///
/// Any Rust panic that escapes across a `extern "C"` / `extern "system"` boundary
/// into C/C++ code is **undefined behaviour** — the unwinding mechanism is not
/// guaranteed to be compatible. In practice on HLDS/Windows this causes the
/// server process to crash or silently corrupt state.
///
/// Use [`catch_ffi_panic`] in **every** `#[no_mangle] pub extern` entry-point
/// and every `unsafe extern "C"` hook callback registered with the engine.
use std::panic::{AssertUnwindSafe, catch_unwind};

/// Marker wrapper to make a `&'static T` value `Sync` when it is only ever
/// accessed from a single thread (the GoldSrc engine is single-threaded).
///
/// # Safety
/// The wrapped reference must never be shared across threads. This holds for
/// the engine-provided globals/function tables, which are set once at init and
/// only read from the server thread afterwards.
pub struct SyncWrapper<T: ?Sized>(T);

// SAFETY: `T` is a `&'static` reference to a value that outlives all calls;
// access happens only on the single server thread.
unsafe impl<T: ?Sized> Sync for SyncWrapper<T> {}

// SAFETY: same reasoning as `Sync` — never moved to another thread, and the
// wrapped `&'static` target outlives all uses on the server thread.
unsafe impl<T: ?Sized> Send for SyncWrapper<T> {}

impl<T: Sized> SyncWrapper<T> {
    /// Wrap a value.
    pub const fn new(value: T) -> Self {
        Self(value)
    }
}

impl<T: ?Sized> std::ops::Deref for SyncWrapper<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

/// Executes `f` inside [`std::panic::catch_unwind`], returning `default` if
/// a panic occurs. The panic payload is reported via `eprintln!` which writes
/// directly to stderr — this always works even before the engine is fully
/// initialised, and even if the server console is unavailable.
///
/// # Usage
/// ```ignore
/// pub unsafe extern "C" fn GetEntityAPI2(table: *mut DLL_FUNCTIONS, ver: *mut i32) -> i32 {
///     // SAFETY: table and ver are valid pointers provided by the engine.
///     catch_ffi_panic("GetEntityAPI2", 0, || {
///         // ... real implementation ...
///     })
/// }
/// ```
#[inline]
pub fn catch_ffi_panic<T, F>(name: &str, default: T, f: F) -> T
where
    F: FnOnce() -> T,
{
    // SAFETY: AssertUnwindSafe is sound here because:
    // 1. We are at an FFI boundary — if we don't catch the panic, the process
    //    crashes or enters UB. Catching is strictly safer than not catching.
    // 2. The caller is responsible for ensuring the pointers passed into `f`
    //    remain valid for the duration of the call, which is guaranteed by the
    //    engine contract for all registered callbacks.
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(val) => val,
        Err(payload) => {
            let msg = payload
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
                .unwrap_or("unknown panic payload");
            eprintln!("[GoldSrc.rs PANIC] caught in `{name}`: {msg}");
            default
        }
    }
}
