use rand::prelude::*;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Color {
    Empty,
    Garbage,
    Red,
    Green,
    Blue,
    Yellow,
    Violet,
}

use Color::*;

static NORMAL_COLORS: [Color; 5] = [Red, Green, Blue, Yellow, Violet];

impl Color {
    pub fn any<R: Rng + ?Sized>(rng: &mut R) -> Self {
        *NORMAL_COLORS.choose(rng).unwrap()
    }
    
    pub fn exclude<R: Rng + ?Sized>(rng: &mut R, exclude: Color) -> Self {
        let exclude_index = NORMAL_COLORS.iter().position(|c| *c == exclude);
        let mut choice: usize = rng.gen_range(0, NORMAL_COLORS.len() - 1);
        if let Some(xi) = exclude_index {
            if choice == xi {
                choice += 1;
            }
        }
        NORMAL_COLORS[choice]
    }

    pub fn is_normal(&self) -> bool {
        match self {
            Empty | Garbage => false,
            _ => true,
        }
    }
}
