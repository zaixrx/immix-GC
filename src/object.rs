use crate::{error::AllocError, rawptr::{AllocHeader, AllocObject, AllocTypeId}};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mark {
    Live,
    Free,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseType {
    SynInteger,
    SynFloating,
    SynObject,
    SynArray,
}

impl AllocTypeId for BaseType {}

pub struct ObjectHeader {
    size: usize,
    mark: Mark,
    type_id: BaseType,
}

impl AllocHeader for ObjectHeader {
    type TypeId = BaseType;

    fn new<O: AllocObject<Self::TypeId>>(size: usize, mark: Mark) -> Self {
        Self {
            type_id: O::TYPE_ID,
            size,
            mark
        }
    }

    fn new_array(size: usize, mark: Mark) -> Self {
        Self {
            type_id: BaseType::SynArray,
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