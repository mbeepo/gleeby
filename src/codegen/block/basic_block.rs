use std::collections::HashMap;

use crate::codegen::allocator::{AllocErrorTrait, Allocator, ConstAllocError, ConstAllocator};
use crate::codegen::assembler::{Context, ErrorTrait};
use crate::codegen::meta_instr::MetaInstructionTrait;
use crate::codegen::variables::{Constant, RegVariable, StoredConstant, Variabler};
use crate::codegen::{Assembler, AssemblerError, Id, LoopCondition, MacroAssembler};
use crate::codegen::{Block, LoopBlock};
use crate::codegen::{IdInner, Variable};
use crate::cpu::instructions::Instruction;
use crate::cpu::SplitError;

use super::BlockTrait;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BasicBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    next_id: IdInner,
    pub contents: Vec<Block<Meta>>,
    pub allocator: ConstAllocator,
    pub variables: HashMap<Id, Variable>,
    pub consts: HashMap<Id, (StoredConstant, Vec<u8>)>,
}

impl<Meta> Default for BasicBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    fn default() -> Self {
        Self {
            next_id: Default::default(),
            contents: Default::default(),
            allocator: Default::default(),
            variables: Default::default(),
            consts: Default::default(),
        }
    }
}

impl<Meta> From<Vec<Instruction<Meta>>> for BasicBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    fn from(instructions: Vec<Instruction<Meta>>) -> Self {
        Self { 
            next_id: Default::default(),
            contents: vec![instructions.into()],
            allocator: ConstAllocator::default(),
            variables: Default::default(),
            consts: Default::default(),
        }
    }
}

impl<Meta> TryFrom<BasicBlock<Meta>> for Vec<u8>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    type Error = Vec<AssemblerError>;

    fn try_from(value: BasicBlock<Meta>) -> Result<Self, Self::Error> {
        let (out, errors) = value.contents.into_iter().fold((Vec::with_capacity(8), Vec::with_capacity(4)), |mut acc, instruction| {
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
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
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
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    type Alloc = ConstAllocator;

    fn new_var(&mut self, len: u16) -> Variable {
        let id = self.new_id();
        let var = Variable::Unallocated { len, id };
        self.variables.insert(id, var);
        var
    }
    
    fn allocator(&mut self) -> &mut Self::Alloc {
        &mut self.allocator
    }
}

impl<Meta> MacroAssembler<Meta, AssemblerError, ConstAllocError> for BasicBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    fn basic_block(&mut self) -> &mut BasicBlock<Meta> {
        let block: BasicBlock<Meta> = Vec::with_capacity(4).into();
        self.contents.push(Block::Basic(block));

        if let Block::Basic(ref mut last) = self.contents.last_mut().unwrap() {
            last
        } else {
            unreachable!()
        }
    }

    fn loop_block(&mut self, condition: LoopCondition) -> &mut LoopBlock<Meta> {
        let block: LoopBlock<Meta> = LoopBlock::<Meta>::new(condition, Vec::with_capacity(4).into());
        self.contents.push(block.into());

        if let Block::Loop(ref mut last) = self.contents.last_mut().unwrap() {
            last
        } else {
            unreachable!()
        }
    }

    fn new_stored_const(&mut self, data: &[u8]) -> Result<StoredConstant, AssemblerError> {
        let addr = self.allocator.alloc_const(data.len() as u16)?;
        let id = self.new_id();
        let constant = StoredConstant {
            id,
            addr,
            len: data.len() as u16
        };

        self.consts.insert(id, (constant, data.to_vec()));

        Ok(constant)
    }

    fn new_inline_const_r8(&mut self, data: u8) -> Constant {
        Constant::Inline8(data)
    }

    fn new_inline_const_r16(&mut self, data: u16) -> Constant {
        Constant::Inline16(data)
    }

    fn free_var(&mut self, var: Variable) -> Result<(), AssemblerError> {
        match var {
            Variable::Reg(var) => {
                match var {
                    RegVariable::R8 { reg, id } => {
                        self.variables.remove(&id);
                        self.allocator.release_reg(reg.into());
                    },
                    RegVariable::MemR8 { addr, reg, id } => {
                        self.variables.remove(&id);
                        self.allocator.release_reg(reg.into());
                        self.allocator.dealloc_var(addr)?;
                    },
                    RegVariable::R16 { reg_pair, id } => {
                        self.variables.remove(&id);
                        self.allocator.release_reg(reg_pair.into());
                    },
                    RegVariable::MemR16 { addr, reg_pair, id } => {
                        self.variables.remove(&id);
                        self.allocator.release_reg(reg_pair.into());
                        self.allocator.dealloc_var(addr)?;
                    },
                    RegVariable::UnallocatedR8(id)
                    | RegVariable::UnallocatedR16(id) => {
                        self.variables.remove(&id);
                    }
                }
            }
            Variable::Memory(var) => {
                self.variables.remove(&var.id);
                self.allocator.dealloc_var(var.addr)?;
            }
            Variable::Unallocated { .. } => {}
        }

        Ok(())
    }

    fn evaluate_meta(&mut self) -> Result<(), AssemblerError> {
        todo!()
    }

    fn gather_consts(&mut self) -> Vec<(Constant, Vec<u8>)> {
        let mut consts: Vec<(Constant, Vec<u8>)> = self.consts.drain().map(|(_, constant)| (Constant::Addr(constant.0), constant.1)).collect();
        consts.extend(self.contents.iter_mut().flat_map(|block| block.gather_consts()));

        consts
    }
}

impl<Meta> Context for BasicBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    fn next_id(&self) -> IdInner {
        self.next_id
    }

    fn next_id_mut(&mut self) -> &mut IdInner {
        &mut self.next_id
    }
}

impl<Meta> AsRef<BasicBlock<Meta>> for BasicBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    fn as_ref(&self) -> &BasicBlock<Meta> {
        self
    }
}

impl<Meta, Error, AllocError> BlockTrait<Error, AllocError> for BasicBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait,
            Error: Clone + std::fmt::Debug + From<SplitError> + From<AllocError> + From<AssemblerError> + From<ConstAllocError> + ErrorTrait, // TODO: Not this
            AllocError: Clone + std::fmt::Debug + Into<Error> + AllocErrorTrait, {
    type Contents = Vec<Block<Meta>>;

    fn contents(&self) -> &Self::Contents {
        &self.contents
    }

    fn contents_mut(&mut self) -> &mut Self::Contents {
        &mut self.contents
    }
}