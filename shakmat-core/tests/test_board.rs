use dashmap::DashMap;
use shakmat_core::{Board, DEFAULT_FEN};

// Perft positions and results obtained from: https://www.chessprogramming.org/Perft_Results
///////////////////////////////////////////////////////////////////////////////

fn test_perft(fen: &str, expected: &[u64]) {
    let board = Board::from_fen(fen).unwrap();
    let cache = DashMap::default();
    for (i, expected) in expected.iter().copied().enumerate() {
        assert_eq!(board.perft_with_cache(i + 1, &cache), expected);
    }
}

///////////////////////////////////////////////////////////////////////////////

#[test]
fn default_pos() {
    test_perft(
        DEFAULT_FEN,
        &[20, 400, 8_902, 197_281, 4_865_609, 119_060_324, 3_195_901_860]
    )
}

#[test]
fn pos2() {
    test_perft(
        "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
        &[48, 2_039, 97_862, 4_085_603, 193_690_690, 8_031_647_685]
    )
}

#[test]
fn pos3() {
    test_perft(
        "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
        &[14, 191, 2_812, 43_238, 674_624, 11_030_083, 178_633_661]
    )
}

#[test]
fn pos4() {
    test_perft(
        "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
        &[6, 264, 9_467, 422_333, 15_833_292, 706_045_033]
    )
}

#[test]
fn pos4_mirrored() {
    test_perft(
        "r2q1rk1/pP1p2pp/Q4n2/bbp1p3/Np6/1B3NBn/pPPP1PPP/R3K2R b KQ - 0 1",
        &[6, 264, 9_467, 422_333, 15_833_292, 706_045_033]
    )
}

#[test]
fn pos5() {
    test_perft(
        "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
        &[44, 1_486, 62_379, 2_103_487, 89_941_194]
    )
}

#[test]
fn pos6() {
    test_perft(
        "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
        &[46, 2_079, 89_890, 3_894_594, 164_075_551, 6_923_051_137]
    )
}