use std::{collections::HashMap, fs::File, io, os::windows::fs::FileExt};

use codegen::{BasicBlock, Block};
use cpu::{instructions::Instruction, Register, RegisterPair};
use memory::{Addr, IoReg};
use ppu::{objects::SpriteIdx, palettes::{CgbPalette, Color, PaletteSelector}, tiles::{Tile, TileIdx}, TiledataSelector, TilemapSelector};

pub mod codegen;
pub mod cpu;
pub mod memory;
pub mod ppu;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cgb<'a> {
    output: Vec<Block>,
    labels: HashMap<String, Addr>,
    palettes: [[Color; 4]; 8],
    next_const: Addr,
    consts: Vec<(Addr, Box<&'a [u8]>)>,
    tilemap: TilemapSelector,
}

impl<'a> Cgb<'a> {
    pub fn new() -> Self {
        let mut output: Vec<Block> = Vec::new();

        Self {
            output,
            labels: HashMap::new(),
            palettes: [[Color::WHITE; 4]; 8],
            next_const: 0xc000,
            consts: Vec::new(),
            tilemap: TilemapSelector::Tilemap9800,
        }
    }

    pub fn set_palette(&mut self, palette: CgbPalette, colors: [Color; 4]) {
        use Instruction::*;
        use Register::*;
        use RegisterPair::*;

        let mut out: Vec<Block> = Vec::with_capacity(5);
        out.push(Block::Basic(BasicBlock::from(vec![
            // Select palette to change
            LdR16Imm(HL, IoReg::Bcps as u16),
            LdR8Imm(A, PaletteSelector::new(true, palette).into()),
            LdToHlAdd,
        ])));

        for color in colors {
            let color = color.0.to_le_bytes();
            out.push(Block::Basic(BasicBlock::from(vec![
                LdHlImm(color[1].into()),
                LdHlImm(color[0].into()),
            ])));
        }

        let idx: usize = palette.into();
        self.palettes[idx] = colors;
        self.output.extend(out);
    }

    pub fn write_tile_data(&mut self, area: TiledataSelector, idx: TileIdx, data: Tile) {
        use Instruction::*;
        use Register::*;
        use RegisterPair::*;

        let addr = area.from_idx(idx);
        let mut out: Vec<Block> = Vec::with_capacity(9);
        
        out.push(Block::Basic(BasicBlock::from(
            LdR16Imm(HL, addr)
        )));

        for byte in data.as_bytes() {
            out.push(Block::Basic(BasicBlock::from(vec![
                LdR8Imm(A, byte),
                LdToHlAdd,
            ])));
        }

        self.output.extend(out);
    }

    pub fn set_tilemap<F>(&mut self, selector: TilemapSelector, setter: F) 
        // where F: Fn(u8, u8) -> Tile
        where F: Fn(u8, u8) -> TileIdx
    {
        use Instruction::*;
        use Register::*;
        use RegisterPair::*;

        let mut out: Vec<Block> = Vec::with_capacity(32 * 32);

        out.push(Block::Basic(BasicBlock::from(
            LdR16Imm(HL, selector.base())
        )));

        for x in 0..32 {
            for y in 0..32 {
                let idx = setter(x, y);
                // let palette = self.get_palette(tile.colors);
                let palette = CgbPalette::_0;
                let addr = self.tilemap.from_idx(idx);   

                out.push(Block::Basic(BasicBlock::from(vec![
                    LdR8Imm(A, idx),
                    LdToHlAdd,
                ])));
            }
        }

        self.output.extend(out);
    }

    pub fn set_sprite(&mut self, sprite: Tile, idx: SpriteIdx) {
        todo!()
    }

    pub fn const_alloc(&'a mut self, data: &'a [u8], label: &str) -> Result<Addr, ()> {
        let len = data.len();
        self.consts.push((self.next_const, Box::new(data)));
        let addr = self.next_const;
        self.next_const += len as u16;
        self.labels.insert(label.to_owned(), addr);

        Ok(addr)
    }

    pub fn push(&mut self, block: Block) {
        self.output.push(block);
    }

    pub fn save(&self, file: File) -> io::Result<()>{
        // set CGB mode
        file.seek_write(&[0x80], 0x143)?;
    
        // jump to main code
        let trampoline: Vec<u8> = Instruction::Jp(0x150).into();
        file.seek_write(&trampoline, 0x100)?;
    
        let output: Vec<u8> = self.output.iter().flat_map(|block| { let out: Vec<u8> = block.into(); out }).collect::<Vec<u8>>();
        file.seek_write(&output, 0x150)?;

        io::Result::Ok(())
    }
}