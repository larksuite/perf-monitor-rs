use std::ptr::NonNull;

/// Triage a pointer to `Some(NonNull<T>)` or `None`
pub trait PointerUpgrade<T>: Sized {
    fn upgrade(self) -> Option<NonNull<T>>;
}

impl<T> PointerUpgrade<T> for *const T {
    #[inline]
    fn upgrade(self) -> Option<NonNull<T>> {
        NonNull::new(self as *mut _)
    }
}

impl<T> PointerUpgrade<T> for *mut T {
    #[inline]
    fn upgrade(self) -> Option<NonNull<T>> {
        NonNull::new(self)
    }
}
