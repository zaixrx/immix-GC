use crate::stickyimmix::StickyImmixHeap;
use crate::object::{BaseHeader, BaseType};
use crate::rawptr::{AllocObject, AllocRaw};
use crate::error::{AllocError, RuntimeError};
use crate::safeptr::{MutatorScope, ScopedPtr, ScopedRef};

type Heap = StickyImmixHeap<BaseHeader>;

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

/// Used as a temporary guard for the heap, must be constructed in `Memory::mutate`
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