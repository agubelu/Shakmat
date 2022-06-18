use shakmat_core::{Board, Pieces, BitBoard, Color::{*, self}};
use super::{Evaluation, EvalScore, masks};

// Auxiliary struct to store values that are used in different parts
// of the evaluation, to avoid calculating them multiple times
pub struct EvalData<'a> {
    pub board: &'a Board,
    pub game_phase: EvalScore,
    pub score_midgame: EvalScore,
    pub score_endgame: EvalScore,
    pub white_pieces: &'a Pieces,
    pub black_pieces: &'a Pieces,

    // Info about king position and attackers
    pub king_inner_rings: [BitBoard; 2],
    pub king_outer_rings: [BitBoard; 2],
    pub attackers_count: [EvalScore; 2],
    pub attacks_weight: [EvalScore; 2],

    // Info about the safe mobility squares, i.e., not controlled by enemy pawns 
    pub safe_mobility_area: [BitBoard; 2],
}


impl<'a> EvalData<'a> {
    pub fn new(board: &'a Board) -> Self {
        let black_pieces = board.get_pieces(Black);
        let br = black_pieces.rooks.count() as EvalScore;
        let bn = black_pieces.knights.count() as EvalScore;
        let bb = black_pieces.bishops.count() as EvalScore;
        let bq = black_pieces.queens.count() as EvalScore;

        let white_pieces = board.get_pieces(White);
        let wr = white_pieces.rooks.count() as EvalScore;
        let wn = white_pieces.knights.count() as EvalScore;
        let wb = white_pieces.bishops.count() as EvalScore;
        let wq = white_pieces.queens.count() as EvalScore;

        let attackers_count = [0; 2];
        let attacks_weight = [0; 2];
        let black_king_pos = board.get_pieces(Black).king.first_piece_index();
        let white_king_pos = board.get_pieces(White).king.first_piece_index();

        // Arrays: Always [black, white]
        let king_inner_rings = [masks::king_inner_ring(black_king_pos),
                                masks::king_inner_ring(white_king_pos)]; 
        let king_outer_rings = [masks::king_outer_ring(black_king_pos),
                                masks::king_outer_ring(white_king_pos)]; 
        let safe_mobility_area = [BitBoard::ones(); 2];

        let mut game_phase = 24;
        game_phase -= wn + bn + wb + bb;
        game_phase -= 2 * (wr + br);
        game_phase -= 4 * (wq + bq);
        game_phase = (game_phase * 256 + 12) / 24;

        Self {board, white_pieces, black_pieces, safe_mobility_area,
             attackers_count, attacks_weight, king_inner_rings, king_outer_rings,
             game_phase, score_endgame: 0, score_midgame: 0}
    }

    pub fn compute_score(&self) -> Evaluation {
        // The values are temporarily promoted to i32 to avoid overflowing when
        // multiplying by the game phase
        let eval = ((self.score_midgame as i32 * (256 - self.game_phase as i32)) + (self.score_endgame as i32 * self.game_phase as i32)) / 256;
        Evaluation::new(eval as EvalScore * self.board.turn_color().sign() as EvalScore)
    }

    pub fn get_pieces(&self, color: Color) -> &Pieces {
        match color {
            Black => self.black_pieces,
            White => self.white_pieces,
        }
    }
}