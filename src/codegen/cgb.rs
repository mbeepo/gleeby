use std::io::{Seek, Write};
use std::{collections::HashMap, fs::File, io};

use super::assembler::Context;
use super::block::Block;
use super::variables::IdInner;
use super::{Assembler, BasicBlock, LoopBlock, LoopCondition, MacroAssembler};
use crate::cpu::instructions::Instruction;
use crate::memory::Addr;
use crate::ppu::{palettes::Color, TilemapSelector};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InterruptHandlers {
    
}

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct ConstAllocator {
    pub next_const: Addr,
    pub max_const: Addr,
    pub next_var: Addr,
    pub max_var: Addr,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cgb {
    output: Vec<Block>,
    labels: HashMap<String, Addr>,
    palettes: [[Color; 4]; 8],
    consts: Vec<(Addr, Vec<u8>)>,
    tilemap: TilemapSelector,
    handlers: InterruptHandlers,
    allocator: ConstAllocator,
    next_id: IdInner,
}

impl Cgb {
    pub fn new() -> Self {
        let output: Vec<Block> = Vec::with_capacity(4);
        let allocator = ConstAllocator {
            next_const: 0x7800,
            max_const: 0x7fff,
            next_var: 0xc000,
            max_var: 0xcfff,
        };

        Self {
            output,
            labels: HashMap::new(),
            palettes: [[Color::WHITE; 4]; 8],
            allocator,
            consts: Vec::new(),
            tilemap: TilemapSelector::Tilemap9800,
            handlers: Default::default(),
            next_id: Default::default(),
        }
    }

    pub fn save(&self, file: &mut File) -> io::Result<()>{
        // set CGB mode
        file.seek(io::SeekFrom::Start(0x143))?;
        file.write_all(&[0x80])?;
    
        // jump to main code
        let trampoline: Vec<u8> = Instruction::Jp(0x150).into();
        file.seek(io::SeekFrom::Start(0x100))?;
        file.write_all(&trampoline)?;
    
        let output: Vec<u8> = self.output.iter().flat_map(|block| { let out: Vec<u8> = block.into(); out }).collect::<Vec<u8>>();
        file.seek(io::SeekFrom::Start(0x150))?;
        file.write_all(&output)?;

        for (addr, bytes) in &self.consts {
            file.seek(io::SeekFrom::Start(*addr as u64))?;
            file.write_all(&bytes)?;
        }

        io::Result::Ok(())
    }
}

impl Assembler for Cgb {
    fn push_instruction(&mut self, instruction: Instruction) {
        self.push_buf(&[instruction]);
    }

    fn push_buf(&mut self, buf: &[Instruction]) {
        if let Some(Block::Raw(block)) = self.output.last_mut() {
            block.0.extend(buf);
        } else {
            let mut new: Vec<Instruction> = Vec::with_capacity(buf.len() + 2);
            new.extend(buf);
            self.output.push(new.into());
        }
    }

    fn len(&self) -> usize {
        self.output.iter().fold(0, |acc, block| acc + block.len())
    }
}

impl MacroAssembler for Cgb {
    type AllocError = ConstAllocError;

    /// [BasicBlock] builder
    fn basic_block<F>(&mut self, inner: F) -> &mut Self
            where F: Fn(&mut BasicBlock) {
        let mut block: BasicBlock = Vec::with_capacity(4).into();
        inner(&mut block);

        let ctx = self.new_ctx();
        block.ctx = ctx;
        self.output.push(block.into());

        self
    }

    /// [Loop] builder
    fn loop_block<F>(&mut self, condition: LoopCondition, inner: F) -> &mut Self
           where F: Fn(&mut LoopBlock) {
        let mut block: LoopBlock = LoopBlock::new(condition, Vec::with_capacity(4).into());
        inner(&mut block);

        let ctx = self.new_ctx();
        block.ctx = ctx;
        self.output.push(block.into());

        self
    }

    fn new_var<T>(&mut self, var: T) -> super::Variable
           where T: super::assembler::AsBuf {
        todo!()
    }

    fn new_const(&mut self, data: &[u8]) -> Result<Addr, Self::AllocError> {
        let addr = self.allocator.new_const(data)?;
        self.consts.push((addr, data.to_vec()));

        Ok(addr)
    }
}

impl Context for Cgb {
    fn next_id(&self) -> IdInner {
        self.next_id
    }

    fn next_id_mut(&mut self) -> &mut IdInner {
        &mut self.next_id
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstAllocError {

}

impl ConstAllocator {
    pub fn new_const(&mut self, data: &[u8]) -> Result<Addr, ConstAllocError> {
        let len = data.len();
        let addr = self.next_const;
        self.next_const += len as u16;

        Ok(addr)
    }
}