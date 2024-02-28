#![no_std]
#![forbid(clippy::undocumented_unsafe_blocks, clippy::missing_safety_doc)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

use core::{marker::PhantomData, ops, ptr::NonNull};

/// # Safety
///
/// If Self: [`ops::Deref`], then [`ErasablePtr::into_raw`] must point to the value
/// `Self::Target` and must be safe to convert to a reference `&Self::Target` while `self` is alive
///
/// If Self: [`ops::DerefMut`], then [`ops::Deref::deref`] and [`ops::DerefMut::deref_mut`] must
///     point to the same value. And the pointer returned from [`ErasablePtr::into_raw`] must be
///     safe to convert to a reference `&mut Self::Target` while `self` is alive
///
/// If `Self: Copy` then `into_raw` will not end self's lifetime
pub unsafe trait ErasablePtr {
    fn into_raw(self) -> NonNull<()>;

    /// # Safety
    ///
    /// This pointer must have been obtained from [`into_raw`] or [`clone_from_raw`]
    unsafe fn from_raw(ptr: NonNull<()>) -> Self;
}

/// # Safety
///
/// If Self: [`ops::Deref`], then [`ErasablePtr::clone_from_raw`]
///     must point to the value `Self::Target` and must be safe to convert to a reference
///     `&Self::Target` while `self` is alive
///
/// If Self: [`ops::DerefMut`], then [`ops::Deref::deref`] and [`ops::DerefMut::deref_mut`] must
///     point to the same value. And the pointer returned from [`ErasablePtr::clone_from_raw`] must
///     be safe to convert to a reference `&mut Self::Target` while `self` is alive
///
/// If `Self: Copy` then `into_raw` will not end self's lifetime
pub unsafe trait CloneFromRaw: Clone + ErasablePtr {
    /// # Safety
    ///
    /// This pointer must have been obtained from [`into_raw`] or a previous call to [`clone_from_raw`]
    unsafe fn clone_from_raw(_ptr: NonNull<()>) -> NonNull<()>;
}

/// # Safety
///
/// you must not create a reference to the value behind the ptr in unerase
/// not a shared reference (&T) or a mutable reference (&mut T)
///
/// you must not write to the value behind the ptr in unerase
///
/// you may only read from the ptr using core::ptr::read
pub unsafe trait Erasable {
    /// # Safety
    ///
    /// The ptr must point to an allocated and initialized value of Self
    unsafe fn unerase(ptr: NonNull<()>) -> NonNull<Self>;
}

// SAFETY: no references are created, no reads/writes are done
unsafe impl<T> Erasable for T {
    unsafe fn unerase(ptr: NonNull<()>) -> NonNull<Self> {
        ptr.cast()
    }
}

// SAFETY: into_raw and clone_from_raw both create pointers to the same value as deref
// and are both safe to convert to references
unsafe impl<T: ?Sized + Erasable> ErasablePtr for &T {
    #[inline]
    fn into_raw(self) -> NonNull<()> {
        NonNull::from(self).cast()
    }

    #[inline]
    unsafe fn from_raw(ptr: NonNull<()>) -> Self {
        // SAFETY: this is a pointer passed to into_raw or clone_from_raw
        // so it is safe to convert it back to a reference
        unsafe { &*T::unerase(ptr).as_ptr() }
    }
}

// SAFETY: Self is Copy, so a passthru implementation is correct
unsafe impl<T: ?Sized + Erasable> CloneFromRaw for &T {
    #[inline]
    unsafe fn clone_from_raw(ptr: NonNull<()>) -> NonNull<()> {
        ptr
    }
}

// SAFETY: into_raw and clone_from_raw both create pointers to the same value as deref[_mut]
// and are both safe to convert to references
unsafe impl<T: ?Sized + Erasable> ErasablePtr for &mut T {
    #[inline]
    fn into_raw(self) -> NonNull<()> {
        NonNull::from(self).cast()
    }

    #[inline]
    unsafe fn from_raw(ptr: NonNull<()>) -> Self {
        // SAFETY: this is a pointer passed to into_raw or clone_from_raw
        // so it is safe to convert it back to a reference
        unsafe { &mut *T::unerase(ptr).as_ptr() }
    }
}

// SAFETY: deref[_mut] is not implemnted for RawThin and from_raw doesn't end it's lifetime
unsafe impl<P> ErasablePtr for RawThin<P> {
    #[inline]
    fn into_raw(self) -> NonNull<()> {
        self.ptr
    }

    #[inline]
    unsafe fn from_raw(ptr: NonNull<()>) -> Self {
        Self {
            ptr,
            _ty: PhantomData,
        }
    }
}

// SAFETY: Self is Copy, so a passthru implementation is correct
unsafe impl<P> CloneFromRaw for RawThin<P> {
    #[inline]
    unsafe fn clone_from_raw(ptr: NonNull<()>) -> NonNull<()> {
        ptr
    }
}

// SAFETY: deref[_mut] is not implemnted for RawThin and from_raw doesn't end it's lifetime
unsafe impl<P> ErasablePtr for CopyThin<P> {
    #[inline]
    fn into_raw(self) -> NonNull<()> {
        self.raw.ptr
    }

    #[inline]
    unsafe fn from_raw(ptr: NonNull<()>) -> Self {
        Self {
            raw: RawThin {
                ptr,
                _ty: PhantomData,
            },
        }
    }
}

// SAFETY: Self is Copy, so a passthru implementation is correct
unsafe impl<P> CloneFromRaw for CopyThin<P> {
    #[inline]
    unsafe fn clone_from_raw(ptr: NonNull<()>) -> NonNull<()> {
        ptr
    }
}

// SAFETY: into_raw and clone_from_raw both create pointers to the same value as deref[_mut]
// and are both safe to convert to references
unsafe impl<P: ErasablePtr> ErasablePtr for Thin<P> {
    #[inline]
    fn into_raw(self) -> NonNull<()> {
        self.raw.ptr
    }

    #[inline]
    unsafe fn from_raw(ptr: NonNull<()>) -> Self {
        Self {
            raw: RawThin {
                ptr,
                _ty: PhantomData,
            },
        }
    }
}

// SAFETY: defering to P::clone_from_raw
unsafe impl<P: CloneFromRaw> CloneFromRaw for Thin<P> {
    #[inline]
    unsafe fn clone_from_raw(ptr: NonNull<()>) -> NonNull<()> {
        P::clone_from_raw(ptr)
    }
}

#[repr(transparent)]
pub struct RawThin<P> {
    ptr: NonNull<()>,
    _ty: PhantomData<P>,
}

impl<P> Copy for RawThin<P> {}
impl<P> Clone for RawThin<P> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: ErasablePtr> RawThin<P> {
    pub fn new(ptr: P) -> Self {
        let ptr = P::into_raw(ptr);
        Self {
            ptr,
            _ty: PhantomData,
        }
    }

    /// # Safety
    ///
    /// the caller must own a valid RawThin<P>
    pub unsafe fn into_inner(self) -> P {
        P::from_raw(self.ptr)
    }

    /// # Safety
    ///
    /// the caller must own a valid RawThin<P>
    pub unsafe fn clone_ptr(self) -> Self
    where
        P: CloneFromRaw,
    {
        Self {
            ptr: P::clone_from_raw(self.ptr),
            _ty: PhantomData,
        }
    }
}

#[repr(transparent)]
pub struct CopyThin<P> {
    raw: RawThin<P>,
}

impl<P> Copy for CopyThin<P> {}
impl<P> Clone for CopyThin<P> {
    #[inline]
    fn clone(&self) -> Self {
        *self
    }
}

impl<P: ErasablePtr + Copy> CopyThin<P> {
    pub fn new(ptr: P) -> Self {
        Self {
            raw: RawThin::new(ptr),
        }
    }
}

impl<P: ErasablePtr> CopyThin<P> {
    #[inline]
    pub fn into_inner(self) -> P {
        // SAFETY:
        // CopyThin can only be constructed from pointer types that are themselves Copy
        // so every value of `CopyThin` owns a value of `P`, even if copies are made
        unsafe { self.raw.into_inner() }
    }
}

#[repr(transparent)]
pub struct Thin<P: ErasablePtr> {
    raw: RawThin<P>,
}

impl<P: CloneFromRaw> Clone for Thin<P> {
    fn clone(&self) -> Self {
        Self {
            // SAFETY: Thin has an invariant that the pointer P is still alive
            // as long as the Thin<P> is alive
            raw: unsafe { self.raw.clone_ptr() },
        }
    }
}

impl<P: ErasablePtr> Drop for Thin<P> {
    fn drop(&mut self) {
        if core::mem::needs_drop::<P>() {
            // SAFETY: Thin has an invariant that the pointer P is still alive
            // as long as the Thin<P> is alive. And now that we are in the
            // destructor, we are finally allowed to destroy the underlying P
            let _ = unsafe { self.raw.into_inner() };
        }
    }
}

impl<P: ErasablePtr> Thin<P> {
    pub fn new(ptr: P) -> Self {
        Self {
            raw: RawThin::new(ptr),
        }
    }

    pub fn into_inner(self) -> P {
        let raw = self.raw;
        core::mem::forget(self);
        // SAFETY: we don't run Thin<P>'s destructor and take ownership of the
        // Thin<P>, so we have ownership over the P inside as well
        unsafe { raw.into_inner() }
    }
}

impl<P: ErasablePtr + ops::Deref> ops::Deref for Thin<P>
where
    P::Target: Erasable,
{
    type Target = P::Target;

    fn deref(&self) -> &Self::Target {
        // SAFETY: by Erasable's safety requirements, since P: ops::Deref
        // the pointer given by P::into_raw is safe to convert to &T::Target
        // while P is alive
        unsafe { &*<P::Target as Erasable>::unerase(self.raw.ptr).as_ptr() }
    }
}

impl<P: ErasablePtr + ops::DerefMut> ops::DerefMut for Thin<P>
where
    P::Target: Erasable,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        // SAFETY: by Erasable's safety requirements, since P: ops::DerefMut
        // the pointer given by P::into_raw is safe to convert to &mut T::Target
        // while P is alive
        unsafe { &mut *<P::Target as Erasable>::unerase(self.raw.ptr).as_ptr() }
    }
}
