use shakmat_core::{Board, Move, DEFAULT_FEN};

// Tests that the zobrist keys are calculated correctly w.r.t. the random
// values defined by PolyGlot.
// Test cases from: http://hgm.nubati.net/book_format.html

fn run_test(moves: &[&str], fen: &str, zobrist_hex: &str) {
    // Tests whether the zobrist key matches for a position, both loading
    // it from FEN and by making the moves. It is important to test both
    // because the key is calculated differently in each case
    // (moves update it incrementally)
    let mut board = Board::default();

    for mv in moves {
        board = board.make_move(&Move::from_notation(mv).unwrap());
    }

    let board_fen = Board::from_fen(fen).unwrap();
    assert_eq!(zobrist_hex, format!("{:#x}", board_fen.zobrist_key()));
    assert_eq!(zobrist_hex, format!("{:#x}", board.zobrist_key()));
}

#[test]
fn initial_pos() {
    run_test(&[], DEFAULT_FEN, "0x463b96181691fc9c");
}

#[test]
fn pos1() {
    run_test(
        &["e2e4"], 
        "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq e3 0 1", 
        "0x823c9b50fd114196"
    );
}

#[test]
fn pos2() {
    run_test(
        &["e2e4", "d7d5"], 
        "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2", 
        "0x756b94461c50fb0"
    );
}

#[test]
fn pos3() {
    run_test(
        &["e2e4", "d7d5", "e4e5"], 
        "rnbqkbnr/ppp1pppp/8/3pP3/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2", 
        "0x662fafb965db29d4"
    );
}

#[test]
fn pos4() {
    run_test(
        &["e2e4", "d7d5", "e4e5", "f7f5"], 
        "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPP1PPP/RNBQKBNR w KQkq f6 0 3", 
        "0x22a48b5a8e47ff78"
    );
}

#[test]
fn pos5() {
    run_test(
        &["e2e4", "d7d5", "e4e5", "f7f5", "e1e2"], 
        "rnbqkbnr/ppp1p1pp/8/3pPp2/8/8/PPPPKPPP/RNBQ1BNR b kq - 0 3", 
        "0x652a607ca3f242c1"
    );
}

#[test]
fn pos6() {
    run_test(
        &["e2e4", "d7d5", "e4e5", "f7f5", "e1e2", "e8f7"], 
        "rnbq1bnr/ppp1pkpp/8/3pPp2/8/8/PPPPKPPP/RNBQ1BNR w - - 0 4", 
        "0xfdd303c946bdd9"
    );
}

#[test]
fn pos7() {
    run_test(
        &["a2a4", "b7b5", "h2h4", "b5b4", "c2c4"], 
        "rnbqkbnr/p1pppppp/8/8/PpP4P/8/1P1PPPP1/RNBQKBNR b KQkq c3 0 3", 
        "0x3c8123ea7b067637"
    );
}

#[test]
fn pos8() {
    run_test(
        &["a2a4", "b7b5", "h2h4", "b5b4", "c2c4", "b4c3", "a1a3"], 
        "rnbqkbnr/p1pppppp/8/8/P6P/R1p5/1P1PPPP1/1NBQKBNR b Kkq - 0 4", 
        "0x5c3f9b829b279560"
    );
}