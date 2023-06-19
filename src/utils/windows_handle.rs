use std::os::raw::c_void;
use std::ptr::NonNull;
use winapi::um::handleapi::CloseHandle;

/// Windows handle wrapper
pub struct Handle(NonNull<c_void>);

impl Handle {
    pub unsafe fn new(handle: NonNull<c_void>) -> Self {
        Handle(handle)
    }

    pub fn as_ptr(&self) -> *mut c_void {
        self.0.as_ptr()
    }
}

impl Drop for Handle {
    fn drop(&mut self) {
        unsafe { CloseHandle(self.0.as_ptr()) };
    }
}
