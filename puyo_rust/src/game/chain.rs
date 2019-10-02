use std::collections::HashSet;
use std::cmp;

use super::color::Color;

const POINTS_PER_GARBAGE: u32 = 70;

#[derive(Debug)]
pub struct ChainTracker {
    total_cleared: u32,
    num_chains: u32,
    colors: HashSet<Color>,
    group_bonus: u32,
    leftover: u32,
}

impl ChainTracker {
    pub fn new() -> Self {
        ChainTracker {
            total_cleared: 0,
            num_chains: 0,
            colors: HashSet::new(),
            group_bonus: 0,
            leftover: 0,
        }
    }

    pub fn record_group(&mut self, color: Color, num_puyo: u32){
        self.total_cleared += num_puyo;
        self.colors.insert(color);
        let group_bonus = if num_puyo < 5 {
            0
        } else if num_puyo >= 11 {
            10
        } else {
            num_puyo - 3
        };
        self.group_bonus += group_bonus;
    }

    pub fn end_cycle(&mut self){
        self.num_chains += 1;
    }

    pub fn convert_to_garbage(&mut self) -> u32 {
        if self.total_cleared == 0 {
            return 0;
        }
        let score = self.get_score() + self.leftover;
        let garbage = score / POINTS_PER_GARBAGE;
        self.total_cleared = 0;
        self.num_chains = 0;
        self.colors.clear();
        self.group_bonus = 0;
        self.leftover = score % POINTS_PER_GARBAGE;
        return garbage;
    }

    fn get_score(&self) -> u32 {
        let chain_power = if self.num_chains <= 1 {
            0
        } else if self.num_chains >= 9 {
            999
        } else {
            8 * (2u32).pow(self.num_chains - 2)
        };
        let color_bonus = match self.colors.len() {
            1 => 0,
            2 => 3,
            3 => 6,
            4 => 12,
            5 => 24,
            _ => unreachable!(),
        };
        let total_bonus = chain_power + color_bonus + self.group_bonus;
        let total_bonus = cmp::max(total_bonus, 1);
        let total_bonus = cmp::min(total_bonus, 999);
        return 10 * self.total_cleared * total_bonus;
    }
}
