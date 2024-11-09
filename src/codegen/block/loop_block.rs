use crate::{
    codegen::{
        allocator::{Allocator, ConstAllocError, ConstAllocator}, assembler::AsBuf, meta_instr::MetaInstructionTrait, variables::Variabler, Assembler, AssemblerError, MacroAssembler, Variable
    },
    cpu::{
        instructions::{
            Condition,
            Instruction
        },
        CpuFlag
    }, memory::Addr
};

use super::{basic_block::BasicBlock, EmitterError};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopCondition {
    Native(Condition),
    // Constructed conditions
    /// Decrements `counter` until it reaches `end`, then stops iterating
    Countdown { counter: Variable, end: u8 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoopBlock<Meta>
        where Meta: Clone + Copy + std::fmt::Debug + MetaInstructionTrait {
    pub condition: LoopCondition,
    pub inner: BasicBlock<Meta>,
}

impl<Meta> LoopBlock<Meta>
        where Meta: Clone + Copy + std::fmt::Debug + MetaInstructionTrait {
    const JR_LEN: usize = 2;

    pub fn new(condition: LoopCondition, inner: BasicBlock<Meta>) -> Self {
        Self {
            condition,
            inner,
        }
    }

    pub fn new_native(condition: Condition, inner: BasicBlock<Meta>) -> Self {
        Self {
            condition: LoopCondition::Native(condition),
            inner,
        }
    }
}

impl<Meta> TryFrom<&LoopBlock<Meta>> for Vec<u8>
        where Meta: Clone + Copy + std::fmt::Debug + MetaInstructionTrait {
    type Error = Vec<EmitterError>;

    fn try_from(value: &LoopBlock<Meta>) -> Result<Self, Self::Error> {
        // when jumping backwards the offset must include the Jr itself (2 bytes)
        let block_length = value.inner.len() + LoopBlock::<Meta>::JR_LEN;

        // Jr takes a signed 8-bit integer
        if block_length as isize * -1 < i8::MIN as isize {
            todo!("Loop body too big")
        }

        let jump: Vec<u8> = match value.condition {
            LoopCondition::Native(condition) => {
                let mut buffer = BasicBlock::<Meta>::default();
                buffer.jr(condition, block_length as i8 * -1);
                buffer
            },
            LoopCondition::Countdown { counter, end } => {
                if end == 0 {
                    let mut buffer = BasicBlock::default();
                    buffer.dec_var(counter);

                    let block_length = block_length + buffer.len();
                    
                    if block_length as isize * -1 < i8::MIN as isize {
                        todo!("Loop body too big")
                    }

                    buffer.jr(Condition::Flag(CpuFlag::NZ), block_length as i8 * -1);
                    buffer
                } else {
                    todo!()
                }
            }
        }.as_ref().try_into()?;

        let mut out: Vec<u8> = (&value.inner).try_into()?;
        out.extend(jump);
        Ok(out)
    }
}

impl<Meta> Assembler<Meta> for LoopBlock<Meta>
        where Meta: Clone + Copy + std::fmt::Debug + MetaInstructionTrait {
    fn push_instruction(&mut self, instruction: Instruction<Meta>) {
        self.inner.push_instruction(instruction);
    }

    fn push_buf(&mut self, buf: &[Instruction<Meta>]) {
        self.inner.push_buf(buf);
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl<Meta> Variabler<Meta, AssemblerError, ConstAllocError> for LoopBlock<Meta>
        where Meta: Clone + Copy + std::fmt::Debug + MetaInstructionTrait {
    type Alloc = ConstAllocator;

    fn new_var(&mut self, len: u16) -> Variable {
        self.inner.new_var(len)
    }

    fn allocator(&mut self) -> &mut Self::Alloc {
        self.inner.allocator()
    }
}

impl<Meta> MacroAssembler<Meta, AssemblerError, ConstAllocError> for LoopBlock<Meta>
        where Meta: Clone + Copy + std::fmt::Debug + MetaInstructionTrait {
    fn basic_block<F>(&mut self, inner: F) -> &mut Self
            where F: Fn(&mut BasicBlock<Meta>) {
        self.inner.basic_block(inner);
        self
    }

    fn loop_block<F>(&mut self, condition: LoopCondition, inner: F) -> &mut Self
            where F: Fn(&mut LoopBlock<Meta>) {
        self.inner.loop_block(condition, inner);
        self
    }

    fn new_const(&mut self, data: &[u8]) -> Result<Addr, ConstAllocError> {
        self.inner.new_const(data)
    }
}