use basic_block::BasicBlock;
use loop_block::LoopBlock;
use raw_block::RawBlock;

use crate::cpu::instructions::Instruction;

use super::{Assembler, Variable};

pub mod basic_block;
pub mod loop_block;
pub mod raw_block;

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    Basic(BasicBlock),
    Labeled(String, BasicBlock),
    Loop(LoopBlock),
    Raw(RawBlock),
}

impl Default for Block {
    fn default() -> Self {
        Self::Raw(Default::default())
    }
}

impl From<LoopBlock> for Block {
    fn from(value: LoopBlock) -> Self {
        Block::Loop(value)
    }
}

impl From<BasicBlock> for Block {
    fn from(value: BasicBlock) -> Self {
        Block::Basic(value)
    }
}

impl From<Instruction> for Block {
    fn from(value: Instruction) -> Self {
        Self::Raw(RawBlock(vec![value]))
    }
}

impl From<&Instruction> for Block {
    fn from(value: &Instruction) -> Self {
        (*value).into()
    }
}

impl From<Vec<Instruction>> for Block {
    fn from(value: Vec<Instruction>) -> Self {
        Self::Raw(RawBlock(value))
    }
}

impl TryFrom<&Block> for Vec<u8> {
    type Error = Vec<EmitterError>;

    fn try_from(value: &Block) -> Result<Self, Self::Error> {
        match value {
            Block::Basic(block) => block.try_into(),
            Block::Loop(block) => block.try_into(),
            Block::Raw(block) => Ok(block.into()),
            _ => unimplemented!()
        }
    }
}

impl Assembler for Block {
    fn push_instruction(&mut self, instruction: Instruction) {
        match self {
            Self::Basic(block) => block.push_instruction(instruction),
            Self::Loop(block) => block.push_instruction(instruction),
            Self::Raw(block) => block.push_instruction(instruction),
            _ => unimplemented!()
        }
    }

    fn push_buf(&mut self, buf: &[Instruction]) {
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
