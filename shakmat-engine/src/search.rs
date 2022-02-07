use std::cmp::{max, min};
use shakmat_core::{Board, Move};

use crate::evaluation::{evaluate_position, Evaluation};
use crate::trasposition::{TTable, TTEntry, NodeType};

// Number of entries of the trasposition table.
const TRASPOSITION_TABLE_SIZE: usize = 1 << 22;

// The contempt factor is the score that the engine associates with a draw.
// A negative value means that the engine assumes it is superior to its opponent,
// so drawing is penalized. Conversely, a positive value means that the engine assumes
// itself to be inferior, so it encourages drawing when it cannot find a decisive advantage.
const CONTEMPT: i16 = 0;

// Struct to hold a pair of evaluation and best move. Hopefully this can be removed
// in the future and make the negamax work with only evaluations, and grab the best
// move from the PV (TO-DO)
pub struct NegamaxResult {
    pub eval: Evaluation,
    pub best: Option<Move>,
}

// Wrapper function over the negamax algorithm, returning the best move
// along with the associated score
pub fn find_best(board: &Board, depth: u8, past_positions: &[u64]) -> NegamaxResult {
    let trans_table = TTable::new(TRASPOSITION_TABLE_SIZE);
    // The array of zobrist keys corresponding to all past positions is cloned so that
    // the search function can take ownership of it, adding and removing new positions
    // during the search process.
    negamax(board, depth, 0, Evaluation::min_val(), Evaluation::max_val(), &trans_table, &mut past_positions.to_vec())
}

pub fn negamax(
    board: &Board, 
    mut depth_remaining: u8, 
    current_depth: u8, 
    mut alpha: Evaluation,
    mut beta: Evaluation, 
    trans_table: &TTable,
    past_positions: &mut Vec<u64>
) -> NegamaxResult {

    let zobrist = board.zobrist_key();
    // Check whether the current position is in the trasposition table. Getting the
    // entry itself from the table is unsafe since there will be lockless concurrent
    // access (in the future), however, the .get_entry() method does some sanity
    // checks and only returns an entry if the data inside it is valid and the
    // stored zobrist key matches.
    if let Some(tt_data) = trans_table.get_entry(zobrist) {
        // Use the data contained in the entry depending on the type of node that
        // this is, and only if the depth is >= the current one
        if tt_data.depth >= depth_remaining {
            let stored_score = tt_data.eval_score();
            match tt_data.node_type() {
                NodeType::Exact => return NegamaxResult::new(stored_score, *tt_data.best_move()),
                NodeType::AlphaCutoff => alpha = max(alpha, stored_score),
                NodeType::BetaCutoff => beta = min(beta, stored_score),
            };

            // Check whether the evaluation window has closed completely
            if alpha >= beta {
                return NegamaxResult::new(stored_score, *tt_data.best_move());
            }
        }
    }

    // If this is an immediate draw, we don't have to do anything else
    if is_draw_by_repetition(board, current_depth, past_positions) {
        return NegamaxResult::new(Evaluation::new(CONTEMPT), None);
    }

    // The current position is not stored, perform the full search from here.
    // If the current side to move is in check, extend the search by 1 more move to
    // avoid misevaluating dangerous positions
    let color_moving = board.turn_color();
    if board.is_check(color_moving) {
        depth_remaining += 1;
    }

    // If we are on a leaf node, use the static evaluation and return it right away.
    if depth_remaining == 0 {
        let score = evaluate_position(board);
        trans_table.write_entry(zobrist, TTEntry::new(zobrist, depth_remaining, score, NodeType::Exact, None));
        return NegamaxResult::new(score, None);
    }

    let mut best_score = Evaluation::min_val();
    let mut best_move = None;
    let mut node_type = NodeType::AlphaCutoff;

    // We use the pseudolegal move generator to construct the new board ourselves
    // and filter out moves that result in illegal positions. This is exactly what
    // board.legal_moves() does, so this way we avoid doing it twice.
    for mv in board.pseudolegal_moves() {
        let next_board = board.make_move(&mv, false).unwrap();

        // This is a pseudo-legal move, we must make sure that the side moving is not in check.
        // Castling moves are always legal since their legality is checked in move generation,
        // for anything else, we must check that the moving side isn't in check
        if matches!(mv, Move::Normal{..} | Move::PawnPromotion{..}) && next_board.is_check(color_moving) {
            continue;
        }

        // Update the vec of past positions with the current zobrist key before the recursive call
        past_positions.push(zobrist);

        // Evaluate the next position recursively and update the current best score
        let next_score = -negamax(&next_board, depth_remaining - 1, current_depth + 1, -beta, -alpha, trans_table, past_positions).eval;

        // We're done calling recursively, remove the current state from the history
        past_positions.pop();

        if next_score > best_score {
            best_move = Some(mv);
            best_score = next_score;
        }

        if best_score > alpha {
            alpha = best_score;
            node_type = NodeType::Exact;
        }

        if best_score >= beta {
            node_type = NodeType::BetaCutoff;
            break;
        }
    }

    // If the evaluation hasn't changed from the worst possible one, no legal moves
    // are available. Check whether this is a checkmate or a stalemate and assign
    // the corresponding score.
    if best_move.is_none() {
        best_score = if board.is_check(color_moving) {
            // Checkmate
            Evaluation::min_val() + current_depth as i16
        } else {
            // Stalemate or other cause of draw
            Evaluation::new(CONTEMPT)
        };
    }

    // Update the transposition table with the information that we have obtained
    // for this position
    trans_table.write_entry(zobrist, TTEntry::new(zobrist, depth_remaining, best_score, node_type, best_move));

    NegamaxResult::new(best_score, best_move)
}

// Determines if a given position is a draw by repetition considering the previous history.
// This function returns true if the current state is either:
// - The third repetition of a position, where the previous two have happened
//   during the previous moves that have been played
// - The second repetition of a position that occured entirely during the search process
// This is because we assume that if a position has repeated twice during
// the search, it is likely that a third repetition will occur, so we save time.
pub fn is_draw_by_repetition(board: &Board, cur_depth: u8, history: &[u64]) -> bool {
    let current_zobrist = board.zobrist_key();
    let mut rep_count = 1;

    // We don't actually have to consider all past states. Moves which update the
    // 50 move rule are irreversible, and thus no repetitions can occur before them.
    let last_irr_move = board.current_ply() - board.fifty_move_rule_counter();

    // This is a board state that occured during the search, so we're a number of moves
    // ahead of the actual game. Determine the last ply that was actually played so we
    // know if we should stop searching at 2 repetitions or 3 (see comment above the function)
    let last_played_ply = board.current_ply() - cur_depth as u16;

    let prev_states = history.iter()
        .copied() // Copy the u64 references into this iter
        .enumerate() // Associate each board state with the (0-based) ply in which it occured
        .skip(last_irr_move as usize) // Fast forward to the last irreversible state of the board
        .rev() // Start with the most recent move and go backwards
        .step_by(2) // We only need to consider every other state, since reps can only
                    // occur when the side to play is the same as the current one
        .skip(1); // We don't need to consider the current state 

    for (ply, zobrist) in prev_states {
        if zobrist == current_zobrist { // We have a repetition!
            rep_count += 1;
            // Stop if we're still inside the search and it's the second rep,
            // or if it's the third one
            if rep_count == 2 && ply as u16 > last_played_ply || rep_count == 3 {
                return true;
            }
        }
    };
        
    false
}

impl NegamaxResult {
    pub fn new(eval: Evaluation, best: Option<Move>) -> Self {
        Self { eval, best }
    }
}