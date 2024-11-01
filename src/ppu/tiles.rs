use super::{ConversionError, palettes::PaletteColor};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Tile {
    // pub colors: [Color; 4],
    pub pixel_data: [TileRow; 8],
}

impl Tile {
    pub fn new(pixels: [[PaletteColor; 8]; 8]) -> Self {
        // `pixels` has 8 top level elements so this will always result in an 8-element collection
        let rows: [TileRow; 8] = pixels.into_iter().map(|row| row.into()).take(8).collect::<Vec<TileRow>>().try_into().unwrap();

        Self {
            pixel_data: rows,
        }
    }

    pub fn try_from_bytes(pixels: [[u8; 8]; 8]) -> Result<Self, ConversionError> {
        let mut checked = [[PaletteColor::default(); 8]; 8];

        for (y, row) in pixels.iter().enumerate() {
            for (x, &px) in row.iter().enumerate() {
                checked[y][x] = px.try_into()?;
            }
        }

        Ok(Self::new(checked))
    }

    pub fn flat(color: PaletteColor) -> Self {
        let row = TileRow::flat(color);
        let pixel_data = [
            row, row, row, row,
            row, row, row, row
        ];

        Self { pixel_data }
    }

    pub fn as_bytes(&self) -> [u8; 16] {
        self.pixel_data.into_iter().flat_map(|row| [row.pixel_data.0, row.pixel_data.1]).collect::<Vec<u8>>().try_into().unwrap()
    }
}

pub type TileIdx = u8;
