use crate::{block::BlockError};
use std::{cell::Cell, ptr::NonNull};

#[derive(Clone, Copy, PartialEq)]
pub enum Mark {
    Live,
    Free,
}

#[derive(Debug)]
pub enum AllocError {
    /// requested invalid `size`
    BadRequest,
    /// out of memory
    OOM,
}

pub trait ScopedRef<T> {
    fn scoped_ref<'scope>(&self, guard: &'scope dyn MutatorScope) -> &'scope T;
}

#[derive(Debug, Clone, Copy)]
pub struct RawPtr<T: Sized> {
    ptr: NonNull<T>,
}

/// used to bridge `RawPtr` and `ScopedPtr`
/// by allowing to apply interior mutability
#[derive(Clone)]
pub struct CellPtr<T: Sized>
where 
    RawPtr<T>: Copy,
{
    inner: Cell<RawPtr<T>>
}

/// used to safely derefrence `RawPtr`
pub struct ScopedPtr<'guard, T: Sized> {
    pub ptr: &'guard T,
}

/// used to define the 'guard lifetime
pub trait MutatorScope {}

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

impl <T: Sized> RawPtr<T> {
    /// Creates a new RawPtr
    /// 
    /// SAFETY: `ptr` must not be null
    pub fn new(ptr: *mut T) -> Self {
        unsafe {
            Self {
                ptr: NonNull::new_unchecked(ptr)
            }
        }
    }
}

impl<T: Sized> ScopedRef<T> for RawPtr<T> {
    fn scoped_ref<'scope>(&self, _guard: &'scope dyn MutatorScope) -> &'scope T {
        unsafe { &*self.ptr.as_ptr() }
    }
}

impl<T: Sized> CellPtr<T>
where 
    RawPtr<T>: Copy,
{
    pub fn get<'guard>(&self, guard: &'guard dyn MutatorScope) -> ScopedPtr<'guard, T> {
        ScopedPtr { ptr: self.inner.get().scoped_ref(guard) }
    }
}

impl<'guard, T: Sized> ScopedPtr<'guard, T> {
    pub fn new(guard: &'guard dyn MutatorScope, ptr: RawPtr<T>) -> Self 
    where 
        RawPtr<T>: Copy
    {
        CellPtr::<T>{
            inner: Cell::new(ptr),
        }.get(guard)
    }
}

impl From<BlockError> for AllocError {
    fn from(e: BlockError) -> Self {
        match e {
            BlockError::BadRequest => AllocError::BadRequest,
            BlockError::OOM => AllocError::OOM,
        }
    }
}

impl<'guard, T: Sized> std::ops::Deref for ScopedPtr<'guard, T> {
    type Target = T;

    fn deref(&self) -> &'guard Self::Target {
        self.ptr
    }
}