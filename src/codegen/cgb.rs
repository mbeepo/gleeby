use std::cell::RefCell;
use std::io::{Seek, Write};
use std::rc::Rc;
use std::{fs::File, io};

use super::allocator::{ConstAllocError, ConstAllocator};
use super::assembler::Context;
use super::meta_instr::MetaInstruction;
use super::variables::{Constant, IdInner, StoredConstant, Variabler};
use super::{Assembler, AssemblerError, BasicBlock, LoopBlock, LoopCondition, MacroAssembler};
use crate::cpu::instructions::Instruction;
use crate::cpu::Condition;
use crate::ppu::{palettes::Color, TilemapSelector};

#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct InterruptHandlers {
    
}

#[derive(Clone, Debug)]
pub struct Cgb {
    inner: BasicBlock<MetaInstruction>,
    palettes: [[Color; 4]; 8],
    tilemap: TilemapSelector,
    handlers: InterruptHandlers,
}

impl Cgb {
    pub fn new() -> Self {
        let mut allocator = ConstAllocator::default();
        allocator.constants.offset = 0x0800;
        allocator.variables.offset = 0xc000;

        let inner: BasicBlock<MetaInstruction> = BasicBlock::new(Rc::new(RefCell::new(allocator)));

        Self {
            inner,
            palettes: [[Color::WHITE; 4]; 8],
            tilemap: TilemapSelector::Tilemap9800,
            handlers: Default::default(),
        }
    }

    pub fn save(mut self, file: &mut File) -> io::Result<()>{
        // set CGB mode
        file.seek(io::SeekFrom::Start(0x143))?;
        file.write_all(&[0x80])?;

        // jump to main code
        let trampoline: Vec<u8> = Instruction::<MetaInstruction>::Jp(Condition::Always, 0x150).into();
        file.seek(io::SeekFrom::Start(0x100))?;
        file.write_all(&trampoline)?;

        // TODO: Fix constant allocation
        for (constant, bytes) in self.inner.gather_consts() {
            if let Constant::Addr(constant) = constant {
                file.seek(io::SeekFrom::Start(constant.addr as u64))?;
                file.write_all(&bytes)?;
            }
        }

        let output: Vec<u8> = self.inner.try_into().expect("Blorp");
        file.seek(io::SeekFrom::Start(0x150))?;
        file.write_all(&output)?;


        io::Result::Ok(())
    }
}

impl Assembler<MetaInstruction> for Cgb {
    fn push_instruction(&mut self, instruction: Instruction<MetaInstruction>) {
        self.inner.push_instruction(instruction)
    }

    fn push_buf(&mut self, buf: &[Instruction<MetaInstruction>]) {
        self.inner.push_buf(buf)
    }

    fn len(&self) -> usize {
        self.inner.len()
    }
}

impl Variabler<MetaInstruction, AssemblerError, ConstAllocError> for Cgb {
    type Alloc = ConstAllocator;

    fn new_var(&mut self, len: u16) -> super::Variable {
        self.inner.new_var(len)
    }

    fn allocator(&self) -> Rc<RefCell<Self::Alloc>> {
        self.inner.allocator()
    }
}

impl MacroAssembler<MetaInstruction, AssemblerError, ConstAllocError> for Cgb {
    /// [BasicBlock] builder
    fn basic_block(&mut self) -> &mut BasicBlock<MetaInstruction> {
        self.inner.basic_block()
    }

    /// [Loop] builder
    fn loop_block(&mut self, condition: LoopCondition) -> &mut LoopBlock<MetaInstruction> {
        self.inner.loop_block(condition)
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

    fn evaluate_meta(&mut self) -> Result<(), AssemblerError> {
        self.inner.evaluate_meta()
    }

    fn gather_consts(&mut self) -> Vec<(Constant, Vec<u8>)> {
        self.inner.gather_consts()
    }
}

impl Context for Cgb {
    fn next_id(&self) -> IdInner {
        self.inner.next_id()
    }

    fn next_id_mut(&mut self) -> &mut IdInner {
        self.inner.next_id_mut()
    }
}