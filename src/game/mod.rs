#![allow(unused_parens)]

use std::collections::HashSet;

pub mod color;
pub mod util;
mod puyo;
mod chain;
pub mod render;

use color::Color;
use color::Color::*;
use util::*;
use puyo::*;
use chain::ChainTracker;
pub use render::Renderer;
pub use util::Direction;

pub struct Game {
    is_over: bool,
    tick_num: u32,
    front_board: Vec<Vec<Color>>,
    board: Vec<Vec<Color>>,
    excluded_color: Color,
    current: Option<Puyo<Coord>>,
    next: Puyo<Color>,
    chain: ChainTracker,
    incoming_garbage: u32,
    garbage_column_index: usize,
    motion: Direction,
    rotate: Rotation,
    outgoing_garbage: u32,
}

pub const BOARD_WIDTH: usize = 6;
pub const BOARD_HEIGHT: usize = 13;
const DROP_POS: Coord = Coord { x: 3, y: 11 };
const GARBAGE_COLUMNS: [usize; 6] = [0, 3, 2, 5, 1, 4];

impl Game {
    pub fn new() -> Game {
        let mut rng = rand::thread_rng();
        let excluded_color = Color::any(&mut rng);
        Game {
            is_over: false,
            tick_num: 50,
            front_board: vec![vec![excluded_color; BOARD_WIDTH]; BOARD_HEIGHT],
            board: vec![vec![Empty; BOARD_WIDTH]; BOARD_HEIGHT],
            excluded_color,
            current: None,
            next: Puyo::<Color>::from_excluded(excluded_color),
            chain: ChainTracker::new(),
            incoming_garbage: 0,
            garbage_column_index: 0,
            motion: Direction::None,
            rotate: Rotation::None,
            outgoing_garbage: 0,
        }
    }

    pub fn is_over(&self) -> bool {
        self.is_over
    }

    pub fn next_puyo(&self) -> (Color, Color) {
        (*self.next.pivot(), *self.next.wheel())
    }

    pub fn move_(&mut self, motion: Direction){
        self.motion = match (self.motion, motion) {
            (Direction::Up, _) => unreachable!(),
            // same == same
            (Direction::Left, Direction::Left) => Direction::Left,
            (Direction::Right, Direction::Right) => Direction::Right,
            (Direction::Down, Direction::Down) => Direction::Down,
            // down & up = none
            (Direction::Down, Direction::Up) => Direction::None,
            // ignore other ups
            (existing, Direction::Up) => existing,
            // main case, if we were nothing, do something
            (Direction::None, something) => something,
            // None is a non-op
            (existing, Direction::None) => existing,
            // down has lower priority than left and right
            (existing, Direction::Down) => existing,
            (Direction::Down, something) => something,
            // opposites cancel out
            (Direction::Left, Direction::Right) => Direction::None,
            (Direction::Right, Direction::Left) => Direction::None,
        };
    }

    pub fn rotate(&mut self){
        if self.current.is_some(){
            self.rotate = match self.rotate {
                Rotation::None => Rotation::Single,
                _ => Rotation::Double,
            }
        }
    }

    pub fn pending_garbage(&self) -> u32 {
        self.incoming_garbage
    }

    pub fn add_garbage(&mut self, amount: u32){
        self.incoming_garbage += amount;
    }

    pub fn get_garbage(&mut self) -> u32 {
        let value = self.outgoing_garbage;
        self.outgoing_garbage = 0;
        return value;
    }

    pub fn tick(&mut self){
        if self.is_over {
            return;
        }
        self.tick_num += 1;
        if self.tick_num > 50 {
            self.tick_num = 0;
        }
        if self.check_motion() {
            return;
        }
        if self.check_rotation() {
            return;
        }
        if self.check_drop(self.tick_num == 0) {
            return;
        }
        if (self.tick_num & 0b11) != 0 {
            return;
        }
        if self.check_gravity() {
            return;
        }
        if self.check_chains() {
            return;
        }
        if self.apply_score() {
            return;
        }
        if self.spawn_garbage() {
            return;
        }
        if self.spawn_puyo() {
            return;
        }
    }

    pub fn render(&self) -> Renderer {
        Renderer::new(&self.front_board, &self.board)
    }

    pub fn finish_render(&mut self){
        let zipper = self.front_board.iter_mut().zip(self.board.iter_mut());
        for (front_row, back_row) in zipper {
            front_row.copy_from_slice(&back_row);
        }
    }

    fn is_empty(&self, coord: &Coord) -> bool {
        match self.board.get(coord.y as usize) {
            None => false,
            Some(row) => match row.get(coord.x as usize) {
                Some(&Empty) => true,
                _ => false,
            }
        }
    }

    fn swap_color(&mut self, coord: &Coord, color: Color) -> Color {
        let row = self.board.get_mut(coord.y as usize).unwrap();
        let item = row.get_mut(coord.x as usize).unwrap();
        let orig = *item;
        *item = color;
        orig
    }

    fn swap_puyo(&mut self, pos: &Puyo<Coord>, colors: Puyo<Color>)
            -> Puyo<Color> {
        Puyo::<Color>::new(
            self.swap_color(pos.pivot(), *colors.pivot()),
            self.swap_color(pos.wheel(), *colors.wheel()),
        )
    }

    fn check_new_pos(&mut self, mut pos: Puyo<Coord>, colors: Puyo<Color>)
            -> bool {
        let mut valid = true;

        if !(self.is_empty(pos.pivot()) && self.is_empty(pos.wheel())) {
            valid = false;
            pos = self.current.clone().unwrap();
        }

        self.swap_puyo(&pos, colors);
        self.current = Some(pos);
        return valid;
    }

    fn check_motion(&mut self) -> bool {
        if let Direction::None = self.motion {
            return false;
        }
        let motion = self.motion;
        self.motion = Direction::None;
        let mut puyo_pos = match &self.current {
            Some(current) => current.clone(),
            None => return false,
        };
        let puyo_colors = self.swap_puyo(&puyo_pos, Puyo::empty());
        puyo_pos.move_(motion);
        return self.check_new_pos(puyo_pos, puyo_colors);
    }

    fn check_rotation(&mut self) -> bool {
        if let Rotation::None = self.rotate {
            return false;
        }

        let mut puyo_pos = match &self.current {
            Some(current) => current.clone(),
            None => return false,
        };

        let puyo_colors = self.swap_puyo(&puyo_pos, Puyo::empty());

        if let Rotation::Double = self.rotate {
            if puyo_pos.is_vertical() {
                puyo_pos.flip();
            } else {
                self.rotate = Rotation::Single;
            }
        }

        if let Rotation::Single = self.rotate {
            puyo_pos.rotate();
        }

        let mut valid = true;

        if self.is_empty(puyo_pos.wheel()){
            // free rotate
        } else if puyo_pos.is_vertical() {
            // floor kick
            puyo_pos.move_(Direction::Up);
        } else {
            // wall kick
            match puyo_pos.rotation() {
                Direction::Left => puyo_pos.move_(Direction::Right),
                Direction::Right => puyo_pos.move_(Direction::Left),
                _ => unreachable!(),
            }
            if !self.is_empty(&puyo_pos.pivot()) {
                // rotation denied
                valid = false;
                puyo_pos = self.current.clone().unwrap();
            }
        }
        self.rotate = Rotation::None;
        self.swap_puyo(&puyo_pos, puyo_colors);
        self.current = Some(puyo_pos);
        return valid;
    }

    fn check_drop(&mut self, full: bool) -> bool {
        let mut puyo_pos = match &self.current {
            Some(current) => current.clone(),
            None => return false,
        };

        if !full {
            return true;
        }

        let puyo_colors = self.swap_puyo(&puyo_pos, Puyo::empty());
        
        puyo_pos.move_(Direction::Down);

        let valid = self.check_new_pos(puyo_pos, puyo_colors);
        if !valid {
            // puyo is no longer under user control
            self.current = None;
        }
        return valid;
    }

    fn check_gravity(&mut self) -> bool {
        let mut did_something = false;

        for above_index in 1..self.board.len() {
            let (lower_rows, upper_rows) = (
                self.board.split_at_mut(above_index));
            let row = lower_rows.last_mut().unwrap();
            let above = upper_rows.first_mut().unwrap();
            for (item, item_above) in row.iter_mut().zip(above.iter_mut()) {
                if *item == Empty && *item_above != Empty {
                    *item = *item_above;
                    *item_above = Empty;
                    did_something = true;
                }
            }
        }
        return did_something;
    }

    fn check_chains(&mut self) -> bool {
        let mut not_part_of_chain: HashSet<Coord> = HashSet::new();
        let mut found_coords;
        let mut pending_coords = HashSet::new();
        let mut garbage_coords;
        let mut any_cleared = false;

        for y in 0i32..((BOARD_HEIGHT - 1) as i32) {
            for x in 0i32..(BOARD_WIDTH as i32) {
                found_coords = HashSet::new();
                garbage_coords = HashSet::new();
                let coord = Coord { x, y };
                if not_part_of_chain.contains(&coord) {
                    continue;
                }
                let blob_color = match self.board[y as usize][x as usize] {
                    color if color.is_normal() => color,
                    _ => continue,
                };
                pending_coords.insert(coord);
                while let Some(item) = pending_coords.iter().next() {
                    let item = item.clone();
                    pending_coords.remove(&item);
                    let color = self.board[item.y as usize][item.x as usize];
                    if color == Garbage {
                        garbage_coords.insert(item);
                        continue;
                    } else if color != blob_color {
                        continue;
                    }
                    for dir in Direction::each_real() {
                        let possible = item.apply_motion(*dir);
                        if (not_part_of_chain.contains(&possible)
                            || found_coords.contains(&possible)
                            || possible.x < 0
                            || possible.y < 0
                            || possible.x >= BOARD_WIDTH as i32
                            || possible.y >= (BOARD_HEIGHT - 1) as i32){
                            continue;
                        }
                        pending_coords.insert(possible);
                    }
                    found_coords.insert(item);
                }
                if found_coords.len() < 4 {
                    not_part_of_chain.extend(found_coords);
                } else {
                    any_cleared = true;
                    let mut group_num = 0;
                    for puyo in found_coords {
                        self.swap_color(&puyo, Empty);
                        group_num += 1;
                    }
                    for puyo in garbage_coords {
                        self.swap_color(&puyo, Empty);
                        group_num += 1;
                    }
                    self.chain.record_group(blob_color, group_num);
                }
            }
        }

        if any_cleared {
            self.chain.end_cycle();
        }
        
        return any_cleared;
    }

    fn apply_score(&mut self) -> bool {
        //print!("\x1b[20;2HChain: {:?}", self.chain);
        let mut garbage = self.chain.convert_to_garbage();
        if self.incoming_garbage < garbage {
            garbage -= self.incoming_garbage;
            self.incoming_garbage = 0;
        } else {
            self.incoming_garbage -= garbage;
            garbage = 0;
        }
        self.outgoing_garbage += garbage;

        return false;
    }

    fn spawn_garbage(&mut self) -> bool {
        let mut did_something = false;
        let mut full_columns = 0;
        while self.incoming_garbage > 0 && full_columns < BOARD_WIDTH {
            let x = GARBAGE_COLUMNS[self.garbage_column_index];
            for y in (0..BOARD_HEIGHT).rev(){
                match self.board[y][x] {
                    Garbage => continue,
                    Empty => {
                        self.board[y][x] = Garbage;
                        self.incoming_garbage -= 1;
                        did_something = true;
                        break;
                    }
                    _ => {
                        full_columns += 1;
                        break;
                    }
                }
            }
            self.garbage_column_index = (
                (self.garbage_column_index + 1) % GARBAGE_COLUMNS.len());
        }
        return did_something;
    }

    fn spawn_puyo(&mut self) -> bool {
        if self.board[DROP_POS.y as usize][DROP_POS.x as usize] != Empty {
            self.is_over = true;
            return true;
        }
        let colors = std::mem::replace(
            &mut self.next, Puyo::from_excluded(self.excluded_color));
        let pos = Puyo::<Coord>::new(
            DROP_POS, DROP_POS.apply_motion(Direction::Up));
        self.swap_puyo(&pos, colors);
        self.current = Some(pos);
        return true;
    }
}
