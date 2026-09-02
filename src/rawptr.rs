use std::ptr::NonNull;

use crate::{error::AllocError, object::Mark};

/// by default a instance of `AllowRaw` returns a `RawPtr`, which
/// are unsafe to use because it requires dereferncing a pointer
/// so `ScopedPtr` alongside
#[derive(Debug)]
pub struct RawPtr<T: Sized> {
    pub(crate) ptr: NonNull<T>,
}

impl<T: Sized> Clone for RawPtr<T> {
    fn clone(&self) -> Self {
        Self {
            ptr: self.ptr.clone()
        }
    }
}

impl<T: Sized> Copy for RawPtr<T> {}

impl <T: Sized> RawPtr<T> {
    /// Creates a new `RawPtr` containing the given pointer (`ptr`).
    ///
    /// # Safety
    ///
    /// `ptr` must not be null.
    pub unsafe fn new(ptr: *const T) -> Self {
        unsafe {
            Self {
                ptr: NonNull::new_unchecked(ptr as *mut T)
            }
        }
    }
}

pub trait AllocTypeId: Copy + Clone {}

pub trait AllocObject<T: AllocTypeId> {
    const TYPE_ID: T;
}

pub trait AllocHeader: Sized {
    /// Associated type that identifies the allocated object type
    type TypeId: AllocTypeId;

    /// Create a new header for object type O
    fn new<O: AllocObject<Self::TypeId>>(size: usize, mark: Mark) -> Self;

    /// Create a new header for an array type
    fn new_array(size: usize, mark: Mark) -> Self;

    /// Get the size of the object in bytes
    fn size(&self) -> usize;

    /// Get the type of the object
    fn type_id(&self) -> Self::TypeId;
}

pub trait AllocRaw {
    /// An implementation of an object header type
    type Header: AllocHeader;

    /// Allocate a single object of type T.
    fn alloc<T>(&self, object: T) -> Result<RawPtr<T>, AllocError>
    where
        T: AllocObject<<Self::Header as AllocHeader>::TypeId>;

    /// Allocating an array allows the user to put anything in the resulting data
    /// block but the type of the memory block will simply be 'Array'. No other
    /// type information will be stored in the object header.
    /// This is just a special case of alloc<T>() for T=u8 but a count > 1 of u8
    /// instances.  The caller is responsible for the content of the array.
    fn alloc_array(&self, size_bytes: usize) -> Result<RawPtr<u8>, AllocError>;

    /// Given a bare pointer to an object, return the expected header address
    fn get_header(object: NonNull<()>) -> NonNull<Self::Header>;

    /// Given a bare pointer to an object's header, return the expected object address
    fn get_object(header: NonNull<Self::Header>) -> NonNull<()>;
}