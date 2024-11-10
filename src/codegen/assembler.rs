use crate::{codegen::block::BlockTrait, cpu::{instructions::{Bit, Instruction, PrefixInstruction}, Condition, GpRegister, IndirectPair, RegisterPair, SplitError, StackPair}, memory::{Addr, IoReg}, ppu::{objects::{Sprite, SpriteIdx}, palettes::{CgbPalette, Color, PaletteSelector}, tiles::{Tile, TileIdx}, TiledataSelector, TilemapSelector}};

use super::{allocator::{AllocErrorTrait, ConstAllocError}, block::basic_block::BasicBlock, meta_instr::{MetaInstructionTrait, VarOrConst}, variables::{Constant, Variabler}, AssemblerError, Id, IdInner, LoopBlock, LoopCondition, Variable};

pub trait Assembler<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait {

    fn push_instruction(&mut self, instruction: Instruction<Meta>);
    fn push_buf(&mut self, buf: &[Instruction<Meta>]);
    fn len(&self) -> usize;

    /// `ld rr, n16`
    /// 
    /// Load the little endian immediate `imm` into `reg`
    fn ld_r16_imm(&mut self, reg_pair: RegisterPair, imm: u16) -> &mut Self {
        self.push_instruction(Instruction::LdR16Imm(reg_pair, imm));
        self
    }

    /// `ld a, [rr]`
    /// 
    /// Load the byte in `[reg]` into `a`
    fn ld_a_from_r16(&mut self, reg_pair: IndirectPair) -> &mut Self {
        self.push_instruction(Instruction::LdAFromR16(reg_pair));
        self
    }

    /// `inc rr`
    /// 
    /// Increment register pair `rr`
    fn inc_r16(&mut self, reg_pair: RegisterPair) -> &mut Self {
        self.push_instruction(Instruction::IncR16(reg_pair));
        self
    }

    /// `inc r`
    /// 
    /// Increment register `r`
    fn inc_r8(&mut self, reg: GpRegister) -> &mut Self {
        self.push_instruction(Instruction::IncR8(reg));
        self
    }

    /// `dec r`
    /// 
    /// Decrement register `r`
    fn dec_r8(&mut self, reg: GpRegister) -> &mut Self {
        self.push_instruction(Instruction::DecR8(reg));
        self
    }
    
    /// `ld r, n8`
    /// 
    /// Load the immediate `imm` into `reg`
    fn ld_r8_imm(&mut self, reg: GpRegister, imm: u8) -> &mut Self {
        self.push_instruction(Instruction::LdR8Imm(reg, imm));
        self
    }

    /// `dec rr`
    /// 
    /// Decrement register pair `rr`
    fn dec_r16(&mut self, reg_pair: RegisterPair) -> &mut Self {
        self.push_instruction(Instruction::DecR16(reg_pair));
        self
    }

    /// `ld [rr], a`
    /// 
    /// Load the byte in `a` into `[reg]`
    fn ld_a_to_r16(&mut self, reg_pair: IndirectPair) -> &mut Self {
        self.push_instruction(Instruction::LdAToR16(reg_pair));
        self
    }

    /// `jr cc, e8`
    /// 
    /// Jump by `offset` bytes if `condition` is true
    fn jr(&mut self, condition: Condition, offset: i8) -> &mut Self {
        self.push_instruction(Instruction::Jr(condition, offset));
        self
    }

    /// `ld r, r`
    /// 
    /// Load the byte in `src` into `dest`
    fn ld_r8_from_r8(&mut self, dest: GpRegister, src: GpRegister) -> &mut Self {
        self.push_instruction(Instruction::LdR8FromR8(dest, src));
        self
    }

    /// `pop rr`
    /// 
    /// Pops 2 bytes off the stack into `rr`
    fn pop(&mut self, reg_pair: StackPair) -> &mut Self {
        self.push_instruction(Instruction::Pop(reg_pair));
        self
    }

    /// `jp a16`
    /// 
    /// Jump to address `addr`
    fn jp(&mut self, condition: Condition, addr: Addr) -> &mut Self {
        self.push_instruction(Instruction::Jp(condition, addr));
        self
    }

    /// `push rr`
    /// 
    /// Pushes `rr` onto the stack
    fn push(&mut self, reg_pair: StackPair) -> &mut Self {
        self.push_instruction(Instruction::Push(reg_pair));
        self
    }

    /// `ld [$ff00+imm], a`
    /// 
    /// Load the byte in `a` into the zero page offset by `imm`
    fn ldh_from_a(&mut self, imm: u8) -> &mut Self {
        self.push_instruction(Instruction::LdhFromA(imm));
        self
    }

    /// `ld [imm], a`
    /// 
    /// Load the byte at the immediate 16-bit address into `a`
    fn ld_a_to_ind(&mut self, imm: Addr) -> &mut Self {
        self.push_instruction(Instruction::LdAToInd(imm));
        self
    }

    /// `ld a, [$ff00+imm]`
    /// 
    /// Load the byte in the zero page at offset `imm` into `a`
    fn ldh_to_a(&mut self, imm: u8) -> &mut Self {
        self.push_instruction(Instruction::LdhToA(imm));
        self
    }

    /// `ld a, [imm]`
    /// 
    /// Load the byte in `a` into the immediate 16-bit address
    fn ld_a_from_ind(&mut self, imm: Addr) -> &mut Self {
        self.push_instruction(Instruction::LdAFromInd(imm));
        self
    }

    /// bit `bit`, `reg`
    /// 
    /// Tests bit `bit` of `reg`, setting the Zero flag if not set
    fn bit(&mut self, reg: GpRegister, bit: Bit) -> &mut Self {
        self.push_instruction(PrefixInstruction::Bit(bit, reg).into());
        self
    }

    /// res `bit`, `reg`
    /// 
    /// Resets bit `bit` of `reg`
    fn res(&mut self, reg: GpRegister, bit: Bit) -> &mut Self {
        self.push_instruction(PrefixInstruction::Res(bit, reg).into());
        self
    }

    /// set `bit`, `reg`
    /// 
    /// Sets bit `bit` of `reg`
    fn set(&mut self, reg: GpRegister, bit: Bit) -> &mut Self {
        self.push_instruction(PrefixInstruction::Set(bit, reg).into());
        self
    }

    /// Metadata tag for assembler usage
    fn meta(&mut self, meta: Meta) -> &mut Self {
        self.push_instruction(Instruction::Meta(meta));
        self
    }
}

pub trait MacroAssembler<Meta, Error, AllocError>: Assembler<Meta> + Variabler<Meta, Error, AllocError>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait,
            Error: Clone + std::fmt::Debug + From<SplitError> + From<AllocError> + From<AssemblerError> + From<ConstAllocError>, // TODO: Not this
            AllocError: Clone + std::fmt::Debug + Into<Error> + AllocErrorTrait, {
    /// [BasicBlock] builder
    fn basic_block(&mut self) -> &mut BasicBlock<Meta>;
    /// [Loop] builder
    fn loop_block(&mut self, condition: LoopCondition) -> &mut LoopBlock<Meta>;
    fn new_const(&mut self, data: &[u8]) -> Result<Constant, Error>;

    fn set_palette(&mut self, palette: CgbPalette, colors: [Color; 4]) -> Result<(), Error> {
        use GpRegister::*;
        use RegisterPair::*;

        let colors: Vec<u8> = colors.iter().flat_map(|color| color.0.to_be_bytes()).collect();
        let addr = self.new_const(&colors)?;

        let block = self.basic_block().open(|block| {
            block.ld_r16_imm(HL, addr.addr);
            block.ld_r8_imm(A, PaletteSelector::new(true, palette).into());
            block.ldh_from_a(IoReg::Bcps.into());
        });
        
        let counter = block.init_var(colors.len() as u16)?;
        block.loop_block(LoopCondition::Countdown { counter, end: 0 }).open(|block| {
            block.ld_a_from_r16(IndirectPair::HLInc).ldh_from_a(IoReg::Bcpd.into());
        });

        Ok(())
    }

    fn copy(&mut self, src: Addr, dest: Addr, len: u8) -> Result<(), Error> {
        use RegisterPair::*;

        self.ld_r16_imm(HL, dest);
        self.ld_r16_imm(BC, src);
        let counter = self.init_var(len)?;

        self.loop_block(LoopCondition::Countdown { counter , end: 0 }).open(|block| {
            block.ld_a_from_r16(IndirectPair::BC);
            block.ld_a_to_r16(IndirectPair::HLInc);
            block.inc_r16(BC);
        });

        Ok(())
    }

    fn write_tile_data(&mut self, area: TiledataSelector, idx: TileIdx, data: &Tile) -> Result<(), Error> {
        let src = self.new_const(&data.as_bytes())?;
        let dest = area.from_idx(idx);

        self.basic_block().copy(src.addr, dest, Tile::MEM_SIZE as u8)?;

        Ok(())
    }

    fn set_tilemap<F>(&mut self, selector: TilemapSelector, setter: F) 
        // where F: Fn(u8, u8) -> Tile
            where F: Fn(u8, u8) -> TileIdx {
        use GpRegister::*;
        use RegisterPair::*;

        self.basic_block().open(|block| {
            block.ld_r16_imm(HL, selector.base());

            // TODO: Roll this up
            for x in 0..32 {
                for y in 0..32 {
                    let idx = setter(x, y);
    
                    block.ld_r8_imm(A, idx);
                    block.ld_a_to_r16(IndirectPair::HLInc);
                }
            }
        });
    }

    fn set_sprite(&mut self, sprite: Sprite, idx: SpriteIdx) {
        let (_, _) = (sprite, idx);
        todo!()
    }

    /// Disables the LCD immediately, granting full access to PPU related memory
    /// 
    /// This should only be called during VBlank: [https://gbdev.io/pandocs/LCDC.html#lcdc7--lcd-enable]
    fn disable_lcd_now(&mut self) {
        self.basic_block().open(|block| {
            block.ldh_to_a(IoReg::Lcdc.into());
            block.res(GpRegister::A, Bit::_7);
            block.ldh_from_a(IoReg::Lcdc.into());
        });
    }

    fn enable_lcd_now(&mut self) {
        self.basic_block().open(|block| {
            block.ldh_to_a(IoReg::Lcdc.into());
            block.set(GpRegister::A, Bit::_7);
            block.ldh_from_a(IoReg::Lcdc.into());
        });
    }
    
    fn init_var<T>(&mut self, value: T) -> Result<Variable, Error>
            where T: AsBuf {
        let buf = value.as_buf();
        let var = self.new_var(buf.len() as u16);
        let val_const = VarOrConst::Const(self.new_const(&buf)?);

        self.set_var(var, val_const)?;
        Ok(var)
    }
}

pub trait Context {
    fn next_id(&self) -> IdInner;
    fn next_id_mut(&mut self) -> &mut IdInner;

    fn new_id_inner(&mut self) -> IdInner {
        let out = self.next_id();
        *self.next_id_mut() += 1;

        out
    }

    fn new_id(&mut self) -> Id {
        Id::Set(self.new_id_inner())
    }
}

pub trait AsBuf {
    fn as_buf(&self) -> Vec<u8>;
}

impl AsBuf for u8 {
    fn as_buf(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}

impl AsBuf for u16 {
    fn as_buf(&self) -> Vec<u8> {
        self.to_le_bytes().to_vec()
    }
}