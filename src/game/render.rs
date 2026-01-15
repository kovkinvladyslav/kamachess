use anyhow::{anyhow, Result};
use chess::{Board, Color, Piece, Square};
use image::{ImageBuffer, Rgba};
use std::str::FromStr;

const SQUARE_SIZE: u32 = 64;
const BOARD_SIZE: u32 = SQUARE_SIZE * 8;

const LIGHT_SQUARE: Rgba<u8> = Rgba([240, 217, 181, 255]);
const DARK_SQUARE: Rgba<u8> = Rgba([181, 136, 99, 255]);

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
                
                let x = (file * SQUARE_SIZE + 8) as i32;
                let y = (rank * SQUARE_SIZE + 8) as i32;
                
                // Draw shadow
                draw_piece(img, piece, x + 2, y + 2, Rgba([60, 60, 60, 200]));
                
                // Draw piece
                let piece_color = if color == Color::White {
                    Rgba([255, 255, 255, 255])
                } else {
                    Rgba([40, 40, 40, 255])
                };
                draw_piece(img, piece, x, y, piece_color);
                
                // Draw outline for white pieces (better visibility)
                if color == Color::White {
                    draw_piece_outline(img, piece, x, y, Rgba([60, 60, 60, 255]));
                }
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

fn draw_piece(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    piece: Piece,
    x: i32,
    y: i32,
    color: Rgba<u8>,
) {
    let pattern = piece_pattern(piece);
    let scale = 3;
    for (row, bits) in pattern.iter().enumerate() {
        for col in 0..16 {
            if (bits >> (15 - col)) & 1 == 1 {
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = x + (col as i32 * scale) + dx;
                        let py = y + (row as i32 * scale) + dy;
                        if px >= 0 && py >= 0 && px < img.width() as i32 && py < img.height() as i32 {
                            img.put_pixel(px as u32, py as u32, color);
                        }
                    }
                }
            }
        }
    }
}

fn draw_piece_outline(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    piece: Piece,
    x: i32,
    y: i32,
    color: Rgba<u8>,
) {
    let pattern = piece_pattern(piece);
    let scale = 3;
    for (row, bits) in pattern.iter().enumerate() {
        for col in 0..16 {
            let is_filled = (bits >> (15 - col)) & 1 == 1;
            if !is_filled {
                continue;
            }
            // Check if this is an edge pixel
            let left = col > 0 && (bits >> (15 - col + 1)) & 1 == 0;
            let right = col < 15 && (bits >> (15 - col - 1)) & 1 == 0;
            let up = row > 0 && (pattern[row - 1] >> (15 - col)) & 1 == 0;
            let down = row < 15 && (pattern[row + 1] >> (15 - col)) & 1 == 0;
            
            if left || right || up || down {
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = x + (col as i32 * scale) + dx;
                        let py = y + (row as i32 * scale) + dy;
                        if px >= 0 && py >= 0 && px < img.width() as i32 && py < img.height() as i32 {
                            img.put_pixel(px as u32, py as u32, color);
                        }
                    }
                }
            }
        }
    }
}

// 16x16 bitmap patterns for chess pieces (recognizable shapes)
fn piece_pattern(piece: Piece) -> [u16; 16] {
    match piece {
        // King - crown shape with cross on top
        Piece::King => [
            0b0000000100000000,
            0b0000001110000000,
            0b0000000100000000,
            0b0000011111000000,
            0b0000111111100000,
            0b0001111111110000,
            0b0001110001110000,
            0b0001100000110000,
            0b0001100000110000,
            0b0001110001110000,
            0b0000111111100000,
            0b0000111111100000,
            0b0001111111110000,
            0b0011111111111000,
            0b0011111111111000,
            0b0000000000000000,
        ],
        // Queen - crown with points
        Piece::Queen => [
            0b0001000100010000,
            0b0001100100011000,
            0b0001110101110000,
            0b0000111111100000,
            0b0000011111000000,
            0b0000011111000000,
            0b0000111111100000,
            0b0000111111100000,
            0b0001111111110000,
            0b0001111111110000,
            0b0001111111110000,
            0b0011111111111000,
            0b0011111111111000,
            0b0111111111111100,
            0b0111111111111100,
            0b0000000000000000,
        ],
        // Rook - castle tower
        Piece::Rook => [
            0b0000000000000000,
            0b0011011011011000,
            0b0011111111111000,
            0b0001111111110000,
            0b0000111111100000,
            0b0000111111100000,
            0b0000111111100000,
            0b0000111111100000,
            0b0000111111100000,
            0b0000111111100000,
            0b0001111111110000,
            0b0001111111110000,
            0b0011111111111000,
            0b0011111111111000,
            0b0000000000000000,
            0b0000000000000000,
        ],
        // Bishop - mitre shape
        Piece::Bishop => [
            0b0000001100000000,
            0b0000011110000000,
            0b0000111111000000,
            0b0001111111100000,
            0b0001111011100000,
            0b0001111111100000,
            0b0000111111000000,
            0b0000011110000000,
            0b0000011110000000,
            0b0000111111000000,
            0b0001111111100000,
            0b0011111111110000,
            0b0011111111110000,
            0b0111111111111000,
            0b0000000000000000,
            0b0000000000000000,
        ],
        // Knight - horse head
        Piece::Knight => [
            0b0000011110000000,
            0b0000111111000000,
            0b0001111111100000,
            0b0011111111110000,
            0b0111111111110000,
            0b0111111111100000,
            0b0011111111100000,
            0b0001111111100000,
            0b0000111111100000,
            0b0000111111000000,
            0b0001111111100000,
            0b0001111111110000,
            0b0011111111110000,
            0b0111111111111000,
            0b0111111111111000,
            0b0000000000000000,
        ],
        // Pawn - simple round top
        Piece::Pawn => [
            0b0000000000000000,
            0b0000011110000000,
            0b0000111111000000,
            0b0000111111000000,
            0b0000111111000000,
            0b0000011110000000,
            0b0000001100000000,
            0b0000011110000000,
            0b0000111111000000,
            0b0001111111100000,
            0b0001111111100000,
            0b0011111111110000,
            0b0011111111110000,
            0b0111111111111000,
            0b0000000000000000,
            0b0000000000000000,
        ],
    }
}
