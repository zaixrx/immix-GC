mod bump;
mod alloc;
mod alloc_api;
mod block;

use alloc_api::*;
use alloc::ScopedGlobalAllocator;

#[derive(Clone, Copy)]
enum ObjectType {
    SynBool,
    SynInt,
    SynString,
    SynStruct,
    SynArrayU8,
}

impl AllocTypeId for ObjectType {}

#[derive(Debug, Copy, Clone)]
struct Person {
    name: &'static str,
    score: usize, 
}

impl AllocObject<ObjectType> for Person {
    const TYPE_ID: ObjectType = ObjectType::SynStruct;
}

#[allow(unused)]
struct ObjectHeader {
    type_id: ObjectType,
    size: usize,
    mark: Mark,
}

impl AllocHeader for ObjectHeader {
    type TypeId = ObjectType;

    fn new<O: AllocObject<Self::TypeId>>(size: usize, mark: Mark) -> Self {
        Self {
            type_id: O::TYPE_ID,
            size,
            mark
        }
    }

    fn new_array(size: usize, mark: Mark) -> Self {
        Self {
            type_id: ObjectType::SynArrayU8,
            size,
            mark
        }
    }

    fn size(&self) -> usize {
        self.size
    }

    fn type_id(&self) -> Self::TypeId {
        self.type_id
    }
}

fn main() -> Result<(), AllocError> {
    let mutator = ScopedGlobalAllocator::<'static, ObjectHeader>::new_static();

    let person = mutator.alloc(Person {
        name: "KOUA Mohamed Anis",
        score: 69420,
    })?;

    println!("[{:016X}] -> name: {}, score: {} ", person.ptr as *const Person as usize, person.name, person.score);

    Ok(())
}
