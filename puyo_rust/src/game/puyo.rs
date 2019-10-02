use super::color::*;
use super::util::*;

#[derive(Clone)]
pub struct Puyo<T> {
    pivot: T,
    wheel: T,
}

impl<T> Puyo<T> {
    pub fn pivot(&self) -> &T {
        &self.pivot
    }

    pub fn wheel(&self) -> &T {
        &self.wheel
    }
}

impl Puyo<Color> {
    pub fn new(pivot: Color, wheel: Color) -> Self {
        Puyo { pivot, wheel }
    }

    pub fn from_excluded(excluded_color: Color) -> Self {
        let mut rng = rand::thread_rng();
        Puyo {
            pivot: Color::exclude(&mut rng, excluded_color),
            wheel: Color::exclude(&mut rng, excluded_color),
        }
    }

    pub fn empty() -> Self {
        Puyo {
            pivot: Color::Empty,
            wheel: Color::Empty,
        }
    }
}

impl Puyo<Coord> {
    pub fn new(pivot: Coord, wheel: Coord) -> Self {
        assert!(wheel.is_adjacent(&pivot));
        Puyo { pivot, wheel }
    }

    pub fn is_vertical(&self) -> bool {
        self.wheel.x == self.pivot.x
    }

    pub fn rotation(&self) -> Direction {
        self.pivot.motion_to(&self.wheel)
    }

    pub fn move_(&mut self, motion: Direction){
        self.pivot = self.pivot.apply_motion(motion);
        self.wheel = self.wheel.apply_motion(motion);
    }

    pub fn flip(&mut self) {
        if self.wheel.y < self.pivot.y {
            self.wheel.y = self.pivot.y + 1;
        } else {
            self.wheel.y = self.pivot.y - 1;
        }
    }

    pub fn rotate(&mut self){
        let xdiff = self.wheel.x - self.pivot.x;
        let ydiff = self.wheel.y - self.pivot.y;
        let newdiff: (i32, i32) = match (xdiff, ydiff) {
            (1, 0) => (0, -1),
            (0, -1) => (-1, 0),
            (-1, 0) => (0, 1),
            (0, 1) => (1, 0),
            _ => unreachable!(),
        };
        let (xdiff, ydiff) = newdiff;
        self.wheel.x = self.pivot.x + xdiff;
        self.wheel.y = self.pivot.y + ydiff;
    }
}
