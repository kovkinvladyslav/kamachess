use chess::{Board, Color, Piece, Square};
use kamachess::game::parse_move;
use std::str::FromStr;

fn apply_moves(moves: &[&str]) -> Board {
    let mut board = Board::default();
    for mv in moves {
        let parsed = parse_move(&board, mv)
            .unwrap_or_else(|err| panic!("Failed to parse move '{mv}': {err}"));
        board = board.make_move_new(parsed);
    }
    board
}

fn assert_piece(board: &Board, square: &str, piece: Piece, color: Color) {
    let sq = Square::from_str(square).expect("invalid square");
    assert_eq!(
        board.piece_on(sq),
        Some(piece),
        "piece mismatch on {square}"
    );
    assert_eq!(
        board.color_on(sq),
        Some(color),
        "color mismatch on {square}"
    );
}

#[test]
fn opera_game_finishing_position() {
    let moves = [
        "e4", "e5", "Nf3", "d6", "d4", "Bg4", "dxe5", "Bxf3", "Qxf3", "dxe5", "Bc4", "Nf6", "Qb3",
        "Qe7", "Nc3", "c6", "Bg5", "b5", "Nxb5", "cxb5", "Bxb5+", "Nbd7", "O-O-O", "Rd8", "Rxd7",
        "Rxd7", "Rd1", "Qe6", "Bxd7+", "Nxd7", "Qb8+", "Nxb8", "Rd8#",
    ];
    let board = apply_moves(&moves);

    assert_eq!(board.status(), chess::BoardStatus::Checkmate);
    assert_piece(&board, "d8", Piece::Rook, Color::White);
    assert_piece(&board, "g5", Piece::Bishop, Color::White);
    assert_piece(&board, "e8", Piece::King, Color::Black);
    assert_piece(&board, "e6", Piece::Queen, Color::Black);
    assert_piece(&board, "h8", Piece::Rook, Color::Black);
    assert_piece(&board, "b8", Piece::Knight, Color::Black);
}

#[test]
fn legall_mate_position() {
    let moves = [
        "e4", "e5", "Nf3", "Nc6", "Bc4", "d6", "Nc3", "Bg4", "Nxe5", "Bxd1", "Bxf7+", "Ke7", "Nd5#",
    ];
    let board = apply_moves(&moves);

    assert_eq!(board.status(), chess::BoardStatus::Checkmate);
    assert_piece(&board, "f7", Piece::Bishop, Color::White);
    assert_piece(&board, "d5", Piece::Knight, Color::White);
    assert_piece(&board, "e7", Piece::King, Color::Black);
    assert_piece(&board, "d1", Piece::Bishop, Color::Black);
}

#[test]
fn blackburne_shilling_mate_position() {
    let moves = [
        "e4", "e5", "Nf3", "Nc6", "Bc4", "Nd4", "Nxe5", "Qg5", "Nxf7", "Qxg2", "Rf1", "Qxe4+",
        "Be2", "Nf3#",
    ];
    let board = apply_moves(&moves);

    assert_eq!(board.status(), chess::BoardStatus::Checkmate);
    assert_piece(&board, "f3", Piece::Knight, Color::Black);
    assert_piece(&board, "e4", Piece::Queen, Color::Black);
    assert_piece(&board, "f7", Piece::Knight, Color::White);
    assert_piece(&board, "f1", Piece::Rook, Color::White);
    assert_piece(&board, "e8", Piece::King, Color::Black);
}

#[test]
fn pawn_short_notation_does_not_move_other_pieces() {
    let moves = ["e4", "e5", "Bc4", "Nc6"];
    let board = apply_moves(&moves);

    assert!(parse_move(&board, "a6").is_err());
    let bishop_move = parse_move(&board, "Ba6").expect("Ba6 should be legal");
    assert_eq!(bishop_move.get_source(), Square::from_str("c4").unwrap());
    assert_eq!(bishop_move.get_dest(), Square::from_str("a6").unwrap());
}
