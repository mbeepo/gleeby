use palettes::PaletteColor;
use tiles::TileRow;

pub mod palettes;
pub mod tiles;
pub mod objects;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConversionError {
    InvalidPaletteColor,
    InvalidPaletteIndex,
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