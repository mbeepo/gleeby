use crate::{codegen::block::Block, cpu::{instructions::{Bit, Instruction}, Condition, Register, RegisterPair}, memory::{Addr, IoReg}, ppu::{objects::{Sprite, SpriteIdx}, palettes::{CgbPalette, Color, PaletteSelector}, tiles::{Tile, TileIdx}, TiledataSelector, TilemapSelector}};

use super::{block::basic_block::BasicBlock, Ctx, Id, IdInner, LoopBlock, LoopCondition, Variable};

pub trait Assembler {
    fn push_instruction(&mut self, instruction: Instruction);
    fn push_buf(&mut self, buf: &[Instruction]);
    fn len(&self) -> usize;

    /// ld r, `[hl]`
    /// 
    /// Load the byte at `[hl]` into `reg`
    fn LdR8FromHl(&mut self, reg: Register) -> &mut Self {
        self.push_instruction(Instruction::LdR8FromHl(reg));
        self
    }

    /// ld `[hl]`, r
    /// 
    /// Load the byte in `reg` into `[hl]`
    fn LdR8ToHl(&mut self, reg: Register) -> &mut Self {
        self.push_instruction(Instruction::LdR8ToHl(reg));
        self
    }

    /// ld r, n8
    /// 
    /// Load the immediate `imm` into `reg`
    fn LdR8Imm(&mut self, reg: Register, imm: u8) -> &mut Self {
        self.push_instruction(Instruction::LdR8Imm(reg, imm));
        self
    }

    /// ld r, r
    /// 
    /// Load the byte in `src` into `dest`
    fn LdR8FromR8(&mut self, dest: Register, src: Register) -> &mut Self {
        self.push_instruction(Instruction::LdR8FromR8(dest, src));
        self
    }

    /// ld rr, n16
    /// 
    /// Load the little endian immediate `imm` into `reg`
    fn LdR16Imm(&mut self, reg_pair: RegisterPair, imm: u16) -> &mut Self {
        self.push_instruction(Instruction::LdR16Imm(reg_pair, imm));
        self
    }

    /// ld a, `[rr]`
    /// 
    /// Load the byte in `[reg]` into `a`
    fn LdAFromR16(&mut self, reg_pair: RegisterPair) -> &mut Self {
        self.push_instruction(Instruction::LdAFromR16(reg_pair));
        self
    }

    /// ld `[rr]`, a
    /// 
    /// Load the byte in `a` into `[reg]`
    fn LdAToR16(&mut self, reg_pair: RegisterPair) -> &mut Self {
        self.push_instruction(Instruction::LdAToR16(reg_pair));
        self
    }

    /// jr cc, e8
    /// 
    /// Jump by `offset` bytes if `condition` is true
    fn Jr(&mut self, condition: Condition, offset: i8) -> &mut Self {
        self.push_instruction(Instruction::Jr(condition, offset));
        self
    }

    /// inc r
    /// 
    /// Increment register `r`
    fn IncR8(&mut self, reg: Register) -> &mut Self {
        self.push_instruction(Instruction::IncR8(reg));
        self
    }

    /// dec r
    /// 
    /// Decrement register `r`
    fn DecR8(&mut self, reg: Register) -> &mut Self {
        self.push_instruction(Instruction::DecR8(reg));
        self
    }

    /// inc rr
    /// 
    /// Increment register pair `rr`
    fn IncR16(&mut self, reg_pair: RegisterPair) -> &mut Self {
        self.push_instruction(Instruction::IncR16(reg_pair));
        self
    }

    /// dec rr
    /// 
    /// Decrement register pair `rr`
    fn DecR16(&mut self, reg_pair: RegisterPair) -> &mut Self {
        self.push_instruction(Instruction::DecR16(reg_pair));
        self
    }

    /// ld [hl+], a
    /// 
    /// Load the byte in `a` into `[hl]`, then increment hl
    fn LdToHlInc(&mut self) -> &mut Self {
        self.push_instruction(Instruction::LdToHlInc);
        self
    }

    /// ld [hl-], a
    /// 
    /// Load the byte in `a` into `[hl]`, then decrement hl
    fn LdToHlDec(&mut self) -> &mut Self {
        self.push_instruction(Instruction::LdToHlDec);
        self
    }

    /// ld a, [hl+]
    /// 
    /// Load the byte in `[hl]` into `a`, then increment hl
    fn LdFromHlInc(&mut self) -> &mut Self {
        self.push_instruction(Instruction::LdFromHlInc);
        self
    }

    /// ld a, [hl-]
    /// 
    /// Load the byte in `[hl]` into `a`, the decrement hl
    fn LdFromHlDec(&mut self) -> &mut Self {
        self.push_instruction(Instruction::LdFromHlDec);
        self
    }

    /// ld `[hl]`, n16
    /// 
    /// Load the little endian immediate `imm` into `[hl]`
    fn LdHlImm(&mut self, imm: u8) -> &mut Self {
        self.push_instruction(Instruction::LdHlImm(imm));
        self
    }

    /// ld `[$ff00+imm]`, a
    /// 
    /// Load the byte in `a` into the zero page offset by `imm`
    fn LdhFromA(&mut self, imm: u8) -> &mut Self {
        self.push_instruction(Instruction::LdhFromA(imm));
        self
    }

    /// ld a, `[$ff00+imm]`
    /// 
    /// Load the byte in the zero page at offset `imm` into `a`
    fn LdhToA(&mut self, imm: u8) -> &mut Self {
        self.push_instruction(Instruction::LdhToA(imm));
        self
    }

    /// jp a16
    /// 
    /// Jump to address `addr`
    fn Jp(&mut self, addr: u16) -> &mut Self {
        self.push_instruction(Instruction::Jp(addr));
        self
    }

    /// bit `bit`, `reg`
    /// 
    /// Tests bit `bit` of `reg, settings the Zero flag if not set
    fn Bit(&mut self, reg: Register, bit: Bit) -> &mut Self {
        self.push_instruction(Instruction::Bit(reg, bit));
        self
    }

    /// res `bit`, `reg`
    /// 
    /// Resets bit `bit` of `reg`
    fn Res(&mut self, reg: Register, bit: Bit) -> &mut Self {
        self.push_instruction(Instruction::Res(reg, bit));
        self
    }

    /// set `bit`, `reg`
    /// 
    /// Sets bit `bit` of `reg`
    fn Set(&mut self, reg: Register, bit: Bit) -> &mut Self {
        self.push_instruction(Instruction::Set(reg, bit));
        self
    }
}

pub trait MacroAssembler: Assembler {
    type AllocError: Clone + std::fmt::Debug;

    /// [BasicBlock] builder
    fn basic_block<F>(&mut self, inner: F) -> &mut Self
        where F: Fn(&mut BasicBlock);
    /// [Loop] builder
    fn loop_block<F>(&mut self, condition: LoopCondition, inner: F) -> &mut Self
        where F: Fn(&mut LoopBlock);
    fn new_var<T>(&mut self, var: T) -> Variable
        where T: AsBuf;
    fn new_const(&mut self, data: &[u8]) -> Result<Addr, Self::AllocError>;

    fn set_palette(&mut self, palette: CgbPalette, colors: [Color; 4]) -> Result<(), Self::AllocError> {
        use Register::*;
        use RegisterPair::*;

        let colors: Vec<u8> = colors.iter().flat_map(|color| color.0.to_be_bytes()).collect();
        let addr = self.new_const(&colors)?;

        self.basic_block(|block| {
            block.LdR16Imm(HL, addr);
            block.LdR8Imm(B, colors.len() as u8);
            block.LdR8Imm(A, PaletteSelector::new(true, palette).into());
            block.LdhFromA(IoReg::Bcps.into());

            let counter = Variable::StaticR8(B);
            block.loop_block(LoopCondition::Countdown { counter, end: 0 }, |block| {
                block.LdFromHlInc().LdhFromA(IoReg::Bcpd.into());
            });
        });

        Ok(())
    }

    fn copy(&mut self, src: Addr, dest: Addr, len: u8) {
        use Register::*;
        use RegisterPair::*;

        self.LdR16Imm(HL, dest);
        self.LdR16Imm(BC, src);
        self.LdR8Imm(D, len);
        let counter = Variable::StaticR8(D);

        self.loop_block(LoopCondition::Countdown { counter , end: 0 }, |block| {
            block.LdAFromR16(BC);
            block.LdToHlInc();
            block.IncR16(BC);
        });
    }

    fn write_tile_data(&mut self, area: TiledataSelector, idx: TileIdx, data: &Tile) -> Result<(), Self::AllocError> {
        let src = self.new_const(&data.as_bytes())?;
        let dest = area.from_idx(idx);

        self.basic_block(|block| block.copy(src, dest, Tile::MEM_SIZE as u8));

        Ok(())
    }

    fn set_tilemap<F>(&mut self, selector: TilemapSelector, setter: F) 
        // where F: Fn(u8, u8) -> Tile
            where F: Fn(u8, u8) -> TileIdx {
        use Register::*;
        use RegisterPair::*;

        self.basic_block(|block| {
            block.LdR16Imm(HL, selector.base());

            // TODO: Roll this up
            for x in 0..32 {
                for y in 0..32 {
                    let idx = setter(x, y);
    
                    block.LdR8Imm(A, idx);
                    block.LdToHlInc();
                }
            }
        });
    }

    fn set_sprite(&mut self, sprite: Sprite, idx: SpriteIdx) {
        todo!()
    }

    /// Disables the LCD immediately, granting full access to PPU related memory
    /// 
    /// This should only be called during VBlank: [https://gbdev.io/pandocs/LCDC.html#lcdc7--lcd-enable]
    fn disable_lcd_now(&mut self) {
        self.basic_block(|block| {
            block.LdhToA(IoReg::Lcdc.into());
            block.Res(Register::A, Bit::_7);
            block.LdhFromA(IoReg::Lcdc.into());
        });
    }

    fn enable_lcd_now(&mut self) {
        self.basic_block(|block| {
            block.LdhToA(IoReg::Lcdc.into());
            block.Set(Register::A, Bit::_7);
            block.LdhFromA(IoReg::Lcdc.into());
        });
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

    fn new_ctx(&mut self) -> Ctx {
        Ctx::Set(self.new_id_inner())
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