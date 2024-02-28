use alloc::{
    alloc::{alloc, handle_alloc_error},
    boxed::Box,
    rc::Rc,
    sync::Arc,
};

use super::*;

// SAFETY: into_raw and clone_from_raw both create pointers to the same value as deref[_mut]
// and are both safe to convert to references
unsafe impl<T: ?Sized + Erasable> ErasablePtr for Box<T> {
    fn into_raw(self) -> NonNull<()> {
        let ptr = alloc::boxed::Box::into_raw(self).cast();

        // SAFETY: Box is always non-null
        unsafe { NonNull::new_unchecked(ptr) }
    }

    unsafe fn from_raw(ptr: NonNull<()>) -> Self {
        // SAFETY: ptr points to a value of type P which is owned by a box
        unsafe { alloc::boxed::Box::from_raw(T::unerase(ptr).as_ptr()) }
    }
}

// SAFETY: into_raw and clone_from_raw both create pointers to the same value as deref[_mut]
// and are both safe to convert to references
unsafe impl<T: Clone> CloneFromRaw for Box<T> {
    unsafe fn clone_from_raw(value: NonNull<()>) -> NonNull<()> {
        let value = move || {
            // SAFETY: value is a pointer to a valid T
            let value = unsafe { T::unerase(value) };

            // SAFETY: value is a pointer to a valid T
            unsafe { value.as_ref().clone() }
        };

        if core::mem::size_of::<T>() == 0 {
            core::mem::forget(value());
            NonNull::<T>::dangling().cast()
        } else {
            let layout = core::alloc::Layout::new::<T>();
            // SAFETY: layout is not zero-sized
            let Some(ptr) = NonNull::new(unsafe { alloc(layout) }) else {
                handle_alloc_error(layout)
            };

            let ptr = ptr.cast::<T>();
            ptr.as_ptr().write(value());
            ptr.cast()
        }
    }
}

// SAFETY: into_raw and clone_from_raw both create pointers to the same value as deref[_mut]
// and are both safe to convert to references
unsafe impl<T: ?Sized + Erasable> ErasablePtr for Rc<T> {
    fn into_raw(self) -> NonNull<()> {
        let ptr = Rc::into_raw(self).cast_mut().cast();

        // SAFETY: Rc is always non-null
        unsafe { NonNull::new_unchecked(ptr) }
    }

    unsafe fn from_raw(ptr: NonNull<()>) -> Self {
        // SAFETY: ptr points to a value of type P which is owned by a box
        unsafe { Rc::from_raw(T::unerase(ptr).as_ptr()) }
    }
}

// SAFETY: into_raw and clone_from_raw both create pointers to the same value as deref[_mut]
// and are both safe to convert to references
unsafe impl<T: ?Sized + Erasable> CloneFromRaw for Rc<T> {
    unsafe fn clone_from_raw(value: NonNull<()>) -> NonNull<()> {
        {
            // SAFETY: value is a pointer to a valid T
            let value = unsafe { T::unerase(value) };

            Rc::increment_strong_count(value.as_ptr())
        }

        value
    }
}

// SAFETY: into_raw and clone_from_raw both create pointers to the same value as deref[_mut]
// and are both safe to convert to references
unsafe impl<T: ?Sized + Erasable> ErasablePtr for Arc<T> {
    fn into_raw(self) -> NonNull<()> {
        let ptr = Arc::into_raw(self).cast_mut().cast();

        // SAFETY: Rc is always non-null
        unsafe { NonNull::new_unchecked(ptr) }
    }

    unsafe fn from_raw(ptr: NonNull<()>) -> Self {
        // SAFETY: ptr points to a value of type P which is owned by a box
        unsafe { Arc::from_raw(T::unerase(ptr).as_ptr()) }
    }
}

// SAFETY: into_raw and clone_from_raw both create pointers to the same value as deref[_mut]
// and are both safe to convert to references
unsafe impl<T: ?Sized + Erasable> CloneFromRaw for Arc<T> {
    unsafe fn clone_from_raw(value: NonNull<()>) -> NonNull<()> {
        {
            // SAFETY: value is a pointer to a valid T
            let value = unsafe { T::unerase(value) };

            Arc::increment_strong_count(value.as_ptr())
        }

        value
    }
}
