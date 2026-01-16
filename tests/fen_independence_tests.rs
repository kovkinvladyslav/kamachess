use chess::Board;
use kamachess::game::parse_move;
use std::str::FromStr;

/// Test that different input formats for the same move produce identical FEN strings.
/// This is critical for image caching and database consistency.

#[test]
fn test_pawn_move_fen_independence() {
    let board = Board::default();

    // Parse e4 in different formats
    let mv1 = parse_move(&board, "e4").unwrap();
    let mv2 = parse_move(&board, "e2e4").unwrap();

    // Apply moves to separate boards
    let board1 = board.make_move_new(mv1);
    let board2 = board.make_move_new(mv2);

    // FEN strings must be identical
    assert_eq!(board1.to_string(), board2.to_string());
}

#[test]
fn test_knight_move_fen_independence() {
    let board = Board::default();

    // Parse Nf3 in different formats
    let mv1 = parse_move(&board, "Nf3").unwrap();
    let mv2 = parse_move(&board, "nf3").unwrap();
    let mv3 = parse_move(&board, "g1f3").unwrap();

    let board1 = board.make_move_new(mv1);
    let board2 = board.make_move_new(mv2);
    let board3 = board.make_move_new(mv3);

    assert_eq!(board1.to_string(), board2.to_string());
    assert_eq!(board1.to_string(), board3.to_string());
}

#[test]
fn test_castling_fen_independence() {
    let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
    let board = Board::from_str(fen).unwrap();

    // Short castling in different notations
    let mv1 = parse_move(&board, "O-O").unwrap();
    let mv2 = parse_move(&board, "0-0").unwrap();
    let mv3 = parse_move(&board, "00").unwrap();
    let mv4 = parse_move(&board, "oo").unwrap();
    let mv5 = parse_move(&board, "OO").unwrap();

    let board1 = board.make_move_new(mv1);
    let board2 = board.make_move_new(mv2);
    let board3 = board.make_move_new(mv3);
    let board4 = board.make_move_new(mv4);
    let board5 = board.make_move_new(mv5);

    assert_eq!(board1.to_string(), board2.to_string());
    assert_eq!(board1.to_string(), board3.to_string());
    assert_eq!(board1.to_string(), board4.to_string());
    assert_eq!(board1.to_string(), board5.to_string());
}

#[test]
fn test_long_castling_fen_independence() {
    let fen = "r3k2r/pppppppp/8/8/8/8/PPPPPPPP/R3K2R w KQkq - 0 1";
    let board = Board::from_str(fen).unwrap();

    // Long castling in different notations
    let mv1 = parse_move(&board, "O-O-O").unwrap();
    let mv2 = parse_move(&board, "0-0-0").unwrap();
    let mv3 = parse_move(&board, "000").unwrap();
    let mv4 = parse_move(&board, "ooo").unwrap();
    let mv5 = parse_move(&board, "OOO").unwrap();

    let board1 = board.make_move_new(mv1);
    let board2 = board.make_move_new(mv2);
    let board3 = board.make_move_new(mv3);
    let board4 = board.make_move_new(mv4);
    let board5 = board.make_move_new(mv5);

    assert_eq!(board1.to_string(), board2.to_string());
    assert_eq!(board1.to_string(), board3.to_string());
    assert_eq!(board1.to_string(), board4.to_string());
    assert_eq!(board1.to_string(), board5.to_string());
}

#[test]
fn test_promotion_fen_independence() {
    let fen = "4k3/P7/8/8/8/8/8/4K3 w - - 0 1";
    let board = Board::from_str(fen).unwrap();

    // Promotion in different formats
    let mv1 = parse_move(&board, "a8=Q").unwrap();
    let mv2 = parse_move(&board, "a7a8q").unwrap();

    let board1 = board.make_move_new(mv1);
    let board2 = board.make_move_new(mv2);

    assert_eq!(board1.to_string(), board2.to_string());
}

#[test]
fn test_capture_fen_independence() {
    let fen = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";
    let board = Board::from_str(fen).unwrap();

    // Pawn capture in different formats
    let mv1 = parse_move(&board, "exd5").unwrap();
    let mv2 = parse_move(&board, "e4d5").unwrap();

    let board1 = board.make_move_new(mv1);
    let board2 = board.make_move_new(mv2);

    assert_eq!(board1.to_string(), board2.to_string());
}

#[test]
fn test_full_game_sequence_fen_independence() {
    // Play the same opening using different notations and verify FEN equality
    let board = Board::default();

    // Game 1: Using SAN notation
    let mut board1 = board;
    board1 = board1.make_move_new(parse_move(&board1, "e4").unwrap());
    board1 = board1.make_move_new(parse_move(&board1, "e5").unwrap());
    board1 = board1.make_move_new(parse_move(&board1, "Nf3").unwrap());
    board1 = board1.make_move_new(parse_move(&board1, "Nc6").unwrap());

    // Game 2: Using coordinate notation
    let mut board2 = board;
    board2 = board2.make_move_new(parse_move(&board2, "e2e4").unwrap());
    board2 = board2.make_move_new(parse_move(&board2, "e7e5").unwrap());
    board2 = board2.make_move_new(parse_move(&board2, "g1f3").unwrap());
    board2 = board2.make_move_new(parse_move(&board2, "b8c6").unwrap());

    // Game 3: Mixed notation
    let mut board3 = board;
    board3 = board3.make_move_new(parse_move(&board3, "e4").unwrap());
    board3 = board3.make_move_new(parse_move(&board3, "e7e5").unwrap());
    board3 = board3.make_move_new(parse_move(&board3, "g1f3").unwrap());
    board3 = board3.make_move_new(parse_move(&board3, "Nc6").unwrap());

    // All three games should have identical FEN
    assert_eq!(board1.to_string(), board2.to_string());
    assert_eq!(board1.to_string(), board3.to_string());
}

#[test]
fn test_bishop_move_fen_independence() {
    let fen = "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 1";
    let board = Board::from_str(fen).unwrap();

    // Bishop move in different formats
    let mv1 = parse_move(&board, "Bc4").unwrap();
    let mv2 = parse_move(&board, "bc4").unwrap();
    let mv3 = parse_move(&board, "f1c4").unwrap();

    let board1 = board.make_move_new(mv1);
    let board2 = board.make_move_new(mv2);
    let board3 = board.make_move_new(mv3);

    assert_eq!(board1.to_string(), board2.to_string());
    assert_eq!(board1.to_string(), board3.to_string());
}

#[test]
fn test_queen_move_fen_independence() {
    let fen = "rnbqkbnr/pppp1ppp/8/4p3/2B1P3/8/PPPP1PPP/RNBQK1NR w KQkq - 0 1";
    let board = Board::from_str(fen).unwrap();

    // Queen move in different formats
    let mv1 = parse_move(&board, "Qf3").unwrap();
    let mv2 = parse_move(&board, "qf3").unwrap();
    let mv3 = parse_move(&board, "d1f3").unwrap();

    let board1 = board.make_move_new(mv1);
    let board2 = board.make_move_new(mv2);
    let board3 = board.make_move_new(mv3);

    assert_eq!(board1.to_string(), board2.to_string());
    assert_eq!(board1.to_string(), board3.to_string());
}

#[test]
fn test_rook_move_fen_independence() {
    let fen = "r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1";
    let board = Board::from_str(fen).unwrap();

    // Rook move in different formats
    let mv1 = parse_move(&board, "Ra3").unwrap();
    let mv2 = parse_move(&board, "ra3").unwrap();
    let mv3 = parse_move(&board, "a1a3").unwrap();

    let board1 = board.make_move_new(mv1);
    let board2 = board.make_move_new(mv2);
    let board3 = board.make_move_new(mv3);

    assert_eq!(board1.to_string(), board2.to_string());
    assert_eq!(board1.to_string(), board3.to_string());
}
