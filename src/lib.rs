
extern crate cfg_if;
extern crate wasm_bindgen;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;
use web_sys::{Document, Element};

use puyo_game::game;
use puyo_game::game::color::Color;

mod utils;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

const IMGURL: &str = "img/puyo_puyo-Alpha.png";
const IMGURL_CLEAR: &str = "img/000000-0.png";
const IMGURL_GARBAGE: &str = "img/Nuisance_large.png";
const GARBAGE_IMGURLS: [(u32, &str); 8] = [
    (1440, "img/Nuisance_comet.png"),
    (720, "img/Nuisance_crown.png"),
    (360, "img/Nuisance_moon.png"),
    (180, "img/Nuisance_star.png"),
    (30, "img/Nuisance_rock.png"),
    (6, IMGURL_GARBAGE),
    (1, "img/Nuisance_small.png"),
    (0, IMGURL_CLEAR),
];

pub struct GameCell {
    image: Element,
    container: Element,
    normal_imgurl: bool,
}

impl GameCell {
    fn style(color: Color) -> String {
        let color = match color {
            Color::Empty => "rgba(0,0,0,0)",
            Color::Garbage => "#FFFFFF",
            Color::Red => "#FF0000",
            Color::Green => "#00FF00",
            Color::Blue => "#0000FF",
            Color::Yellow => "#FFFF00",
            Color::Violet => "#FF00FF",
        };
        format!("width: 32px; height: 32px; background-color: {}", color)
    }

    pub fn new(document: &Document) -> Result<Self, JsValue> {
        let image = document.create_element("img")?;
        image.set_attribute("src", IMGURL)?;
        image.set_attribute("style", &Self::style(Color::Empty))?;
        let container = document.create_element("td")?;
        container.append_child(&image)?;
        Ok(GameCell { image, container, normal_imgurl: true })
    }

    pub fn element(&self) -> &Element {
        &self.container
    }

    pub fn set_color(&mut self, color: Color) -> Result<(), JsValue> {
        let change_img = match (color, self.normal_imgurl) {
            (Color::Garbage, normal) => normal,
            (_, normal) => !normal,
        };
        if change_img {
            self.normal_imgurl = !self.normal_imgurl;
            self.image.set_attribute("src",
                match color {
                    Color::Garbage => IMGURL_GARBAGE,
                    _ => IMGURL,
                }
            )?;
        }
        self.image.set_attribute("style", &Self::style(color))?;
        Ok(())
    }
}

pub struct GameView {
    board: Vec<Vec<GameCell>>,
    table: Element,
    garbage_row: Vec<Element>,
    score: Element,
    next_pivot: GameCell,
    next_wheel: GameCell,
}

impl GameView {
    pub fn new(document: &Document) -> Result<Self, JsValue> {
        let table = document.create_element("table")?;
        table.set_attribute("style",
            "display: inline-block;
             border: 1px solid black;
             border-collapse: collapse;
        ")?;
        // garbage row
        let html_row = document.create_element("tr")?;
        table.append_child(&html_row)?;
        html_row.set_attribute("style",
            "border-bottom: 1px solid black;
        ")?;
        let mut garbage_row = Vec::with_capacity(game::BOARD_WIDTH);
        for _x in 0..game::BOARD_WIDTH {
            let td = document.create_element("td")?;
            html_row.append_child(&td)?;
            let img = document.create_element("img")?;
            td.append_child(&img)?;
            img.set_attribute("style", "width: 32px; height: 32px")?;
            img.set_attribute("src", IMGURL_CLEAR)?;
            garbage_row.push(img);
        }
        // main board
        let mut board = Vec::with_capacity(game::BOARD_HEIGHT);
        for _y in 0..game::BOARD_HEIGHT {
            let html_row = document.create_element("tr")?;
            table.append_child(&html_row)?;
            let mut row = Vec::with_capacity(game::BOARD_WIDTH);
            for _x in 0..game::BOARD_WIDTH {
                let cell = GameCell::new(document)?;
                html_row.append_child(cell.element())?;
                row.push(cell);
            }
            board.push(row);
        }
        // score
        let html_row = document.create_element("tr")?;
        table.append_child(&html_row)?;
        html_row.set_attribute("style",
            "border-top: 1px solid black;
        ")?;
        for _x in 0..(game::BOARD_WIDTH - 1) {
            let td = document.create_element("td")?;
            html_row.append_child(&td)?;
        }
        let td = document.create_element("td")?;
        html_row.append_child(&td)?;
        let score = document.create_element("div")?;
        td.append_child(&score)?;
        score.set_attribute("style",
            "width: 32px;
             height: 32px;
             font-size: 32px;
             direction: rtl;
             overflow: visible;
             white-space: nowrap;
        ")?;
        // next
        let html_row = document.create_element("tr")?;
        table.append_child(&html_row)?;
        html_row.set_attribute("style",
            "border-top: 1px solid black;
        ")?;
        for _x in 0..(game::BOARD_WIDTH - 2) {
            let td = document.create_element("td")?;
            html_row.append_child(&td)?;
        }
        let next_pivot = GameCell::new(document)?;
        html_row.append_child(next_pivot.element())?;
        let next_wheel = GameCell::new(document)?;
        html_row.append_child(next_wheel.element())?;

        Ok(GameView {
            board,
            table,
            garbage_row,
            score,
            next_pivot,
            next_wheel,
        })
    }

    pub fn element(&self) -> &Element {
        &self.table
    }

    pub fn set_over(&self, score: u32, win: bool) {
        self.score.set_inner_html(
            &if win {
                format!("Won ({})", score)
            } else {
                format!("Lost ({})", score)
            }
        )
    }

    pub fn render(&mut self, game: &game::Game) -> Result<(), JsValue> {
        // render score
        self.score.set_inner_html(&game.score().to_string());
        // render garbage
        let mut garbage = game.pending_garbage();
        let mut col = 0;
        let mut symbol_index = 0;
        while col < self.garbage_row.len() {
            let (amount, symbol) = GARBAGE_IMGURLS[symbol_index];
            if amount <= garbage {
                garbage -= amount;
                self.garbage_row[col].set_attribute("src", symbol)?;
                col += 1;
            } else {
                symbol_index += 1;
            }
        }
        // render board
        let mut row: usize = 0;
        let mut col: usize = 0;
        for cmd in game.render() {
            match cmd {
                game::render::RenderCommand::Goto(coords) => {
                    let top_row = game::BOARD_HEIGHT - 1;
                    col = coords.x as usize;
                    row = top_row - coords.y as usize;
                }
                game::render::RenderCommand::Paint(color) => {
                    self.board[row][col].set_color(color)?;
                    col += 1;
                }
            }
        }
        // render next
        let (next_pivot, next_wheel) = game.next_puyo();
        self.next_pivot.set_color(next_pivot)?;
        self.next_wheel.set_color(next_wheel)?;
        Ok(())
    }
}

#[wasm_bindgen]
pub struct TwoPlayerGame {
    player_one: game::Game,
    player_two: game::Game,
    view_one: GameView,
    view_two: GameView,
}

#[wasm_bindgen]
impl TwoPlayerGame {
    pub fn new() -> Result<TwoPlayerGame, JsValue> {
        utils::set_panic_hook();
        let player_one = game::Game::new();
        let player_two = game::Game::new();
        let window = web_sys::window().expect("no global `window` exists");
        let document = window.document().expect("should have a document on window");
        let body = document.body().expect("document should have a body");

        let view_one = GameView::new(&document)?;
        body.append_child(view_one.element())?;

        let spacer = document.create_element("div")?;
        spacer.set_attribute("style", "display: inline-block; width: 100px")?;
        body.append_child(&spacer)?;

        let view_two = GameView::new(&document)?;
        body.append_child(view_two.element())?;
        Ok(TwoPlayerGame {
            player_one,
            player_two,
            view_one, view_two,
        })
    }

    pub fn restart(&mut self) {
        self.player_one = game::Game::new();
        self.player_two = game::Game::new();
    }

    pub fn tick(&mut self) -> Result<bool, JsValue> {
        if self.player_one.is_over() {
            self.view_one.set_over(self.player_one.score(), false);
            self.view_two.set_over(self.player_two.score(), true);
            return Ok(false);
        } else if self.player_two.is_over() {
            self.view_one.set_over(self.player_one.score(), true);
            self.view_two.set_over(self.player_two.score(), false);
            return Ok(false);
        }

        self.player_one.tick();
        self.player_two.tick();
        self.player_one.add_garbage(self.player_two.get_garbage());
        self.player_two.add_garbage(self.player_one.get_garbage());
        
        self.view_one.render(&self.player_one)?;
        self.view_two.render(&self.player_two)?;
        Ok(true)
    }

    pub fn p1_left(&mut self) {
        self.player_one.move_(game::Direction::Left);
    }

    pub fn p1_right(&mut self) {
        self.player_one.move_(game::Direction::Right);
    }

    pub fn p1_up(&mut self) {
        self.player_one.move_(game::Direction::Up);
    }

    pub fn p1_down(&mut self) {
        self.player_one.move_(game::Direction::Down);
    }

    pub fn p1_rotate(&mut self) {
        self.player_one.rotate();
    }
 
    pub fn p2_left(&mut self) {
        self.player_two.move_(game::Direction::Left);
    }

    pub fn p2_right(&mut self) {
        self.player_two.move_(game::Direction::Right);
    }
 
    pub fn p2_up(&mut self) {
        self.player_two.move_(game::Direction::Up);
    }

    pub fn p2_down(&mut self) {
        self.player_two.move_(game::Direction::Down);
    }

    pub fn p2_rotate(&mut self) {
        self.player_two.rotate();
    }
}
