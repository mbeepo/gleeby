use crate::codegen::allocator::{Allocator, ConstAllocError, ConstAllocator};
use crate::codegen::assembler::Context;
use crate::codegen::meta_instr::MetaInstructionTrait;
use crate::codegen::variables::Variabler;
use crate::codegen::{Assembler, AssemblerError, LoopCondition, MacroAssembler};
use crate::codegen::{Block, LoopBlock};
use crate::codegen::{IdInner, Variable};
use crate::cpu::instructions::Instruction;
use crate::memory::Addr;

use super::EmitterError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BasicBlock<Meta>
        where Meta: Clone + Copy + std::fmt::Debug + MetaInstructionTrait {
    next_id: IdInner,
    pub allocator: ConstAllocator,
    pub variables: Vec<Variable>,
    pub contents: Vec<Block<Meta>>,
    pub consts: Vec<(Addr, Vec<u8>)>,
}

impl<Meta> Default for BasicBlock<Meta>
        where Meta: Clone + Copy + std::fmt::Debug + MetaInstructionTrait {
    fn default() -> Self {
        Self {
            next_id: Default::default(),
            allocator: Default::default(),
            variables: Default::default(),
            contents: Default::default(),
            consts: Default::default(),
        }
    }
}

impl<Meta> From<Vec<Instruction<Meta>>> for BasicBlock<Meta>
        where Meta: Clone + Copy + std::fmt::Debug + MetaInstructionTrait {
    fn from(instructions: Vec<Instruction<Meta>>) -> Self {
        Self { 
            contents: vec![instructions.into()],
            next_id: Default::default(),
            allocator: ConstAllocator::default(),
            variables: Default::default(),
            consts: Default::default(),
        }
    }
}

impl<Meta> TryFrom<&BasicBlock<Meta>> for Vec<u8>
        where Meta: Clone + Copy + std::fmt::Debug + MetaInstructionTrait {
    type Error = Vec<EmitterError>;

    fn try_from(value: &BasicBlock<Meta>) -> Result<Self, Self::Error> {
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

impl<Meta> Assembler<Meta> for BasicBlock<Meta>
        where Meta: Clone + Copy + std::fmt::Debug + MetaInstructionTrait {

    fn push_instruction(&mut self, instruction: Instruction<Meta>) {
        self.push_buf(&[instruction])
    }

    fn push_buf(&mut self, buf: &[Instruction<Meta>]) {
        if let Some(Block::Raw(block)) = self.contents.last_mut() {
            block.0.extend(buf.to_vec());
        } else {
            let mut new: Vec<Instruction<Meta>> = Vec::with_capacity(buf.len() + 2);
            new.extend(buf.to_vec());
            self.contents.push(new.into());
        }
    }

    fn len(&self) -> usize {
        self.contents.iter().fold(0, |acc, block| { acc + block.len() })
    }
}

impl<Meta> Variabler<Meta, AssemblerError, ConstAllocError> for BasicBlock<Meta>
        where Meta: Clone + Copy + std::fmt::Debug + MetaInstructionTrait {
    type Alloc = ConstAllocator;

    fn new_var(&mut self, len: u16) -> Variable {
        let var = Variable::Unallocated { len, id: self.new_id() };
        self.variables.push(var);
        var
    }

    fn allocator(&mut self) -> &mut Self::Alloc {
        &mut self.allocator
    }
}

impl<Meta> MacroAssembler<Meta, AssemblerError, ConstAllocError> for BasicBlock<Meta>
        where Meta: Clone + Copy + std::fmt::Debug + MetaInstructionTrait {
    fn basic_block<F>(&mut self, inner: F) -> &mut Self
            where F: Fn(&mut BasicBlock<Meta>) {
        let mut block: BasicBlock<Meta> = Vec::with_capacity(4).into();
        inner(&mut block);

        self.contents.push(block.into());
        self
    }

    fn loop_block<F>(&mut self, condition: LoopCondition, inner: F) -> &mut Self
            where F: Fn(&mut LoopBlock<Meta>) {
        let mut block: LoopBlock<Meta> = LoopBlock::new(condition, Vec::with_capacity(4).into());
        inner(&mut block);

        self.contents.push(block.into());
        self
    }

    fn new_const(&mut self, data: &[u8]) -> Result<Addr, ConstAllocError> {
        let addr = self.allocator.alloc_const(data.len() as u16)?;
        self.consts.push((addr, data.to_vec()));
        Ok(addr)
    }
}

impl<Meta> Context for BasicBlock<Meta>
        where Meta: Clone + Copy + std::fmt::Debug + MetaInstructionTrait {
    fn next_id(&self) -> IdInner {
        self.next_id
    }

    fn next_id_mut(&mut self) -> &mut IdInner {
        &mut self.next_id
    }
}

impl<Meta> AsRef<BasicBlock<Meta>> for BasicBlock<Meta>
    where Meta: Clone + Copy + std::fmt::Debug + MetaInstructionTrait {
    fn as_ref(&self) -> &BasicBlock<Meta> {
        self
    }
}