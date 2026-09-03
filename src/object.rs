use crate::{error::AllocError, rawptr::{AllocHeader, AllocObject, AllocTypeId}};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mark {
    Live,
    Free,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    SynI32,
    SynObject,
    SynArray,
}

impl AllocTypeId for ObjectType {}

pub struct ObjectHeader {
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

    fn mark(&mut self) {
        self.mark = Mark::Live;
    }

    fn is_marked(&self) -> bool {
        self.mark == Mark::Live
    }

    fn size(&self) -> usize {
        self.size
    }

    fn type_id(&self) -> Self::TypeId {
        self.type_id
    }
}

// API(API): make this user-defined
#[derive(Debug)]
pub enum RuntimeError {
    MemoryError(AllocError),
    // ...
}