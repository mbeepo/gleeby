use crate::codegen::assembler::Context;
use crate::codegen::cgb::ConstAllocError;
use crate::codegen::{Assembler, LoopCondition, MacroAssembler};
use crate::codegen::{Block, LoopBlock};
use crate::codegen::{Ctx, IdInner, Variable};
use crate::cpu::instructions::Instruction;

#[derive(Clone, Default, Debug, PartialEq, Eq)]
pub struct BasicBlock {
    next_id: IdInner,
    pub ctx: Ctx,
    pub variables: Vec<Variable>,
    pub contents: Vec<Block>,
}

impl From<Vec<Instruction>> for BasicBlock {
    fn from(instructions: Vec<Instruction>) -> Self {
        Self { 
            contents: vec![instructions.into()],
            ..Default::default()
        }
    }
}

impl From<BasicBlock> for Vec<u8> {
    fn from(value: BasicBlock) -> Self {
        (&value).into()
    }
}

impl From<&BasicBlock> for Vec<u8> {
    fn from(value: &BasicBlock) -> Self {
        value.contents.iter().flat_map(|block| { let out: Vec<u8> = block.into(); out }).collect::<Vec<u8>>()
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

impl MacroAssembler for BasicBlock {
    type AllocError = ConstAllocError;

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

    fn new_const(&mut self, data: &[u8]) -> Result<crate::memory::Addr, Self::AllocError> {
        todo!()
    }

    fn new_var<T>(&mut self, var: T) -> Variable
            where T: crate::codegen::assembler::AsBuf {
        todo!()
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