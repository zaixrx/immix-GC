#[derive(Debug)]
pub(crate) enum BlockError {
    BadRequest,
    OOM, 
}

#[derive(Debug)]
pub enum AllocError {
    /// requested invalid `size`
    BadRequest,
    /// out of memory
    OOM,
}

impl From<BlockError> for AllocError {
    fn from(e: BlockError) -> Self {
        match e {
            BlockError::BadRequest => AllocError::BadRequest,
            BlockError::OOM => AllocError::OOM,
        }
    }
}