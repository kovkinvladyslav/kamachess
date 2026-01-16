use anyhow::{anyhow, Result};
use chess::{Board, Color, Piece, Square};
use image::{ImageBuffer, Rgba};
use std::str::FromStr;

use super::glyphs::{glyph_for_file, glyph_for_rank, piece_pattern};

const SQUARE_SIZE: u32 = 64;
const COORD_MARGIN: u32 = 20;
const BOARD_SIZE: u32 = SQUARE_SIZE * 8 + COORD_MARGIN * 2;

const LIGHT_SQUARE: Rgba<u8> = Rgba([240, 217, 181, 255]);
const DARK_SQUARE: Rgba<u8> = Rgba([181, 136, 99, 255]);
const COORD_BORDER: Rgba<u8> = Rgba([101, 76, 59, 255]);

pub fn render_board_png(board: &Board) -> Result<Vec<u8>> {
    let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_pixel(BOARD_SIZE, BOARD_SIZE, COORD_BORDER);

    draw_board_squares(&mut img);
    draw_coordinates(&mut img);
    draw_pieces(board, &mut img)?;

    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )?;

    Ok(bytes)
}

fn draw_board_squares(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
    let origin_x = COORD_MARGIN;
    let origin_y = COORD_MARGIN;
    for rank in 0..8 {
        for file in 0..8 {
            let x0 = origin_x + file * SQUARE_SIZE;
            let y0 = origin_y + rank * SQUARE_SIZE;
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

fn draw_coordinates(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) {
    let scale: i32 = 2;
    let file_glyph_w: i32 = 5 * scale;
    let file_glyph_h: i32 = 9 * scale;
    let rank_glyph_w: i32 = 7 * scale;
    let rank_glyph_h: i32 = 7 * scale;
    let pad: i32 = 1;
    let origin_x = COORD_MARGIN as i32;
    let origin_y = COORD_MARGIN as i32;
    let margin = COORD_MARGIN as i32;
    let board_span = (SQUARE_SIZE * 8) as i32;
    let label_color = Rgba([220, 200, 180, 255]);

    for rank in 0..8 {
        for file in 0..8 {
            let x0 = origin_x + (file * SQUARE_SIZE) as i32;
            let y0 = origin_y + (rank * SQUARE_SIZE) as i32;

            if rank == 0 {
                let letter = (b'a' + file as u8) as char;
                let glyph = glyph_for_file(letter);
                let x = x0 + (SQUARE_SIZE as i32 - file_glyph_w) / 2;
                let y = (margin - file_glyph_h) / 2 + pad;
                draw_glyph_file(img, x, y, label_color, &glyph, scale);
            }
            if rank == 7 {
                let letter = (b'a' + file as u8) as char;
                let glyph = glyph_for_file(letter);
                let x = x0 + (SQUARE_SIZE as i32 - file_glyph_w) / 2;
                let y = origin_y + board_span + (margin - file_glyph_h) / 2 - pad;
                draw_glyph_file(img, x, y, label_color, &glyph, scale);
            }

            if file == 0 {
                let number = (8 - rank) as u8;
                let glyph = glyph_for_rank(number);
                let x = (margin - rank_glyph_w) / 2 + pad;
                let y = y0 + (SQUARE_SIZE as i32 - rank_glyph_h) / 2;
                draw_glyph_rank(img, x, y, label_color, &glyph, scale);
            }
            if file == 7 {
                let number = (8 - rank) as u8;
                let glyph = glyph_for_rank(number);
                let x = origin_x + board_span + (margin - rank_glyph_w) / 2 - pad;
                let y = y0 + (SQUARE_SIZE as i32 - rank_glyph_h) / 2;
                draw_glyph_rank(img, x, y, label_color, &glyph, scale);
            }
        }
    }
}

fn draw_glyph_file(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: i32,
    y: i32,
    color: Rgba<u8>,
    glyph: &[u8; 9],
    scale: i32,
) {
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..5 {
            if (bits >> (4 - col)) & 1 == 1 {
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = x + col * scale + dx;
                        let py = y + (row as i32 * scale) + dy;
                        if px >= 0 && py >= 0 && px < img.width() as i32 && py < img.height() as i32
                        {
                            img.put_pixel(px as u32, py as u32, color);
                        }
                    }
                }
            }
        }
    }
}

fn draw_glyph_rank(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: i32,
    y: i32,
    color: Rgba<u8>,
    glyph: &[u8; 7],
    scale: i32,
) {
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..7 {
            if (bits >> (6 - col)) & 1 == 1 {
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = x + col * scale + dx;
                        let py = y + (row as i32 * scale) + dy;
                        if px >= 0 && py >= 0 && px < img.width() as i32 && py < img.height() as i32
                        {
                            img.put_pixel(px as u32, py as u32, color);
                        }
                    }
                }
            }
        }
    }
}

fn draw_pieces(board: &Board, img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<()> {
    for rank in 0..8 {
        for file in 0..8 {
            let square = square_from_coords(file, 7 - rank)?;
            if let Some(piece) = board.piece_on(square) {
                let color = board.color_on(square).unwrap_or(Color::White);

                let x = (COORD_MARGIN + file * SQUARE_SIZE + 8) as i32;
                let y = (COORD_MARGIN + rank * SQUARE_SIZE + 8) as i32;

                draw_piece(img, piece, x + 2, y + 2, Rgba([60, 60, 60, 200]));

                let piece_color = if color == Color::White {
                    Rgba([255, 255, 255, 255])
                } else {
                    Rgba([40, 40, 40, 255])
                };
                draw_piece(img, piece, x, y, piece_color);

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
                        let px = x + col * scale + dx;
                        let py = y + (row as i32 * scale) + dy;
                        if px >= 0 && py >= 0 && px < img.width() as i32 && py < img.height() as i32
                        {
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
            let left = col > 0 && (bits >> (15 - col + 1)) & 1 == 0;
            let right = col < 15 && (bits >> (15 - col - 1)) & 1 == 0;
            let up = row > 0 && (pattern[row - 1] >> (15 - col)) & 1 == 0;
            let down = row < 15 && (pattern[row + 1] >> (15 - col)) & 1 == 0;

            if left || right || up || down {
                for dy in 0..scale {
                    for dx in 0..scale {
                        let px = x + col * scale + dx;
                        let py = y + (row as i32 * scale) + dy;
                        if px >= 0 && py >= 0 && px < img.width() as i32 && py < img.height() as i32
                        {
                            img.put_pixel(px as u32, py as u32, color);
                        }
                    }
                }
            }
        }
    }
}
