use shakmat_core::{Board, Move, PieceType::*};
use std::cmp::{min, max};

use super::move_ordering::{order_moves, RatedMove, MoveScore};
use super::pv_line::PVLine;
use super::history::HistoryTable;
use crate::evaluation::{evaluate_position, Evaluation, EvalScore};
use crate::trasposition::{TTable, TTEntry, NodeType};
use crate::time::TimeManager;

// Number of entries of the trasposition table.
const TRASPOSITION_TABLE_SIZE: usize = 1 << 22;

// The maximum depth that will be reached under any circumstances
const LIMIT_DEPTH: usize = 100;

// Number of killer moves to store in each ply
const MAX_KILLERS: usize = 2;

// Depth to reduce a null move search. Maybe try dynamic values in the future?
const NULL_MOVE_REDUCTION: u8 = 2;

// Width for the aspiration window
const ASP_WINDOW: EvalScore = 30;

// The amount that a score must drop between iterations for
// panic time to be allocated
const PANIC_DROP: EvalScore = 30;

// Number of legal moves after which to start applying late move reductions
const LMR_MOVES: usize = 1;

// Score margins for futility pruning
const FUTILIY_MARGINS: [EvalScore; 6] = [0, 100, 160, 220, 280, 340];

// Score margin for reverse futility pruning, scaling with depth
const REV_FUTILITY_MARGIN: EvalScore = 80;

// Typedef for the killer moves table
pub type Killers = [[Move; MAX_KILLERS]; LIMIT_DEPTH + 2];

// Typedef for the pair (alpha, beta) of score bounds
pub type Bounds = (Evaluation, Evaluation);

// Calculates the number of quiet moves to explore for a certain depth
// const fn max_movecount(depth: u8) -> usize {
//     3 + (depth as usize * depth as usize)
// }

// The Search struct contains all necessary parameters for the search and stores
// relevant information between iterations. All search-related functions
// are implemented as methods of this struct.
pub struct Search {
    timer: TimeManager,
    max_depth: u8,
    past_positions: Vec<u64>,
    killers: Killers,
    tt: TTable,
    node_count: u32,
    history: HistoryTable,
}

// The SearchConfig struct contains a series of parameters for the search
pub struct SearchOptions {
    pub total_time_remaining: Option<u64>, // Milliseconds remaining in our clock
    pub moves_until_control: Option<u64>, // Moves remaining until the next time control stage
    pub time_for_move: Option<u64>, // Millis designated for this move, overrides previous two
    pub max_depth: Option<u8>, // Maximum depth for the search
}

// SearchResult a pair of evaluation and best move, so we can return the current evaluation to
// the front-end in addition to the best move
pub struct SearchResult {
    pub score: Evaluation,
    pub best_move: Option<Move>,
}

impl Search {
    pub fn from_config(config: SearchOptions, past_positions: &[u64]) -> Self {
        Self {
            timer: TimeManager::new(&config),
            max_depth: min(config.max_depth.unwrap_or(LIMIT_DEPTH as u8), LIMIT_DEPTH as u8),
            tt: TTable::new(TRASPOSITION_TABLE_SIZE),
            killers: [[Move::empty(); MAX_KILLERS]; LIMIT_DEPTH + 2],
            node_count: 0,
            past_positions: past_positions.to_vec(),
            history: HistoryTable::new(),
        }
    }

    // Wrapper function over the negamax algorithm, returning the best move
    // along with the associated score
    pub fn find_best(&mut self, board: &Board) -> SearchResult {
        // If there is only one legal move, return it immediately
        let legal_moves = board.legal_moves();
        if legal_moves.len() == 1 {
            return SearchResult { score: Evaluation::new(0), best_move: Some(legal_moves[0]) };
        }

        let mut previous_score = Evaluation::new(0);
        let mut score = Evaluation::min_val();
        let mut best_move = None;

        let mut alpha = Evaluation::min_val();
        let mut beta = Evaluation::max_val();

        // Iterative deepening: instead of diving directly into a search of depth `max_depth`,
        // increase the depth by 1 every time. This may seem counter-intuitive, but it actually
        // makes it run faster. The reason is that we can use the best move from the previous
        // search as the temptative best move in this one in the move ordering, which makes
        // the alpha-beta pruning remove many more branches during the search.
        let mut depth = 1;

        // The PV line found by the engine
        let mut pv_line = PVLine::new();

        while depth <= self.max_depth && !self.timer.times_up() {
            let t_start = self.timer.elapsed_micros();
            score = self.negamax(board, depth, 0, (alpha, beta), true, &mut pv_line);
            let search_time = self.timer.elapsed_micros() - t_start;

            // If we ran out of time during the search, stop and
            // return the score from the previous one
            if self.timer.times_up() {
                score = previous_score;
                break;
            }

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

            // The best move will be the first one in the PV line
            best_move = pv_line.first();

            // If the currest best score is a forced mate, either for us or for
            // the opponent, return the move right away.
            if score.is_mate() {
                break;
            }

            // The score dropped a worrying amount w.r.t. the last iteration,
            // add some extra time to make sure we investigate it and maybe
            // find a better move
            if depth > 3 && previous_score - score >= PANIC_DROP {
                let worry = (previous_score - score).score() / PANIC_DROP;
                println!("{}", "😰".to_owned().repeat(min(worry as usize, 10))); // 😰
                self.timer.add_panic_time();
            }

            // It is reasonable to assume that the search time increases with
            // increasing depth. So, if the last search took more time than
            // the time we have remaining, and we are not given a hard time
            // limit to make this move, save time by avoiding entering a search
            // that will most likely be interrupted
            if !self.timer.hard_limit() && search_time > self.timer.remaining_micros() {
                break;
            }

            alpha = score - ASP_WINDOW;
            beta = score + ASP_WINDOW;
            previous_score = score;
            depth += 1;
        }

        if depth > self.max_depth {
            // We exited the loop after reaching the maximum depth,
            // reduce it by 1 as it is the depth that was
            // actually reached
            depth -= 1;
        }

        // Print some stats before returning the result
        let total_micros = self.timer.elapsed_micros();
        let knps = self.node_count as u64 * 1_000 / total_micros;
        println!("KNPS: {}, max. depth: {}", knps, depth);

        SearchResult { score, best_move }
    }

    fn negamax(
        &mut self,
        board: &Board, 
        mut depth_remaining: u8, 
        current_depth: u8, 
        (mut alpha, mut beta): Bounds,
        can_null: bool,
        pv_line: &mut PVLine,
    ) -> Evaluation {
        self.node_count += 1;

        // If, for some reason, we go past the limit depth, return the static
        // evaluation value right away.
        if current_depth >= LIMIT_DEPTH as u8 {
            return evaluate_position(board);
        }

        // Update the timer every 4096 nodes. Using a power of 2 makes things
        // faster since we can use a bitwise AND to check instead of modulo
        // (Rust does compile modulo N == 0 to bitwise ANDs when N is
        // a power of 2, but we do it explicitly anyways)
        if self.node_count & 4095 == 0 {
            self.timer.update();
        }

        // If we ran out of time, exit immediately returning whatever. The value
        // will not be used anyway, since the best move from a search is only
        // used if there is time remaining after the search finishes, to avoid
        // using the result of an unfinished search.
        if self.timer.times_up() {
            return Evaluation::new(0);
        }

        // Check whether the current position is in the trasposition table. Getting the
        // entry itself from the table is unsafe since there will be lockless concurrent
        // access (in the future), however, the .get_entry() method does some sanity
        // checks and only returns an entry if the data inside it is valid and the
        // stored zobrist key matches.
        let mut tt_move = None;
        let zobrist = board.zobrist_key();
        if let Some(tt_data) = self.tt.get_entry(zobrist, depth_remaining, &mut tt_move) {
            let tt_score = tt_data.eval_score();
            match tt_data.node_type() {
                NodeType::Exact => return tt_score,
                NodeType::Lowerbound => alpha = max(alpha, tt_score),
                NodeType::Upperbound => beta = min(beta, tt_score),
            };

            if alpha >= beta {
                return tt_score;
            }
        }

        // If this is an immediate draw, we don't have to do anything else
        if is_draw_by_repetition(board, current_depth, &self.past_positions) {
            return Evaluation::contempt();
        }

        // The current position is not stored, perform the full search from here.
        // If the current side to move is in check, extend the search by 1 more move to
        // avoid misevaluating dangerous positions and prevent the search from
        // entering in quiesence mode
        let color_moving = board.turn_color();
        let is_check = board.is_check(color_moving);
        if is_check {
            depth_remaining += 1;
        }

        // If we are on a leaf node, use the quiesence search to make sure the
        // static evaluation is reliable
        if depth_remaining == 0 {
            return self.quiesence_search(board, current_depth, alpha, beta, pv_line);
        }

        let is_pv = beta - alpha != 1;

        // Reverse futility pruning: if the current score exceeds what the
        // opponent can already guarantee, even if we substract a margin from it,
        // we can assume that they will not allow this position and prune it.
        // TO-DO: probably add a depth condition to avoid calling the evaluation
        // in early depths where the margin is huge and see how that works
        if !is_pv && !is_check && !beta.is_mate() {
            let score = evaluate_position(board);
            let margin = depth_remaining as EvalScore * REV_FUTILITY_MARGIN;
            let reduced = score - margin;

            if reduced > beta {
                return reduced;
            }
        }

        // PV line for the recursive calls
        let mut next_pv_line = PVLine::new();

        // Null move pruning: pass the turn, and see if the opponent can improve
        // their position playing two turns in a row doing a reduced depth
        // search. If they can't, we can assume that they will not allow this
        // position to happen, and we can produce a beta cutoff. Special care
        // must be given to not use this kind of pruning in positions where the
        // current side to move is in check (it would be illegal), or in late
        // game positions where not moving is actually the best move. Also, don't
        // do it in positions close to the horizon.

        if can_null && !is_check && depth_remaining > NULL_MOVE_REDUCTION && !board.only_pawns_or_endgame() && !is_pv {
            let new_board = board.make_null_move();
            let score = -self.negamax(&new_board, depth_remaining - NULL_MOVE_REDUCTION - 1, current_depth + 1, (-beta, -beta + 1), false, &mut next_pv_line);

            // If the opponent can't improve their position, return beta
            if score >= beta && !score.is_positive_mate() {
                return beta;
            // If we get checkmated if we don't do anything, increase the depth 
            } else if score.is_negative_mate() {
                depth_remaining += 1;
            }
        }

        // Futility pruning: if we are close to the horizon, and the score
        // is very far away from alpha, it is unlikely that we will improve it,
        // to prune everything except the PV. However, don't do this in tactical
        // positions such as checks and in the PV.
        let mut do_futility = false;
        if (depth_remaining as usize)  < FUTILIY_MARGINS.len() && !is_pv && !is_check
        && !alpha.is_mate() {
            let eval = evaluate_position(board);
            if eval + FUTILIY_MARGINS[depth_remaining as usize] < alpha {
                do_futility = true;
            }
        }

        let mut best_score = Evaluation::min_val();
        let mut best_move = None;
        let mut node_type = NodeType::Upperbound;

        // We use the pseudolegal move generator to construct the new board ourselves
        // and filter out moves that result in illegal positions. This is exactly what
        // board.legal_moves() does, so this way we avoid doing it twice.
        let moves = board.pseudolegal_moves();
        let mut analyzed_moves = 0;
        let rated_moves = order_moves(moves, board, tt_move, &self.killers[current_depth as usize], &self.history);

        // A list with the quiet (non-capture) moves that we have analyzed
        let mut analyzed_quiets = Vec::with_capacity(64);

        for RatedMove{mv, ..} in rated_moves {
            let next_board = board.make_move(&mv);

            // This is a pseudo-legal move, we must make sure that the side moving is not in check.
            // Castling moves are always legal since their legality is checked in move generation,
            // for anything else, we must check that the moving side isn't in check
            if matches!(mv, Move::Normal{..} | Move::PawnPromotion{..}) && next_board.is_check(color_moving) {
                continue;
            }

            // Some information about this move
            let is_capture = mv.is_capture(board);
            let cap_or_prom = is_capture || matches!(mv, Move::PawnPromotion{..});
            let gives_check = next_board.is_check(next_board.turn_color());
            let is_pawn_move = mv.piece_moving(board) == Pawn;
            let is_tactical = is_check || gives_check || cap_or_prom || is_pawn_move || self.is_killer(&mv, current_depth);

            // Late move pruning: in non-PV nodes, skip late quiet moves since they are less
            // likely to be interesting. The closer we are to the horizon, the more
            // moves we prune. However, we only do that when there are non-pawn pieces on
            // the board, not in the root node, and we're not under a checkmate threat
            // Not working very well ATM
            // if !is_pv && !is_tactical && current_depth != 0 && !board.only_pawns()
            //    && !best_score.is_negative_mate() && analyzed_moves >= max_movecount(depth_remaining) {
            //     continue;
            // }

            // Futility pruning, part 2: if we decided earlier that we can
            // use this pruning, and the current move is not a tactical one,
            // skip this move entirely unless its the first one (PV)
            if do_futility && analyzed_moves != 0 && !is_tactical {
                continue;
            }

            // Update the vec of past positions with the current zobrist key before the recursive calls
            self.past_positions.push(zobrist);

            // Late move reduction: Moves after the first one are less likely
            // to be interesting, so we search them with a reduced depth and
            // window. However, the following moves are not reduced: checks in general,
            // captures, promotions, PV nodes, shallow depth, killers and pawn moves
            // Also, we never reduce at the root
            let mut red = 0;
            if !is_pv && !is_tactical && depth_remaining >= 3 && analyzed_moves >= LMR_MOVES && current_depth != 0 {
                // The base reduction starts at 2 because it's one more than the
                // usual reduction in depth by 1 when calling recursively
                // The reduction increases by 1 for each 5 moves after the LMR move limit.
                red = 2 + (analyzed_moves - LMR_MOVES) as u8 / 5;

                // Make sure that we don't reduce directly into quiesence search
                if red >= depth_remaining {
                    red = depth_remaining - 1;
                }
            }
            
            // The score for the current move
            let mut score = Evaluation::new(0);

            // Whether we must do a full fledged search, which is true
            // by default unless the reduced search fails low and we
            // can skip it
            let mut do_full_depth = true;

            // If we are reducing, try to search with reduced depth first
            if red != 0 {
                score = -self.negamax(&next_board, depth_remaining - red, current_depth + 1, ((-alpha)-1, -alpha), true, &mut next_pv_line);
                // If the reduced search fails low, we don't have to search using full depth
                do_full_depth = score > alpha;
            }

            // Since the moves are ordered, only evaluate the first move with a full window
            if analyzed_moves == 0 {
                score = -self.negamax(&next_board, depth_remaining - 1, current_depth + 1, (-beta, -alpha), true, &mut next_pv_line);
            } else if do_full_depth {
                // Try a minimal window first. If the value falls under [alpha, beta] then use the standard window
                score = -self.negamax(&next_board, depth_remaining - 1, current_depth + 1, ((-alpha)-1, -alpha), true, &mut next_pv_line);

                if score > alpha && score < beta {
                    // Do a full evaluation since the position was not significantly worsened
                    score = -self.negamax(&next_board, depth_remaining - 1, current_depth + 1, (-beta, -alpha), true, &mut next_pv_line);
                }
            };

            // We're done calling recursively, remove the current state from the history
            self.past_positions.pop();
            analyzed_moves += 1;

            // Update alpha, beta and the scores
            if score > best_score {
                // This move improves our previous score, update the score
                // and the current new move with the PV line
                best_move = Some(mv);
                best_score = score;
                pv_line.update_line(mv, &mut next_pv_line);
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
                node_type = NodeType::Lowerbound;
                break;
            }

            // If the current move is not the best and it's quiet, store it
            if Some(mv) != best_move && !is_capture {
                analyzed_quiets.push(mv);
            }

            // Clear the next PV line for the following iteration
            next_pv_line.clear();
        }

        // Check the time again after the recursive calls. The value returned
        // if the time is up doesn't matter, as explained above.
        if self.timer.times_up() {
            return Evaluation::new(0);
        }

        // If we have a best move, update history stats and killers
        if let Some(bm) = best_move {
            self.update_histories(&bm, &analyzed_quiets, board, depth_remaining);
        } else {
            // Otherwise, there are no legal moves available.
            // Check whether this is a checkmate or a draw, and assign
            // the corresponding score.
            best_score = if board.is_check(color_moving) {
                // Checkmate
                Evaluation::min_val() + current_depth as EvalScore
            } else {
                // Stalemate or other cause of draw
                Evaluation::contempt()
            };
        }

        // Update the transposition table with the information that we have obtained
        // for this position
        self.tt.write_entry(zobrist, TTEntry::new(zobrist, depth_remaining, best_score, node_type, best_move));
        best_score
    }

    // The quiesence search is a simplified version of the negamax search that only
    // expands captures. This runs in terminal nodes in the standard search, and mitigates
    // the horizon effect by making sure that we are not misevaluating a position where
    // a piece is hanging and can be easily captured in the next move.
    fn quiesence_search(
        &mut self, 
        board: &Board, 
        current_depth: u8, 
        mut alpha: Evaluation, 
        beta: Evaluation,
        pv_line: &mut PVLine
    ) -> Evaluation {
        self.node_count += 1;

        // If, for some reason, we go past the limit depth, return the static
        // evaluation value right away.
        if current_depth >= LIMIT_DEPTH as u8 {
            return evaluate_position(board);
        }

        // Update the timer every 4096 nodes.
        if self.node_count & 4095 == 0 {
            self.timer.update();
        }

        // If we ran out of time, exit immediately returning whatever. The value
        // will not be used anyway, since the best move from a search is only
        // used if there is time remaining after the search finishes, to avoid
        // using the result of an unfinished search.
        if self.timer.times_up() {
            return Evaluation::new(0);
        }

        let static_score = evaluate_position(board);

        if static_score >= beta {
            return beta;
        } else if static_score > alpha {
            alpha = static_score;
        }

        let mut next_pv_line = PVLine::new();

        // Only consider moves that are captures or pawn promotions
        let moves = board.pseudolegal_caps();
        let rated_moves = order_moves(moves, board, None, &self.killers[current_depth as usize], &self.history);
        for RatedMove{mv, ..} in rated_moves {
            // As in the normal search, we are using pseudolegal moves, so we must make sure that
            // the moving side is not in check. Castling moves are not generated now so we
            // don't have to worry about them
            let next_board = board.make_move(&mv);
            if next_board.is_check(board.turn_color()) {
                continue;
            }

            let next_score = -self.quiesence_search(&next_board, current_depth + 1, -beta, -alpha, &mut next_pv_line);

            if next_score >= beta {
                return beta;
            } else if next_score > alpha {
                alpha = next_score;
                // Update the PV line
                pv_line.update_line(mv, &mut next_pv_line);
            }

            // Clear the next PV line for the next iteration
            next_pv_line.clear();
        }

        alpha
    }

    fn is_killer(&self, mv: &Move, depth: u8) -> bool {
        self.killers[depth as usize][0] == *mv || self.killers[depth as usize][1] == *mv
    }

    fn update_histories(&mut self, best_move: &Move, quiet_moves: &[Move], board: &Board, depth: u8) {
        // We only need to update histories if the best move is a quiet one
        if !best_move.is_capture(board) {
            let color = board.turn_color();
            let bonus = (depth as MoveScore) * (depth as MoveScore);
            // Increase stats for the best move and store it as a killer
            self.history.add_bonus(best_move, color, bonus);
            let i = depth as usize;
            if *best_move != self.killers[i][0] {
                self.killers[i][1] = self.killers[i][0];
                self.killers[i][0] = *best_move;
            }

            // Decrease history stats for the other quiet moves
            quiet_moves.iter().for_each(|mv| {
                self.history.add_bonus(mv, color, -bonus);
            });
        }
    }
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            total_time_remaining: None,
            moves_until_control: None,
            time_for_move: None,
            max_depth: Some(7),
        }
    }
}

///////////////////////////////////////////////////////////////////////////////
// Some auxiliary functions:

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
