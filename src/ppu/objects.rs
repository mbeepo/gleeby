use bitflags::bitflags;

use super::{palettes::CgbPalette, tiles::TileIdx};

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
pub struct Sprite {
    pub pos: Pos2,
    pub tile: TileIdx,
    pub attr: ObjAttributes,
}

impl Sprite {
    pub fn as_bytes(&self) -> [u8; 4] {
        [self.pos.y, self.pos.x, self.tile as u8, self.attr.into()]
    }
}


#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpriteIdx {
     _0,  _1,  _2,  _3,  _4,  _5,  _6,  _7,  _8,  _9,
    _10, _11, _12, _13, _14, _15, _16, _17, _18, _19,
    _20, _21, _22, _23, _24, _25, _26, _27, _28, _29,
    _30, _31, _32, _33, _34, _35, _36, _37, _38, _39,
}