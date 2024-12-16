use std::ops::{Add, Sub, Index, IndexMut};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Position {
    pub x: u8,
    pub y: u8,
}

impl Position {
    /// Creates a new `Position` with bounds checking.
    pub fn new(x: u8, y: u8) -> Result<Self, &'static str> {
        if x < 8 && y < 8 {
            Ok(Position { x, y })
        } else {
            Err("Position out of bounds")
        }
    }

    /// Converts a linear index (0-63) to a `Position`.
    pub fn from_sqr(sqr: u8) -> Result<Self, &'static str> {
        if sqr < 64 {
            Ok(Position::new(sqr % 8, sqr / 8).unwrap())
        } else {
            Err("Square index out of bounds")
        }
    }

    /// Converts a linear index (0-63) to a `Position`.
    pub fn from_index(index: u8) -> Result<Self, &'static str> {
        Position::from_sqr(index)
    }

    /// Converts a chess notation string (e.g., "e2") to a `Position`.
    pub fn from_chess_notation(notation: &str) -> Result<Self, &'static str> {
        if notation.len() != 2 {
            return Err("Notation must be exactly two characters");
        }
        let chars: Vec<char> = notation.chars().collect();
        let file = chars[0].to_ascii_lowercase();
        let rank = chars[1];
        if file < 'a' || file > 'h' || rank < '1' || rank > '8' {
            return Err("Invalid chess notation: must be between 'a1' and 'h8'");
        }
        let x = (file as u8) - b'a';
        let y = (rank as u8) - b'1';
        Position::new(x, y)
    }

    /// Converts a `Position` to a linear index (0-63).
    pub fn to_sqr(&self) -> u8 {
        self.y * 8 + self.x
    }

    /// Converts a `Position` to chess notation (e.g., "e2").
    pub fn to_chess_notation(&self) -> Result<String, &'static str> {
        if self.x < 8 && self.y < 8 {
            let file = (b'a' + self.x) as char;
            let rank = (self.y + 1).to_string();
            Ok(format!("{}{}", file, rank))
        } else {
            Err("Position out of bounds for chess notation")
        }
    }

    /// Checks if another `Position` is adjacent to the current one.
    pub fn is_adjacent(&self, other: &Self) -> bool {
        let dx = (self.x as i8 - other.x as i8).abs();
        let dy = (self.y as i8 - other.y as i8).abs();
        dx <= 1 && dy <= 1 && !(dx == 0 && dy == 0)
    }
}

impl Add for Position {
    type Output = Result<Self, &'static str>;

    fn add(self, rhs: Self) -> Self::Output {
        let x = self.x.checked_add(rhs.x).ok_or("Addition overflow for x")?;
        let y = self.y.checked_add(rhs.y).ok_or("Addition overflow for y")?;
        Position::new(x, y)
    }
}

impl Sub for Position {
    type Output = Result<Self, &'static str>;

    fn sub(self, rhs: Self) -> Self::Output {
        let x = self.x.checked_sub(rhs.x).ok_or("Subtraction underflow for x")?;
        let y = self.y.checked_sub(rhs.y).ok_or("Subtraction underflow for y")?;
        Position::new(x, y)
    }
}

impl std::fmt::Debug for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.x, self.y)
    }
}

// Implement Index for Position to index slices
impl<T> Index<Position> for [T] {
    type Output = T;

    fn index(&self, index: Position) -> &Self::Output {
        &self[index.to_sqr() as usize]
    }
}

// Implement IndexMut for Position to index slices mutably
impl<T> IndexMut<Position> for [T] {
    fn index_mut(&mut self, index: Position) -> &mut Self::Output {
        &mut self[index.to_sqr() as usize]
    }
}
