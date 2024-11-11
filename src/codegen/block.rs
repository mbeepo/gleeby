use basic_block::BasicBlock;
use loop_block::LoopBlock;
use raw_block::RawBlock;

use crate::cpu::{instructions::Instruction, SplitError};

use super::{allocator::{AllocErrorTrait, ConstAllocError}, assembler::ErrorTrait, meta_instr::MetaInstructionTrait, variables::Constant, Assembler, AssemblerError, MacroAssembler, Variable};

pub mod basic_block;
pub mod loop_block;
pub mod raw_block;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    Basic(BasicBlock<Meta>),
    Loop(LoopBlock<Meta>),
    Raw(RawBlock<Meta>),
}

impl<Meta> Block<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    pub fn gather_consts(&mut self) -> Vec<(Constant, Vec<u8>)> {
        match self {
            Self::Basic(block) => block.gather_consts(),
            Self::Loop(block) => block.gather_consts(),
            Self::Raw(_) => Vec::new(),
        }
    }
}

impl<Meta> Default for Block<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    fn default() -> Self {
        Self::Raw(Default::default())
    }
}

impl<Meta> From<LoopBlock<Meta>> for Block<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    fn from(value: LoopBlock<Meta>) -> Self {
        Block::<_>::Loop(value)
    }
}

impl<Meta> From<BasicBlock<Meta>> for Block<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    fn from(value: BasicBlock<Meta>) -> Self {
        Block::<_>::Basic(value)
    }
}

impl<Meta> From<Instruction<Meta>> for Block<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    fn from(value: Instruction<Meta>) -> Self {
        Self::Raw(RawBlock(vec![value]))
    }
}

impl<Meta> From<&Instruction<Meta>> for Block<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    fn from(value: &Instruction<Meta>) -> Self {
        value.clone().into()
    }
}

impl<Meta> From<Vec<Instruction<Meta>>> for Block<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    fn from(value: Vec<Instruction<Meta>>) -> Self {
        Self::Raw(RawBlock(value))
    }
}

impl<Meta> TryFrom<Block<Meta>> for Vec<u8>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    type Error = Vec<AssemblerError>;

    fn try_from(value: Block<Meta>) -> Result<Self, Self::Error> {
        match value {
            Block::Basic(block) => block.try_into(),
            Block::Loop(block) => block.try_into(),
            Block::Raw(block) => Ok(block.into()),
            _ => unimplemented!()
        }
    }
}

impl<Meta> Assembler<Meta> for Block<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    fn push_instruction(&mut self, instruction: Instruction<Meta>) {
        match self {
            Self::Basic(block) => block.push_instruction(instruction),
            Self::Loop(block) => block.push_instruction(instruction),
            Self::Raw(block) => block.push_instruction(instruction),
            _ => unimplemented!()
        }
    }

    fn push_buf(&mut self, buf: &[Instruction<Meta>]) {
        match self {
            Self::Basic(block) => block.push_buf(buf),
            Self::Loop(block) => block.push_buf(buf),
            Self::Raw(block) => block.push_buf(buf),
            _ => unimplemented!()
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::Basic(block) => block.len(),
            Self::Loop(block) => block.len(),
            Self::Raw(block) => block.len(),
            _ => unimplemented!()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EmitterError {
    UnallocatedVariable(Variable),
}

pub trait BlockTrait<Error, AllocError>
        where Error: Clone + std::fmt::Debug + From<SplitError> + From<AllocError> + From<AssemblerError> + From<ConstAllocError> + ErrorTrait, // TODO: Not this
            AllocError: Clone + std::fmt::Debug + Into<Error> + AllocErrorTrait, {
    type Contents;

    fn contents(&self) -> &Self::Contents;
    fn contents_mut(&mut self) -> &mut Self::Contents;

    fn open<F>(&mut self, inner: F) -> Result<&mut Self, Error>
            where F: Fn(&mut Self) -> Result<(), Error> {
        inner(self)?;
        Ok(self)
    }

    fn ok(&self) -> Result<(), Error> {
        Ok(())
    }
}