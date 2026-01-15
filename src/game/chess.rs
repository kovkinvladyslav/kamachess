use anyhow::{anyhow, Result};
use chess::{Board, ChessMove, Color, MoveGen, Piece, Square};
use std::str::FromStr;
use crate::models::DbUser;

pub fn parse_move(board: &Board, input: &str) -> Result<ChessMove> {
    let mv = input.to_lowercase();
    
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
            "Ambiguous or illegal move. Use coordinate form like e2e4."
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

    Err(anyhow!("Illegal move. Try e4 or e2e4."))
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
