use std::fmt::{Formatter, Display};
use std::ops::{Neg, Add, Sub};
use shakmat_core::{Board, Color::{*, self}, BitBoard, PieceType::{*, self}};
use super::{tables, EvalData};

// Represents the evaluation of a position. The goal of using a struct instead of an i16
// directly is to implement Display, to be able to show the score in a much nicer way
// (for example, plies to checkmate instead of the raw score)
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Evaluation { score: i16 } 

// The contempt factor is the score that the engine associates with a draw.
// A negative value means that the engine assumes it is superior to its opponent,
// so drawing is penalized. Conversely, a positive value means that the engine assumes
// itself to be inferior, so it encourages drawing when it cannot find a decisive advantage.
const CONTEMPT: i16 = 0;

// Bonuses and penalties, measured in centipawns
// Values that are pairs represent the scores for the middlegame and endgame phases
type ScorePair = (i16, i16);

const PAWN_BASE_VALUE: i16 = 100;
const BISHOP_BASE_VALUE: i16 = 300;
const KNIGHT_BASE_VALUE: i16 = 300;
const ROOK_BASE_VALUE: i16 = 500;
const QUEEN_BASE_VALUE: i16 = 900;

const TEMPO_BONUS: i16 = 28;
const BISHOP_PAIR_BONUS: ScorePair = (20, 60);
const ROOK_OPEN_FILE_BONUS: ScorePair = (20, 20);

// Evaluate how favorable a position is for the current side to move
// A positive score favors the current side, while a negative one
// favors the rival.
pub fn evaluate_position(board: &Board) -> Evaluation {
    let mut eval_data = EvalData::new(board);

    calc_piece_score(&mut eval_data);
    calc_positional_score(&mut eval_data);
    calc_control_score(&mut eval_data);
    calc_bishop_pair_bonus(&mut eval_data);
    calc_tempo(&mut eval_data);
    eval_data.compute_score()
}

// Computes the total piece score of a color, using the specialized functions
fn calc_piece_score(eval_data: &mut EvalData) {  
    let (wp_mg, wp_eg) = eval_bitboard(eval_data.white_pieces.pawns, Pawn, eval_data.board, White);
    let (bp_mg, bp_eg) = eval_bitboard(eval_data.black_pieces.pawns, Pawn, eval_data.board, Black);

    let (wb_mg, wb_eg) = eval_bitboard(eval_data.white_pieces.bishops, Bishop, eval_data.board, White);
    let (bb_mg, bb_eg) = eval_bitboard(eval_data.black_pieces.bishops, Bishop, eval_data.board, Black);

    let (wn_mg, wn_eg) = eval_bitboard(eval_data.white_pieces.knights, Knight, eval_data.board, White);
    let (bn_mg, bn_eg) = eval_bitboard(eval_data.black_pieces.knights, Knight, eval_data.board, Black);

    let (wr_mg, wr_eg) = eval_bitboard(eval_data.white_pieces.rooks, Rook, eval_data.board, White);
    let (br_mg, br_eg) = eval_bitboard(eval_data.black_pieces.rooks, Rook, eval_data.board, Black);

    let (wq_mg, wq_eg) = eval_bitboard(eval_data.white_pieces.queens, Queen, eval_data.board, White);
    let (bq_mg, bq_eg) = eval_bitboard(eval_data.black_pieces.queens, Queen, eval_data.board, Black);

    let (wk_mg, wk_eg) = eval_bitboard(eval_data.white_pieces.king, King, eval_data.board, White);
    let (bk_mg, bk_eg) = eval_bitboard(eval_data.black_pieces.king, King, eval_data.board, Black);

    eval_data.score_opening = wp_mg + wb_mg + wn_mg + wr_mg + wq_mg + wk_mg - bp_mg - bb_mg - bn_mg - br_mg - bq_mg - bk_mg;
    eval_data.score_endgame = wp_eg + wb_eg + wn_eg + wr_eg + wq_eg + wk_eg - bp_eg - bb_eg - bn_eg - br_eg - bq_eg - bk_eg;
}

// Gives an extra centipoint for each square controlled, and 2 points
// for each one in the endgame stage.
fn calc_control_score(eval_data: &mut EvalData) {
    let control_white = eval_data.board.get_attack_bitboard(White).count() as i16;
    let control_black = eval_data.board.get_attack_bitboard(Black).count() as i16;
    eval_data.score_opening += control_white - control_black;
    eval_data.score_endgame += 2 * (control_white - control_black);
}

// Gives positional bonuses to each piece using the corresponding table,
// for both the middlegame and endgame phases.
fn calc_positional_score(eval_data: &mut EvalData) {
    let wp = eval_data.white_pieces;
    let bp = eval_data.black_pieces;

    let score_opening = pos_score(wp.pawns, &tables::WHITE_PAWN_OPENING)
        - pos_score(bp.pawns, &tables::BLACK_PAWN_OPENING)
        + pos_score(wp.rooks, &tables::WHITE_ROOK_OPENING)
        - pos_score(bp.rooks, &tables::BLACK_ROOK_OPENING)
        + pos_score(wp.knights, &tables::WHITE_KNIGHT_OPENING)
        - pos_score(bp.knights, &tables::BLACK_KNIGHT_OPENING)
        + pos_score(wp.knights, &tables::WHITE_BISHOP_OPENING)
        - pos_score(bp.knights, &tables::BLACK_BISHOP_OPENING)
        + pos_score(wp.queens, &tables::WHITE_QUEEN_OPENING)
        - pos_score(bp.queens, &tables::BLACK_QUEEN_OPENING)
        + pos_score(wp.king, &tables::WHITE_KING_OPENING)
        - pos_score(bp.king, &tables::BLACK_KING_OPENING);
    
    let score_endgame = pos_score(wp.pawns, &tables::WHITE_PAWN_ENDGAME)
        - pos_score(bp.pawns, &tables::BLACK_PAWN_ENDGAME)
        + pos_score(wp.knights, &tables::WHITE_KNIGHT_ENDGAME)
        - pos_score(bp.knights, &tables::BLACK_KNIGHT_ENDGAME)
        + pos_score(wp.knights, &tables::WHITE_BISHOP_ENDGAME)
        - pos_score(bp.knights, &tables::BLACK_BISHOP_ENDGAME)
        + pos_score(wp.queens, &tables::WHITE_QUEEN_ENDGAME)
        - pos_score(bp.queens, &tables::BLACK_QUEEN_ENDGAME)
        + pos_score(wp.king, &tables::WHITE_KING_ENDGAME)
        - pos_score(bp.king, &tables::BLACK_KING_ENDGAME);  

    eval_data.score_opening += score_opening;
    eval_data.score_endgame += score_endgame;
}

fn calc_bishop_pair_bonus(eval_data: &mut EvalData) {
    let bonus_early = BISHOP_PAIR_BONUS.0;
    let bonus_late = BISHOP_PAIR_BONUS.1;

    let white_pair = (eval_data.wb >= 2) as i16;
    let black_pair = (eval_data.bb >= 2) as i16;
    
    eval_data.score_opening += bonus_early * white_pair - bonus_early * black_pair;
    eval_data.score_endgame += bonus_late * white_pair - bonus_late * black_pair;
}

fn calc_tempo(eval_data: &mut EvalData) {
    // Small bonus for having the right to move, only
    // in the early game
    eval_data.score_opening += TEMPO_BONUS;
}

fn pos_score(bb: BitBoard, pos_table: &[i16]) -> i16 {
    bb.piece_indices().map(|i| pos_table[i as usize]).sum()
}

///////////////////////////////////////////////////////////////////////////////
/// Aux function to evaluate a whole bitboard of pieces of a given type
fn eval_bitboard(bb: BitBoard, piece_type: PieceType, board: &Board, color: Color) -> ScorePair {
    let eval_func = match piece_type {
        Pawn => eval_pawn,
        Knight => eval_knight,
        Bishop => eval_bishop,
        Rook => eval_rook,
        Queen => eval_queen,
        King => eval_king,
    };

    bb.piece_indices()
        .map(|i| eval_func(i, bb, board, color))
        .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1))
}

///////////////////////////////////////////////////////////////////////////////
/// Specialized functions for each piece type
fn eval_pawn(pos: u8, bb: BitBoard, board: &Board, color: Color) -> ScorePair {
    (PAWN_BASE_VALUE, PAWN_BASE_VALUE)
}

fn eval_bishop(pos: u8, bb: BitBoard, board: &Board, color: Color) -> ScorePair {
    (BISHOP_BASE_VALUE, BISHOP_BASE_VALUE)
}

fn eval_knight(pos: u8, bb: BitBoard, board: &Board, color: Color) -> ScorePair {
    (KNIGHT_BASE_VALUE, KNIGHT_BASE_VALUE)
}

fn eval_rook(pos: u8, bb: BitBoard, board: &Board, color: Color) -> ScorePair {
    let mut mg = ROOK_BASE_VALUE;
    let mut eg = ROOK_BASE_VALUE;

    let file = tables::FILES[pos as usize % 8];
    if (file & (board.get_pieces(White).pawns | board.get_pieces(Black).pawns)).is_empty() {
        mg += ROOK_OPEN_FILE_BONUS.0;
        eg += ROOK_OPEN_FILE_BONUS.1;
    }

    (mg, eg)
}

fn eval_queen(pos: u8, bb: BitBoard, board: &Board, color: Color) -> ScorePair {
    (QUEEN_BASE_VALUE, QUEEN_BASE_VALUE)
}

fn eval_king(pos: u8, bb: BitBoard, board: &Board, color: Color) -> ScorePair {
    (0, 0)
}

///////////////////////////////////////////////////////////////////////////////

impl Evaluation {
    pub const fn new(score: i16) -> Self {
        Self { score }
    }

    pub const fn contempt() -> Self {
        Self::new(CONTEMPT)
    } 

    // The min value is set to i16::MIN + 1, so that -min_val() == max_val()
    // and viceversa. Otherwise, it overflows when swapping its sign
    // and all sort of bad things happen.
    pub fn min_val() -> Self {
        Self::new(i16::MIN + 1)
    }

    pub fn max_val() -> Self {
        Self::new(i16::MAX)
    }

    pub fn score(&self) -> i16 {
        self.score
    }

    pub fn is_positive_mate(&self) -> bool {
        *self >= Self::max_val() - 100
    }

    pub fn is_negative_mate(&self) -> bool {
        *self <= Self::min_val() + 100
    }

    pub fn is_mate(&self) -> bool {
        self.is_negative_mate() || self.is_positive_mate()
    }
}

impl Neg for Evaluation {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.score)
    }
}

impl Sub<i16> for Evaluation {
    type Output = Self;

    fn sub(self, rhs: i16) -> Self::Output {
        Self::new(self.score - rhs)
    }
}

impl Add<i16> for Evaluation {
    type Output = Self;

    fn add(self, rhs: i16) -> Self::Output {
        Self::new(self.score + rhs)
    }
}

impl Sub<Self> for Evaluation {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.score - rhs.score)
    }
}

impl Add<Self> for Evaluation {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.score + rhs.score)
    }
}

impl PartialOrd<i16> for Evaluation {
    fn partial_cmp(&self, other: &i16) -> Option<std::cmp::Ordering> {
        self.score.partial_cmp(other)
    }
}

impl PartialEq<i16> for Evaluation {
    fn eq(&self, other: &i16) -> bool {
        self.score == *other
    }
}

impl Display for Evaluation {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.is_positive_mate() {
            write!(f, "M{}", (i16::MAX - self.score()) / 2)
        } else if self.is_negative_mate() {
            write!(f, "-M{}", (self.score() - i16::MIN - 1) / 2)
        } else {
            write!(f, "{:+.2}", self.score() as f32 / 100.0)
        }
    }
}