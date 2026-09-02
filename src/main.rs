use crate::rawptr::AllocObject;
use crate::object::{ObjectType, RuntimeError};
use crate::memory::{Memory, Mutator, MutatorView};

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

struct ExampleMutator;

impl Mutator for ExampleMutator {
    type Input = ();
    type Output = ();

    fn run<'memory>(&self, mem: &'memory MutatorView, _input: Self::Input) -> Result<Self::Output, RuntimeError> {
        let object = SynI32 { value: 694201337 };

        let p_object = mem.alloc(object).map_err(|err| RuntimeError::MemoryError(err))?;

        assert_eq!(p_object.value.value, object.value);

        println!(":3");

        Ok(())
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
