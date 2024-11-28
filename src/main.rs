use std::fs::File;

use gleeby::{codegen::{allocator::GpRegisters, variables::Variabler, Assembler, MacroAssembler}, cpu::instructions::Condition, ppu::{palettes::{CgbPalette, Color, PaletteColor}, tiles::{Tile, Tilemap}, TiledataSelector, TilemapSelector}, Cgb};

fn main() {
    let mut sys = Cgb::new();
    let colors = [Color::BLACK, Color::RED | Color::GREEN, Color::GREEN, Color::BLUE];
    let flat = Tile::flat(PaletteColor::_3);
    let smiley = Tile::try_from_bytes([
        [0, 0, 0, 0, 0, 0, 0, 0],
        [0, 1, 1, 1, 1, 1, 1, 0],
        [0, 1, 0, 1, 1, 0, 1, 0],
        [0, 1, 1, 1, 1, 1, 1, 0],
        [0, 1, 0, 1, 1, 0, 1, 0],
        [0, 1, 0, 0, 0, 0, 1, 0],
        [0, 1, 1, 1, 1, 1, 1, 0],
        [0, 0, 0, 0, 0, 0, 0, 0]
    ]).expect("Someone did a silly >:#");

    let regs = GpRegisters {
        a: None,
        b: None,
        c: None,
        d: None,
        e: None,
        h: None,
        l: None,
        bleh: None,
    };

    assert_eq!(sys.allocator().borrow().registers.borrow().clone(), regs);
    dbg!(&sys.allocator().borrow().registers);

    sys.disable_lcd_now();
    assert_eq!(sys.allocator().borrow().registers.borrow().clone(), regs);
    dbg!(&sys.allocator().borrow().registers);

    sys.set_palette(CgbPalette::_0, colors).unwrap();
    assert_eq!(sys.allocator().borrow().registers.borrow().clone(), regs);
    dbg!(&sys.allocator().borrow().registers);

    sys.write_tile_data(TiledataSelector::Tiledata8000, 1, &smiley).unwrap();
    assert_eq!(sys.allocator().borrow().registers.borrow().clone(), regs);
    dbg!(&sys.allocator().borrow().registers);
    
    sys.write_tile_data(TiledataSelector::Tiledata8000, 2, &flat).unwrap();
    assert_eq!(sys.allocator().borrow().registers.borrow().clone(), regs);
    dbg!(&sys.allocator().borrow().registers);

    let setter = |x, y| {
        if (x > 8 && x < 12 && y > 8 && y < 12)
        || (y == 10 && (x == 8 || x == 12))
        || (x == 10 && (y == 8 || y == 12)) {
            1
        } else if (x + y) % 2 == 0 {
            2
        } else {
            0
        }
    };
    let mut tilemap: [u8; 32 * 32] = [0; 32 * 32];

    for x in 0..32 {
        for y in 0..32 {
            tilemap[(y * 32) + x] = setter(x, y);
        }
    }

    dbg!(&sys.allocator().borrow().registers);
    sys.set_tilemap(TilemapSelector::Tilemap9800, Tilemap::from(tilemap)).unwrap();
    sys.set_tile(TilemapSelector::Tilemap9800, 0, 1).unwrap();
    sys.set_tile(TilemapSelector::Tilemap9800, 8, 1).unwrap();
    assert_eq!(sys.allocator().borrow().registers.borrow().clone(), regs);
    dbg!(&sys.allocator().borrow().registers);


    sys.enable_lcd_now();
    sys.jr(Condition::Always, -2);

    let mut file = File::create("out.gb").unwrap();

    sys.save(&mut file).unwrap();
}

#[cfg(test)]
mod tests {
    use gleeby::ppu::{palettes::PaletteColor, tiles::{Tile, TileRow}};

    #[test]
    fn tile_from_u8() {
        let smiley = Tile::try_from_bytes([
            [0, 0, 0, 0, 0, 0, 0, 0],
            [0, 1, 1, 1, 1, 1, 1, 0],
            [0, 1, 0, 1, 1, 0, 1, 0],
            [0, 1, 1, 1, 1, 1, 1, 0],
            [0, 1, 0, 1, 1, 0, 1, 0],
            [0, 1, 0, 0, 0, 0, 1, 0],
            [0, 1, 1, 1, 1, 1, 1, 0],
            [0, 0, 0, 0, 0, 0, 0, 0]
        ]).expect("Someone did a silly >:#");

        let target = Tile { pixel_data: [
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
        let smiley = {
            use PaletteColor::*;
            Tile::new([
                [ _0, _0, _0, _0, _0, _0, _0, _0 ],
                [ _0, _1, _1, _1, _1, _1, _1, _0 ],
                [ _0, _1, _0, _1, _1, _0, _1, _0 ],
                [ _0, _1, _1, _1, _1, _1, _1, _0 ],
                [ _0, _1, _0, _1, _1, _0, _1, _0 ],
                [ _0, _1, _0, _0, _0, _0, _1, _0 ],
                [ _0, _1, _1, _1, _1, _1, _1, _0 ],
                [ _0, _0, _0, _0, _0, _0, _0, _0 ]
            ])
        };

        let target = Tile { pixel_data: [
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
        let tile = Tile::try_from_bytes([
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
