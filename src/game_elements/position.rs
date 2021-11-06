use std::result::Result;

pub const UP: (isize, isize) = (0, -1);
pub const DOWN: (isize, isize) = (0, 1);
pub const LEFT: (isize, isize) = (-1, 0);
pub const RIGHT: (isize, isize) = (1, 0);
pub const UP_LEFT: (isize, isize) = (-1, -1);
pub const UP_RIGHT: (isize, isize) = (1, -1);
pub const DOWN_LEFT: (isize, isize) = (-1, 1);
pub const DOWN_RIGHT: (isize, isize) = (1, 1);

pub const KNIGHT_MOVES: [(isize, isize); 8] = [(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)];
pub const KING_MOVES: [(isize, isize); 8] = [UP, DOWN, LEFT, RIGHT, UP_LEFT, UP_RIGHT, DOWN_LEFT, DOWN_RIGHT];
    
#[derive(Debug, Clone, Copy)]
pub struct Position {
    file: isize,
    rank: isize
}

impl Position {
    pub fn new_0based(file: isize, rank: isize) -> Self {
        Position { file, rank}
    }

    pub fn new_1based(file: isize, rank: isize) -> Self {
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

    pub fn to_notation(&self) -> String {
        let file = match self.file {
            0 => 'a',
            1 => 'b',
            2 => 'c',
            3 => 'd',
            4 => 'e',
            5 => 'f',
            6 => 'g',
            7 => 'h',
             _ => panic!("Invalid file: {}", self.file),
        };

        let rank = match self.rank {
            0 => '1',
            1 => '2',
            2 => '3',
            3 => '4',
            4 => '5',
            5 => '6',
            6 => '7',
            7 => '8',
             _ => panic!("Invalid rank: {}", self.rank),
        };

        format!("{}{}", file, rank)
    }

    pub fn knight_jumps(&self) -> Vec<Position> {
        KNIGHT_MOVES.iter()
            .map(|(df, dr)| Position::new_0based(self.file + df, self.rank + dr))
            .filter(|pos| pos.is_valid())
            .collect()
    }

    pub fn king_moves(&self) -> Vec<Position> {
        KING_MOVES.iter()
            .map(|(df, dr)| Position::new_0based(self.file + df, self.rank + dr))
            .filter(|pos| pos.is_valid())
            .collect()
    }

    pub fn file_u(&self) -> usize {
        self.file as usize
    }

    pub fn rank_u(&self) -> usize {
        self.rank as usize
    }

    pub fn is_valid(&self) -> bool {
        self.rank >= 0 && self.rank < 8 && self.file >= 0 && self.file < 8
    }
}