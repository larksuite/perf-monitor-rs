use std::num::NonZeroIsize;

/// Triage a return value of windows handle to `Some(handle)` or `None`
pub trait HandleUpgrade: Sized {
    fn upgrade(self) -> Option<NonZeroIsize>;
}

impl HandleUpgrade for isize {
    #[inline]
    fn upgrade(self) -> Option<NonZeroIsize> {
        NonZeroIsize::new(self)
    }
}
