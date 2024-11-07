pub mod allocator;
pub mod assembler;
pub mod block;
pub mod cgb;
pub mod variables;

use allocator::ConstAllocError;
pub use assembler::{
    Assembler,
    MacroAssembler,
};

use block::EmitterError;
pub use block::{
    Block,
    basic_block::BasicBlock,
    loop_block::{
        LoopBlock,
        LoopCondition,
    }
};

pub use variables::{
    Ctx,
    Id,
    Variable
};

pub(crate) use variables::IdInner;

use crate::cpu::SplitError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssemblerError {
    AllocError(ConstAllocError),
    EmitterError(EmitterError),
    RegSplitError(SplitError),
}

impl From<ConstAllocError> for AssemblerError {
    fn from(value: ConstAllocError) -> Self {
        Self::AllocError(value)
    }
}

impl From<EmitterError> for AssemblerError {
    fn from(value: EmitterError) -> Self {
        Self::EmitterError(value)
    }
}