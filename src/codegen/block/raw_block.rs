use crate::{codegen::{meta_instr::MetaInstructionTrait, Assembler}, cpu::instructions::Instruction};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RawBlock<Meta>(pub Vec<Instruction<Meta>>)
    where Meta: Clone + std::fmt::Debug + MetaInstructionTrait;

impl<Meta> From<RawBlock<Meta>> for Vec<u8>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait {
    fn from(value: RawBlock<Meta>) -> Self {
        value.0.into_iter().flat_map(|instruction| { let out: Vec<u8> = instruction.into(); out }).collect()
    }
}

impl<Meta> Assembler<Meta> for RawBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait {
    fn push_instruction(&mut self, instruction: Instruction<Meta>) {
        self.0.push(instruction);
    }

    fn push_buf(&mut self, buf: &[Instruction<Meta>]) {
        self.0.extend(buf.to_vec());
    }

    fn len(&self) -> usize {
        self.0.iter().fold(0, |acc, instruction| acc + instruction.len())
    }
}

impl<Meta> Default for RawBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait {
    fn default() -> Self {
        Self(Default::default())
    }
}