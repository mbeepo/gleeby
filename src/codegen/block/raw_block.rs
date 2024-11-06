use crate::{codegen::Assembler, cpu::instructions::Instruction};

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct RawBlock(pub Vec<Instruction>);

impl From<RawBlock> for Vec<u8> {
    fn from(value: RawBlock) -> Self {
        (&value).into()
    }
}

impl From<&RawBlock> for Vec<u8> {
    fn from(value: &RawBlock) -> Self {
        value.0.iter().flat_map(|&instruction| { let out: Vec<u8> = instruction.into(); out }).collect()
    }
}

impl Assembler for RawBlock {
    fn push_instruction(&mut self, instruction: Instruction) {
        self.0.push(instruction);
    }

    fn push_buf(&mut self, buf: &[Instruction]) {
        self.0.extend(buf);
    }

    fn len(&self) -> usize {
        self.0.iter().fold(0, |acc, instruction| acc + instruction.len())
    }
}