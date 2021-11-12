use std::result::Result;
use super::Color;
use crate::board::{Piece, Board};
use crate::game_elements::PieceType;

type CoordElem = i8;
type Coord = (CoordElem, CoordElem);

pub const UP: Coord = (0, 1);
pub const DOWN: Coord = (0, -1);
pub const LEFT: Coord = (-1, 0);
pub const RIGHT: Coord = (1, 0);
pub const UP_LEFT: Coord = (-1, 1);
pub const UP_RIGHT: Coord = (1, 1);
pub const DOWN_LEFT: Coord = (-1, -1);
pub const DOWN_RIGHT: Coord = (1, -1);

pub const KNIGHT_MOVES: [Coord; 8] = [(-2, -1), (-2, 1), (-1, -2), (-1, 2), (1, -2), (1, 2), (2, -1), (2, 1)];
pub const KING_MOVES: [Coord; 8] = [UP, DOWN, LEFT, RIGHT, UP_LEFT, UP_RIGHT, DOWN_LEFT, DOWN_RIGHT];
    
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub file: CoordElem,
    pub rank: CoordElem
}

impl Position {
    pub fn new_0based(file: CoordElem, rank: CoordElem) -> Self {
        Position { file, rank}
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

    pub fn as_notation(&self) -> String {
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

    // Traces rays from the position in a certain direction until the edge of
    // the board, or the first piece hit of the opposite color.
    // Returns the list of visited positions and the piece found at the end,
    // or None if the ray ends at the edge of the board.
    pub fn trace_ray<'a>(&self, board: &'a Board, dir: Coord, color_moving: Color) -> (Vec<Position>, Option<&'a Piece>) {
        let mut positions = Vec::new();
        let mut next_pos = self.add_delta(&dir);
        let mut piece_found = None;

        while next_pos.is_valid() {
            if let Some(piece) = board.get_pos(&next_pos) {
                piece_found = Some(piece);
                if piece.color() != color_moving {
                    // The color of the piece that is moving can capture that piece
                    positions.push(next_pos);
                    return (positions, Some(piece));
                }
                break;
            }
            positions.push(next_pos);
            next_pos = next_pos.add_delta(&dir);
        }

        (positions, piece_found)
    }

    pub fn is_attacked_by(&self, board: &Board, attacker_color: Color) -> bool {
        // Check for horsies
        for pos in self.knight_jumps() {
            if matches!(board.get_pos(&pos), Some(Piece{color, piece_type: PieceType::Knight}) if *color == attacker_color) {
                return true;
            }
        }

        // Check for sliding pieces and pawns. First, diagonals
        for dir in [UP_LEFT, UP_RIGHT, DOWN_LEFT, DOWN_RIGHT].iter() {
            let (moves, piece_opt) = self.trace_ray(board, *dir, !attacker_color);
            let piece = match piece_opt {
                // The piece must be of the attacking color
                Some(p) if p.color() == attacker_color => p,
                _ => continue,
            };

            // The piece must be: a queen, a bishop or
            // - a king, if the length of the ray is 1 (it's right next to the square)
            // - a pawn, if the length of the ray is 1 and the direction of the ray
            //   is the opposite of the pawn's attacking direction
            let vertical_dir_pawn_attack = match attacker_color {
                Color::White => -1,
                Color::Black =>  1,
            };

            let typ = piece.piece_type();
            if typ == PieceType::Queen || typ == PieceType::Bishop ||
                 (moves.len() == 1 && (typ == PieceType::King || (typ == PieceType::Pawn && dir.1 == vertical_dir_pawn_attack))) {
                    return true;
            }
        }

        // Then, horizontal and vertical
        for dir in [UP, DOWN, LEFT, RIGHT].iter() {
            let (moves, piece_opt) = self.trace_ray(board, *dir, !attacker_color);
            let piece = match piece_opt {
                // The piece must be of the attacking color
                Some(p) if p.color() == attacker_color => p,
                _ => continue,
            };

            // In this case, the piece must be either a queen, a rook
            // or a king at distance 1
            let typ = piece.piece_type();
            if typ == PieceType::Queen || typ == PieceType::Rook || (moves.len() == 1 && typ == PieceType::King) {
                return true;
            }
        }

        false
    }

    pub fn add_delta(&self, delta: &Coord) -> Position {
        Position::new_0based(self.file + delta.0, self.rank + delta.1)
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