SECTION "Header", ROM0[$100]
    
    nop
	jr EntryPoint

	; Make sure to allocate some space for the header, so no important
	; code gets put there and later overwritten by RGBFIX.
	; RGBFIX is designed to operate over a zero-filled header, so make
	; sure to put zeros regardless of the padding value. (This feature
	; was introduced in RGBDS 0.4.0, but the -MG etc flags were also
	; introduced in that version.)
	ds $150 - @, 0

EntryPoint:
    ld a, %00011011
    ldh [rBGP], a
    
    ld a, $80 ; start at palette 0 color 0 and autoincrement
    ldh [rBCPS], a
    
    ld hl, BgPalette
    ld b, BgPaletteEnd - BgPalette
:   ld a, [hl+]
    ldh [rBCPD], a
    dec b
    jr nz, :-


SECTION "Palettes", ROM0

BgPalette:
    db $c4, $34
    db $ff, $ff
    db $ff, $ff
    db $ff, $ff
BgPaletteEnd: