use shakmat_core::Move;


// A struct holding the principal variation line for a search
pub struct PVLine {
    moves: Vec<Move>
}

impl PVLine {
    pub fn new() -> Self {
        Self { moves: vec![] }
    }

    pub fn clear(&mut self) {
        self.moves.clear();
    }

    pub fn first(&self) -> Option<Move> {
        self.moves.first().copied()
    }

    pub fn update_line(&mut self, mv: Move, child_line: &mut Self) {
        self.clear();
        self.moves[0] = mv;
        self.moves.append(&mut child_line.moves);
    }
}