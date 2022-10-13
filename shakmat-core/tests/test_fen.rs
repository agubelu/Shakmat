use shakmat_core::{Board, DEFAULT_FEN, Move};

// Tests the FEN generation by comparing the known ones against
// the expected output from the position
#[test]
fn test_known_fens() {
    let fens = [
        DEFAULT_FEN,
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10"
    ];

    for fen in fens {
        let board = Board::from_fen(fen).unwrap();
        assert_eq!(fen, board.fen());
    }
}

// Tests the generated FENS when some moves are done from the initial position
#[test]
fn test_dynamic_fens() {
    let moves = ["e2e4", "c7c5", "g1f3"];
    let fens = [
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1",
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/8/PPPP1PPP/RNBQKBNR w KQkq c6 0 2",
        "rnbqkbnr/pp1ppppp/8/2p5/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2"
    ];

    let mut board = Board::default();

    for (movstr, &fen) in moves.iter().zip(fens.iter()) {
        let mv = Move::from_notation(movstr).unwrap();
        board = board.make_move(&mv);
        assert_eq!(fen, board.fen());
    }
}