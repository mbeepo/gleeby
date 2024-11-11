pub mod allocator;
pub mod assembler;
pub mod block;
pub mod cgb;
pub mod meta_instr;
pub mod variables;

use allocator::ConstAllocError;
use assembler::ErrorTrait;
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
    Id,
    Variable
};

pub(crate) use variables::IdInner;

use crate::cpu::{GpRegister, IndirectPair, RegConversionError, RegisterPair, SplitError, StackPair};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssemblerError {
    AllocError(ConstAllocError),
    EmitterError(EmitterError),
    RegSplitError(SplitError),
    ConversionError(RegConversionError),
}

impl ErrorTrait for AssemblerError {}

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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum R8Arg {
    GpRegister(GpRegister),
    Variable(Variable),
}

impl From<GpRegister> for R8Arg {
    fn from(value: GpRegister) -> Self {
        Self::GpRegister(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum R16Arg {
    RegisterPair(RegisterPair),
    Variable(Variable),
}

impl From<RegisterPair> for R16Arg {
    fn from(value: RegisterPair) -> Self {
        Self::RegisterPair(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IndirectArg {
    IndirectPair(IndirectPair),
    Variable(Variable),
}

impl From<IndirectPair> for IndirectArg {
    fn from(value: IndirectPair) -> Self {
        Self::IndirectPair(value)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StackArg {
    StackPair(StackPair),
    Variable(Variable),
}

impl From<StackPair> for StackArg {
    fn from(value: StackPair) -> Self {
        Self::StackPair(value)
    }
}
