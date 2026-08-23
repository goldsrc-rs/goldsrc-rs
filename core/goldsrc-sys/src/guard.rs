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

#[cfg(not(windows))]
mod posix {
    use super::*;

    pub const SIGSEGV: i32 = 11;
    pub const SIGFPE: i32 = 8;
    pub const SIGILL: i32 = 4;
    pub const SIGBUS: i32 = 7;

    pub const SA_SIGINFO: i32 = 4;
    pub const SA_ONSTACK: i32 = 0x08000000;

    #[repr(C)]
    pub struct siginfo_t {
        pub si_signo: i32,
        pub si_errno: i32,
        pub si_code: i32,
        pub _pad: [u8; 116],
    }

    #[repr(C)]
    pub struct sigaction_t {
        pub sa_sigaction: Option<extern "C" fn(i32, *mut siginfo_t, *mut c_void)>,
        pub sa_mask: [u64; 16],
        pub sa_flags: i32,
        pub sa_restorer: Option<extern "C" fn()>,
    }

    #[repr(C)]
    pub struct stack_t {
        pub ss_sp: *mut c_void,
        pub ss_flags: i32,
        pub ss_size: usize,
    }

    unsafe extern "C" {
        pub fn sigaction(signum: i32, act: *const sigaction_t, oldact: *mut sigaction_t) -> i32;
        pub fn sigaltstack(ss: *const stack_t, old_ss: *mut stack_t) -> i32;
        pub fn write(fd: i32, buf: *const c_void, count: usize) -> isize;
    }

    static mut ALT_STACK: [u8; 64 * 1024] = [0; 64 * 1024];

    pub extern "C" fn posix_signal_handler(
        sig: i32,
        _info: *mut siginfo_t,
        _ucontext: *mut c_void,
    ) {
        let sig_name = match sig {
            SIGSEGV => "SIGSEGV (Segmentation Fault)\n\0",
            SIGFPE => "SIGFPE (Arithmetic Exception / DivByZero)\n\0",
            SIGILL => "SIGILL (Illegal Instruction)\n\0",
            SIGBUS => "SIGBUS (Bus Error / Unaligned Access)\n\0",
            _ => "UNKNOWN_SIGNAL\n\0",
        };

        let msg = b"[GoldSrc.rs CRASH GUARD] Intercepted POSIX Signal: ";
        unsafe {
            write(2, msg.as_ptr() as *const c_void, msg.len());
            let name_bytes = sig_name.as_bytes();
            write(
                2,
                name_bytes.as_ptr() as *const c_void,
                name_bytes.len() - 1,
            );
        }
    }

    pub fn install_posix_guard() {
        unsafe {
            let alt = stack_t {
                ss_sp: std::ptr::addr_of_mut!(ALT_STACK) as *mut c_void,
                ss_flags: 0,
                ss_size: 64 * 1024,
            };
            sigaltstack(&alt, std::ptr::null_mut());

            let mut sa: sigaction_t = std::mem::zeroed();
            sa.sa_sigaction = Some(posix_signal_handler);
            sa.sa_flags = SA_SIGINFO | SA_ONSTACK;

            for &sig in &[SIGSEGV, SIGFPE, SIGILL, SIGBUS] {
                sigaction(sig, &sa, std::ptr::null_mut());
            }

            let init_msg = b"[GoldSrc.rs] OS Crash Guard (POSIX Signals) installed successfully.\n";
            write(2, init_msg.as_ptr() as *const c_void, init_msg.len());
        }
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

    #[cfg(not(windows))]
    {
        posix::install_posix_guard();
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
