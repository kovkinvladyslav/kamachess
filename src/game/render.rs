use anyhow::Result;
use chess::{Board, Color, File, Piece, Rank, Square};
use image::{ImageBuffer, Rgba};

use super::cache;
use super::glyphs::{glyph_for_file, glyph_for_rank, piece_pattern};

const SQUARE_SIZE: u32 = 64;
const COORD_MARGIN: u32 = 20;
const BOARD_SIZE: u32 = SQUARE_SIZE * 8 + COORD_MARGIN * 2;

const LIGHT_SQUARE: Rgba<u8> = Rgba([240, 217, 181, 255]);
const DARK_SQUARE: Rgba<u8> = Rgba([181, 136, 99, 255]);
const COORD_BORDER: Rgba<u8> = Rgba([101, 76, 59, 255]);

pub fn render_board_png(board: &Board, flip_board: bool) -> Result<Vec<u8>> {
    cache::get_or_create(board, flip_board, || {
        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_pixel(BOARD_SIZE, BOARD_SIZE, COORD_BORDER);

        draw_board_squares(&mut img);
        draw_coordinates(&mut img, flip_board);
        draw_pieces(board, &mut img, flip_board);

        let mut bytes = Vec::new();
        img.write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )?;

        Ok(bytes)
    })
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

fn draw_coordinates(img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, flip_board: bool) {
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

    let (side1_y, side2_y) =
        calculate_vertical_side_positions(margin, file_glyph_h, origin_y, board_span, pad);
    let (side1_x, side2_x) =
        calculate_horizontal_side_positions(margin, rank_glyph_w, origin_x, board_span, pad);

    draw_labels_on_two_sides(
        img,
        0..8,
        |file| {
            let file_idx = if flip_board { 7 - file } else { file };
            let letter = (b'a' + file_idx as u8) as char;
            glyph_for_file(letter)
        },
        |file| {
            let x =
                origin_x + (file * SQUARE_SIZE) as i32 + (SQUARE_SIZE as i32 - file_glyph_w) / 2;
            ((x, side1_y), (x, side2_y))
        },
        label_color,
        scale,
        |img,
                 x,
                 y,
                 glyph,
                 color,
                 scale| {
            draw_glyph_file(img, x, y, color, glyph, scale);
        },
    );

    draw_labels_on_two_sides(
        img,
        0..8,
        |rank| {
            let rank_num = if flip_board { rank + 1 } else { 8 - rank };
            let number = rank_num as u8;
            glyph_for_rank(number)
        },
        |rank| {
            let y = origin_y + (rank * SQUARE_SIZE) as i32 + (SQUARE_SIZE as i32 - rank_glyph_h) / 2;
            ((side1_x, y), (side2_x, y))
        },
        label_color,
        scale,
        |img, x, y, glyph, color, scale| {
            draw_glyph_rank(img, x, y, color, glyph, scale);
        },
    );
}

fn calculate_vertical_side_positions(margin: i32, glyph_h: i32, origin_y: i32, board_span: i32, pad: i32) -> (i32, i32) {
    let side1 = (margin - glyph_h) / 2 + pad;
    let side2 = origin_y + board_span + (margin - glyph_h) / 2 - pad;
    (side1, side2)
}

fn calculate_horizontal_side_positions(margin: i32, glyph_w: i32, origin_x: i32, board_span: i32, pad: i32) -> (i32, i32) {
    let side1 = (margin - glyph_w) / 2 + pad;
    let side2 = origin_x + board_span + (margin - glyph_w) / 2 - pad;
    (side1, side2)
}

fn draw_labels_on_two_sides<G, F1, F4, const N: usize>(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    indices: impl Iterator<Item = u32>,
    get_glyph: G,
    get_coords: F1,
    color: Rgba<u8>,
    scale: i32,
    draw_fn: F4,
) where
    G: Fn(u32) -> [u8; N],
    F1: Fn(u32) -> ((i32, i32), (i32, i32)),
    F4: Fn(&mut ImageBuffer<Rgba<u8>, Vec<u8>>, i32, i32, &[u8; N], Rgba<u8>, i32),
{
    for idx in indices {
        let glyph = get_glyph(idx);
        let ((x1, y1), (x2, y2)) = get_coords(idx);
        
        draw_fn(img, x1, y1, &glyph, color, scale);
        draw_fn(img, x2, y2, &glyph, color, scale);
    }
}

fn draw_scaled_pixel(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    base_x: i32,
    base_y: i32,
    col: usize,
    row: usize,
    scale: i32,
    color: Rgba<u8>,
) {
    for dy in 0..scale {
        for dx in 0..scale {
            let px = base_x + (col as i32 * scale) + dx;
            let py = base_y + (row as i32 * scale) + dy;
            if px >= 0 && py >= 0 && px < img.width() as i32 && py < img.height() as i32 {
                img.put_pixel(px as u32, py as u32, color);
            }
        }
    }
}

struct GlyphParams {
    width: usize,
    bit_shift: usize,
}

fn draw_glyph(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: i32,
    y: i32,
    color: Rgba<u8>,
    glyph: &[u8],
    params: GlyphParams,
    scale: i32,
) {
    for (row, bits) in glyph.iter().enumerate() {
        for col in 0..params.width {
            if (bits >> (params.bit_shift - col)) & 1 == 1 {
                draw_scaled_pixel(img, x, y, col, row, scale, color);
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
    draw_glyph(img, x, y, color, glyph, GlyphParams { width: 5, bit_shift: 4 }, scale);
}

fn draw_glyph_rank(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    x: i32,
    y: i32,
    color: Rgba<u8>,
    glyph: &[u8; 7],
    scale: i32,
) {
    draw_glyph(img, x, y, color, glyph, GlyphParams { width: 7, bit_shift: 6 }, scale);
}

fn draw_pieces(board: &Board, img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>, flip_board: bool) {
    for rank in 0..8 {
        for file in 0..8 {
            let board_rank = if flip_board { rank } else { 7 - rank };
            let board_file = if flip_board { 7 - file } else { file };
            let square = square_from_coords(board_file, board_rank);
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
}

fn square_from_coords(file: u32, rank: u32) -> Square {
    let f = File::from_index(file as usize);
    let r = Rank::from_index(rank as usize);
    Square::make_square(r, f)
}

fn draw_piece_pattern_pixels(
    img: &mut ImageBuffer<Rgba<u8>, Vec<u8>>,
    pattern: &[u16; 16],
    x: i32,
    y: i32,
    color: Rgba<u8>,
    scale: i32,
    should_draw: impl Fn(usize, usize, &[u16; 16]) -> bool,
) {
    for row in 0..16 {
        for col in 0..16 {
            if should_draw(row, col, pattern) {
                draw_scaled_pixel(img, x, y, col, row, scale, color);
            }
        }
    }
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
    draw_piece_pattern_pixels(img, &pattern, x, y, color, scale, |_row, col, pattern| {
        (pattern[_row] >> (15 - col)) & 1 == 1
    });
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
    draw_piece_pattern_pixels(img, &pattern, x, y, color, scale, |row, col, pattern| {
        let is_filled = (pattern[row] >> (15 - col)) & 1 == 1;
        if !is_filled {
            return false;
        }
        let left = col > 0 && (pattern[row] >> (15 - col + 1)) & 1 == 0;
        let right = col < 15 && (pattern[row] >> (15 - col - 1)) & 1 == 0;
        let up = row > 0 && (pattern[row - 1] >> (15 - col)) & 1 == 0;
        let down = row < 15 && (pattern[row + 1] >> (15 - col)) & 1 == 0;
        left || right || up || down
    });
}
