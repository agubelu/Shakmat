use std::fmt::{Formatter, Display};
use std::ops::{Neg, Add, Sub};
use shakmat_core::{Board, Color::{*, self}, BitBoard, PieceType::{*, self}, move_gen};
use super::{piece_tables, EvalData, masks};

// TODO: Note to self: in the future, refactor all of this using const enum generics
// for the evaluation functions, to reduce duplicities
// TODO: change to i32 probably to avoid funny overflows

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
pub type ScorePair = (i16, i16);

const PAWN_BASE_VALUE: i16 = 100;
const BISHOP_BASE_VALUE: i16 = 300;
const KNIGHT_BASE_VALUE: i16 = 300;
const ROOK_BASE_VALUE: i16 = 500;
const QUEEN_BASE_VALUE: i16 = 900;

const TEMPO_BONUS: i16 = 28;
const BISHOP_PAIR_BONUS: ScorePair = (20, 60);
const ROOK_OPEN_FILE_BONUS: ScorePair = (50, 25);
const ROOK_SEMIOPEN_FILE_BONUS: ScorePair = (20, 10);
const ROOK_CLOSED_FILE_PENALTY: ScorePair = (-10, -5);
const PASSED_PAWN_BONUS: [ScorePair; 7] = [
    (0, 0), (10, 1), (5, 5), (1, 25), (15, 50), (50, 100), (100, 150)
];
const CONNECTED_PAWN_BONUS: [i16; 7] = [0, 5, 10, 10, 15, 55, 85];

const KNIGHT_ATTACK_WEIGHT: i16 = 15;
const BISHOP_ATTACK_WEIGHT: i16 = 45;
const ROOK_ATTACK_WEIGHT: i16 = 50;
const QUEEN_ATTACK_WEIGHT: i16 = 75;

// Evaluate how favorable a position is for the current side to move
// We always calculate it so that positive scores favor white, while
// negative scores favor black.
// eval_data.compute_score() adapts the final sign to make it from
// the point of view of the current player.
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
// It's very important that we evaluate the king last, since the king safety
// evaluation depends on data that will be saved in the eval_data by other pieces
fn calc_piece_score(eval_data: &mut EvalData) {  
    let (wp_mg, wp_eg) = eval_bitboard(eval_data.white_pieces.pawns, Pawn, White, eval_data);
    let (bp_mg, bp_eg) = eval_bitboard(eval_data.black_pieces.pawns, Pawn, Black, eval_data);

    let (wb_mg, wb_eg) = eval_bitboard(eval_data.white_pieces.bishops, Bishop, White, eval_data);
    let (bb_mg, bb_eg) = eval_bitboard(eval_data.black_pieces.bishops, Bishop, Black, eval_data);

    let (wn_mg, wn_eg) = eval_bitboard(eval_data.white_pieces.knights, Knight, White, eval_data);
    let (bn_mg, bn_eg) = eval_bitboard(eval_data.black_pieces.knights, Knight, Black, eval_data);

    let (wr_mg, wr_eg) = eval_bitboard(eval_data.white_pieces.rooks, Rook, White, eval_data);
    let (br_mg, br_eg) = eval_bitboard(eval_data.black_pieces.rooks, Rook, Black, eval_data);

    let (wq_mg, wq_eg) = eval_bitboard(eval_data.white_pieces.queens, Queen, White, eval_data);
    let (bq_mg, bq_eg) = eval_bitboard(eval_data.black_pieces.queens, Queen, Black, eval_data);

    // Always last!
    let (wk_mg, wk_eg) = eval_bitboard(eval_data.white_pieces.king, King, White, eval_data);
    let (bk_mg, bk_eg) = eval_bitboard(eval_data.black_pieces.king, King, Black, eval_data);

    eval_data.score_midgame += wp_mg + wb_mg + wn_mg + wr_mg + wq_mg + wk_mg - bp_mg - bb_mg - bn_mg - br_mg - bq_mg - bk_mg;
    eval_data.score_endgame += wp_eg + wb_eg + wn_eg + wr_eg + wq_eg + wk_eg - bp_eg - bb_eg - bn_eg - br_eg - bq_eg - bk_eg;
}

// Gives an extra centipoint for each square controlled, and 2 points
// for each one in the endgame stage.
fn calc_control_score(eval_data: &mut EvalData) {
    let control_white = eval_data.board.get_attack_bitboard(White).count() as i16;
    let control_black = eval_data.board.get_attack_bitboard(Black).count() as i16;
    eval_data.score_midgame += control_white - control_black;
    eval_data.score_endgame += 2 * (control_white - control_black);
}

// Gives positional bonuses to each piece using the corresponding table,
// for both the middlegame and endgame phases.
fn calc_positional_score(eval_data: &mut EvalData) {
    let wp = eval_data.white_pieces;
    let bp = eval_data.black_pieces;

    add_pos_scores(eval_data, wp.pawns, &piece_tables::WHITE_PAWNS);
    add_pos_scores(eval_data, wp.rooks, &piece_tables::WHITE_ROOKS);
    add_pos_scores(eval_data, wp.knights, &piece_tables::WHITE_KNIGHTS);
    add_pos_scores(eval_data, wp.bishops, &piece_tables::WHITE_BISHOPS);
    add_pos_scores(eval_data, wp.queens, &piece_tables::WHITE_QUEENS);
    add_pos_scores(eval_data, wp.king, &piece_tables::WHITE_KING);

    sub_pos_scores(eval_data, bp.pawns, &piece_tables::BLACK_PAWNS);
    sub_pos_scores(eval_data, bp.rooks, &piece_tables::BLACK_ROOKS);
    sub_pos_scores(eval_data, bp.knights, &piece_tables::BLACK_KNIGHTS);
    sub_pos_scores(eval_data, bp.bishops, &piece_tables::BLACK_BISHOPS);
    sub_pos_scores(eval_data, bp.queens, &piece_tables::BLACK_QUEENS);
    sub_pos_scores(eval_data, bp.king, &piece_tables::BLACK_KING);
}

fn calc_bishop_pair_bonus(eval_data: &mut EvalData) {
    let bonus_early = BISHOP_PAIR_BONUS.0;
    let bonus_late = BISHOP_PAIR_BONUS.1;

    let white_pair = (eval_data.wb >= 2) as i16;
    let black_pair = (eval_data.bb >= 2) as i16;
    
    eval_data.score_midgame += bonus_early * white_pair - bonus_early * black_pair;
    eval_data.score_endgame += bonus_late * white_pair - bonus_late * black_pair;
}

fn calc_tempo(eval_data: &mut EvalData) {
    // Small bonus for having the right to move, only
    // in the early game
    eval_data.score_midgame += TEMPO_BONUS;
}

///////////////////////////////////////////////////////////////////////////////
/// Specialized functions for each piece type
fn eval_pawn(pos: u8, _: BitBoard, color: Color, eval_data: &mut EvalData) -> ScorePair {
    let mut mg = PAWN_BASE_VALUE;
    let mut eg = PAWN_BASE_VALUE;
    let enemy = (!color).to_index();

    // Check if this pawn attacks the enemy king ring. Pawns have no attack weight
    // but still count towards the attacker count
    let attack_bb = move_gen::pawn_attacks(pos as usize, color);
    if (attack_bb & eval_data.king_rings[enemy]).is_not_empty() {
        eval_data.attackers_count[enemy] += 1;
    }

    // Check if this is a passed pawn, and add bonuses acordingly
    let (enemy_pawns, passed_mask, rel_rank) = match color {
        White => (eval_data.black_pieces.pawns, masks::white_passed_pawn(pos), pos / 8),
        Black => (eval_data.white_pieces.pawns, masks::black_passed_pawn(pos), 7 - (pos / 8)),
    };

    if (enemy_pawns & passed_mask).is_empty() {
        // This pawn is a passer, assign a bonus depending on its relative rank
        let (mg_bonus, eg_bonus) = PASSED_PAWN_BONUS[rel_rank as usize];
        mg += mg_bonus;
        eg += eg_bonus;
    }

    // Check if this pawn is connected to friendly pawns
    let our_pawns = eval_data.get_pieces(color).pawns;
    if (attack_bb & our_pawns).is_not_empty() {
        let bonus = CONNECTED_PAWN_BONUS[rel_rank as usize];
        mg += bonus;
        eg += bonus;
    }

    (mg, eg)
}

fn eval_bishop(pos: u8, bb: BitBoard, color: Color, eval_data: &mut EvalData) -> ScorePair {
    // Check if this bishop attacks the enemy king ring.
    let attack_bb = move_gen::bishop_moves(pos as usize, eval_data.board.get_all_bitboard());
    let enemy = (!color).to_index();
    if (attack_bb & eval_data.king_rings[enemy]).is_not_empty() {
        eval_data.attackers_count[enemy] += 1;
        eval_data.attacks_weight[enemy] += BISHOP_ATTACK_WEIGHT;
    }

    (BISHOP_BASE_VALUE, BISHOP_BASE_VALUE)
}

fn eval_knight(pos: u8, bb: BitBoard, color: Color, eval_data: &mut EvalData) -> ScorePair {
    // Check if this knight attacks the enemy king ring.
    let attack_bb = move_gen::knight_moves(pos as usize);
    let enemy = (!color).to_index();
    if (attack_bb & eval_data.king_rings[enemy]).is_not_empty() {
        eval_data.attackers_count[enemy] += 1;
        eval_data.attacks_weight[enemy] += KNIGHT_ATTACK_WEIGHT;
    }

    (KNIGHT_BASE_VALUE, KNIGHT_BASE_VALUE)
}

fn eval_rook(pos: u8, bb: BitBoard, color: Color, eval_data: &mut EvalData) -> ScorePair {
    let mut mg = ROOK_BASE_VALUE;
    let mut eg = ROOK_BASE_VALUE;

    // Check if this rook attacks the enemy king ring.
    let attack_bb = move_gen::rook_moves(pos as usize, eval_data.board.get_all_bitboard());
    let enemy = (!color).to_index();
    if (attack_bb & eval_data.king_rings[enemy]).is_not_empty() {
        eval_data.attackers_count[enemy] += 1;
        eval_data.attacks_weight[enemy] += ROOK_ATTACK_WEIGHT;
    }

    let file = masks::file(pos);
    let (friendly_pawns, enemy_pawns) = match color {
        White => (eval_data.white_pieces.pawns, eval_data.black_pieces.pawns),
        Black => (eval_data.black_pieces.pawns, eval_data.white_pieces.pawns),
    };

    // Check if the rook is in a closed, semi-open or open file
    if (file & friendly_pawns).is_not_empty() {
        // Friendly pawns on this file, we consider it closed and substract a penalty
        mg += ROOK_CLOSED_FILE_PENALTY.0;
        eg += ROOK_CLOSED_FILE_PENALTY.1;
    } else if (file & enemy_pawns).is_not_empty() {
        // Only enemy pawns, we consider it semi-open and add a bonus
        mg += ROOK_SEMIOPEN_FILE_BONUS.0;
        eg += ROOK_SEMIOPEN_FILE_BONUS.1;
    } else {
        // No pawns, we consider it open
        mg += ROOK_OPEN_FILE_BONUS.0;
        eg += ROOK_OPEN_FILE_BONUS.1;
    }

    (mg, eg)
}

fn eval_queen(pos: u8, bb: BitBoard, color: Color, eval_data: &mut EvalData) -> ScorePair {
    // Check if this queen attacks the enemy king ring.
    let attack_bb = move_gen::queen_moves(pos as usize, eval_data.board.get_all_bitboard());
    let enemy = (!color).to_index();
    if (attack_bb & eval_data.king_rings[enemy]).is_not_empty() {
        eval_data.attackers_count[enemy] += 1;
        eval_data.attacks_weight[enemy] += QUEEN_ATTACK_WEIGHT;
    }

    (QUEEN_BASE_VALUE, QUEEN_BASE_VALUE)
}

// There are approximately 99999 ways to evaluate a king's safety, so here we
// follow the path of our lord and savior Stockfish and compute a safety value
// by multiplying the number of attackers with the total weight of their attacks
fn eval_king(pos: u8, bb: BitBoard, color: Color, eval_data: &mut EvalData) -> ScorePair {
    // Calculate the threat score
    /*let us = color.to_index();
    let threat = eval_data.attackers_count[us] * eval_data.attacks_weight[us];*/

    let (mut mg, mut eg) = (0, 0);

    /*if threat > 100 {
        let k = threat as i32;
        mg -= ((k * k) / 4096) as i16;
        eg -= threat / 16;
    }*/

    (mg, eg)
}

///////////////////////////////////////////////////////////////////////////////
/// Aux function to evaluate a whole bitboard of pieces of a given type
fn eval_bitboard(bb: BitBoard, piece_type: PieceType, color: Color, eval_data: &mut EvalData) -> ScorePair {
    let eval_func = match piece_type {
        Pawn => eval_pawn,
        Knight => eval_knight,
        Bishop => eval_bishop,
        Rook => eval_rook,
        Queen => eval_queen,
        King => eval_king,
    };

    bb.piece_indices()
        .map(|i| eval_func(i, bb, color, eval_data))
        .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1))
}

///////////////////////////////////////////////////////////////////////////////
/// Aux functions to add/substract positional scores
fn add_pos_scores(eval_data: &mut EvalData, bb: BitBoard, table: &[ScorePair]) {
    bb.piece_indices().for_each(|pos| {
        // All positions are <64, so it's safe to skip bounds checking
        let (mg, eg) = unsafe { table.get_unchecked(pos as usize) };
        eval_data.score_midgame += mg;
        eval_data.score_endgame += eg;
    });
}

fn sub_pos_scores(eval_data: &mut EvalData, bb: BitBoard, table: &[ScorePair]) {
    bb.piece_indices().for_each(|pos| {
        // All positions are <64, so it's safe to skip bounds checking
        let (mg, eg) = unsafe { table.get_unchecked(pos as usize) };
        eval_data.score_midgame -= mg;
        eval_data.score_endgame -= eg;
    });
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