use std::ops::{Add, Sub, Mul};
use std::fmt;
use num_traits::AsPrimitive;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Position {
    pub x: i8,
    pub y: i8,
}

impl Position {
    /// Creates a new `Position` without bounds checking.
    pub fn new(x: i8, y: i8) -> Self {
        Position { x, y }
    }

    /// Converts a linear index (0-63) to a `Position`. Returns `None` if invalid square.
    pub fn from_sqr(sqr: i8) -> Option<Self> {
        if sqr >= 0 && sqr < 64 {
            Some(Position::new(sqr % 8, sqr / 8))
        } else {
            None
        }
    }

    /// Converts a chess notation string (e.g., "e2") to a `Position`. Panics if invalid notation.

    /// Converts a linear index (0-63) to a `Position`. Alias for `from_sqr`.
    pub fn from_index(index: i8) -> Option<Self> {
        Position::from_sqr(index)
    }

    /// Converts a `Position` to a linear index (0-63). Returns `None` if out of bounds.
    pub fn to_sqr(&self) -> Option<i8> {
        if self.x >= 0 && self.x < 8 && self.y >= 0 && self.y < 8 {
            Some(self.y * 8 + self.x)
        } else {
            None
        }
    }

    pub fn from_chess_notation(notation: &str) -> Option<Self> {
        if notation.len() != 2 {
            return None;
        }
        let chars: Vec<char> = notation.chars().collect();
        let file = chars[0].to_ascii_lowercase();
        let rank = chars[1];
        if file < 'a' || file > 'h' || rank < '1' || rank > '8' {
            return None;
        }
        let x = (file as i8) - b'a' as i8;
        let y = (rank as i8) - b'1' as i8;
        Some(Position::new(x, y))
    }

    /// Converts a `Position` to chess notation, e.g., "e2". Returns `None` if out of bounds.
    pub fn to_chess_notation(&self) -> Option<String> {
        if self.x < 0 || self.x >= 8 || self.y < 0 || self.y >= 8 {
            return None;
        }
        let file = (b'a' as i8 + self.x) as u8 as char;
        let rank = (self.y + 1).to_string();
        Some(format!("{}{}", file, rank))
    }


    /// Checks if another `Position` is adjacent to the current one.
    pub fn is_adjacent(&self, other: &Self) -> bool {
        let dx = (self.x - other.x).abs();
        let dy = (self.y - other.y).abs();
        dx <= 1 && dy <= 1 && !(dx == 0 && dy == 0)
    }
}

impl Add for Position {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Position::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl Sub for Position {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Position::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl<T: AsPrimitive<i8>> Mul<T> for Position {
    type Output = Self;

    fn mul(self, rhs: T) -> Self::Output {
        let factor: i8 = rhs.as_(); // Convert to i8
        Self {
            x: self.x * factor,
            y: self.y * factor,
        }
    }
}

impl fmt::Debug for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}