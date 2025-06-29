//! This an `Arc` reimplementation, specifically tailored to be used in a container
//!
//! Main difference from regular `Arc` is a [`Arc::try_move_out`] method, performing same counter checks as `Drop` implementation does. This essentially allows you to guarantee that if ALL the live [`Arc`]s are consumed with [`Arc::try_move_out`], then **exactly one** of these calls will succeed

use alloc::boxed::Box;
use core::{
    fmt::Debug,
    mem::MaybeUninit,
    ops::Deref,
    sync::atomic::{AtomicUsize, Ordering},
};

#[derive(Debug)]
#[repr(C)]
struct ArcInner<T> {
    count: AtomicUsize,
    data: T,
}

// NOTE: container pointer is **usually non-null**, but will be nullptr if the struct was moved out of
pub(super) struct Arc<T>(*mut ArcInner<T>);

// SAFETY: clone and drop logic are implemented with proper atomic checks
unsafe impl<T: Send + Sync> Send for Arc<T> {}
// SAFETY: clone and drop logic are implemented with proper atomic checks
unsafe impl<T: Send + Sync> Sync for Arc<T> {}

impl<T> Deref for Arc<T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner().data
    }
}

impl<T: Debug> Debug for Arc<T> {
    #[inline]
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        T::fmt(self, f)
    }
}

impl<T> Clone for Arc<T> {
    #[inline]
    fn clone(&self) -> Self {
        const MAX_REFCOUNT: usize = (isize::MAX) as usize;
        let old_size = self.inner().count.fetch_add(1, Ordering::Relaxed);
        assert!(
            old_size <= MAX_REFCOUNT,
            "Suspiciously many `Arc`s pointing to the same location"
        );
        Self(self.0)
    }
}

impl<T> Arc<MaybeUninit<T>> {
    #[inline]
    pub(super) fn uninit() -> Self {
        let inner = Box::new(ArcInner {
            count: AtomicUsize::new(1),
            data: MaybeUninit::uninit(),
        });
        let ptr = Box::into_raw(inner);
        Self(ptr)
    }

    #[inline]
    pub(super) unsafe fn assume_init(mut self) -> Arc<T> {
        let uptr = core::mem::replace(&mut self.0, core::ptr::null_mut());
        drop(self);
        let ptr = uptr.cast::<ArcInner<T>>();
        Arc(ptr)
    }
}

impl<T> Arc<T> {
    #[inline]
    fn ptr(&self) -> *const ArcInner<T> {
        self.0.cast_const()
    }

    #[inline]
    fn ptr_mut(&mut self) -> *mut ArcInner<T> {
        self.0
    }

    #[inline]
    fn inner(&self) -> &ArcInner<T> {
        // SAFETY: while `Arc` exists, it is guaranteed to have uncontested read access to the inner value
        unsafe { &*self.ptr() }
    }

    #[inline]
    pub(super) fn count(this: &Self) -> usize {
        this.inner().count.load(Ordering::Acquire)
    }

    #[inline]
    pub(super) fn is_unique(&self) -> bool {
        Self::count(self) == 1
    }

    #[inline]
    pub(super) fn get_mut(&mut self) -> Option<&mut T> {
        self.is_unique().then(|| {
            // SAFETY: unique access ensured by the check above
            unsafe { self.get_mut_unchecked() }
        })
    }

    /// ### Safety
    /// Should only be called if current [`Arc`] has unique access to the data
    #[inline]
    pub(super) unsafe fn get_mut_unchecked(&mut self) -> &mut T {
        // SAFETY: (function invariant)
        &mut unsafe { &mut *self.ptr_mut() }.data
    }

    #[inline]
    pub(super) fn try_move_out<O>(mut self, op: impl FnOnce(*mut T) -> O) -> Option<O> {
        // leave nullptr behind, so that actual drop won't double-free
        let ptr = core::mem::replace(&mut self.0, core::ptr::null_mut());
        drop(self);
        {
            // SAFETY: `ptr` points to still-live version of the inner struct
            let count = &unsafe { &*ptr }.count;
            // same code as in drop, that's the point
            if count.fetch_sub(1, Ordering::Release) != 1 {
                return None;
            }
        }
        // get the data pointer
        // SAFETY: unique access ensured by logic above
        let inner = unsafe { &mut *ptr };
        let data_ptr = core::ptr::from_mut(&mut inner.data);
        #[expect(dropping_references, reason = "Ensuring the reference is not live")]
        drop(inner);
        // NOTE: NO references exist at this point, `&mut _` is not Copy!
        // perform the op
        let res = op(data_ptr);
        // deallocate memory as a if it contains `MaybeUninit`
        let uptr = ptr.cast::<ArcInner<MaybeUninit<T>>>();
        // SAFETY: `uptr` was initially created by `Bow::into_raw`
        drop(unsafe { Box::from_raw(uptr) });
        Some(res)
    }
}

impl<T> Drop for Arc<T> {
    #[inline]
    fn drop(&mut self) {
        if self.0.is_null() {
            // moved out of struct, nothing to do
            return;
        }
        if self.inner().count.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }
        self.inner().count.load(Ordering::Acquire);
        // SAFETY: container poainter was initially created by `Bow::into_raw`
        drop(unsafe { Box::from_raw(self.ptr_mut()) });
    }
}
