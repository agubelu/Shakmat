use std::fmt::{Formatter, Display};
use std::ops::{Neg, Add, Sub};
use shakmat_core::{Board, Color::{*, self}, BitBoard, PieceType::{*, self}, move_gen};
use super::{piece_tables, EvalData, masks};

// TODO: change to i32 probably to avoid funny overflows
pub type EvalScore = i16;
pub type ScorePair = (i16, i16);

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

// Attack values for the different pieces for the outer and inner rings
const MINOR_PIECE_ATTACK: ScorePair = (8, 21);
const ROOK_ATTACK: ScorePair = (7, 18);
const QUEEN_ATTACK: ScorePair = (14, 33);

// Danger values for a king on a semi-open file or with semi-open flanks
const KING_SEMIOPEN_FILE_DANGER: i16 = 70;
const KING_SEMIOPEN_FLANK_DANGER: i16 = 50;

// King danger reduction if the opponent doesn't have a queen
const NO_QUEEN_DANGER_RED: i16 = 800;

// Penalties for a king under different attack values
const ATTACKED_PENALTIES: [i16; 64] = [0,0,-1,-2,-4,-6,-8,-11,-14,-18,-21,-25,-30,-35,-40,-45,-51,-57,-63,-69,-76,-83,-91,-98,-106,-114,-123,-132,-141,-150,-159,-169,-179,-189,-200,-211,-222,-233,-245,-257,-269,-281,-294,-306,-319,-333,-346,-360,-374,-388,-403,-418,-433,-448,-463,-479,-495,-511,-527,-544,-561,-578,-595,-613];

// Bonuses and penalties for the mobility of different pieces
const KNIGHT_MOBILITY_BONUS: [ScorePair; 9] = [(-62, -79), (-53, -57), (-12, -31), (-3, -17), (3, 7), (12, 13), (21, 16), (28, 21), (37, 26)];
const BISHOP_MOBILITY_BONUS: [ScorePair; 14] = [(-47, -59), (-20, -25), (14, -8), (29, 12), (39, 21), (53, 40), (53, 56), (60, 58), (62, 65), (69, 72), (78, 78), (83, 87), (91, 88), (96, 98)];
const ROOK_MOBILITY_BONUS: [ScorePair; 15] = [(-60, -82), (-24,-15), (0, 17), (3, 43), (4, 72), (14, 100), (20, 102), (30, 122), (41, 133), (41, 139), (41, 153), (45, 160), (57, 165), (58, 170), (67, 175)];
const QUEEN_MOBILITY_BONUS: [ScorePair; 28] = [(-29, -49), (-16, -29), (-8, -8), (-8, 17), (18, 39), (25, 54), (23, 59), (37, 73), (41, 76), (54, 95), (65, 95), (68, 101), (69, 124), (70, 128), (70, 132), (70, 133), (71, 136), (72, 140), (74, 147), (76, 149), (90, 153), (104, 169), (105, 171), (106, 171), (112, 178), (114, 185), (114, 187), (119, 221)];

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
// It's very important that we evaluate the different pieces in the current order,
// since some evaluation terms depend on things that are calculated during the
// evaluation of other pieces
fn calc_piece_score(eval_data: &mut EvalData) {  
    // Pawns go first, since we use their evaluation to update the squares
    // controlled by the pawns of both sides
    let (wp_mg, wp_eg) = eval_bitboard::<{White}, {Pawn}>(eval_data.white_pieces.pawns, eval_data);
    let (bp_mg, bp_eg) = eval_bitboard::<{Black}, {Pawn}>(eval_data.black_pieces.pawns, eval_data);

    let (wb_mg, wb_eg) = eval_bitboard::<{White}, {Bishop}>(eval_data.white_pieces.bishops, eval_data);
    let (bb_mg, bb_eg) = eval_bitboard::<{Black}, {Bishop}>(eval_data.black_pieces.bishops, eval_data);

    let (wn_mg, wn_eg) = eval_bitboard::<{White}, {Knight}>(eval_data.white_pieces.knights, eval_data);
    let (bn_mg, bn_eg) = eval_bitboard::<{Black}, {Knight}>(eval_data.black_pieces.knights, eval_data);

    let (wr_mg, wr_eg) = eval_bitboard::<{White}, {Rook}>(eval_data.white_pieces.rooks, eval_data);
    let (br_mg, br_eg) = eval_bitboard::<{Black}, {Rook}>(eval_data.black_pieces.rooks, eval_data);

    let (wq_mg, wq_eg) = eval_bitboard::<{White}, {Queen}>(eval_data.white_pieces.queens, eval_data);
    let (bq_mg, bq_eg) = eval_bitboard::<{Black}, {Queen}>(eval_data.black_pieces.queens, eval_data);

    // The king goes always last, because many king safety terms depend on 
    // the squares attacked by the previous pieces
    let (wk_mg, wk_eg) = eval_bitboard::<{White}, {King}>(eval_data.white_pieces.king, eval_data);
    let (bk_mg, bk_eg) = eval_bitboard::<{Black}, {King}>(eval_data.black_pieces.king, eval_data);

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
fn eval_pawn<const COLOR: Color>(pos: u8, _: BitBoard, eval_data: &mut EvalData) -> ScorePair {
    let mut mg = PAWN_BASE_VALUE;
    let mut eg = PAWN_BASE_VALUE;
    let them = (!COLOR).to_index();

    // Check the squares controlled by this pawn
    let attack_bb = move_gen::pawn_attacks(pos as usize, COLOR);
    eval_data.safe_mobility_area[them] &= !attack_bb;

    // Check if this is a passed pawn, and add bonuses acordingly
    let (enemy_pawns, passed_mask, rel_rank) = match COLOR {
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
    let our_pawns = match COLOR {
        Black => eval_data.black_pieces.pawns,
        White => eval_data.white_pieces.pawns,
    };
    if (attack_bb & our_pawns).is_not_empty() {
        let bonus = CONNECTED_PAWN_BONUS[rel_rank as usize];
        mg += bonus;
        eg += bonus;
    }

    (mg, eg)
}

fn eval_bishop<const COLOR: Color>(pos: u8, _: BitBoard, eval_data: &mut EvalData) -> ScorePair {
    let (mut mg, mut eg) = (BISHOP_BASE_VALUE, BISHOP_BASE_VALUE);
    let us = COLOR.to_index();

    // Check if this bishop attacks the enemy king rings.
    // X-ray attacks: bishops can see through queens, so we remove them
    // when calculating bishop attacks to the enemy king
    let our_queens_mask = !eval_data.board.get_pieces(COLOR).queens;
    let attack_bb = move_gen::bishop_moves(pos as usize, eval_data.board.get_all_bitboard() & our_queens_mask);
    add_attack_values::<COLOR>(attack_bb, eval_data, MINOR_PIECE_ATTACK);

    // Calculate the mobility score for this bishop
    let moves = move_gen::bishop_moves(pos as usize, eval_data.board.get_all_bitboard());
    let safe_moves = (moves & eval_data.safe_mobility_area[us]).count() as usize;

    let (mg_mob_bonus, eg_mob_bonus) = BISHOP_MOBILITY_BONUS[safe_moves];
    mg += mg_mob_bonus;
    eg += eg_mob_bonus;

    (mg, eg)
}

fn eval_knight<const COLOR: Color>(pos: u8, _: BitBoard, eval_data: &mut EvalData) -> ScorePair {
    let (mut mg, mut eg) = (KNIGHT_BASE_VALUE, KNIGHT_BASE_VALUE);
    let us = COLOR.to_index();

    // Check if this knight attacks the enemy king ring.
    let attack_bb = move_gen::knight_moves(pos as usize);
    add_attack_values::<COLOR>(attack_bb, eval_data, MINOR_PIECE_ATTACK);

    // Calculate the mobility score for this knight
    let safe_moves = (attack_bb & eval_data.safe_mobility_area[us]).count() as usize;

    let (mg_mob_bonus, eg_mob_bonus) = KNIGHT_MOBILITY_BONUS[safe_moves];
    mg += mg_mob_bonus;
    eg += eg_mob_bonus;

    (mg, eg)
}

fn eval_rook<const COLOR: Color>(pos: u8, bb: BitBoard, eval_data: &mut EvalData) -> ScorePair {
    let mut mg = ROOK_BASE_VALUE;
    let mut eg = ROOK_BASE_VALUE;
    let us = COLOR.to_index();

    // Check if this rook attacks the enemy king ring.
    // X-ray attacks: rooks can see through queens and other rooks, so we remove them
    // when calculating rook attacks to the enemy king
    let our_pieces_mask = !(eval_data.board.get_pieces(COLOR).queens | bb);
    let attack_bb = move_gen::rook_moves(pos as usize, eval_data.board.get_all_bitboard() & our_pieces_mask);
    add_attack_values::<COLOR>(attack_bb, eval_data, ROOK_ATTACK);

    // Calculate the mobility score for this rook
    let moves = move_gen::rook_moves(pos as usize, eval_data.board.get_all_bitboard());
    let safe_moves = (moves & eval_data.safe_mobility_area[us]).count() as usize;

    let (mg_mob_bonus, eg_mob_bonus) = ROOK_MOBILITY_BONUS[safe_moves];
    mg += mg_mob_bonus;
    eg += eg_mob_bonus;

    let file = masks::file(pos);
    let (friendly_pawns, enemy_pawns) = match COLOR {
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

fn eval_queen<const COLOR: Color>(pos: u8, _: BitBoard, eval_data: &mut EvalData) -> ScorePair {
    let (mut mg, mut eg) = (QUEEN_BASE_VALUE, QUEEN_BASE_VALUE);
    let us = COLOR.to_index();

    // Check if this queen attacks the enemy king ring.
    let attack_bb = move_gen::queen_moves(pos as usize, eval_data.board.get_all_bitboard());
    add_attack_values::<COLOR>(attack_bb, eval_data, QUEEN_ATTACK);

    // Calculate the mobility score for this queen
    let safe_moves = (attack_bb & eval_data.safe_mobility_area[us]).count() as usize;

    let (mg_mob_bonus, eg_mob_bonus) = QUEEN_MOBILITY_BONUS[safe_moves];
    mg += mg_mob_bonus;
    eg += eg_mob_bonus;

    (mg, eg)
}

// There are approximately 99999 ways to evaluate a king's safety, so here we
// follow the path of our lord and savior Stockfish and compute a safety value
// by multiplying the number of attackers with the total weight of their attacks
fn eval_king<const COLOR: Color>(pos: u8, _: BitBoard, eval_data: &mut EvalData) -> ScorePair {
    let (mut mg, eg) = (0, 0);
    let enemy = !COLOR;
    let our_pawns = match COLOR {
        Black => eval_data.black_pieces.pawns,
        White => eval_data.white_pieces.pawns,
    };

    let file = pos % 8;
    let king_file_mask = masks::file(pos);

    // Calculate the threat score from the attacks from other pieces
    let us = COLOR.to_index();
    let mut threat = eval_data.attacks_weight[us];

    // Assignate a penalty if the king is in a semi-open file
    if (our_pawns & king_file_mask).is_empty() {
        threat += KING_SEMIOPEN_FILE_DANGER;
    }

    // Penalty if the king has semi-open flanks to its sides
    // The right flank is analyzed if the king is not on the H file
    if file != 0 && (our_pawns & (king_file_mask >> 1)).is_empty() {
        threat += KING_SEMIOPEN_FLANK_DANGER;
    }

    // And the left flank is analyzed if the king is not on the A file
    if file != 7 && (our_pawns & (king_file_mask << 1)).is_empty() {
        threat += KING_SEMIOPEN_FLANK_DANGER;
    }

    // Reduce king danger if the enemy doesn't have a queen
    let enemy_queens = eval_data.get_pieces(enemy).queens;
    threat -= NO_QUEEN_DANGER_RED * enemy_queens.is_empty() as i16;

    // Index the king safety penalty using the threat value and
    // setting it to 0 if it's negative
    let threat_index = threat.max(0);
    mg += ATTACKED_PENALTIES[(threat_index as usize / 8).min(ATTACKED_PENALTIES.len() - 1)];

    (mg, eg)
}

///////////////////////////////////////////////////////////////////////////////
/// Aux function to evaluate a whole bitboard of pieces of a given type
fn eval_bitboard<const PIECE_COLOR: Color, const PIECE_TYPE: PieceType>(bb: BitBoard, eval_data: &mut EvalData) -> ScorePair {
    let eval_func = match PIECE_TYPE {
        Pawn => eval_pawn::<PIECE_COLOR>,
        Knight => eval_knight::<PIECE_COLOR>,
        Bishop => eval_bishop::<PIECE_COLOR>,
        Rook => eval_rook::<PIECE_COLOR>,
        Queen => eval_queen::<PIECE_COLOR>,
        King => eval_king::<PIECE_COLOR>,
    };

    bb.piece_indices()
      .map(|i| eval_func(i, bb, eval_data))
      .fold((0, 0), |a, b| (a.0 + b.0, a.1 + b.1))
}

///////////////////////////////////////////////////////////////////////////////
/// Aux function to add attack values from a certain piece to the enemy king
fn add_attack_values<const COLOR: Color>(attack_bb: BitBoard, eval_data: &mut EvalData, weights: ScorePair) {
    let enemy = !COLOR;
    let enemy_i = enemy.to_index();
    let outer_ring_attacks = (attack_bb & eval_data.king_outer_rings[enemy_i]).count();
    let inner_ring_attacks = (attack_bb & eval_data.king_inner_rings[enemy_i]).count();
    eval_data.attacks_weight[enemy_i] = outer_ring_attacks as i16 * weights.0 + inner_ring_attacks as i16 * weights.1;
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