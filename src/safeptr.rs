use crate::rawptr::RawPtr;

use std::cell::Cell;

/// used to bridge `RawPtr` and `ScopedPtr`by allowing to apply
/// interior mutability specifically allowing for there to be 
/// mutable pointers
#[derive(Clone)]
pub struct CellPtr<T: Sized> {
    inner: Cell<RawPtr<T>>
}

impl<T: Sized> CellPtr<T> {
    /// assumes `ptr` is the result of a success `Heap::alloc` call
    pub fn new_from(ptr: ScopedPtr<'_, T>) -> Self {
        Self {
            inner: Cell::new(
                RawPtr::new(ptr.value)
            )
        }
    }

    pub fn get<'guard>(&self, guard: &'guard dyn MutatorScope) -> ScopedPtr<'guard, T> {
        ScopedPtr {
            value: self.inner.get().scoped_ref(guard)
        }
    }

    pub fn set<'guard>(&self, scoped: ScopedPtr<'guard, T>) {
        let ptr = RawPtr::new(scoped.value);
        self.inner.set(ptr);
    }
}

/// used to define the 'guard lifetime
pub trait MutatorScope {}

/// used to safely derefrence `RawPtr`
pub struct ScopedPtr<'guard, T: Sized> {
    value: &'guard T,
}

impl<'guard, T: Sized> ScopedPtr<'guard, T> {
    // 'guard here may be redudent, but is a good enforcement in case
    // `value` lifetime outlives `_guard`'s
    pub fn new(_guard: &'guard dyn MutatorScope, value: &'guard T) -> Self {
        Self {
            value
        }
    }

    /// give access to the underlying reference held by `ScopedPtr`
    pub fn as_ref(&self) -> &'guard T {
        self.value
    }
}

impl<'guard, T: Sized> std::ops::Deref for ScopedPtr<'guard, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.value
    }
}


/// Used as a safety extension for `RawPtr`. See [this](https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html#aliasing-rules) 
/// about rust's aliasing rules, and [that](https://rust-hosted-langs.github.io/book/chapter-interp-alloc.html#pointers) 
/// on how that would look like (implementation-wise) here
pub trait ScopedRef<T> {
    fn scoped_ref<'scope>(&self, guard: &'scope dyn MutatorScope) -> &'scope T;
}

impl<T: Sized> ScopedRef<T> for RawPtr<T> {
    fn scoped_ref<'scope>(&self, _guard: &'scope dyn MutatorScope) -> &'scope T {
        unsafe { &*self.ptr.as_ptr() }
    }
}
