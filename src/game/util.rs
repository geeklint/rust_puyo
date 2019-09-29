#[derive(Copy, Clone, PartialEq)]
pub enum Direction {
    None, Left, Right, Up, Down,
}

static ALL_DIRECTIONS: [Direction; 4] = [
    Direction::Left,
    Direction::Right,
    Direction::Up,
    Direction::Down
];

impl Direction {
    pub fn each_real() -> std::slice::Iter<'static, Self> {
        ALL_DIRECTIONS.iter()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Coord {
    pub x: i32,
    pub y: i32,
}

impl Coord {
    fn up(&self) -> Self {
        Coord { x: self.x, y: self.y + 1 }
    }

    fn down(&self) -> Self {
        Coord { x: self.x, y: self.y - 1 }
    }

    fn left(&self) -> Self {
        Coord { x: self.x - 1, y: self.y }
    }

    fn right(&self) -> Self {
        Coord { x: self.x + 1, y: self.y }
    }

    pub fn apply_motion(&self, motion: Direction) -> Self {
        match motion {
            Direction::Left => self.left(),
            Direction::Right => self.right(),
            Direction::Up => self.up(),
            Direction::Down => self.down(),
            Direction::None => *self,
        }
    }

    pub fn is_adjacent(&self, other: &Self) -> bool {
        let xdiff = (other.x - self.x).abs();
        let ydiff = (other.y - self.y).abs();
        match (xdiff, ydiff) {
            (0, 1) | (1, 0) => true,
            _ => false,
        }
    }

    pub fn motion_to(&self, other: &Self) -> Direction {
        let xdiff = other.x - self.x;
        let ydiff = other.y - self.y;
        match (xdiff, ydiff) {
            (-1, 0) => Direction::Left,
            (1, 0) => Direction::Right,
            (0, 1) => Direction::Up,
            (0, -1) => Direction::Up,
            _ => Direction::None,
        }
    }
}

pub enum Rotation {
    None,
    Single,
    Double,
}

