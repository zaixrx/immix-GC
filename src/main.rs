mod tl_alloc;
mod g_alloc;
mod api_alloc;
mod block;

use api_alloc::*;
use g_alloc::ScopedGlobalAllocator;

#[derive(Clone, Copy)]
enum ObjectType {
    SynBool,
    SynInt,
    SynString,
    SynStruct,
    SynArrayU8,
}

impl AllocTypeId for ObjectType {}

#[derive(Debug, Clone)]
struct Person {
    name: String,
    score: usize, 
}

impl AllocObject<ObjectType> for Person {
    const TYPE_ID: ObjectType = ObjectType::SynStruct;
}

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
    let galloc = GlobalAllocator::<ObjectHeader>::new();

    let person_ptr = galloc.alloc(Person {
        name: String::from("KOUA Mohamed Anis"),
        score: 69420,
    })?;

    let player = unsafe { &mut (*player_ptr) };
    player.name = String::from("Hello, World!");
    player.score = 100;
    dbg!(player);

    let space = galloc.alloc(1 << 20);
    println!("{:?}", space);

    Ok(())
}
