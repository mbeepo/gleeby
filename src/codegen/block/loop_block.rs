use std::{cell::{RefCell, RefMut}, rc::Rc};

use crate::{
    codegen::{
        allocator::{ConstAllocError, ConstAllocator}, meta_instr::MetaInstructionTrait, variables::{Constant, StoredConstant, Variabler}, Assembler, AssemblerError, MacroAssembler, Variable
    },
    cpu::{
        instructions::{
            Condition,
            Instruction
        },
        CpuFlag
    }
};

use super::{basic_block::BasicBlock, Block, BlockTrait};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoopCondition {
    Native(Condition),
    // Constructed conditions
    /// Decrements `counter` until it reaches `end`, then stops iterating
    Countdown { counter: Variable, end: u8 },
    /// Increments `counter` until it reaches `end`, then stops iterating
    Countup { counter: Variable, end: u8 },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoopBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    pub condition: LoopCondition,
    pub inner: BasicBlock<Meta>,
}

impl<Meta> LoopBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
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

impl<Meta> TryFrom<LoopBlock<Meta>> for Vec<u8>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    type Error = Vec<AssemblerError>;

    fn try_from(mut value: LoopBlock<Meta>) -> Result<Self, Self::Error> {
        let mut errs: Self::Error = Vec::new();

        // when jumping backwards the offset must include the Jr itself (2 bytes)
        let block_length = value.inner.len() + LoopBlock::<Meta>::JR_LEN;

        // Jr takes a signed 8-bit integer
        if block_length as isize * -1 < i8::MIN as isize {
            todo!("Loop body too big")
        }

        let allocator = value.allocator();
        let jump: Result<Vec<u8>, Self::Error> = match value.condition {
            LoopCondition::Native(condition) => {
                let mut buffer = BasicBlock::<Meta>::new(allocator);
                buffer.jr(condition, block_length as i8 * -1);
                buffer
            },
            LoopCondition::Countdown { ref mut counter, end } => {
                if end == 0 {
                    let mut buffer = BasicBlock::<Meta>::new(allocator);
                    if let Err(err) = buffer.dec_var(counter) {
                        errs.push(err);
                    }
                    
                    // TODO: make compatible with register pairs
                    // `dec r16` leaves F.Z unchanged, so we'll need to do some kind of sneakiness
                    // especially when all registers are in use

                    let block_length = block_length + buffer.len();
                    
                    if block_length as isize * -1 < i8::MIN as isize {
                        todo!("Loop body too big")
                    }

                    buffer.jr(Condition::Flag(CpuFlag::NZ), block_length as i8 * -1);
                    buffer
                } else {
                    todo!()
                }
            },
            LoopCondition::Countup { ref mut counter, end } => {
                let mut buffer = BasicBlock::<Meta>::new(allocator);
                if let Err(err) = buffer.inc_var(counter) {
                    errs.push(err);
                }

                let block_length = block_length + buffer.len();
                    
                if block_length as isize * -1 < i8::MIN as isize {
                    todo!("Loop body too big")
                }

                buffer.jr(Condition::Flag(CpuFlag::NZ), block_length as i8 * -1);
                buffer
            }
        }.try_into();

        let mut out: Vec<u8> = value.inner.try_into()?;
        
        match jump {
            Ok(jump) => {
                out.extend(jump);
            }
            Err(err) => {
                errs.extend(err);
            }
        }

        if errs.len() > 0 {
            Err(errs)
        } else {
            Ok(out)
        }
    }
}

impl<Meta> Assembler<Meta> for LoopBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
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
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    type Alloc = ConstAllocator;

    fn new_var(&mut self, len: u16) -> Variable {
        self.inner.new_var(len)
    }

    fn allocator(&self) -> Rc<RefCell<ConstAllocator>> {
        self.inner.allocator()
    }

    fn allocator_mut(&mut self) -> RefMut<Self::Alloc> {
        self.inner.allocator_mut()
    }
}

impl<Meta> MacroAssembler<Meta, AssemblerError, ConstAllocError> for LoopBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    fn basic_block(&mut self) -> &mut BasicBlock<Meta> {
        self.inner.basic_block()
    }

    fn loop_block(&mut self, condition: LoopCondition) -> &mut LoopBlock<Meta> {
        self.inner.loop_block(condition);
        self
    }

    fn new_stored_const(&mut self, data: &[u8]) -> Result<StoredConstant, AssemblerError> {
        self.inner.new_stored_const(data)
    }

    fn new_inline_const_r8(&mut self, data: u8) -> Constant {
        self.inner.new_inline_const_r8(data)
    }

    fn new_inline_const_r16(&mut self, data: u16) -> Constant {
        self.inner.new_inline_const_r16(data)
    }

    fn free_var(&mut self, var: Variable) -> Result<(), AssemblerError> {
        self.inner.free_var(var)
    }

    fn evaluate_meta(&mut self) -> Result<(), AssemblerError> {
        self.inner.evaluate_meta()
    }

    fn gather_consts(&mut self) -> Vec<(Constant, Vec<u8>)> {
        self.inner.gather_consts()
    }
}

impl<Meta> BlockTrait for LoopBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait {
    type Contents = Vec<Block<Meta>>;
        
    fn contents(&self) -> &Self::Contents {
        &self.inner.contents
    }

    fn contents_mut(&mut self) -> &mut Self::Contents {
        &mut self.inner.contents
    }
}