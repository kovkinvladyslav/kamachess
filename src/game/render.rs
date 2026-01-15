use anyhow::{anyhow, Result};
use chess::{Board, Color, Piece, Square};
use image::{ImageBuffer, Rgba};
use std::str::FromStr;

const SQUARE_SIZE: u32 = 64;
const BOARD_SIZE: u32 = SQUARE_SIZE * 8;

const LIGHT_SQUARE: Rgba<u8> = Rgba([240, 217, 181, 255]);
const DARK_SQUARE: Rgba<u8> = Rgba([181, 136, 99, 255]);
const WHITE_PIECE: Rgba<u8> = Rgba([250, 250, 250, 255]);
const BLACK_PIECE: Rgba<u8> = Rgba([20, 20, 20, 255]);
const WHITE_SHADOW: Rgba<u8> = Rgba([20, 20, 20, 255]);
const BLACK_SHADOW: Rgba<u8> = Rgba([250, 250, 250, 255]);

pub fn render_board_png(board: &Board) -> Result<Vec<u8>> {
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(BOARD_SIZE, BOARD_SIZE, Rgba([255, 255, 255, 255]));

    draw_board_squares(&mut img);
    draw_pieces(board, &mut img)?;

    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )?;
    
    Ok(bytes)
}

fn draw_board_squares(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
    for rank in 0..8 {
        for file in 0..8 {
            let x0 = file * SQUARE_SIZE;
            let y0 = rank * SQUARE_SIZE;
            let is_light = (rank + file) % 2 == 0;
            let color = if is_light { LIGHT_SQUARE } else { DARK_SQUARE };
            
            for y in y0..(y0 + SQUARE_SIZE) {
                for x in x0..(x0 + SQUARE_SIZE) {
                    img.put_pixel(x, y, color);
                }
            }
        }
    }
}

fn draw_pieces(
    board: &Board,
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
) -> Result<()> {
    for rank in 0..8 {
        for file in 0..8 {
            let square = square_from_coords(file, 7 - rank)?;
            if let Some(piece) = board.piece_on(square) {
                let color = board.color_on(square).unwrap_or(Color::White);
                let glyph = piece_glyph(piece);
                
                let (draw_color, shadow_color) = if color == Color::White {
                    (WHITE_PIECE, WHITE_SHADOW)
                } else {
                    (BLACK_PIECE, BLACK_SHADOW)
                };
                
                let x = (file * SQUARE_SIZE + 18) as i32;
                let y = (rank * SQUARE_SIZE + 14) as i32;
                
                draw_char(img, glyph, x + 2, y + 2, 4, shadow_color);
                draw_char(img, glyph, x, y, 4, draw_color);
            }
        }
    }
    Ok(())
}

fn square_from_coords(file: u32, rank: u32) -> Result<Square> {
    let file_char = (b'a' + file as u8) as char;
    let rank_char = (b'1' + rank as u8) as char;
    let coord = format!("{}{}", file_char, rank_char);
    Square::from_str(&coord).map_err(|e| anyhow!("Invalid coordinates: {}", e))
}

fn piece_glyph(piece: Piece) -> char {
    match piece {
        Piece::King => 'K',
        Piece::Queen => 'Q',
        Piece::Rook => 'R',
        Piece::Bishop => 'B',
        Piece::Knight => 'N',
        Piece::Pawn => 'P',
    }
}

fn draw_char(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    ch: char,
    x: i32,
    y: i32,
    scale: i32,
    color: Rgba<u8>,
) {
    let pattern = font_pattern(ch);
    for (row, bits) in pattern.iter().enumerate() {
        for col in 0..5 {
            if (bits >> (4 - col)) & 1 == 1 {
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = x + (col as i32 * scale) + dx;
                        let py = y + (row as i32 * scale) + dy;
                        if px >= 0
                            && py >= 0
                            && px < img.width() as i32
                            && py < img.height() as i32
                        {
                            img.put_pixel(px as u32, py as u32, color);
                        }
                    }
                }
            }
        }
    }
}

fn font_pattern(ch: char) -> [u8; 7] {
    match ch {
        'K' => [0b10001, 0b10010, 0b10100, 0b11000, 0b10100, 0b10010, 0b10001],
        'Q' => [0b01110, 0b10001, 0b10001, 0b10001, 0b10101, 0b10010, 0b01101],
        'R' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10100, 0b10010, 0b10001],
        'B' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10001, 0b10001, 0b11110],
        'N' => [0b10001, 0b11001, 0b10101, 0b10011, 0b10001, 0b10001, 0b10001],
        'P' => [0b11110, 0b10001, 0b10001, 0b11110, 0b10000, 0b10000, 0b10000],
        _ => [0, 0, 0, 0, 0, 0, 0],
    }
}
