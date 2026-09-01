use crate::alloc_api::{AllocError, AllocHeader, AllocObject, AllocTypeId, Mark, MutatorScope};

mod block;
mod bump;
mod alloc;
mod alloc_api;

#[derive(Clone, Copy)]
enum ObjectType {
    SynStruct,
    SynArray,
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
    size: usize,
    mark: Mark,
    type_id: ObjectType,
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
            type_id: ObjectType::SynArray,
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

fn introduce_guard_scope() {

    
}

fn main() -> () {
}
