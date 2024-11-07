use crate::codegen::allocator::{Allocator, ConstAllocError, ConstAllocator};
use crate::codegen::assembler::{AsBuf, Context};
use crate::codegen::variables::{MemoryVariable, RegVariable, Variabler};
use crate::codegen::{Assembler, AssemblerError, LoopCondition, MacroAssembler};
use crate::codegen::{Block, LoopBlock};
use crate::codegen::{Ctx, IdInner, Variable};
use crate::cpu::instructions::Instruction;
use crate::memory::Addr;

use super::EmitterError;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BasicBlock {
    next_id: IdInner,
    pub allocator: ConstAllocator,
    pub ctx: Ctx,
    pub variables: Vec<(Vec<u8>, Variable)>,
    pub contents: Vec<Block>,
    pub consts: Vec<(Addr, Vec<u8>)>,
}

impl From<Vec<Instruction>> for BasicBlock {
    fn from(instructions: Vec<Instruction>) -> Self {
        Self { 
            contents: vec![instructions.into()],
            next_id: Default::default(),
            allocator: ConstAllocator::default(),
            ctx: Default::default(),
            variables: Default::default(),
            consts: Default::default(),
        }
    }
}

impl TryFrom<&BasicBlock> for Vec<u8>  {
    type Error = Vec<EmitterError>;

    fn try_from(value: &BasicBlock) -> Result<Self, Self::Error> {
        let (out, errors) = value.contents.iter().fold((Vec::with_capacity(8), Vec::with_capacity(4)), |mut acc, instruction| {
            let out: Result<Vec<u8>, Self::Error> = instruction.try_into();

            match out {
                Ok(buf) => acc.0.extend(buf),
                Err(e) => acc.1.extend(e),
            };

            acc
        });

        if errors.len() > 0 {
            Err(errors)
        } else {
            Ok(out)
        }
    }
}

impl Assembler for BasicBlock {
    fn push_instruction(&mut self, instruction: Instruction) {
        self.push_buf(&[instruction])
    }

    fn push_buf(&mut self, buf: &[Instruction]) {
        if let Some(Block::Raw(block)) = self.contents.last_mut() {
            block.0.extend(buf);
        } else {
            let mut new: Vec<Instruction> = Vec::with_capacity(buf.len() + 2);
            new.extend(buf);
            self.contents.push(new.into());
        }
    }

    fn len(&self) -> usize {
        self.contents.iter().fold(0, |acc, block| { acc + block.len() })
    }
}

impl Variabler<AssemblerError, ConstAllocError> for BasicBlock {
    type Alloc = ConstAllocator;

    fn new_var<T>(&mut self, initial: T) -> Variable
           where T: AsBuf {
        let var = Variable::Unallocated { id: self.new_id(), ctx: self.new_ctx() };
        self.variables.push((initial.as_buf(), var));
        var
    }

    fn allocator(&mut self) -> &mut Self::Alloc {
        &mut self.allocator
    }
}

impl MacroAssembler<AssemblerError, ConstAllocError> for BasicBlock {
    fn basic_block<F>(&mut self, inner: F) -> &mut Self
            where F: Fn(&mut BasicBlock) {
        let mut block: BasicBlock = Vec::with_capacity(4).into();
        inner(&mut block);

        let ctx = self.new_ctx();
        block.ctx = ctx;
        self.contents.push(block.into());

        self
    }

    fn loop_block<F>(&mut self, condition: LoopCondition, inner: F) -> &mut Self
            where F: Fn(&mut LoopBlock) {
        let mut block: LoopBlock = LoopBlock::new(condition, Vec::with_capacity(4).into());
        inner(&mut block);

        let ctx = self.new_ctx();
        block.ctx = ctx;
        self.contents.push(block.into());
        self
    }

    fn new_const(&mut self, data: &[u8]) -> Result<Addr, AssemblerError> {
        let addr = self.allocator.new_const(data).map_err(AssemblerError::AllocError)?;
        self.consts.push((addr, data.to_vec()));
        Ok(addr)
    }
}

impl Context for BasicBlock {
    fn next_id(&self) -> IdInner {
        self.next_id
    }

    fn next_id_mut(&mut self) -> &mut IdInner {
        &mut self.next_id
    }
}

impl AsRef<BasicBlock> for BasicBlock {
    fn as_ref(&self) -> &BasicBlock {
        self
    }
}