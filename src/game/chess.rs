use anyhow::{anyhow, Result};
use chess::{Board, ChessMove, Color, MoveGen, Piece, Square, Rank, File};
use std::str::FromStr;
use crate::models::DbUser;

pub fn parse_move(board: &Board, input: &str) -> Result<ChessMove> {
    let trimmed = input.trim();
    
    // Try SAN notation first (e.g., Nf6, nf6, Qxd5, O-O, e4)
    // SAN parsing is case-insensitive for piece letters
    match parse_san(board, trimmed) {
        Ok(mv) => return Ok(mv),
        Err(_) => {
            // If SAN parsing fails, try coordinate notation as fallback
        }
    }
    
    // Fall back to coordinate notation
    let mv = trimmed.to_lowercase();
    
    if mv.len() == 2 {
        let dest = Square::from_str(&mv)
            .map_err(|e| anyhow!("Invalid square: {}", e))?;
        
        let mut matches: Vec<ChessMove> = MoveGen::new_legal(board)
            .filter(|m| m.get_dest() == dest)
            .collect();
            
        if matches.len() == 1 {
            return Ok(matches.remove(0));
        }
        
        return Err(anyhow!(
            "Ambiguous or illegal move. Use coordinate form like e2e4 or SAN like Nf6."
        ));
    }

    if mv.len() == 4 || mv.len() == 5 {
        let from = Square::from_str(&mv[0..2])
            .map_err(|e| anyhow!("Invalid source square: {}", e))?;
        let to = Square::from_str(&mv[2..4])
            .map_err(|e| anyhow!("Invalid destination square: {}", e))?;
        let promo = if mv.len() == 5 {
            Some(parse_promotion(&mv[4..5])?)
        } else {
            None
        };
        
        let candidate = ChessMove::new(from, to, promo);
        if MoveGen::new_legal(board).any(|m| m == candidate) {
            return Ok(candidate);
        }
    }

    Err(anyhow!("Illegal move. Try e4, e2e4, or Nf6."))
}

fn parse_san(board: &Board, input: &str) -> Result<ChessMove> {
    let s = input.trim();
    let side = board.side_to_move();
    
    // Handle castling
    if s == "O-O" || s == "o-o" || s == "0-0" {
        return parse_castling(board, side, false);
    }
    if s == "O-O-O" || s == "o-o-o" || s == "0-0-0" {
        return parse_castling(board, side, true);
    }
    
    // Remove check/checkmate markers
    let s = s.trim_end_matches('+').trim_end_matches('#');
    
    // Parse promotion (e.g., e8=Q, e8Q)
    let (move_part, promo) = if let Some(pos) = s.find('=') {
        (s[..pos].to_string(), Some(parse_promotion_char(&s[pos+1..])?))
    } else if s.len() >= 2 && s.chars().nth(s.len() - 2).map_or(false, |c| c.is_ascii_digit()) {
        let last = s.chars().last().unwrap();
        if let Ok(p) = parse_promotion_char(&last.to_string()) {
            (s[..s.len()-1].to_string(), Some(p))
        } else {
            (s.to_string(), None)
        }
    } else {
        (s.to_string(), None)
    };
    
    // Remove capture marker 'x' or 'X' (it doesn't affect move parsing, just notation)
    let move_part = move_part.replace('x', "").replace('X', "");
    
    // Extract destination square (last 2 characters that look like a square)
    // For moves like "nf6", "Nxf6", "Nf6+", the destination is always the last 2 chars
    if move_part.len() < 2 {
        return Err(anyhow!("Invalid SAN move: too short"));
    }
    
    let dest_str = &move_part[move_part.len() - 2..];
    
    // Convert to lowercase for square parsing (squares are always lowercase)
    let dest = Square::from_str(&dest_str.to_lowercase())
        .map_err(|_| anyhow!("Invalid destination square in SAN: {}", dest_str))?;
    
    // Get all legal moves to this destination
    let candidates: Vec<ChessMove> = MoveGen::new_legal(board)
        .filter(|m| {
            m.get_dest() == dest && 
            promo.map_or(true, |p| m.get_promotion() == Some(p))
        })
        .collect();
    
    if candidates.is_empty() {
        return Err(anyhow!("No legal moves to {}", dest_str));
    }
    
    // Parse piece type and disambiguation
    let piece_type = if move_part.len() > 2 {
        parse_piece_char(move_part.chars().next().unwrap())
    } else {
        None // Pawn move
    };
    
    let matches: Vec<ChessMove> = candidates.iter()
        .filter(|&m| {
            let piece = board.piece_on(m.get_source()).unwrap_or(Piece::Pawn);
            let expected_piece = piece_type.unwrap_or(Piece::Pawn);
            
            if piece != expected_piece {
                return false;
            }
            
            // Check disambiguation if present (e.g., Nbd7, R1e2, Rae1)
            if move_part.len() > 3 {
                let disambig = &move_part[1..move_part.len()-2];
                if disambig.len() == 1 {
                    let ch = disambig.chars().next().unwrap();
                    // File disambiguation (e.g., Nbd7 - 'b' is the file)
                    if ch.is_ascii_lowercase() {
                        let file_idx = (ch as u8 - b'a') as usize;
                        return m.get_source().get_file().to_index() == file_idx;
                    }
                    // Rank disambiguation (e.g., N1f3 - '1' is the rank)
                    if ch.is_ascii_digit() {
                        if let Ok(rank_num) = disambig.parse::<u8>() {
                            let rank_idx = (rank_num - 1) as usize;
                            return m.get_source().get_rank().to_index() == rank_idx;
                        }
                    }
                } else if disambig.len() == 2 {
                    // Full square disambiguation (e.g., Rae1 - 'e1' is the source)
                    if let Ok(sq) = Square::from_str(disambig) {
                        return m.get_source() == sq;
                    }
                }
                // If disambiguation doesn't match, exclude this move
                return false;
            }
            
            true
        })
        .copied()
        .collect();
    
    if matches.len() == 1 {
        Ok(matches[0])
    } else if matches.is_empty() {
        // Provide more helpful error message
        let piece_info = piece_type
            .map(|p| format!("{:?}", p))
            .unwrap_or_else(|| "pawn".to_string());
        Err(anyhow!(
            "No legal {:?} move to {} for SAN: {}. Try a different move or use coordinate notation like e2e4.",
            piece_info,
            dest_str,
            input
        ))
    } else {
        Err(anyhow!("Ambiguous SAN move: {}. Use disambiguation like Nbd7 or R1e2.", input))
    }
}

fn parse_castling(board: &Board, side: Color, queenside: bool) -> Result<ChessMove> {
    let rank = if side == Color::White { Rank::First } else { Rank::Eighth };
    let king_from = Square::make_square(rank, File::E);
    let king_to = Square::make_square(rank, if queenside { File::C } else { File::G });
    
    let candidate = ChessMove::new(king_from, king_to, None);
    
    if MoveGen::new_legal(board).any(|m| m == candidate) {
        Ok(candidate)
    } else {
        Err(anyhow!("Castling not legal"))
    }
}

fn parse_piece_char(c: char) -> Option<Piece> {
    match c.to_ascii_uppercase() {
        'K' => Some(Piece::King),
        'Q' => Some(Piece::Queen),
        'R' => Some(Piece::Rook),
        'B' => Some(Piece::Bishop),
        'N' => Some(Piece::Knight),
        _ => None,
    }
}

fn parse_promotion_char(s: &str) -> Result<Piece> {
    match s.to_ascii_uppercase().as_str() {
        "Q" => Ok(Piece::Queen),
        "R" => Ok(Piece::Rook),
        "B" => Ok(Piece::Bishop),
        "N" => Ok(Piece::Knight),
        _ => Err(anyhow!("Invalid promotion piece")),
    }
}

fn parse_promotion(token: &str) -> Result<Piece> {
    match token {
        "q" => Ok(Piece::Queen),
        "r" => Ok(Piece::Rook),
        "b" => Ok(Piece::Bishop),
        "n" => Ok(Piece::Knight),
        _ => Err(anyhow!("Unknown promotion piece. Use q, r, b, or n.")),
    }
}

pub fn color_to_turn(color: Color) -> &'static str {
    if color == Color::White {
        "w"
    } else {
        "b"
    }
}

pub fn uci_string(mv: ChessMove) -> String {
    let mut uci = format!("{}{}", mv.get_source(), mv.get_dest());
    if let Some(promo) = mv.get_promotion() {
        let suffix = match promo {
            Piece::Queen => "q",
            Piece::Rook => "r",
            Piece::Bishop => "b",
            Piece::Knight => "n",
            Piece::Pawn | Piece::King => "",
        };
        uci.push_str(suffix);
    }
    uci
}

pub fn build_caption(
    header: &str,
    board: &Board,
    white: &DbUser,
    black: &DbUser,
    to_move: Color,
    result_line: Option<String>,
) -> String {
    let white_name = white.mention_html();
    let black_name = black.mention_html();
    let side = if to_move == Color::White {
        white.mention_html()
    } else {
        black.mention_html()
    };
    
    let mut caption = format!(
        "{}.\nWhite: {}\nBlack: {}\nTo move: {}",
        crate::utils::escape_html(header),
        white_name,
        black_name,
        side
    );
    
    if let Some(advantage) = material_advantage(board, white, black) {
        caption.push_str(&format!("\n{}", advantage));
    }
    
    if let Some(result) = result_line {
        caption.push_str(&format!("\n{}", result));
    }
    
    caption
}

pub fn material_advantage(board: &Board, white: &DbUser, black: &DbUser) -> Option<String> {
    let score = material_score(board);
    if score == 0 {
        return None;
    }
    
    if score > 0 {
        Some(format!("{} +{}", white.mention_html(), score))
    } else {
        Some(format!("{} +{}", black.mention_html(), score.abs()))
    }
}

fn material_score(board: &Board) -> i32 {
    let mut score = 0;
    for (piece, value) in [
        (Piece::Pawn, 1),
        (Piece::Knight, 3),
        (Piece::Bishop, 3),
        (Piece::Rook, 5),
        (Piece::Queen, 9),
    ] {
        let white = (board.pieces(piece) & board.color_combined(Color::White)).popcnt();
        let black = (board.pieces(piece) & board.color_combined(Color::Black)).popcnt();
        score += (white as i32 - black as i32) * value;
    }
    score
}
