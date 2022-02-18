use std::fmt::{Formatter, Display};
use std::ops::{Neg, Add, Sub};
use shakmat_core::{Board, Color::*, BitBoard, PieceType::*};
use super::positional_tables;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
// Represents the evaluation of a position. The goal of using a struct instead of an i16
// directly is to implement Display, to be able to show the score in a much nicer way
// (for example, plies to checkmate instead of the raw score)
pub struct Evaluation { score: i16 } 

// The contempt factor is the score that the engine associates with a draw.
// A negative value means that the engine assumes it is superior to its opponent,
// so drawing is penalized. Conversely, a positive value means that the engine assumes
// itself to be inferior, so it encourages drawing when it cannot find a decisive advantage.
const CONTEMPT: i16 = 0;

// Auxiliary struct to store values that are used in different parts
// of the evaluation, to avoid calculating them multiple times
struct EvalData<'a> {
    board: &'a Board,
    game_phase: i16,
    score_opening: i16,
    score_endgame: i16,
    // Count of pieces of a certain type for every side
    wp: i16, wr: i16, wb: i16, wn: i16, wq: i16,
    bp: i16, br: i16, bb: i16, bn: i16, bq: i16,
}

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

// Computes the total piece score of a color, using the normal piece scores
fn calc_piece_score(eval_data: &mut EvalData) {  
    let score = 100 * (eval_data.wp - eval_data.bp) +
    300 * (eval_data.wn - eval_data.bn) +
    300 * (eval_data.wb - eval_data.bb) +
    500 * (eval_data.wr - eval_data.br) +
    900 * (eval_data.wq - eval_data.bq);

    eval_data.score_opening += score;
    eval_data.score_endgame += score;
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
    let wp = eval_data.board.get_pieces(White);
    let bp = eval_data.board.get_pieces(Black);

    let score_opening = pos_score(wp.get_pieces_of_type(Pawn), &positional_tables::WHITE_PAWN_OPENING)
        - pos_score(bp.get_pieces_of_type(Pawn), &positional_tables::BLACK_PAWN_OPENING)
        + pos_score(wp.get_pieces_of_type(Rook), &positional_tables::WHITE_ROOK_OPENING)
        - pos_score(bp.get_pieces_of_type(Rook), &positional_tables::BLACK_ROOK_OPENING)
        + pos_score(wp.get_pieces_of_type(Knight), &positional_tables::WHITE_KNIGHT_OPENING)
        - pos_score(bp.get_pieces_of_type(Knight), &positional_tables::BLACK_KNIGHT_OPENING)
        + pos_score(wp.get_pieces_of_type(Bishop), &positional_tables::WHITE_BISHOP_OPENING)
        - pos_score(bp.get_pieces_of_type(Bishop), &positional_tables::BLACK_BISHOP_OPENING)
        + pos_score(wp.get_pieces_of_type(Queen), &positional_tables::WHITE_QUEEN_OPENING)
        - pos_score(bp.get_pieces_of_type(Queen), &positional_tables::BLACK_QUEEN_OPENING)
        + pos_score(wp.get_pieces_of_type(King), &positional_tables::WHITE_KING_OPENING)
        - pos_score(bp.get_pieces_of_type(King), &positional_tables::BLACK_KING_OPENING);
    
    let score_endgame = pos_score(wp.get_pieces_of_type(Pawn), &positional_tables::WHITE_PAWN_ENDGAME)
        - pos_score(bp.get_pieces_of_type(Pawn), &positional_tables::BLACK_PAWN_ENDGAME)
        + pos_score(wp.get_pieces_of_type(Knight), &positional_tables::WHITE_KNIGHT_ENDGAME)
        - pos_score(bp.get_pieces_of_type(Knight), &positional_tables::BLACK_KNIGHT_ENDGAME)
        + pos_score(wp.get_pieces_of_type(Bishop), &positional_tables::WHITE_BISHOP_ENDGAME)
        - pos_score(bp.get_pieces_of_type(Bishop), &positional_tables::BLACK_BISHOP_ENDGAME)
        + pos_score(wp.get_pieces_of_type(Queen), &positional_tables::WHITE_QUEEN_ENDGAME)
        - pos_score(bp.get_pieces_of_type(Queen), &positional_tables::BLACK_QUEEN_ENDGAME)
        + pos_score(wp.get_pieces_of_type(King), &positional_tables::WHITE_KING_ENDGAME)
        - pos_score(bp.get_pieces_of_type(King), &positional_tables::BLACK_KING_ENDGAME);  

    eval_data.score_opening += score_opening;
    eval_data.score_endgame += score_endgame;
}

fn calc_bishop_pair_bonus(eval_data: &mut EvalData) {
    let bonus_early = 20;
    let bonus_late = 60;

    let white_pair = (eval_data.wb >= 2) as i16;
    let black_pair = (eval_data.bb >= 2) as i16;
    
    eval_data.score_opening += bonus_early * white_pair - bonus_early * black_pair;
    eval_data.score_endgame += bonus_late * white_pair - bonus_late * black_pair;
}

fn calc_tempo(eval_data: &mut EvalData) {
    // Small bonus for having the right to move, only
    // in the early game
    eval_data.score_opening += 28;
}

fn pos_score(bb: BitBoard, pos_table: &[i16]) -> i16 {
    bb.piece_indices().map(|i| pos_table[i as usize]).sum()
}

///////////////////////////////////////////////////////////////////////////////

impl<'a> EvalData<'a> {
    pub fn new(board: &'a Board) -> Self {
        let black_pieces = board.get_pieces(Black);
        let bp = black_pieces.pawns.count() as i16;
        let br = black_pieces.rooks.count() as i16;
        let bn = black_pieces.knights.count() as i16;
        let bb = black_pieces.bishops.count() as i16;
        let bq = black_pieces.queens.count() as i16;

        let white_pieces = board.get_pieces(White);
        let wp = white_pieces.pawns.count() as i16;
        let wr = white_pieces.rooks.count() as i16;
        let wn = white_pieces.knights.count() as i16;
        let wb = white_pieces.bishops.count() as i16;
        let wq = white_pieces.queens.count() as i16;

        let mut res = Self {bp, br, bn, bb, bq, wp, wr, wn, wb, wq, board, game_phase: 0, score_endgame: 0, score_opening: 0};
        res.update_game_phase();
        res
    }

    pub fn compute_score(&self) -> Evaluation {
        // The values are temporarily promoted to i32 to avoid overflowing when
        // multiplying by the game phase
        let eval = ((self.score_opening as i32 * (256 - self.game_phase as i32)) + (self.score_endgame as i32 * self.game_phase as i32)) / 256;
        Evaluation::new(eval as i16 * self.board.turn_color().sign())
    }

    fn update_game_phase(&mut self) {
        let mut phase = 24;
        phase -= self.wn + self.bn + self.wb + self.bb;
        phase -= 2 * (self.wr + self.br);
        phase -= 4 * (self.wq + self.bq);
        self.game_phase = (phase * 256 + 12) / 24
    }
}


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
            write!(f, "M{}", i16::MAX - self.score())
        } else if self.is_negative_mate() {
            write!(f, "-M{}", self.score() - i16::MIN - 1)
        } else {
            write!(f, "{:+.2}", self.score() as f32 / 100.0)
        }
    }
}