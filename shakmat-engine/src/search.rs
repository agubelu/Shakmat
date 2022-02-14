use shakmat_core::{Board, Move, PieceType};

use crate::evaluation::{evaluate_position, Evaluation};
use crate::trasposition::{TTable, TTEntry, NodeType};

// Number of entries of the trasposition table.
const TRASPOSITION_TABLE_SIZE: usize = 1 << 22;

// The contempt factor is the score that the engine associates with a draw.
// A negative value means that the engine assumes it is superior to its opponent,
// so drawing is penalized. Conversely, a positive value means that the engine assumes
// itself to be inferior, so it encourages drawing when it cannot find a decisive advantage.
const CONTEMPT: i16 = 0;

// Struct to hold a pair of evaluation and best move, so we can return the current evaluation to
// the front-end in addition to the best move
pub struct SearchResult {
    pub score: Evaluation,
    pub best_move: Option<Move>,
}

// Struct to hold a pair of (Move, move heuristical value)
struct RatedMove {
    pub mv: Move,
    pub score: i16
}

// Wrapper function over the negamax algorithm, returning the best move
// along with the associated score
pub fn find_best(board: &Board, max_depth: u8, past_positions: &[u64]) -> SearchResult {
    let trans_table = TTable::new(TRASPOSITION_TABLE_SIZE);
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
        score = negamax(board, depth, 0, alpha, beta, &trans_table, &mut history);

        // Aspiration window: TO-DO comment
        if score <= alpha {
            println!("Alpha fail, depth {}", depth);
            alpha = Evaluation::min_val();
            continue;
        }

        if score >= beta {
            println!("Beta fail, depth {}", depth);
            beta = Evaluation::max_val();
            continue;
        }

        alpha = score - window;
        beta = score + window;
        
        println!("Depth {}, score {}", depth, score);
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
    past_positions: &mut Vec<u64>
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
    if is_draw_by_repetition(board, current_depth, past_positions) || 
       board.fifty_move_rule_counter() >= 100 || 
       board.is_draw_by_material() {
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
        return quiesence_search(board, alpha, beta, trans_table);
    }

    let mut best_score = Evaluation::min_val();
    let mut best_move = None;
    let mut node_type = NodeType::AlphaCutoff;

    // We use the pseudolegal move generator to construct the new board ourselves
    // and filter out moves that result in illegal positions. This is exactly what
    // board.legal_moves() does, so this way we avoid doing it twice.
    let moves = board.pseudolegal_moves();

    let mut analyzed_moves = 0;

    for RatedMove{mv, ..} in order_moves(moves, board, tt_move) {
        let next_board = board.make_move(&mv, false).unwrap();

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
            -negamax(&next_board, depth_remaining - 1, current_depth + 1, -beta, -alpha, trans_table, past_positions)
        } else {
            // Try a minimal window first. If the value falls under [alpha, beta] then use the standard window
            let mut temptative_score = -negamax(&next_board, depth_remaining - 1, current_depth + 1, (-alpha)-1, -alpha, trans_table, past_positions);

            if temptative_score > alpha && temptative_score < beta {
                // Do a full evaluation since the position was not significantly worsened
                temptative_score = -negamax(&next_board, depth_remaining - 1, current_depth + 1, -beta, -temptative_score, trans_table, past_positions)
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
    best_score
}

// The quiesence search is a simplified version of the negamax search that only
// expands captures. This runs in terminal nodes in the standard search, and mitigates
// the horizon effect by making sure that we are not misevaluating a position where
// a piece is hanging and can be easily captured in the next move.
fn quiesence_search(board: &Board, mut alpha: Evaluation, beta: Evaluation, trans_table: &TTable) -> Evaluation {
    let static_score = evaluate_position(board);

    if static_score >= beta {
        return beta;
    } else if static_score > alpha {
        alpha = static_score;
    }

    // Only consider moves that are captures or pawn promotions
    let moves = board.pseudolegal_caps();
    for RatedMove{mv, ..} in order_moves(moves, board, None) {
        // As in the normal search, we are using pseudolegal moves, so we must make sure that
        // the moving side is not in check. Castling moves are not generated now so we
        // don't have to worry about them
        let next_board = board.make_move(&mv, false).unwrap();
        if next_board.is_check(board.turn_color()) {
            continue;
        }

        let next_score = -quiesence_search(&next_board, -beta, -alpha, trans_table);

        if next_score >= beta {
            return beta;
        } else if next_score > alpha {
            alpha = next_score;
        }
    }

    alpha
}

fn order_moves(moves: Vec<Move>, board: &Board, tt_move: Option<Move>) -> Vec<RatedMove> {
    let mut rated_moves: Vec<RatedMove> = moves.into_iter().map(|mv| rate_move(mv, tt_move, board)).collect();
    rated_moves.sort_unstable_by_key(|rm| -rm.score);
    rated_moves
}

// Takes a move by value and returns a struct with that move
// and its heuristic value. PV moves are rated the highest, then captures
fn rate_move(mv: Move, pv_move: Option<Move>, board: &Board) -> RatedMove {
    let score = if pv_move == Some(mv) {
        10_000 // PV move, should be evaluated first
    } else if let Some(captured) = mv.piece_captured(board) {
        value_of_capture(captured) - value_of_attacker(mv.piece_moving(board))
    } else {
        0
    };

    RatedMove { mv, score }
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

// Most Valuable Victim - Least Valuable Aggressor (MVV-LVA)
// Attempts to provide a heuristic for capturing moves by
// capturing with the least valuable piece
const fn value_of_attacker(piece: PieceType) -> i16 {
    match piece {
        PieceType::Pawn => 10,
        PieceType::Knight => 30,
        PieceType::Bishop => 30,
        PieceType::Rook => 50,
        PieceType::Queen => 90,
        PieceType::King => 99,
    }
}

const fn value_of_capture(piece: PieceType) -> i16 {
    match piece {
        PieceType::Pawn => 100,
        PieceType::Knight => 300,
        PieceType::Bishop => 300,
        PieceType::Rook => 500,
        PieceType::Queen => 900,
        PieceType::King => 9999, // Doesn't happen since the king is never captured
    }
}
