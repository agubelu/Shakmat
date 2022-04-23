use super::move_ordering::{MoveScore, MAX_HISTORY_VAL};
use shakmat_core::{Color::{self, *}, Move::{self, *}};

// Stores move evaluation data indexed by [color][from][to]
pub struct HistoryTable {
    data: [[[MoveScore; 64]; 64]; 2]
}

impl HistoryTable {
    pub fn new() -> Self {
        Self { data: [[[0; 64]; 64]; 2] }
    }

    pub fn get_value(&self, mv: &Move, color: Color) -> MoveScore {
        let (from, to) = get_from_to(mv, color);
        self.data[color.to_index()][from][to]
    } 

    pub fn add_bonus(&mut self, mv: &Move, color: Color, bonus: MoveScore) {
        let (from, to) = get_from_to(mv, color);
        
        let newval = self.data[color.to_index()][from][to] + bonus;
        self.data[color.to_index()][from][to] = newval;
        
        // If we reach the maximum history value, scale down the whole table
        if newval > MAX_HISTORY_VAL {
            self.age();
        }
    }

    pub fn age(&mut self) {
        self.data.iter_mut().for_each(|color| {
            color.iter_mut().for_each(|from_mat| {
                from_mat.iter_mut().for_each(|val| {
                    *val /= 2;
                })
            })
        });
    }
}

fn get_from_to(mv: &Move, color: Color) -> (usize, usize) {
    match (mv, color) {
        (ShortCastle, White) => (3, 1),
        (LongCastle, White) => (3, 5),
        (ShortCastle, Black) => (59, 57),
        (LongCastle, Black) => (59, 61),
        _ => (mv.from() as usize, mv.to() as usize)
    }
}