//! Simple VTable hook abstraction based on virtual function index offsets.

use core::ffi::c_void;

#[cfg(windows)]
pub type VTablePtr = *mut *mut c_void;

/// Basic VTable detour manager for overriding C++ virtual methods.
pub struct VTableHook {
    target_object: *mut c_void,
    vtable: *mut *mut c_void,
    original_vtable: *mut *mut c_void,
}

impl VTableHook {
    /// # Safety
    /// `target_object` must be a valid pointer to a C++ object with a VTable.
    pub unsafe fn new(target_object: *mut c_void) -> Option<Self> {
        if target_object.is_null() {
            return None;
        }

        // SAFETY: `target_object` is checked for null, points to a C++ object where the first word is the vtable pointer.
        let vtable = *(target_object as *mut *mut *mut c_void);
        if vtable.is_null() {
            return None;
        }

        Some(Self {
            target_object,
            vtable,
            original_vtable: vtable,
        })
    }

    /// Read virtual method address at given index offset.
    /// # Safety
    /// `index` must be a valid virtual table offset within bounds.
    pub unsafe fn get_func(&self, index: usize) -> *mut c_void {
        // SAFETY: dereferencing slot pointer at index offset in vtable array.
        *self.vtable.add(index)
    }

    /// Swap virtual method pointer at given index offset with new hook function.
    /// # Safety
    /// Requires valid `index` and `hook_fn` function pointer. Memory write permissions are handled dynamically.
    pub unsafe fn hook_index(&mut self, index: usize, hook_fn: *mut c_void) -> *mut c_void {
        let orig = self.get_func(index);
        let slot = self.vtable.add(index);

        #[cfg(windows)]
        {
            use windows_sys::Win32::System::Memory::{
                VirtualProtect, PAGE_EXECUTE_READWRITE, PAGE_PROTECTION_FLAGS,
            };
            let mut old_protect: PAGE_PROTECTION_FLAGS = 0;
            let slot_ptr = slot as *mut c_void;
            // SAFETY: VirtualProtect changes memory protection for slot pointer on Windows.
            VirtualProtect(
                slot_ptr,
                core::mem::size_of::<*mut c_void>(),
                PAGE_EXECUTE_READWRITE,
                &mut old_protect,
            );
            *slot = hook_fn;
            VirtualProtect(
                slot_ptr,
                core::mem::size_of::<*mut c_void>(),
                old_protect,
                &mut old_protect,
            );
        }

        #[cfg(not(windows))]
        {
            use libc::{
                c_void as libc_void, mprotect, sysconf, _SC_PAGESIZE, PROT_EXEC, PROT_READ,
                PROT_WRITE,
            };
            let page_size = sysconf(_SC_PAGESIZE) as usize;
            let slot_addr = slot as usize;
            let page_start = (slot_addr & !(page_size - 1)) as *mut libc_void;

            // SAFETY: Changing memory page permissions to allow writing new vtable pointer on Unix/Linux.
            mprotect(page_start, page_size, PROT_READ | PROT_WRITE | PROT_EXEC);
            *slot = hook_fn;
            mprotect(page_start, page_size, PROT_READ | PROT_EXEC);
        }

        orig
    }

    pub fn original_vtable(&self) -> *mut *mut c_void {
        self.original_vtable
    }

    pub fn target(&self) -> *mut c_void {
        self.target_object
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    unsafe extern "C" fn original_fn() -> i32 {
        42
    }

    unsafe extern "C" fn hook_fn() -> i32 {
        1337
    }

    #[test]
    fn test_vtable_hook_swap() {
        unsafe {
            let mut fake_vtable: [*mut c_void; 2] =
                [original_fn as *mut c_void, core::ptr::null_mut()];
            let vtable_ptr = fake_vtable.as_mut_ptr();
            let mut object_ptr = vtable_ptr;
            let target_obj = &mut object_ptr as *mut _ as *mut c_void;

            let mut hook = VTableHook::new(target_obj).expect("Failed to create VTableHook");
            assert_eq!(hook.get_func(0), original_fn as *mut c_void);

            let orig = hook.hook_index(0, hook_fn as *mut c_void);
            assert_eq!(orig, original_fn as *mut c_void);
            assert_eq!(hook.get_func(0), hook_fn as *mut c_void);
        }
    }
}
