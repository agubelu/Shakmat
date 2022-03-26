use shakmat_core::{Board, Pieces, Color::*};
use super::Evaluation;

// Auxiliary struct to store values that are used in different parts
// of the evaluation, to avoid calculating them multiple times
pub struct EvalData<'a> {
    pub board: &'a Board,
    pub game_phase: i16,
    pub score_opening: i16,
    pub score_endgame: i16,
    pub white_pieces: &'a Pieces,
    pub black_pieces: &'a Pieces,
    // Count of pieces of a certain type for every side
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

        let mut res = Self {bp, br, bn, bb, bq, wp, wr, wn, wb, wq,
             board, white_pieces, black_pieces,
             game_phase: 0, score_endgame: 0, score_opening: 0};
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