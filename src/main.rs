use std::collections::HashMap;

use bitflags::bitflags;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConversionError {
    InvalidPaletteColor,
    InvalidPaletteIndex,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CgbPalette {
    #[default]
    Palette0,
    Palette1,
    Palette2,
    Palette3,
    Palette4,
    Palette5,
    Palette6,
    Palette7,
}

impl From<CgbPalette> for u8 {
    fn from(value: CgbPalette) -> Self {
        use CgbPalette::*;
        match value {
            Palette0 => 0,
            Palette1 => 1,
            Palette2 => 2,
            Palette3 => 3,
            Palette4 => 4,
            Palette5 => 5,
            Palette6 => 6,
            Palette7 => 7,
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
            0 => Ok(Palette0),
            1 => Ok(Palette1),
            2 => Ok(Palette2),
            3 => Ok(Palette3),
            4 => Ok(Palette4),
            5 => Ok(Palette5),
            6 => Ok(Palette6),
            7 => Ok(Palette7),
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
    Color0,
    Color1,
    Color2,
    Color3,
}

impl From<PaletteColor> for u16 {
    fn from(value: PaletteColor) -> Self {
        match value {
            PaletteColor::Color0 => 0x0000,
            PaletteColor::Color1 => 0x0001,
            PaletteColor::Color2 => 0x0100,
            PaletteColor::Color3 => 0x0101,
        }
    }
}

impl TryFrom<u8> for PaletteColor {
    type Error = ConversionError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PaletteColor::Color0),
            1 => Ok(PaletteColor::Color1),
            2 => Ok(PaletteColor::Color2),
            3 => Ok(PaletteColor::Color3),
            _ => Err(ConversionError::InvalidPaletteColor)
        }
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
    const BLUE: Self = Self(0b00000_00000_11111_0);
    const YELLOW: Self = Self(0b11111_11111_00000_0);
    const WHITE: Self = Self(u16::MAX);
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tile {
    pub colors: [Color; 4],
    pub pixels: [u8; 16],
}

impl Tile {
    pub fn new(colors: [Color; 4], pixels: [[PaletteColor; 8]; 8]) -> Self {
        let pixels: Vec<u8> = pixels.iter().map(|row| {
            let mut out = 0u16;

            for (i, &px) in row.iter().enumerate() {
                let px: u16 = px.into();
                out |= px << i;
            }

            out.to_le_bytes()
        }).flatten().collect();

        let pixels = [
            pixels[0], pixels[1], pixels[2], pixels[3],
            pixels[4], pixels[5], pixels[6], pixels[7],
            pixels[8], pixels[9], pixels[10], pixels[11],
            pixels[12], pixels[13], pixels[14], pixels[15],
        ];

        Self {
            colors,
            pixels,
        }
    }

    pub fn new_silly(colors: [Color; 4], pixels: [[u8; 8]; 8]) -> Result<Self, ConversionError> {
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
        let pixels = [0_u8; 16];

        Self { colors, pixels }
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
    Z, NZ,
    C, NC,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Instruction {
    LoadR8Hl(Register),
    LoadHlR8(Register),
    LoadR8Imm(Register, u8),
    LoadR8R8(Register, Register),
    LoadR16Imm(RegisterPair, u16),
    Jr(Condition, u16),
    IncR16(RegisterPair),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TilemapSelector {
    Tilemap9800,
    Tilemap9C00,
}

impl TilemapSelector {
    pub fn from_idx(&self, idx: u8) -> u16 {
        let base = match self {
            Self::Tilemap9800 => 0x9800,
            Self::Tilemap9C00 => 0x9c00,
        };
        base + idx as u16
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
                0x9800 + idx as u16 * 16
            }
            Self::Tiledata9000 => {
                let offset = idx as i8 as i16 * 16;
                0x9c00_u16.wrapping_add(offset as u16)
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
            palettes: [[Color::BLACK; 4]; 8],
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
            LoadR16Imm(HL, R_BCPS),
            LoadR8Imm(A, PaletteSelector::new(true, palette).into()),
            LoadHlR8(A),
            LoadR16Imm(HL, R_BCPD),
        ])));

        for color in colors {
            let color = color.0.to_le_bytes();
            out.push(Block::Basic(BasicBlock::from(vec![
                LoadR8Imm(A, color[0].into()),
                LoadHlR8(A),
                LoadR8Imm(A, color[1].into()),
                LoadHlR8(A),
            ])));
        }

        let idx: usize = palette.into();
        self.palettes[idx] = colors;
        self.output.extend(out);
    }

    pub fn set_tile_data(&mut self, idx: TileSelector, data: Tile) -> Result<(), ResourceError> {
        use Instruction::*;
        use Register::*;
        use RegisterPair::*;

        let addr = idx.tilemap.from_idx(idx.idx);
        let mut out: Vec<Block> = Vec::with_capacity(9);
        
        out.push(Block::Basic(BasicBlock::from(vec![
            LoadR16Imm(HL, addr),
        ])));

        for byte in data.pixels {
            out.push(Block::Basic(BasicBlock::from(vec![
                LoadR8Imm(A, byte),
                LoadHlR8(A),
                IncR16(HL),
            ])));
        }



        self.output.extend(out);
        Ok(())
    }

    pub fn set_tilemap<F>(&mut self, selector: TilemapSelector, setter: F) 
        // where F: Fn(u8, u8) -> Tile
        where F: Fn(u8, u8) -> TileIdx
    {
        for x in 0..32 {
            for y in 0..32 {
                let idx = setter(x, y);
                // let palette = self.get_palette(tile.colors);
                let palette = CgbPalette::Palette0;
                let addr = self.tilemap.from_idx(idx);

            }
        }
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
}

fn main() {
    let mut sys = Cgb::new();
    let bg = Tile::flat(Color::BLUE);
    let colors = [Color::BLACK, Color::YELLOW, Color::BLUE, Color::WHITE];
    let smiley = Tile::new_silly(colors, [
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 1, 1, 1, 1, 1, 1, 0],
        [0, 1, 0, 1, 1, 0, 1, 0],
        [0, 1, 1, 1, 1, 1, 1, 0],
        [0, 1, 0, 1, 1, 0, 1, 0],
        [0, 1, 0, 0, 0, 0, 1, 0],
        [0, 1, 1, 1, 1, 1, 1, 0],
        [0, 0, 0, 0, 0, 0, 0, 0]
    ]).expect("Someone did a silly >:#");

    sys.set_palette(CgbPalette::Palette0, colors);
    sys.set_tile_data(TileSelector { tilemap: sys.tilemap, idx: 1 }, smiley).unwrap();

    dbg!(sys);

    let sprite = Sprite { pos: pos2(0x20, 0x20), tile: 1, attr: ObjAttributes::default() };

    // sys.set_tilemap(TilemapSelector::Tilemap9800, |x, y| {
    //     bg
    // });
}