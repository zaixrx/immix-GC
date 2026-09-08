use crate::error::{AllocError, RuntimeError};
use crate::rawptr::{AllocObject, AllocRaw};
use crate::safeptr::{MutatorScope, ScopedPtr, ScopedRef};
use crate::stickyimmix::StickyImmixHeap;

// TODO(API): make this user-defined
use crate::object::{ObjectHeader, BaseType};

type Heap = StickyImmixHeap<ObjectHeader>;

/// Used as a temporary view of the heap, must be constructed
/// from `Memory::mutate`
/// 
/// Allows mutation through the internal mutability pattern
pub struct MutatorView<'memory> {
    heap: &'memory Heap
}

impl<'memory> MutatorScope for MutatorView<'memory> { }

impl<'memory> MutatorView<'memory> {
    pub fn new(memory: &'memory Memory) -> Self {
        Self {
            heap: &memory.heap
        }
    }

    pub fn alloc<T>(&'memory self, object: T) -> Result<ScopedPtr<'memory, T>, AllocError>
    where
        T: AllocObject<BaseType> {

        Ok(
            ScopedPtr::new(
                self,
                self.heap.alloc(object)?.scoped_ref(self)
            )
        )
    }
}

pub trait Mutator {
    type Input;
    type Output;

    fn run<'memory>(&self, mem: &'memory MutatorView, input: Self::Input) -> Result<Self::Output, RuntimeError>;
}

pub struct Memory {
    heap: Heap
}

impl Memory {
    pub fn new() -> Self {
        Memory {
            heap: Heap::new()
        }
    }

    pub fn mutate<M: Mutator>(&self, m: &M, input: M::Input) -> Result<M::Output, RuntimeError> {
        let mem = MutatorView::new(self);
        m.run(&mem, input)
    }
}