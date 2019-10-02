use super::util::*;
use super::color::Color;

#[derive(Copy, Clone, PartialEq)]
pub enum RenderCommand {
    Goto(Coord),
    Paint(Color),
}

use RenderCommand::*;

const GOTO_THRESHOLD: usize = 2;

pub struct Renderer<'a> {
    front: &'a Vec<Vec<Color>>,
    back: &'a Vec<Vec<Color>>,
    row: usize,
    col: usize,
    paint_current: bool,
    at_start: bool,
    at_end: bool,
    queue: Vec<Color>,
}

enum AdvanceResult {
    End,
    NewRow,
    Normal,
}

impl<'b> Renderer<'b> {
    pub fn new<'a>(front: &'a Vec<Vec<Color>>, back: &'a Vec<Vec<Color>>) -> Renderer<'a> {
        assert_eq!(front.len(), back.len());
        Renderer {
            front,
            back,
            row: front.len() - 1,
            col: 0,
            paint_current: false,
            at_start: true,
            at_end: false,
            queue: vec![],
        }
    }

    fn advance(&mut self) -> AdvanceResult {
        if self.at_end {
            return AdvanceResult::End;
        }
        self.col += 1;
        if self.col >= self.back[self.row].len() {
            self.col = 0;
            if self.row == 0 {
                self.at_end = true;
                return AdvanceResult::End;
            } else {
                self.row -= 1;
                return AdvanceResult::NewRow;
            }
        }
        return AdvanceResult::Normal;
    }
    
    fn same(&self) -> bool {
        self.front[self.row][self.col] == self.back[self.row][self.col] 
    }
}

impl Iterator for Renderer<'_> {
    type Item = RenderCommand;
    
    fn next(&mut self) -> Option<Self::Item> {
        if self.at_end {
            return None;
        }
        if !self.queue.is_empty() {
            return Some(Paint(self.queue.remove(0)));
        }
        if self.paint_current {
            let color = self.back[self.row][self.col];
            self.paint_current = false;
            self.advance();
            return Some(Paint(color));
        }
        let mut need_goto = self.at_start;
        while !self.at_end && self.same() {
            if !need_goto {
                if self.queue.len() < GOTO_THRESHOLD {
                    self.queue.push(self.back[self.row][self.col])
                } else {
                    need_goto = true;
                    self.queue.clear();
                }
            }
            match self.advance() {
                AdvanceResult::NewRow => {
                    need_goto = true;
                    self.queue.clear();
                },
                AdvanceResult::Normal => (),
                AdvanceResult::End => return None,
            }
        }
        if !self.queue.is_empty(){
            self.paint_current = true;
            return Some(Paint(self.queue.remove(0)));
        }
        if need_goto {
            self.paint_current = true;
            return Some(Goto(
                Coord { x: self.col as i32, y: self.row as i32 }
            ));
        }
        let color = self.back[self.row][self.col];
        self.advance();
        return Some(Paint(color));
    }
}
