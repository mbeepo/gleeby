use std::io::{Seek, Write};
use std::{collections::HashMap, fs::File, io};

use super::allocator::{Allocator, ConstAllocError, ConstAllocator};
use super::assembler::Context;
use super::block::Block;
use super::meta_instr::MetaInstruction;
use super::variables::{Constant, IdInner, Variable, Variabler};
use super::{Assembler, AssemblerError, BasicBlock, Id, LoopBlock, LoopCondition, MacroAssembler};
use crate::cpu::instructions::Instruction;
use crate::cpu::Condition;
use crate::memory::Addr;
use crate::ppu::{palettes::Color, TilemapSelector};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InterruptHandlers {
    
}

#[derive(Clone, Debug)]
pub struct Cgb {
    output: Vec<Block<MetaInstruction>>,
    labels: HashMap<String, Addr>,
    palettes: [[Color; 4]; 8],
    tilemap: TilemapSelector,
    handlers: InterruptHandlers,
    allocator: ConstAllocator,
    next_id: IdInner,
    consts: HashMap<Id, (Constant, Vec<u8>)>,
    variables: HashMap<Id, Variable>,
}

impl Cgb {
    pub fn new() -> Self {
        let output: Vec<Block<MetaInstruction>> = Vec::with_capacity(4);
        let mut allocator = ConstAllocator::default();
        allocator.constants.offset = 0x07ff;
        allocator.variables.offset = 0xc000;

        Self {
            output,
            labels: HashMap::new(),
            palettes: [[Color::WHITE; 4]; 8],
            allocator,
            tilemap: TilemapSelector::Tilemap9800,
            handlers: Default::default(),
            next_id: 1,
            consts: HashMap::with_capacity(4),
            variables: HashMap::with_capacity(4),
        }
    }

    pub fn save(self, file: &mut File) -> io::Result<()>{
        // set CGB mode
        file.seek(io::SeekFrom::Start(0x143))?;
        file.write_all(&[0x80])?;
    
        // jump to main code
        let trampoline: Vec<u8> = Instruction::<MetaInstruction>::Jp(Condition::Always, 0x150).into();
        file.seek(io::SeekFrom::Start(0x100))?;
        file.write_all(&trampoline)?;
    
        let output: Vec<u8> = self.output.into_iter().flat_map(|block| { let out: Vec<u8> = block.try_into().expect("Blorp"); out }).collect::<Vec<u8>>();
        file.seek(io::SeekFrom::Start(0x150))?;
        file.write_all(&output)?;

        for (constant, bytes) in self.consts.values() {
            file.seek(io::SeekFrom::Start(constant.addr as u64))?;
            file.write_all(&bytes)?;
        }

        io::Result::Ok(())
    }
}

impl Assembler<MetaInstruction> for Cgb {
    fn push_instruction(&mut self, instruction: Instruction<MetaInstruction>) {
        self.push_buf(&[instruction]);
    }

    fn push_buf(&mut self, buf: &[Instruction<MetaInstruction>]) {
        if let Some(Block::Raw(block)) = self.output.last_mut() {
            block.0.extend(buf.to_vec());
        } else {
            let mut new = Vec::with_capacity(buf.len() + 2);
            new.extend(buf.to_vec());
            self.output.push(new.into());
        }
    }

    fn len(&self) -> usize {
        self.output.iter().fold(0, |acc, block| acc + block.len())
    }
}

impl Variabler<MetaInstruction, AssemblerError, ConstAllocError> for Cgb {
    type Alloc = ConstAllocator;

    fn new_var(&mut self, len: u16) -> super::Variable {
        let id = self.new_id();
        let var = Variable::Unallocated { len, id };
        self.variables.insert(id, var);
        var
    }
    
    fn allocator(&mut self) -> &mut Self::Alloc {
        &mut self.allocator
    }
}

impl MacroAssembler<MetaInstruction, AssemblerError, ConstAllocError> for Cgb {
    /// [BasicBlock] builder
    fn basic_block(&mut self) -> &mut BasicBlock<MetaInstruction> {
        let block: BasicBlock<_> = Vec::with_capacity(4).into();
        self.output.push(block.into());

        if let Block::Basic(ref mut last) = self.output.last_mut().unwrap() {
            last
        } else {
            unreachable!()
        }
    }

    /// [Loop] builder
    fn loop_block(&mut self, condition: LoopCondition) -> &mut LoopBlock<MetaInstruction> {
        let block: LoopBlock<_> = LoopBlock::<_>::new(condition, Vec::with_capacity(4).into());
        self.output.push(block.into());
        
        if let Block::Loop(ref mut last) = self.output.last_mut().unwrap() {
            last
        } else {
            unreachable!()
        }
    }

    fn new_const(&mut self, data: &[u8]) -> Result<Constant, AssemblerError> {
        let addr = self.allocator.alloc_const(data.len() as u16)?;
        let id = self.new_id();
        let constant = Constant {
            id,
            addr,
            len: data.len() as u16
        };

        self.consts.insert(id, (constant, data.to_vec()));

        Ok(constant)
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