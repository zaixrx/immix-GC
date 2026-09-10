use crate::rawptr::{AllocHeader, AllocObject, AllocTypeId};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mark {
    Live,
    Free,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BaseType {
    SynInteger,
    SynFloat,
    SynArray,
}

impl AllocTypeId for BaseType {}

pub struct BaseHeader {
    size: usize,
    mark: Mark,
    type_id: BaseType,
}

impl AllocHeader for BaseHeader {
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

pub struct SynInteger {
    pub value: isize
}

impl AllocObject<BaseType> for SynInteger {
    const TYPE_ID: BaseType = BaseType::SynInteger;
}

pub struct SynFloat {
    pub value: f64,
}

impl AllocObject<BaseType> for SynFloat {
    const TYPE_ID: BaseType = BaseType::SynFloat;
}

pub struct SynArray {
    ptr: *const u8,
    len: usize,
    cap: usize,
}

impl AllocObject<BaseType> for SynArray {
    const TYPE_ID: BaseType = BaseType::SynArray;
}
