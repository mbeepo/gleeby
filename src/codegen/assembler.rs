use crate::{codegen::block::BlockTrait, cpu::{instructions::{Bit, Instruction, PrefixInstruction}, Condition, GpRegister, IndirectPair, RegisterPair, SplitError, StackPair}, memory::{Addr, IoReg}, ppu::{objects::{Sprite, SpriteIdx}, palettes::{CgbPalette, Color, PaletteSelector}, tiles::{Tile, TileIdx, Tilemap}, TiledataSelector, TilemapSelector}};

use super::{allocator::{AllocErrorTrait, ConstAllocError}, block::basic_block::BasicBlock, meta_instr::{MetaInstructionTrait, VarOrConst}, variables::{Constant, RawRegVariable, RawVariable, StoredConstant, Variabler}, AssemblerError, Id, IdInner, LoopBlock, LoopCondition, Variable};

pub trait Assembler<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait {

    fn push_instruction(&mut self, instruction: Instruction<Meta>);
    fn push_buf(&mut self, buf: &[Instruction<Meta>]);
    fn len(&self) -> usize;

    /// `ld rr, n16`
    /// 
    /// Load the little endian immediate `imm` into `reg`
    fn ld_r16_imm<T>(&mut self, reg_pair: T, imm: u16) -> &mut Self
            where T: Into<RegisterPair> {
        self.push_instruction(Instruction::LdR16Imm(reg_pair.into(), imm));
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
    fn inc_r16<T>(&mut self, reg_pair: T) -> &mut Self
            where T: Into<RegisterPair> {
        self.push_instruction(Instruction::IncR16(reg_pair.into()));
        self
    }

    /// `inc r`
    /// 
    /// Increment register `r`
    fn inc_r8<T>(&mut self, reg: T) -> &mut Self
            where T: Into<GpRegister> {
        self.push_instruction(Instruction::IncR8(reg.into()));
        self
    }

    /// `dec r`
    /// 
    /// Decrement register `r`
    fn dec_r8<T>(&mut self, reg: T) -> &mut Self
            where T: Into<GpRegister> {
        self.push_instruction(Instruction::DecR8(reg.into()));
        self
    }
    
    /// `ld r, n8`
    /// 
    /// Load the immediate `imm` into `reg`
    fn ld_r8_imm<T>(&mut self, reg: T, imm: u8) -> &mut Self
            where T: Into<GpRegister> {
        self.push_instruction(Instruction::LdR8Imm(reg.into(), imm));
        self
    }

    /// `dec rr`
    /// 
    /// Decrement register pair `rr`
    fn dec_r16<T>(&mut self, reg_pair: T) -> &mut Self
            where T: Into<RegisterPair> {
        self.push_instruction(Instruction::DecR16(reg_pair.into()));
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
    fn jr<T>(&mut self, condition: T, offset: i8) -> &mut Self
            where T: Into<Condition> {
        self.push_instruction(Instruction::Jr(condition.into(), offset));
        self
    }

    /// `ld r, r`
    /// 
    /// Load the byte in `src` into `dest`
    fn ld_r8_from_r8<T, U>(&mut self, dest: T, src: U) -> &mut Self
            where T: Into<GpRegister>, U: Into<GpRegister> {
        self.push_instruction(Instruction::LdR8FromR8(dest.into(), src.into()));
        self
    }

    /// `cp a, r`
    /// 
    /// Subtract `r` from `a` without storing the result (still changes flags)
    fn cp<T>(&mut self, reg: T) -> &mut Self
            where T: Into<GpRegister> {
        self.push_instruction(Instruction::Cp(reg.into()));
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

    /// `ld [$ff00+c], a`
    /// 
    /// Load the byte in `a` into the zero page offset by `c`
    fn ldh_from_a_with_c(&mut self) -> &mut Self {
        self.push_instruction(Instruction::LdhFromAWithC);
        self
    }

    /// `ld [imm], a`
    /// 
    /// Load the byte in `a` into the immediate 16-bit address
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

    /// `ld a, [$ff00+c],
    /// 
    /// Load the byte in `a` into the zero page offset by `c`
    fn ldh_to_a_with_c(&mut self) -> &mut Self {
        self.push_instruction(Instruction::LdhToAWithC);
        self
    }

    /// `ld a, [imm]`
    /// 
    /// Load the byte at the immediate 16-bit address into `a`
    fn ld_a_from_ind(&mut self, imm: Addr) -> &mut Self {
        self.push_instruction(Instruction::LdAFromInd(imm));
        self
    }

    /// bit `bit`, `reg`
    /// 
    /// Tests bit `bit` of `reg`, setting the Zero flag if not set
    fn bit<T>(&mut self, reg: T, bit: Bit) -> &mut Self
            where T: Into<GpRegister> {
        self.push_instruction(PrefixInstruction::Bit(bit, reg.into()).into());
        self
    }

    /// res `bit`, `reg`
    /// 
    /// Resets bit `bit` of `reg`
    fn res<T>(&mut self, reg: T, bit: Bit) -> &mut Self
            where T: Into<GpRegister> {
        self.push_instruction(PrefixInstruction::Res(bit, reg.into()).into());
        self
    }

    /// set `bit`, `reg`
    /// 
    /// Sets bit `bit` of `reg`
    fn set<T>(&mut self, reg: T, bit: Bit) -> &mut Self
            where T: Into<GpRegister> {
        self.push_instruction(PrefixInstruction::Set(bit, reg.into()).into());
        self
    }

    /// Metadata tag for assembler usage
    fn meta(&mut self, meta: Meta) -> &mut Self {
        self.push_instruction(Instruction::Meta(meta));
        self
    }
}

pub trait BlockAssembler<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait {
        /// [BasicBlock] builder
    fn basic_block(&mut self) -> &mut BasicBlock<Meta>;
    /// [LoopBlock] builder
    fn loop_block(&mut self, condition: LoopCondition) -> &mut LoopBlock<Meta>;
}

pub trait MacroAssembler<Meta, Error, AllocError>: Assembler<Meta> + Variabler<Meta, Error, AllocError> + BlockAssembler<Meta>
        where Meta: Clone + std::fmt::Debug + MetaInstructionTrait,
            Error: Clone + std::fmt::Debug + From<SplitError> + From<AllocError> + From<AssemblerError> + From<ConstAllocError> + ErrorTrait, // TODO: Not this
            AllocError: Clone + std::fmt::Debug + Into<Error> + AllocErrorTrait, {
    fn new_stored_const(&mut self, data: &[u8]) -> Result<StoredConstant, Error>;
    fn new_inline_const_r8(&mut self, data: u8) -> Constant;
    fn new_inline_const_r16(&mut self, data: u16) -> Constant;
    fn evaluate_meta(&mut self) -> Result<(), Error>;
    fn gather_consts(&mut self) -> Vec<(Constant, Vec<u8>)>;

    fn set_palette(&mut self, palette: CgbPalette, colors: [Color; 4]) -> Result<(), Error> {
        let colors: Vec<u8> = colors.iter().flat_map(|color| color.0.to_be_bytes()).collect();
        let addr = self.new_stored_const(&colors)?;

        let reg_hl = self.claim_reg_pair(RegisterPair::HL, Id::Unset);
        let reg_a = self.claim_reg(GpRegister::A, Id::Unset);

        let block = self.basic_block().open(|block| {
            block.ld_r16_imm(reg_hl.inner, addr.addr);
            block.ld_r8_imm(reg_a.inner, PaletteSelector::new(true, palette).into());
            block.ldh_from_a(IoReg::Bcps.into());
        });
        
        let counter = block.init_var8(colors.len() as u8)?.into_raw();
        block.loop_block(LoopCondition::Countdown { counter: counter.clone(), end: 0 }).open(|block| {
            block.ld_a_from_r16(IndirectPair::HLInc).ldh_from_a(IoReg::Bcpd.into());
        });

        self.dealloc_var(counter.into())?;

        Ok(())
    }

    fn copy(&mut self, src: Addr, dest: Addr, len: u16) -> Result<(), Error> {
        let reg_hl = RegisterPair::HL;
        let hl_stacked = if self.reg_is_used(RegisterPair::HL) {
            self.push(StackPair::HL);
            true
        } else {
            false
        };

        let reg_a = GpRegister::A;
        let af_stacked = if self.reg_is_used(GpRegister::A) {
            self.push(StackPair::AF);
            true
        } else {
            false
        };
        
        self.ld_r16_imm(reg_hl, dest);
        
        let hl_var: RawVariable = RawRegVariable::from(reg_hl).into();
        let mut data_pointer = self.init_var16(src)?;

        let block = (|| {
            if let Ok(len) = len.try_into() {
                if let Ok(reg) = self.alloc_reg() {
                    self.ld_r8_imm(&reg, len);
                    let counter: RawVariable = RawRegVariable::from(reg.inner).into();
                    return self.loop_block(LoopCondition::Countdown { counter, end: 0 });
                }
            }
            self.loop_block(LoopCondition::Countup{ counter: hl_var , end: dest + len })
        })();

        block.ld_a_from_var_ind(&mut data_pointer)?
            .ld_a_to_r16(IndirectPair::HLInc)
            .inc_var(&mut data_pointer)?;

        if hl_stacked { self.pop(StackPair::HL); }
        if af_stacked { self.pop(StackPair::AF); }

        Ok(())
    }

    fn write_tile_data(&mut self, area: TiledataSelector, idx: TileIdx, data: &Tile) -> Result<(), Error> {
        let src = self.new_stored_const(&data.as_bytes())?;
        let dest = area.from_idx(idx);

        self.basic_block().copy(src.addr, dest, Tile::MEM_SIZE as u16)?;

        Ok(())
    }

    fn set_tilemap(&mut self, tilemap: TilemapSelector, data: Tilemap) -> Result<(), Error> {
        let block = self.basic_block();
        let addr = block.new_stored_const((&data).into())?;
        block.copy(addr.addr, tilemap.base(), data.len() as u16)?;

        Ok(())
    }

    /// `tile` is the tile in the map to set, `data` is the tile data to set it to
    fn set_tile(&mut self, tilemap: TilemapSelector, tile: TileIdx, data: TileIdx) -> Result<(), Error> {
        let src = self.init_var8(data)?;
        let dest = tilemap.from_idx(tile);
        self.ld_var_to_ind(&src, dest)?;
        Ok(())
    }

    fn set_sprite(&mut self, sprite: Sprite, idx: SpriteIdx) {
        let (_, _) = (sprite, idx);
        todo!()
    }

    /// Disables the LCD immediately, granting full access to PPU related memory
    /// 
    /// This should only be called during VBlank: [https://gbdev.io/pandocs/LCDC.html#lcdc7--lcd-enable]
    fn disable_lcd_now(&mut self) {
        let reg_a = self.claim_reg(GpRegister::A, Id::Unset);
        
        self.basic_block()
            .ldh_to_a(IoReg::Lcdc.into())
            .res(reg_a.inner, Bit::_7)
            .ldh_from_a(IoReg::Lcdc.into());
    }

    fn enable_lcd_now(&mut self) {
        let reg_a = self.claim_reg(GpRegister::A, Id::Unset);
        
        self.basic_block().open(|block| {
            block.ldh_to_a(IoReg::Lcdc.into());
            block.set(reg_a.inner, Bit::_7);
            block.ldh_from_a(IoReg::Lcdc.into());
        });
    }

    fn store_byte(&mut self, addr: Addr, value: u8) {
        let reg = self.alloc_reg();

        if let Ok(reg) = reg {
            if reg.inner == GpRegister::A {
                self.ld_r8_imm(reg.inner, value);

                if addr > 0xff00 {
                    self.ldh_from_a((addr & 0x00ff) as u8);
                } else {
                    self.ld_a_to_ind(addr);
                }
            } else {
                self.ld_r8_from_r8(reg.inner, GpRegister::A);
                self.ld_r8_imm(GpRegister::A, value);

                if addr > 0xff00 {
                    self.ldh_from_a((addr & 0x00ff) as u8);
                } else {
                    self.ld_a_to_ind(addr);
                }

                self.ld_r8_from_r8(GpRegister::A, reg.inner);
            }
        } else {
            self.push(StackPair::AF);
            self.ld_r8_imm(GpRegister::A, value);

            if addr > 0xff00 {
                self.ldh_from_a((addr & 0x00ff) as u8);
            } else {
                self.ld_a_to_ind(addr);
            }

            self.pop(StackPair::AF);
        }
    }

    fn set_ioreg(&mut self, ioreg: IoReg, value: u8) {
        self.store_byte(ioreg as u16, value);
    }
    
    fn init_var8<T>(&mut self, value: T) -> Result<Variable, Error>
            where T: Clone + Copy + Into<u8> {
        let mut val_const = VarOrConst::Const(self.new_inline_const_r8(value.into()));
        let mut var = self.new_var(1);
        
        self.set_var(&mut var, &mut val_const)?;

        Ok(var)
    }

    fn init_var16<T>(&mut self, value: T) -> Result<Variable, Error>
            where T: Clone + Copy + Into<u16> {
        let mut val_const = VarOrConst::Const(self.new_inline_const_r16(value.into()));
        let mut var = self.new_var(2);
        
        self.set_var(&mut var, &mut val_const)?;
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

pub trait ErrorTrait {
    fn invalid_arg() -> Self where Self: Sized;
}