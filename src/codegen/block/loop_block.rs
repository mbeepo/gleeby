use crate::{
    codegen::{
        allocator::{Allocator, ConstAllocError, ConstAllocator}, assembler::AsBuf, variables::{MemoryVariable, RegVariable, Variabler}, Assembler, AssemblerError, Ctx, MacroAssembler, Variable
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
pub struct LoopBlock {
    pub ctx: Ctx,
    pub condition: LoopCondition,
    pub inner: BasicBlock,
}

impl LoopBlock {
    const JR_LEN: usize = 2;

    pub fn new(condition: LoopCondition, inner: BasicBlock) -> Self {
        Self {
            condition,
            inner,
            ctx: Default::default(),
        }
    }

    pub fn new_native(condition: Condition, inner: BasicBlock) -> Self {
        Self {
            condition: LoopCondition::Native(condition),
            inner,
            ctx: Default::default(),
        }
    }
}

impl TryFrom<&LoopBlock> for Vec<u8> {
    type Error = Vec<EmitterError>;

    fn try_from(value: &LoopBlock) -> Result<Self, Self::Error> {
        // when jumping backwards the offset must include the Jr itself (2 bytes)
        let block_length = value.inner.len() + LoopBlock::JR_LEN;

        // Jr takes a signed 8-bit integer
        if block_length as isize * -1 < i8::MIN as isize {
            todo!("Loop body too big")
        }

        let jump: Vec<u8> = match value.condition {
            LoopCondition::Native(condition) => {
                let mut buffer = BasicBlock::default();
                buffer.jr(condition, block_length as i8 * -1);
                buffer
            },
            LoopCondition::Countdown { counter, end } => {
                if end == 0 {
                    match counter {
                        Variable::Unallocated { id, ctx } => {
                            return Err(vec![EmitterError::UnallocatedVariable(counter)]);
                        }
                        Variable::Allocated(var) => {
                            let mut buffer = BasicBlock::default();
                            buffer.decrement_var(var);

                            let block_length = block_length + buffer.len();
                            
                            if block_length as isize * -1 < i8::MIN as isize {
                                todo!("Loop body too big")
                            }

                            buffer.jr(Condition::Flag(CpuFlag::NZ), block_length as i8 * -1);
                            buffer
                        }
                        _ => todo!()
                    }
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

impl Assembler for LoopBlock {
    fn push_instruction(&mut self, instruction: Instruction) {
        self.inner.push_instruction(instruction);
    }

    fn push_buf(&mut self, buf: &[Instruction]) {
        self.inner.push_buf(buf);
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl Variabler<AssemblerError, ConstAllocError> for LoopBlock {
    type Alloc = ConstAllocator;

    fn new_var<T>(&mut self, initial: T) -> Variable
            where T: AsBuf {
        self.inner.new_var(initial)
    }

    fn allocator(&mut self) -> &mut Self::Alloc {
        self.inner.allocator()
    }
}

impl MacroAssembler<AssemblerError, ConstAllocError> for LoopBlock {
    fn basic_block<F>(&mut self, inner: F) -> &mut Self
            where F: Fn(&mut BasicBlock) {
        self.inner.basic_block(inner);
        self
    }

    fn loop_block<F>(&mut self, condition: LoopCondition, inner: F) -> &mut Self
            where F: Fn(&mut LoopBlock) {
        self.inner.loop_block(condition, inner);
        self
    }

    fn new_const(&mut self, data: &[u8]) -> Result<Addr, AssemblerError> {
        self.inner.new_const(data)
    }
}