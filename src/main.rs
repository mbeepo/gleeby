use std::{collections::HashMap, fs::File, io::Write, ops::BitOr, os::windows::fs::FileExt};

use bitflags::bitflags;

macro_rules! color {
    (0) => { PaletteColor::ColorZero };
    (1) => { PaletteColor::ColorOne };
    (2) => { PaletteColor::ColorTwo };
    (3) => { PaletteColor::ColorThree };
}

macro_rules! palette {
    (0) => { CgbPalette::PaletteZero };
    (1) => { CgbPalette::PaletteOne };
    (2) => { CgbPalette::PaletteTwo };
    (3) => { CgbPalette::PaletteThree };
    (4) => { CgbPalette::PaletteFour };
    (5) => { CgbPalette::PaletteFive };
    (6) => { CgbPalette::PaletteSix };
    (7) => { CgbPalette::PaletteSeven };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConversionError {
    InvalidPaletteColor,
    InvalidPaletteIndex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CgbPalette {
    #[default]
    PaletteZero,
    PaletteOne,
    PaletteTwo,
    PaletteThree,
    PaletteFour,
    PaletteFive,
    PaletteSix,
    PaletteSeven,
}

impl From<CgbPalette> for u8 {
    fn from(value: CgbPalette) -> Self {
        use CgbPalette::*;
        match value {
            PaletteZero => 0,
            PaletteOne => 1,
            PaletteTwo => 2,
            PaletteThree => 3,
            PaletteFour => 4,
            PaletteFive => 5,
            PaletteSix => 6,
            PaletteSeven => 7,
        }
    }
}

impl From<CgbPalette> for usize {
    fn from(value: CgbPalette) -> Self {
        let idx: u8 = value.into();
        idx as usize
    }
}

impl TryFrom<usize> for CgbPalette {
    type Error = ConversionError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        use CgbPalette::*;
        match value {
            0 => Ok(PaletteZero),
            1 => Ok(PaletteOne),
            2 => Ok(PaletteTwo),
            3 => Ok(PaletteThree),
            4 => Ok(PaletteFour),
            5 => Ok(PaletteFive),
            6 => Ok(PaletteSix),
            7 => Ok(PaletteSeven),
            _ => Err(ConversionError::InvalidPaletteIndex),
        }
    }
}

impl CgbPalette {
    pub fn offset(&self) -> u8 {
        let idx: u8 = (*self).into();
        idx * 8
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PaletteColor {
    #[default]
    ColorZero,
    ColorOne,
    ColorTwo,
    ColorThree,
}

impl From<PaletteColor> for (u8, u8) {
    fn from(value: PaletteColor) -> Self {
        match value {
            PaletteColor::ColorZero => (0, 0),
            PaletteColor::ColorOne => (1, 0),
            PaletteColor::ColorTwo => (0, 1),
            PaletteColor::ColorThree => (1, 1),
        }
    }
}

impl TryFrom<u8> for PaletteColor {
    type Error = ConversionError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PaletteColor::ColorZero),
            1 => Ok(PaletteColor::ColorOne),
            2 => Ok(PaletteColor::ColorTwo),
            3 => Ok(PaletteColor::ColorThree),
            _ => Err(ConversionError::InvalidPaletteColor)
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileRow {
    pub pixel_data: (u8, u8)
}

impl TileRow {
    pub fn flat(color: PaletteColor) -> Self {
        [
            color, color, color, color,
            color, color, color, color
        ].into()
    }
}

impl From<[PaletteColor; 8]> for TileRow {
    fn from(value: [PaletteColor; 8]) -> Self {
       Self { pixel_data: value.iter().fold((0, 0), |mut acc, &color| {
            acc.0 <<= 1;
            acc.1 <<= 1;

            let new_data: (u8, u8) = color.into();
            acc.0 |= new_data.0;
            acc.1 |= new_data.1;

            acc
        }) }
    }
}

pub struct PaletteSelector {
    pub autoincrement: bool,
    pub palette: CgbPalette,
}

impl From<PaletteSelector> for u8 {
    fn from(value: PaletteSelector) -> Self {
        let idx: u8 = value.palette.into();

        if value.autoincrement {
            0x80 | (idx & 0x1f)
        } else {
            idx & 0x1f
        }
    }
}

impl PaletteSelector {
    pub fn new(autoincrement: bool, palette: CgbPalette) -> Self {
        Self { autoincrement, palette }
    }
}

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
    pub struct ObjAttributeFlags: u8 {
        const BG_PRIORITY   = 0b10000000;
        const Y_FLIP        = 0b01000000;
        const X_FLIP        = 0b00100000;
        const DMG_PALETTE1  = 0b00010000;
        const CGB_BANK1     = 0b00001000;
    }
}

impl From<ObjAttributes> for u8 {
    fn from(value: ObjAttributes) -> Self {
        let palette: u8 = value.cgb_palette.into();
        value.flags.bits() | palette
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub struct ObjAttributes {
    pub flags: ObjAttributeFlags,
    pub cgb_palette: CgbPalette,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Pos2 {
    pub x: u8,
    pub y: u8,
}

pub fn pos2(x: u8, y: u8) -> Pos2 {
    Pos2 { x, y }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(u16);

impl Color {
    const BLACK: Self = Self(0);
    const DARK_GREY: Self = Self(0b01010_01010_01010_0);
    const LIGHT_GREY: Self = Self(0b10101_10101_10101_0);
    const WHITE: Self = Self(u16::MAX);
    const RED: Self = Self(u16::from_le_bytes([0b00000000, 0b00011111]));
    const GREEN: Self = Self(u16::from_le_bytes([0b00000011, 0b11100000]));
    const BLUE: Self = Self(u16::from_le_bytes([0b01111100, 0b00000000]));
}

impl BitOr<Color> for Color {
    type Output = Color;
    
    fn bitor(self, rhs: Color) -> Self::Output {
        Color(self.0 | rhs.0)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tile {
    pub colors: [Color; 4],
    pub pixel_data: [TileRow; 8],
}

impl Tile {
    pub fn new(colors: [Color; 4], pixels: [[PaletteColor; 8]; 8]) -> Self {
        // `pixels` has 8 top level elements so this will always result in an 8-element collection
        let rows: [TileRow; 8] = pixels.into_iter().map(|row| row.into()).take(8).collect::<Vec<TileRow>>().try_into().unwrap();

        Self {
            colors,
            pixel_data: rows,
        }
    }

    pub fn try_from_bytes(colors: [Color; 4], pixels: [[u8; 8]; 8]) -> Result<Self, ConversionError> {
        let mut checked = [[PaletteColor::default(); 8]; 8];

        for (y, row) in pixels.iter().enumerate() {
            for (x, &px) in row.iter().enumerate() {
                checked[y][x] = px.try_into()?;
            }
        }

        Ok(Self::new(colors, checked))
    }

    pub fn flat(color: Color) -> Self {
        let colors = [color, color, color, color];
        let row = TileRow::flat(color!(0));
        let pixel_data = [
            row, row, row, row,
            row, row, row, row
        ];

        Self { colors, pixel_data }
    }

    pub fn as_bytes(&self) -> [u8; 16] {
        self.pixel_data.into_iter().flat_map(|row| [row.pixel_data.0, row.pixel_data.1]).collect::<Vec<u8>>().try_into().unwrap()
    }
}

pub type TileIdx = u8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sprite {
    pub pos: Pos2,
    // pub tile: Tile,
    pub tile: TileIdx,
    pub attr: ObjAttributes,
}

impl Sprite {
    pub fn unpack(&self) -> [u8; 4] {
        [self.pos.y, self.pos.x, self.tile as u8, self.attr.into()]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Register {
    A,
    B, C,
    D, E,
    H, L,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RegisterPair {
    BC,
    DE,
    HL,
}


#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Condition {
    Always,
    Z, NZ,
    C, NC,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Instruction {
    LdR8FromHl(Register),
    LdR8ToHl(Register),
    LdR8Imm(Register, u8),
    LdR8R8(Register, Register),
    LdR16Imm(RegisterPair, u16),
    Jr(Condition, i8),
    IncR16(RegisterPair),
    LdToHlAdd,
    LdToHlSub,
    LdFromHlAdd,
    LdFromHlA,
    LdHlImm(u8),
    Jp(u16),
}

impl From<Instruction> for Vec<u8> {
    fn from(value: Instruction) -> Self {
        use Instruction::*;
        use Register::*;
        use RegisterPair::*;

        match value {
            LdR8FromHl(A) => vec![0x7e],
            LdR16Imm(r16, imm) => {
                let imm = imm.to_le_bytes();
                let mut opcode = match r16 {
                    HL => vec![0x21],
                    _ => unimplemented!()
                };
                opcode.extend(imm);
                opcode
            }
            LdR8Imm(A, imm) => vec![0x3e, imm],
            LdHlImm(imm) => vec![0x36, imm],
            LdToHlAdd => vec![0x22],
            LdToHlSub => vec![0x32],
            Jp(imm) => {
                let imm = imm.to_le_bytes();
                vec![0xc3, imm[0], imm[1]]
            },
            Jr(Condition::Always, imm) => vec![0x18, imm as u8],
            _ => unimplemented!()
        }.to_vec()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BasicBlock {
    pub instructions: Vec<Instruction>,
}

impl From<Vec<Instruction>> for BasicBlock {
    fn from(instructions: Vec<Instruction>) -> Self {
        Self { instructions }
    }
}

impl From<Instruction> for BasicBlock {
    fn from(value: Instruction) -> Self {
        Self { instructions: vec![value] }
    }
}

impl From<BasicBlock> for Vec<u8> {
    fn from(value: BasicBlock) -> Self {
        value.instructions.iter().flat_map(|&instruction| { let out: Vec<u8> = instruction.into(); out }).collect()
    }
}

impl From<&BasicBlock> for Vec<u8> {
    fn from(value: &BasicBlock) -> Self {
        value.instructions.iter().flat_map(|&instruction| { let out: Vec<u8> = instruction.into(); out }).collect()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct Loop {
    pub condition: Condition,
    pub inner: BasicBlock,
}

impl Loop {
    pub fn new(condition: Condition, inner: BasicBlock) -> Self {
        Self { condition, inner }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum Block {
    Basic(BasicBlock),
    Loop(Loop),
}

impl From<&Instruction> for Block {
    fn from(value: &Instruction) -> Self {
        Self::Basic((*value).into())
    }
}

impl From<Block> for Vec<u8> {
    fn from(value: Block) -> Self {
        match value {
            Block::Basic(block) => block.into(),
            _ => unimplemented!()
        }
    }
}

impl From<&Block> for Vec<u8> {
    fn from(value: &Block) -> Self {
        match value {
            Block::Basic(block) => block.into(),
            _ => unimplemented!()
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TilemapSelector {
    Tilemap9800,
    Tilemap9C00,
}

impl TilemapSelector {
    pub fn from_idx(&self, idx: u8) -> u16 {
        self.base() + idx as u16
    }

    pub fn base(&self) -> u16 {
        match self {
            Self::Tilemap9800 => 0x9800,
            Self::Tilemap9C00 => 0x9c00,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TiledataSelector {
    Tiledata8000,
    Tiledata9000,
}

impl TiledataSelector {
    pub fn from_idx(&self, idx: u8) -> u16 {
        match self {
            Self::Tiledata8000 => {
                0x8000 + idx as u16 * 16
            }
            Self::Tiledata9000 => {
                let offset = idx as i8 as i16 * 16;
                0x9000_u16.wrapping_add(offset as u16)
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TileSelector {
    pub tilemap: TilemapSelector,
    pub idx: u8,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResourceError {
    PaletteMissing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpriteError {
    IndexOutOfRange,
}

pub type Addr = u16;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cgb<'a> {
    output: Vec<Block>,
    labels: HashMap<String, Addr>,
    palettes: [[Color; 4]; 8],
    next_const: Addr,
    consts: Vec<(Addr, Box<&'a [u8]>)>,
    tilemap: TilemapSelector,
}

const R_BCPS: u16 = 0xff68;
const R_BCPD: u16 = 0xff69;

impl<'a> Cgb<'a> {
    pub fn new() -> Self {
        Self {
            output: Vec::new(),
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
            LdR16Imm(HL, R_BCPS),
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

    pub fn write_tile_data(&mut self, area: TiledataSelector, idx: u8, data: Tile) -> Result<(), ResourceError> {
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
        Ok(())
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
                let palette = CgbPalette::PaletteZero;
                let addr = self.tilemap.from_idx(idx);   

                out.push(Block::Basic(BasicBlock::from(vec![
                    LdR8Imm(A, idx),
                    LdToHlAdd,
                ])));
            }
        }

        self.output.extend(out);
    }

    pub fn set_sprite(&mut self, sprite: Sprite, idx: u8) -> Result<(), SpriteError> {
        if idx > 40 {
            return Err(SpriteError::IndexOutOfRange);
        }

        

        Ok(())
    }

    fn get_palette(&self, colors: [Color; 4]) -> Result<CgbPalette, ResourceError> {
        self.palettes.iter().enumerate().find_map(|(i, &p)| { 
            if p == colors { 
                i.try_into().ok()
            } else {
                None
            }
        }).ok_or(ResourceError::PaletteMissing)
    }

    // fn get_tile(&self, pixels: [u8; 16]) -> Result<TileSelector, ResourceError> {
    // }

    fn const_alloc(&'a mut self, data: &'a [u8], label: &str) -> Result<Addr, ()> {
        let len = data.len();
        self.consts.push((self.next_const, Box::new(data)));
        let addr = self.next_const;
        self.next_const += len as u16;
        self.labels.insert(label.to_owned(), addr);

        Ok(addr)
    }

    fn push(&mut self, block: Block) {
        self.output.push(block);
    }
}

fn main() {
    let mut sys = Cgb::new();
    let bg = Tile::flat(Color::BLUE);
    let colors = [Color::BLACK, Color::RED | Color::GREEN, Color::GREEN, Color::BLUE];
    let smiley = Tile::try_from_bytes(colors, [
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 1, 1, 1, 1, 1, 1, 0],
        [0, 1, 0, 1, 1, 0, 1, 0],
        [0, 1, 1, 1, 1, 1, 1, 0],
        [0, 1, 0, 1, 1, 0, 1, 0],
        [0, 1, 0, 0, 0, 0, 1, 0],
        [0, 1, 1, 1, 1, 1, 1, 0],
        [0, 0, 0, 0, 0, 0, 0, 0]
    ]).expect("Someone did a silly >:#");


    sys.set_palette(CgbPalette::PaletteZero, colors);
    sys.write_tile_data(TiledataSelector::Tiledata8000, 1, smiley).unwrap();
    sys.write_tile_data(TiledataSelector::Tiledata8000, 0, bg).unwrap();
    sys.set_tilemap(TilemapSelector::Tilemap9800, |x, y| {
        if (x > 8 && x < 12 && y > 8 && y < 12)
        || (y == 10 && (x == 8 || x == 12))
        || (x == 10 && (y == 8 || y == 12)) {
            1
        } else {
            0
        }
    });

    let mut file = File::create("out.gb").unwrap();

    // set CGB mode
    file.seek_write(&[0x80], 0x143).unwrap();

    // jump to main code
    let trampoline: Vec<u8> = Instruction::Jp(0x150).into();
    file.seek_write(&trampoline, 0x100).unwrap();

    let output: Vec<u8> = sys.output.iter().flat_map(|block| { let out: Vec<u8> = block.into(); out }).collect::<Vec<u8>>();
    file.seek_write(&output, 0x150).unwrap();

    let end: Vec<u8> = Instruction::Jr(Condition::Always, -2).into();
    file.write(&end).unwrap();
}

#[cfg(test)]
mod tests {
    use crate::{Color, PaletteColor, Tile, TileRow};

    #[test]
    fn tile_from_u8() {
        let colors = [Color::BLACK, Color::RED, Color::GREEN, Color::BLUE];
        let smiley = Tile::try_from_bytes(colors, [
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 1, 1, 1, 1, 1, 1, 0],
            [0, 1, 0, 1, 1, 0, 1, 0],
            [0, 1, 1, 1, 1, 1, 1, 0],
            [0, 1, 0, 1, 1, 0, 1, 0],
            [0, 1, 0, 0, 0, 0, 1, 0],
            [0, 1, 1, 1, 1, 1, 1, 0],
            [0, 0, 0, 0, 0, 0, 0, 0]
        ]).expect("Someone did a silly >:#");

        let target = Tile { colors, pixel_data: [
            TileRow { pixel_data: (0x00, 0x00) },
            TileRow { pixel_data: (0x7e, 0x00) },
            TileRow { pixel_data: (0x5a, 0x00) },
            TileRow { pixel_data: (0x7e, 0x00) },
            TileRow { pixel_data: (0x5a, 0x00) },
            TileRow { pixel_data: (0x42, 0x00) },
            TileRow { pixel_data: (0x7e, 0x00) },
            TileRow { pixel_data: (0x00, 0x00) }
        ]};

        assert_eq!(smiley, target);
    }

    #[test]
    fn tile_from_colors() {
        let colors = [Color::BLACK, Color::RED, Color::GREEN, Color::BLUE];
        let smiley = {
            Tile::new(colors, [
                [ color!(0), color!(0), color!(0), color!(0), color!(0), color!(0), color!(0), color!(0) ],
                [ color!(0), color!(1), color!(1), color!(1), color!(1), color!(1), color!(1), color!(0) ],
                [ color!(0), color!(1), color!(0), color!(1), color!(1), color!(0), color!(1), color!(0) ],
                [ color!(0), color!(1), color!(1), color!(1), color!(1), color!(1), color!(1), color!(0) ],
                [ color!(0), color!(1), color!(0), color!(1), color!(1), color!(0), color!(1), color!(0) ],
                [ color!(0), color!(1), color!(0), color!(0), color!(0), color!(0), color!(1), color!(0) ],
                [ color!(0), color!(1), color!(1), color!(1), color!(1), color!(1), color!(1), color!(0) ],
                [ color!(0), color!(0), color!(0), color!(0), color!(0), color!(0), color!(0), color!(0) ]
            ])
        };

        let target = Tile { colors, pixel_data: [
            TileRow { pixel_data: (0x00, 0x00) },
            TileRow { pixel_data: (0x7e, 0x00) },
            TileRow { pixel_data: (0x5a, 0x00) },
            TileRow { pixel_data: (0x7e, 0x00) },
            TileRow { pixel_data: (0x5a, 0x00) },
            TileRow { pixel_data: (0x42, 0x00) },
            TileRow { pixel_data: (0x7e, 0x00) },
            TileRow { pixel_data: (0x00, 0x00) }
        ]};

        assert_eq!(smiley, target);
    }

    #[test]
    fn pandocs_example() {
        let colors = [Color::BLACK, Color::DARK_GREY, Color::LIGHT_GREY, Color::WHITE];
        let tile = Tile::try_from_bytes(colors, [
            [0, 2, 3, 3, 3, 3, 2, 0],
            [0, 3, 0, 0, 0, 0, 3, 0],
            [0, 3, 0, 0, 0, 0, 3, 0],
            [0, 3, 0, 0, 0, 0, 3, 0],
            [0, 3, 1, 3, 3, 3, 3, 0],
            [0, 1, 1, 1, 3, 1, 3, 0],
            [0, 3, 1, 3, 1, 3, 2, 0],
            [0, 2, 3, 3, 3, 2, 0, 0]
        ]).expect("Someone did a silly >:#");

        let target_bytes = [
            0x3c, 0x7e, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42,
            0x7e, 0x5e, 0x7e, 0x0a, 0x7c, 0x56, 0x38, 0x7c
        ];

        assert_eq!(tile.as_bytes(), target_bytes);
    }
}
