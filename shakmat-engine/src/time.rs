use std::time::Instant;
use std::cmp::min;
use crate::search::SearchOptions;

// Offset in microseconds to substract from the allocated time,
// based on my observations. TO-DO: does this really make sense?
const OFFSET: u64 = 10_000;
pub struct TimeManager {
    unlimited: bool, // Whether we have unlimited time to make a move
    allocated_micros: u64, // Amount of us that we have calculated that we 
                           // can spend on this move
    total_remaining: u64, // Total time remaining in micros
    start: Instant, // Instant in which the time started counting
    finished: bool, // Whether the allocated time has passed
    hard_limit: bool, // Whether we are given a hard time limit for the move
}

impl TimeManager {
    pub fn new(options: &SearchOptions) -> Self {
        let mut allocated_micros = 0;
        let mut total_remaining = 0;
        let mut unlimited = false;
        let mut hard_limit = false;

        if let Some(time) = options.time_for_move {
            // We are given a specific value *in millis* for the time we have to
            // make this move, use that value
            allocated_micros = time * 1000 - OFFSET;
            hard_limit = true;
        } else if options.total_time_remaining.is_none() {
            // We are not given a time remaining, so we have
            // unlimited time
            unlimited = true;
        } else {
            // We do have a time remaining:
            total_remaining = options.total_time_remaining.unwrap() * 1000;

            // If we also have the amount of moves until time control,
            // divide that amount over the time remaining to know the
            // average time per move we have. Otherwise, assume that
            // the game will keep going on for 40 more moves.
            let moves_remaining = options.moves_until_control.unwrap_or(40);

            // Aim to make a move in 80% of that time, so that we have
            // some extra time later on if we need to allocate panic time.
            allocated_micros = total_remaining / moves_remaining * 4 / 5 - OFFSET;
        }

        Self { allocated_micros, total_remaining, unlimited, hard_limit, start: Instant::now(), finished: false }
    }

    pub fn add_panic_time(&mut self) {
        // If the search requests to allocate extra time, we increment the 
        // allowed time by 30%, but never to the point where we would use more
        // than 20% of the total time remaining
        
        // Only do this for searches with a total time remaining, otherwise ignore
        // it as we're either in unlimited time mode, or under a hard constraint
        // for the move in question
        if self.total_remaining != 0 {
            self.allocated_micros = min(self.total_remaining / 5, self.allocated_micros * 13 / 10);
        }
    }

    pub fn update(&mut self) {
        if !self.unlimited {
            self.finished = self.elapsed_micros() >= self.allocated_micros;
        }
    }

    pub fn remaining_micros(&mut self) -> u64 {
        self.update();

        if self.times_up() {
            0
        } else if self.unlimited {
            u64::MAX
        } else {
            self.allocated_micros - self.elapsed_micros()
        }
    }

    pub fn elapsed_micros(&self) -> u64 {
        self.start.elapsed().as_micros() as u64
    }

    pub fn times_up(&self) -> bool {
        self.finished
    }

    pub fn hard_limit(&self) -> bool {
        self.hard_limit
    }
}