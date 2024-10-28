use crate::cpu::instructions::{Condition, Instruction};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BasicBlock {
    pub instructions: Vec<Instruction>,
}

impl From<Vec<Instruction>> for BasicBlock {
    fn from(instructions: Vec<Instruction>) -> Self {
        Self { instructions }
    }
}

impl From<Instruction> for BasicBlock {
    fn from(value: Instruction) -> Self {
        Self { instructions: vec![value] }
    }
}

impl From<BasicBlock> for Vec<u8> {
    fn from(value: BasicBlock) -> Self {
        value.instructions.iter().flat_map(|&instruction| { let out: Vec<u8> = instruction.into(); out }).collect()
    }
}

impl From<&BasicBlock> for Vec<u8> {
    fn from(value: &BasicBlock) -> Self {
        value.instructions.iter().flat_map(|&instruction| { let out: Vec<u8> = instruction.into(); out }).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Loop {
    pub condition: Condition,
    pub inner: BasicBlock,
}

impl Loop {
    pub fn new(condition: Condition, inner: BasicBlock) -> Self {
        Self { condition, inner }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Block {
    Basic(BasicBlock),
    Labeled(String, BasicBlock),
    Loop(Loop),
}

impl From<&Instruction> for Block {
    fn from(value: &Instruction) -> Self {
        Self::Basic((*value).into())
    }
}

impl From<Block> for Vec<u8> {
    fn from(value: Block) -> Self {
        match value {
            Block::Basic(block) => block.into(),
            _ => unimplemented!()
        }
    }
}

impl From<&Block> for Vec<u8> {
    fn from(value: &Block) -> Self {
        match value {
            Block::Basic(block) => block.into(),
            _ => unimplemented!()
        }
    }
}