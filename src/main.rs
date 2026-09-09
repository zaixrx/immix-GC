use crate::object::BaseType;
use crate::error::RuntimeError;
use crate::rawptr::AllocObject;
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

struct SynI32 {
    value: i32
}

impl SynI32 {
    fn alloc<'guard>(mem: &'guard MutatorView, value: i32) -> Result<ScopedPtr<'guard, Self>, RuntimeError> {
        mem.alloc(SynI32 { value }).map_err(Into::into)
    }
}

impl<'guard> ScopedPtr<'guard, SynI32> {
    fn mutate<O, F>(&self, f: F) -> O
    where 
        F: for <'a> FnOnce(&'a mut SynI32) -> O
    {
        let m: &mut SynI32 = unsafe {
            std::ptr::from_ref(self.value).cast_mut().as_mut_unchecked()
        };
        f(m)
    }
}

impl AllocObject<BaseType> for SynI32 {
    const TYPE_ID: BaseType = BaseType::SynInteger;
}

struct ExampleMutator;

impl Mutator for ExampleMutator {
    type Input = ();
    type Output = ();

    fn run<'memory>(&self, mem: &'memory MutatorView, _input: Self::Input) -> Result<Self::Output, RuntimeError> {
        let a = SynI32::alloc(mem, 1)?;

        a.mutate(|a| {
            a.value = 255;
        });

        println!("{}", a.value.value);

        Ok(())
    }
}

fn main() {
    let memory = Memory::new();

    let mutator = ExampleMutator;

    memory.mutate(&mutator, ()).unwrap_or_else(|err| {
        eprintln!("ExampleMutator: failed to execute, {:?}", err);
        std::process::exit(1)
    });
}