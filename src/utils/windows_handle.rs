use std::num::NonZeroIsize;
use windows_sys::Win32::Foundation::CloseHandle;

/// Windows handle wrapper
pub struct Handle(NonZeroIsize);

impl Handle {
    /// Invalid handle leads to UB
    pub unsafe fn new(handle: NonZeroIsize) -> Self {
        Handle(handle)
    }

    pub fn as_handle(&self) -> isize {
        self.0.get()
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0.get()) };
    }
}
