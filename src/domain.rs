
pub struct Position {
    x: usize,
    y: usize
}

pub struct Tile {
    coordinates: Position
}

impl Position {
    pub fn new(x: usize, y: usize) -> Self {
        Position {
            x,
            y
        }
    }
}

