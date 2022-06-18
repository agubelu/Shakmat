use shakmat_core::{Board, Pieces, BitBoard, Color::{*, self}};
use super::{Evaluation, masks};

// Auxiliary struct to store values that are used in different parts
// of the evaluation, to avoid calculating them multiple times
pub struct EvalData<'a> {
    pub board: &'a Board,
    pub game_phase: i16,
    pub score_midgame: i16,
    pub score_endgame: i16,
    pub white_pieces: &'a Pieces,
    pub black_pieces: &'a Pieces,

    // Info about king position and attackers
    pub king_inner_rings: [BitBoard; 2],
    pub king_outer_rings: [BitBoard; 2],
    pub attackers_count: [i16; 2],
    pub attacks_weight: [i16; 2],

    // Info about the safe mobility squares, i.e., not controlled by enemy pawns 
    pub safe_mobility_area: [BitBoard; 2],

    // Count of pieces of a certain type for every side
    // Do I really need these in the future...?
    pub wp: i16, pub wr: i16, pub wb: i16, pub wn: i16, pub wq: i16,
    pub bp: i16, pub br: i16, pub bb: i16, pub bn: i16, pub bq: i16,
}


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

        let mut res = Self {bp, br, bn, bb, bq, wp, wr, wn, wb, wq,
             board, white_pieces, black_pieces, safe_mobility_area,
             attackers_count, attacks_weight, king_inner_rings, king_outer_rings,
             game_phase: 0, score_endgame: 0, score_midgame: 0};
        res.update_game_phase();
        res
    }

    pub fn compute_score(&self) -> Evaluation {
        // The values are temporarily promoted to i32 to avoid overflowing when
        // multiplying by the game phase
        let eval = ((self.score_midgame as i32 * (256 - self.game_phase as i32)) + (self.score_endgame as i32 * self.game_phase as i32)) / 256;
        Evaluation::new(eval as i16 * self.board.turn_color().sign())
    }

    fn update_game_phase(&mut self) {
        let mut phase = 24;
        phase -= self.wn + self.bn + self.wb + self.bb;
        phase -= 2 * (self.wr + self.br);
        phase -= 4 * (self.wq + self.bq);
        self.game_phase = (phase * 256 + 12) / 24
    }

    pub fn get_pieces(&self, color: Color) -> &Pieces {
        match color {
            Black => self.black_pieces,
            White => self.white_pieces,
        }
    }
}