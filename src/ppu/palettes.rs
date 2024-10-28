use std::ops::BitOr;

use super::ConversionError;

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CgbPalette {
    #[default]
    _0,
    _1, _2,_3,
    _4, _5, _6, _7
}

impl From<CgbPalette> for u8 {
    fn from(value: CgbPalette) -> Self {
        value as u8
    }
}

impl From<CgbPalette> for usize {
    fn from(value: CgbPalette) -> Self {
        let idx: u8 = value.into();
        idx as usize
    }
}

impl TryFrom<u8> for CgbPalette {
    type Error = ConversionError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(CgbPalette::_0),
            1 => Ok(CgbPalette::_1),
            2 => Ok(CgbPalette::_2),
            3 => Ok(CgbPalette::_3),
            4 => Ok(CgbPalette::_4),
            5 => Ok(CgbPalette::_5),
            6 => Ok(CgbPalette::_6),
            7 => Ok(CgbPalette::_7),
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
    _0,
    _1, _2, _3
}

impl From<PaletteColor> for (u8, u8) {
    fn from(value: PaletteColor) -> Self {
        match value {
            PaletteColor::_0 => (0, 0),
            PaletteColor::_1 => (1, 0),
            PaletteColor::_2 => (0, 1),
            PaletteColor::_3 => (1, 1),
        }
    }
}

impl TryFrom<u8> for PaletteColor {
    type Error = ConversionError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(PaletteColor::_0),
            1 => Ok(PaletteColor::_1),
            2 => Ok(PaletteColor::_2),
            3 => Ok(PaletteColor::_3),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color(pub u16);

impl Color {
    pub const BLACK: Self = Self(0);
    pub const DARK_GREY: Self = Self(0b01010_01010_01010_0);
    pub const LIGHT_GREY: Self = Self(0b10101_10101_10101_0);
    pub const WHITE: Self = Self(u16::MAX);
    pub const RED: Self = Self(u16::from_le_bytes([0b00000000, 0b00011111]));
    pub const GREEN: Self = Self(u16::from_le_bytes([0b00000011, 0b11100000]));
    pub const BLUE: Self = Self(u16::from_le_bytes([0b01111100, 0b00000000]));
}

impl BitOr<Color> for Color {
    type Output = Color;
    
    fn bitor(self, rhs: Color) -> Self::Output {
        Color(self.0 | rhs.0)
    }
}