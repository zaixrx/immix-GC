use std::cell::Cell;

use object::SynInteger;

use crate::error::RuntimeError;
use crate::memory::{Memory, Mutator, MutatorView};
use crate::safeptr::CellPtr;

mod error;
mod block;
mod bump;
mod rawptr;
mod stickyimmix;
mod safeptr;
mod memory;

// TODO(API): make that user defined
mod object;

#[derive(Clone, Copy, PartialEq)]
enum BorrowFlag {
    None,
    Shared,
    Exclusive
}

/// acts as an owner for `SynInteger` providing safe mutable access
/// via RefCell like interior-mutability
struct SynIntegerMut {
    inner: CellPtr<SynInteger>,
    _borrow: Cell<BorrowFlag>
}

impl SynIntegerMut {
    fn alloc<'guard>(
        mem: &'guard MutatorView,
        value: isize
    ) -> Result<Self, RuntimeError> {
        mem.alloc(SynInteger {
            value
        })
        .map(|ptr| {
            Self {
                inner: CellPtr::new_from(ptr),
                _borrow: Cell::new(BorrowFlag::None)
            }
        })
        .map_err(Into::into)
    }

    /// # Advantage
    /// The advatange of having a closure, is you don't need to keep track
    /// of when the mutable borrow's lifetime ends
    /// 
    /// # DisAdvantage
    /// The dis-advatange on the other hand is that you'll need to have shit
    /// ton of closures everywhere which isn't near pleasent if it scales with
    /// other Object, which is the goal
    /// 
    /// # Panic
    /// panics if there is a mutable or an immutable reference in the wild
    fn borrow_mut<'guard, O, F>(
        &self,
        mem: &'guard MutatorView, f: F
    ) -> O
    where 
        F: for <'a> FnOnce(&'a mut SynInteger) -> O
    {
        if self._borrow.get() != BorrowFlag::None {
            panic!("borrow_mut: I dunno what to say to you, just fuck off");
        }

        let mref = unsafe {
            std::ptr::from_ref(self.inner.get(mem).as_ref()).cast_mut().as_mut().expect("unchecked")
        };

        self._borrow.set(BorrowFlag::Exclusive);

        let result = f(mref);

        self._borrow.set(BorrowFlag::None);

        result
    }

    /// # Panic
    /// panics if there is a mutable reference in the wild
    fn borrow<'guard, O, F>(
        &self,
        mem: &'guard MutatorView, f: F
    ) -> O
    where 
        F: for <'a> FnOnce(&'a SynInteger) -> O
    {
        if self._borrow.get() == BorrowFlag::Exclusive {
            panic!("borrow: I dunno what to say to you, just fuck off");
        }

        let imref = self.inner.get(mem).as_ref();

        let prev = self._borrow.replace(BorrowFlag::Shared);

        let result = f(imref);

        self._borrow.set(prev);

        result
    }
}

struct ExampleMutator;

impl Mutator for ExampleMutator {
    type Input = ();
    type Output = ();

    fn run<'memory>(&self, mem: &'memory MutatorView, _input: Self::Input) -> Result<Self::Output, RuntimeError> {
        let int = SynIntegerMut::alloc(mem, 1)?;

        int.borrow(mem, |a: &SynInteger| {
            int.borrow(mem, |b: &SynInteger| {
                println!("did you know cat can meow? also here is before: {}", b.value);
            });

            println!("before: {}", a.value);

            /* the following code panics
            int.borrow_mut(mem, |b: &mut SynInteger| -> () {
                b.value = 0xBAD1DEA;
            });
            */
        });

        // it's best practise to shadow `int` and not nest shit like an asshole would
        let _ = int.borrow_mut(mem, |a: &mut SynInteger| {
            a.value = 255;

            /* the following code panics
            int.borrow_mut(mem, |b: &mut SynInteger| -> () {
                b.value = 0xBAD1DEA;
            });
            */
        });

        int.borrow(mem, |a: &SynInteger| {
            int.borrow(mem, |b: &SynInteger| {
                println!("did you know dogs can bark? cool right? anyways here is after: {}", b.value);
            });

            println!("after: {}", a.value);
        });

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
