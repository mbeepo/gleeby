use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::codegen::allocator::{Allocator, ConstAllocError, ConstAllocator};
use crate::codegen::assembler::{BlockAssembler, Context};
use crate::codegen::meta_instr::MetaInstructionTrait;
use crate::codegen::variables::{Constant, StoredConstant, Variabler};
use crate::codegen::{Assembler, AssemblerError, Id, LoopCondition, MacroAssembler};
use crate::codegen::{Block, LoopBlock};
use crate::codegen::{IdInner, Variable};
use crate::cpu::instructions::Instruction;

use super::BlockTrait;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BasicBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    next_id: IdInner,
    pub contents: Vec<Block<Meta>>,
    pub allocator: Rc<RefCell<ConstAllocator>>,
    // pub variables: HashMap<Id, Variable>,
    pub consts: HashMap<Id, (StoredConstant, Vec<u8>)>,
}

// impl<Meta> Default for BasicBlock<Meta>
//         where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
//     fn default() -> Self {
//         Self {
//             next_id: Default::default(),
//             contents: Default::default(),
//             allocator: Default::default(),
//             variables: Default::default(),
//             consts: Default::default(),
//         }
//     }
// }

impl<Meta> BasicBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    pub fn new(allocator: Rc<RefCell<ConstAllocator>>) -> Self {
        Self {
            next_id: Default::default(),
            contents: Vec::with_capacity(4),
            allocator,
            // variables: Default::default(),
            consts: Default::default(),
        }
    }
}

// impl<Meta> From<Vec<Instruction<Meta>>> for BasicBlock<Meta>
//         where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
//     fn from(instructions: Vec<Instruction<Meta>>) -> Self {
//         Self { 
//             next_id: Default::default(),
//             contents: vec![instructions.into()],
//             allocator: ConstAllocator::default(),
//             variables: Default::default(),
//             consts: Default::default(),
//         }
//     }
// }

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
        // self.variables.insert(id, var.clone());
        var
    }

    fn allocator(&self) -> Rc<RefCell<ConstAllocator>> {
        self.allocator.clone()
    }
}

impl<Meta> BlockAssembler<Meta> for BasicBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    fn basic_block(&mut self) -> &mut BasicBlock<Meta> {
        let block: BasicBlock<Meta> = BasicBlock::new(self.allocator.clone());
        self.contents.push(Block::Basic(block));

        if let Block::Basic(ref mut last) = self.contents.last_mut().unwrap() {
            last
        } else {
            unreachable!()
        }
    }

    fn loop_block(&mut self, condition: LoopCondition) -> &mut LoopBlock<Meta> {
        let block: LoopBlock<Meta> = LoopBlock::<Meta>::new(condition, BasicBlock::new(self.allocator.clone()));
        self.contents.push(block.into());

        if let Block::Loop(ref mut last) = self.contents.last_mut().unwrap() {
            last
        } else {
            unreachable!()
        }
    }
}

impl<Meta> MacroAssembler<Meta, AssemblerError, ConstAllocError> for BasicBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait, {
    fn new_stored_const(&mut self, data: &[u8]) -> Result<StoredConstant, AssemblerError> {
        let addr = self.allocator.borrow_mut().alloc_const(data.len() as u16)?;
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

impl<Meta> BlockTrait for BasicBlock<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait {
    type Contents = Vec<Block<Meta>>;

    fn contents(&self) -> &Self::Contents {
        &self.contents
    }

    fn contents_mut(&mut self) -> &mut Self::Contents {
        &mut self.contents
    }
}