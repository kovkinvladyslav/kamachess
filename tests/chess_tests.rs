use chess::{Board, Piece, Square};
use kamachess::game::parse_move;
use std::str::FromStr;

#[test]
fn test_parse_pawn_move_e4() {
    let board = Board::default();
    let mv = parse_move(&board, "e4").unwrap();
    assert_eq!(mv.get_dest(), Square::from_str("e4").unwrap());
}

#[test]
fn test_parse_pawn_move_e2e4() {
    let board = Board::default();
    let mv = parse_move(&board, "e2e4").unwrap();
    assert_eq!(mv.get_source(), Square::from_str("e2").unwrap());
    assert_eq!(mv.get_dest(), Square::from_str("e4").unwrap());
}

#[test]
fn test_parse_knight_move_nf3() {
    let board = Board::default();
    let mv = parse_move(&board, "Nf3").unwrap();
    assert_eq!(mv.get_dest(), Square::from_str("f3").unwrap());
    assert_eq!(board.piece_on(mv.get_source()), Some(Piece::Knight));
}

#[test]
fn test_parse_knight_move_lowercase_nf3() {
    let board = Board::default();
    let mv = parse_move(&board, "nf3").unwrap();
    assert_eq!(mv.get_dest(), Square::from_str("f3").unwrap());
}

#[test]
fn test_parse_knight_move_nc3() {
    let board = Board::default();
    let mv = parse_move(&board, "Nc3").unwrap();
    assert_eq!(mv.get_dest(), Square::from_str("c3").unwrap());
}

#[test]
fn test_parse_knight_nf6_illegal_for_white() {
    let board = Board::default();
    assert!(parse_move(&board, "Nf6").is_err());
}

#[test]
fn test_parse_black_knight_nf6() {
    let board = Board::default();
    let mv = parse_move(&board, "e4").unwrap();
    let board = board.make_move_new(mv);
    let mv = parse_move(&board, "Nf6").unwrap();
    assert_eq!(mv.get_dest(), Square::from_str("f6").unwrap());
}

#[test]
fn test_parse_coordinate_g1f3() {
    let board = Board::default();
    let mv = parse_move(&board, "g1f3").unwrap();
    assert_eq!(mv.get_source(), Square::from_str("g1").unwrap());
    assert_eq!(mv.get_dest(), Square::from_str("f3").unwrap());
}

#[test]
fn test_castling_short() {
    let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
    let board = Board::from_str(fen).unwrap();
    let mv = parse_move(&board, "O-O").unwrap();
    assert_eq!(mv.get_source(), Square::from_str("e1").unwrap());
    assert_eq!(mv.get_dest(), Square::from_str("g1").unwrap());
}

#[test]
fn test_castling_long() {
    let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
    let board = Board::from_str(fen).unwrap();
    let mv = parse_move(&board, "O-O-O").unwrap();
    assert_eq!(mv.get_source(), Square::from_str("e1").unwrap());
    assert_eq!(mv.get_dest(), Square::from_str("c1").unwrap());
}

#[test]
fn test_castling_zero_notation() {
    let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
    let board = Board::from_str(fen).unwrap();
    let mv = parse_move(&board, "0-0").unwrap();
    assert_eq!(mv.get_dest(), Square::from_str("g1").unwrap());
}
