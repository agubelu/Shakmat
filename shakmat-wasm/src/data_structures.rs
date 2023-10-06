use shakmat_core::Board;
use wasm_bindgen::prelude::*;

/** Basic information about the current turn in a given board. */
#[wasm_bindgen(getter_with_clone)]
pub struct TurnInfo {
    pub turn_number: u16,
    pub color: String,
    pub moves: Vec<JsValue>, /* Strings put into JSValues. Apparently,      */
    pub in_check: bool,      /* returning Vec<String> is *almost* supported */
    pub fen: String,         /* but not fully as of coding this.            */
}

/** Best move and evaluation by the engine. */
#[wasm_bindgen(getter_with_clone)]
pub struct SearchResult {
    pub best_move: Option<String>,
    pub eval: i16,
}

impl TurnInfo {
    pub fn from_board(board: &Board, history: &[u64]) -> Self {
        let moves = if shakmat_engine::is_draw_by_repetition(board, 0, history) {
            vec![]
        } else {
            board.legal_moves().into_iter().map(|mv| mv.to_string().into()).collect()
        };

        Self {
            turn_number: board.turn_number(),
            color: board.turn_color().to_string(),
            in_check: board.is_check(board.turn_color()),
            fen: board.fen(),
            moves
        }
    }
}
