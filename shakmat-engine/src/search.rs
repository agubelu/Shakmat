use shakmat_core::{Board, Move};

use crate::evaluation::{evaluate_position, Evaluation};
use crate::move_ordering::{order_moves, RatedMove};
use crate::trasposition::{TTable, TTEntry, NodeType};

// Number of entries of the trasposition table.
const TRASPOSITION_TABLE_SIZE: usize = 1 << 22;

// The contempt factor is the score that the engine associates with a draw.
// A negative value means that the engine assumes it is superior to its opponent,
// so drawing is penalized. Conversely, a positive value means that the engine assumes
// itself to be inferior, so it encourages drawing when it cannot find a decisive advantage.
const CONTEMPT: i16 = 0;

// The maximum depth that will be reached under any circumstances
const MAX_DEPTH: usize = 100;

// Number of killer moves to store in each ply
const MAX_KILLERS: usize = 2;

// Typedef for the killer moves table
pub type Killers = [[Move; MAX_KILLERS]; MAX_DEPTH];

// Struct to hold a pair of evaluation and best move, so we can return the current evaluation to
// the front-end in addition to the best move
pub struct SearchResult {
    pub score: Evaluation,
    pub best_move: Option<Move>,
}

// Wrapper function over the negamax algorithm, returning the best move
// along with the associated score
pub fn find_best(board: &Board, max_depth: u8, past_positions: &[u64]) -> SearchResult {
    // Refactor all of this to have a Search struct in the future...
    let trans_table = TTable::new(TRASPOSITION_TABLE_SIZE);
    let mut killers = [[Move::empty(); MAX_KILLERS]; MAX_DEPTH];
    let mut score = Evaluation::min_val();

    let mut alpha = Evaluation::min_val();
    let mut beta = Evaluation::max_val();
    let window = 30;

    // Iterative deepening: instead of diving directly into a search of depth `max_depth`,
    // increase the depth by 1 every time. This may seem counter-intuitive, but it actually
    // makes it run faster. The reason is that we can use the best move from the previous
    // search as the temptative best move in this one in the move ordering, which makes
    // the alpha-beta pruning remove many more branches during the search.
    let mut depth = 1;
    while depth <= max_depth {
        // The array of zobrist keys corresponding to all past positions is cloned so that
        // the search function can take ownership of it, adding and removing new positions
        // during the search process.
        let mut history = past_positions.to_vec();
        score = negamax(board, depth, 0, alpha, beta, &trans_table, &mut history, &mut killers);

        // Aspiration windows: the score is unlikely to change a lot between iterations,
        // so we use a window margin around the last score to use as alpha and beta,
        // hoping that this will cause more cutoffs. However, if the score ends up
        // under alpha or over beta, then we must search again using the full window
        // size as the search result is not reliable.
        if score <= alpha {
            alpha = Evaluation::min_val();
            continue;
        }

        if score >= beta {
            beta = Evaluation::max_val();
            continue;
        }

        alpha = score - window;
        beta = score + window;
        depth += 1;
    }

    // The best move will be stored in the corresponding entry in the transposition table.
    // Because we use an "always-replace" scheme, it is guaranteed that the best
    // move for the root position will be stored there when the search finishes.
    let mut best_move = None;

    // The call to tt.get_entry() writes to the best_move parameter
    trans_table.get_entry(board.zobrist_key(), 0, Evaluation::min_val(), Evaluation::max_val(), &mut best_move);

    SearchResult { score, best_move }
}

pub fn negamax(
    board: &Board, 
    mut depth_remaining: u8, 
    current_depth: u8, 
    mut alpha: Evaluation,
    beta: Evaluation, 
    trans_table: &TTable,
    past_positions: &mut Vec<u64>,
    killers: &mut Killers
) -> Evaluation {
    // Check whether the current position is in the trasposition table. Getting the
    // entry itself from the table is unsafe since there will be lockless concurrent
    // access (in the future), however, the .get_entry() method does some sanity
    // checks and only returns an entry if the data inside it is valid and the
    // stored zobrist key matches.
    let mut tt_move = None;
    let zobrist = board.zobrist_key();
    if let Some(eval) = trans_table.get_entry(zobrist, depth_remaining, alpha, beta, &mut tt_move) {
        return eval
    }

    // If this is an immediate draw, we don't have to do anything else
    if is_draw_by_repetition(board, current_depth, past_positions) {
        return Evaluation::new(CONTEMPT);
    }

    // The current position is not stored, perform the full search from here.
    // If the current side to move is in check, extend the search by 1 more move to
    // avoid misevaluating dangerous positions and prevent the search from
    // entering in quiesence mode
    let color_moving = board.turn_color();
    if board.is_check(color_moving) {
        depth_remaining += 1;
    }

    // If we are on a leaf node, use the quiesence search to make sure the
    // static evaluation is reliable
    if depth_remaining == 0 {
        return quiesence_search(board, current_depth, alpha, beta, trans_table, killers);
    }

    let mut best_score = Evaluation::min_val();
    let mut best_move = None;
    let mut node_type = NodeType::AlphaCutoff;

    // We use the pseudolegal move generator to construct the new board ourselves
    // and filter out moves that result in illegal positions. This is exactly what
    // board.legal_moves() does, so this way we avoid doing it twice.
    let moves = board.pseudolegal_moves();

    let mut analyzed_moves = 0;

    for RatedMove{mv, ..} in order_moves(moves, board, tt_move, &killers[current_depth as usize]) {
        let next_board = board.make_move(&mv);

        // This is a pseudo-legal move, we must make sure that the side moving is not in check.
        // Castling moves are always legal since their legality is checked in move generation,
        // for anything else, we must check that the moving side isn't in check
        if matches!(mv, Move::Normal{..} | Move::PawnPromotion{..}) && next_board.is_check(color_moving) {
            continue;
        }

        // Update the vec of past positions with the current zobrist key before the recursive call
        past_positions.push(zobrist);

        // Since the moves are ordered, only evaluate the first move with a full window
        let next_score = if analyzed_moves == 0 {
            -negamax(&next_board, depth_remaining - 1, current_depth + 1, -beta, -alpha, trans_table, past_positions, killers)
        } else {
            // Try a minimal window first. If the value falls under [alpha, beta] then use the standard window
            let mut temptative_score = -negamax(&next_board, depth_remaining - 1, current_depth + 1, (-alpha)-1, -alpha, trans_table, past_positions, killers);

            if temptative_score > alpha && temptative_score < beta {
                // Do a full evaluation since the position was not significantly worsened
                temptative_score = -negamax(&next_board, depth_remaining - 1, current_depth + 1, -beta, -temptative_score, trans_table, past_positions, killers)
            }

            temptative_score
        };

        // We're done calling recursively, remove the current state from the history
        past_positions.pop();
        analyzed_moves += 1;

        // Update alpha, beta and the scores
        if next_score > best_score {
            // This move improves our previous score, update the score
            // and the current new move
            best_move = Some(mv);
            best_score = next_score;
        }

        if best_score > alpha {
            // This move improves the past best score we can get in the search
            alpha = best_score;
            node_type = NodeType::Exact;
        }

        if best_score >= beta {
            // This move is "too good", its score is higher than what our
            // opponent can guarantee earlier in the search. So, we assume
            // that they will avoid this position, and stop evaluating it.
            node_type = NodeType::BetaCutoff;

            // Check if the current move is a killer move, and in that case,
            // store it. Note that we must pass the *previous* board, to
            // determine if the move was a capture
            store_possible_killer(current_depth, board, mv, killers);
            break;
        }
    }

    // If we have no best move, no legal moves  are available. 
    // Check whether this is a checkmate or a draw, and assign
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
    best_score
}

// The quiesence search is a simplified version of the negamax search that only
// expands captures. This runs in terminal nodes in the standard search, and mitigates
// the horizon effect by making sure that we are not misevaluating a position where
// a piece is hanging and can be easily captured in the next move.
fn quiesence_search(board: &Board, current_depth: u8, mut alpha: Evaluation, beta: Evaluation, trans_table: &TTable, killers: &mut Killers) -> Evaluation {
    let static_score = evaluate_position(board);

    if static_score >= beta {
        return beta;
    } else if static_score > alpha {
        alpha = static_score;
    }

    // Only consider moves that are captures or pawn promotions
    let moves = board.pseudolegal_caps();
    for RatedMove{mv, ..} in order_moves(moves, board, None, &killers[current_depth as usize]) {
        // As in the normal search, we are using pseudolegal moves, so we must make sure that
        // the moving side is not in check. Castling moves are not generated now so we
        // don't have to worry about them
        let next_board = board.make_move(&mv);
        if next_board.is_check(board.turn_color()) {
            continue;
        }

        let next_score = -quiesence_search(&next_board, current_depth + 1, -beta, -alpha, trans_table, killers);

        if next_score >= beta {
            return beta;
        } else if next_score > alpha {
            alpha = next_score;
        }
    }

    alpha
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

fn store_possible_killer(depth: u8, board: &Board, mv: Move, killers: &mut Killers) {
    // The move caused a beta cutoff. If it's a quiet move (i.e. it doesn't capture anything),
    // then it is a killer move and it must be stored if it isn't there already
    if !mv.is_capture(board) {
        let i = depth as usize;
        if mv != killers[i][0] {
            killers[i][1] = killers[i][0];
            killers[i][0] = mv;
        }
    }
}
