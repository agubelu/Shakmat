use std::result::Result;

#[derive(Debug, Clone, Copy)]
pub struct Position {
    file: usize,
    rank: usize
}

impl Position {
    pub fn new_0based(file: usize, rank: usize) -> Self {
        Position { file, rank}
    }

    pub fn new_1based(file: usize, rank: usize) -> Self {
        Position { file: file - 1, rank: rank - 1}
    }

    pub fn from_notation(pos: &str) -> Result<Self, String> {
        let pos_chars: Vec<char> = pos.chars().collect();

        if pos_chars.len() != 2 {
            return Err(format!("Invalid position: {}", pos));
        }

        let file = match pos_chars[0] {
            'a' | 'A' => 0,
            'b' | 'B' => 1,
            'c' | 'C' => 2,
            'd' | 'D' => 3,
            'e' | 'E' => 4,
            'f' | 'F' => 5,
            'g' | 'G' => 6,
            'h' | 'H' => 7,
             x  => return Err(format!("Invalid file: {}", x)),
        };

        let rank = match pos_chars[1] {
            '1' => 0,
            '2' => 1,
            '3' => 2,
            '4' => 3,
            '5' => 4,
            '6' => 5,
            '7' => 6,
            '8' => 7,
             x  => return Err(format!("Invalid rank: {}", x)),
        };

        Ok(Position::new_0based(file, rank))
    }
}