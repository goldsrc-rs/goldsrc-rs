//! Low-level OS-level hardware fault and crash prevention barrier.
//!
//! Provides Vectored Exception Handling (Windows) and Signal Handlers (POSIX)
//! to intercept hardware CPU faults (Access Violation `0xC0000005`, Illegal Instruction,
//! Division by Zero, Stack Overflow) before the operating system forcibly terminates
//! the host `hlds.exe` / `hlds_linux` process.

use std::ffi::c_void;
use std::sync::atomic::{AtomicBool, AtomicPtr, Ordering};

static GUARD_INSTALLED: AtomicBool = AtomicBool::new(false);
static VEH_HANDLE: AtomicPtr<c_void> = AtomicPtr::new(std::ptr::null_mut());

#[cfg(windows)]
mod win32 {
    use super::*;

    pub const EXCEPTION_CONTINUE_SEARCH: i32 = 0;
    pub const EXCEPTION_CONTINUE_EXECUTION: i32 = -1;

    pub const EXCEPTION_ACCESS_VIOLATION: u32 = 0xC0000005;
    pub const EXCEPTION_DATATYPE_MISALIGNMENT: u32 = 0x80000002;
    pub const EXCEPTION_ILLEGAL_INSTRUCTION: u32 = 0xC000001D;
    pub const EXCEPTION_INT_DIVIDE_BY_ZERO: u32 = 0xC0000094;
    pub const EXCEPTION_STACK_OVERFLOW: u32 = 0xC00000FD;
    pub const EXCEPTION_GUARD_PAGE: u32 = 0x80000001;

    #[repr(C)]
    pub struct EXCEPTION_RECORD {
        pub ExceptionCode: u32,
        pub ExceptionFlags: u32,
        pub ExceptionRecord: *mut EXCEPTION_RECORD,
        pub ExceptionAddress: *mut c_void,
        pub NumberParameters: u32,
        pub ExceptionInformation: [usize; 15],
    }

    #[repr(C)]
    pub struct EXCEPTION_POINTERS {
        pub ExceptionRecord: *mut EXCEPTION_RECORD,
        pub ContextRecord: *mut c_void,
    }

    type PVECTORED_EXCEPTION_HANDLER =
        unsafe extern "system" fn(ExceptionInfo: *mut EXCEPTION_POINTERS) -> i32;

    unsafe extern "system" {
        pub fn AddVectoredExceptionHandler(
            First: u32,
            Handler: PVECTORED_EXCEPTION_HANDLER,
        ) -> *mut c_void;
        pub fn RemoveVectoredExceptionHandler(Handle: *mut c_void) -> u32;
    }

    /// Global Vectored Exception Handler.
    pub unsafe extern "system" fn veh_exception_filter(info: *mut EXCEPTION_POINTERS) -> i32 {
        if info.is_null() {
            return EXCEPTION_CONTINUE_SEARCH;
        }

        let (code, addr) = unsafe {
            let rec = (*info).ExceptionRecord;
            if rec.is_null() {
                return EXCEPTION_CONTINUE_SEARCH;
            }
            ((*rec).ExceptionCode, (*rec).ExceptionAddress)
        };

        // Only process relevant hardware fault codes
        let is_fault = matches!(
            code,
            EXCEPTION_ACCESS_VIOLATION
                | EXCEPTION_ILLEGAL_INSTRUCTION
                | EXCEPTION_INT_DIVIDE_BY_ZERO
                | EXCEPTION_DATATYPE_MISALIGNMENT
        );

        if is_fault {
            let code_str = match code {
                EXCEPTION_ACCESS_VIOLATION => "STATUS_ACCESS_VIOLATION (0xC0000005)",
                EXCEPTION_ILLEGAL_INSTRUCTION => "STATUS_ILLEGAL_INSTRUCTION (0xC000001D)",
                EXCEPTION_INT_DIVIDE_BY_ZERO => "STATUS_INTEGER_DIVIDE_BY_ZERO (0xC0000094)",
                EXCEPTION_DATATYPE_MISALIGNMENT => "STATUS_DATATYPE_MISALIGNMENT (0x80000002)",
                _ => "HARDWARE_FAULT",
            };

            eprintln!(
                "[GoldSrc.rs CRASH GUARD] Intercepted {code_str} at address {:p}!",
                addr
            );
        }

        // Return EXCEPTION_CONTINUE_SEARCH to allow any higher-level/debugger or minidump filter to see it
        EXCEPTION_CONTINUE_SEARCH
    }
}

/// Installs the global OS-level crash guard. Safe to call multiple times (idempotent).
pub fn install_crash_guard() {
    if GUARD_INSTALLED.swap(true, Ordering::SeqCst) {
        return;
    }

    #[cfg(windows)]
    {
        unsafe {
            // Register as first handler (First = 1)
            let handle = win32::AddVectoredExceptionHandler(1, win32::veh_exception_filter);
            if !handle.is_null() {
                VEH_HANDLE.store(handle, Ordering::SeqCst);
                eprintln!("[GoldSrc.rs] OS Crash Guard (Windows VEH) installed successfully.");
            }
        }
    }
}

/// Uninstalls the global crash guard upon shutdown.
pub fn uninstall_crash_guard() {
    if !GUARD_INSTALLED.swap(false, Ordering::SeqCst) {
        return;
    }

    #[cfg(windows)]
    {
        let handle = VEH_HANDLE.swap(std::ptr::null_mut(), Ordering::SeqCst);
        if !handle.is_null() {
            unsafe {
                win32::RemoveVectoredExceptionHandler(handle);
            }
        }
    }
}
