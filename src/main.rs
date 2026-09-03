use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use crate::rawptr::AllocObject;
use crate::object::{ObjectType, RuntimeError};
use crate::memory::{Memory, Mutator, MutatorView};
use crate::safeptr::{CellPtr, ScopedPtr};

mod error;

mod block;
mod bump;

mod rawptr;
mod stickyimmix;

mod safeptr;
mod memory;

// TODO(API): make that user defined
mod object;

#[derive(Copy, Clone)]
struct SynI32 {
    value: i32
}

impl AllocObject<ObjectType> for SynI32 {
    const TYPE_ID: ObjectType = ObjectType::SynI32;
}

struct SynI32_Holder {
    value: CellPtr<SynI32>
}

impl SynI32_Holder {
    fn new(ptr: ScopedPtr<'_, SynI32>) -> Self {
        Self {
            value: CellPtr::new_from(ptr)
        }
    }
}

struct ExampleMutator;

impl Mutator for ExampleMutator {
    type Input = ();
    type Output = ();

    fn run<'memory>(&self, mem: &'memory MutatorView, _input: Self::Input) -> Result<Self::Output, RuntimeError> {
        let object = SynI32 { value: 694201337 };

        let p_object = mem.alloc(object).map_err(|err| RuntimeError::MemoryError(err))?;

        // assert_eq!(p_object.value.value, v_object.value);

        println!(":3");

        Ok(())
    }
}

struct RefCell<T> {
    data: *mut T,
    is_mutable: AtomicBool,
    mut_refs: AtomicUsize
}

impl<T> RefCell<T> {
    fn borrow_immu<'a>(&'a self) -> &'a T {
        let mut do_panic = false;
        do_panic |= self.is_mutable.load(Ordering::Acquire);
        
        if do_panic {
            panic!("wtf man");
        }

        self.mut_refs.store(self.mut_refs.load(Ordering::Acquire) + 1, Ordering::Release);

        unsafe {
            &*self.data
        }
    }

    fn borrow_mut<'a>(&'a self) -> &'a mut T {
        let mut do_panic = false;
        do_panic |= self.is_mutable.load(Ordering::Acquire);
        do_panic |= self.mut_refs.load(Ordering::Acquire) > 0;

        if do_panic {
            panic!("wtf man");
        }

        self.is_mutable.store(true, Ordering::Release);

        unsafe {
            &mut *self.data
        }
    }
}

fn main() -> () {
    let memory = Memory::new();
    let mutator = ExampleMutator;

    memory.mutate(&mutator, ()).unwrap_or_else(|err| {
        eprintln!("ExampleMutator: failed to execute, {:?}", err);
        std::process::exit(1)
    });
}
