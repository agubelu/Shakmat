use shakmat_core::Board;

// Perft positions and results obtained from: https://www.chessprogramming.org/Perft_Results

///////////////////////////////////////////////////////////////////////////////
struct TestPosition {
    fen: String,
    positions: Vec<u64>,
}

impl TestPosition {
    fn new(fen: &str, positions: Vec<u64>) -> Self {
        Self {fen: fen.to_string(), positions}
    }

    fn run_tests(&self) {
        let board = Board::from_fen(&self.fen).unwrap();
        for (i, expected) in self.positions.iter().copied().enumerate() {
            assert_eq!(board.perft(i + 1), expected);
        }
    }
}
///////////////////////////////////////////////////////////////////////////////

#[test]
fn default_pos() {
    TestPosition::new(
        "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
        vec![20, 400, 8902, 197281, 4865609, 119060324, 3195901860]
    ).run_tests()
}

#[test]
fn pos2() {
    TestPosition::new(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        vec![48, 2039, 97862, 4085603, 193690690, 8031647685]
    ).run_tests()
}

#[test]
fn pos3() {
    TestPosition::new(
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        vec![14, 191, 2812, 43238, 674624, 11030083, 178633661]
    ).run_tests()
}

#[test]
fn pos4() {
    TestPosition::new(
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        vec![6, 264, 9467, 422333, 15833292, 706045033]
    ).run_tests()
}

#[test]
fn pos4_mirrored() {
    TestPosition::new(
        "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
        vec![6, 264, 9467, 422333, 15833292, 706045033]
    ).run_tests()
}

#[test]
fn pos5() {
    TestPosition::new(
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        vec![44, 1486, 62379, 2103487, 89941194]
    ).run_tests()
}

#[test]
fn pos6() {
    TestPosition::new(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        vec![46, 2079, 89890, 3894594, 164075551, 6923051137]
    ).run_tests()
}