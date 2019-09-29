extern crate rand;
extern crate libc;

mod game;
mod ui;

use game::Game;
use game::color::Color::*;
use ui::UI;

fn render_color(color: game::color::Color) {
    let code = match color {
        Empty => 40,
        Garbage => 47,
        Red => 41,
        Green => 42,
        Blue => 44,
        Yellow => 43,
        Violet => 45,
    };
    print!("\x1b[{}m ", code);
}

fn render_at(row: i32, col: i32, game: &Game){
    for cmd in game.render() {
        match cmd {
            game::render::RenderCommand::Goto(coords) => {
                let col = col + coords.x;
                let top_row = (game::BOARD_HEIGHT - 2) as i32;
                let row = row + (top_row - coords.y);
                print!("\x1b[{};{}H", row, col);
            },
            game::render::RenderCommand::Paint(color) => {
                render_color(color);
            },
        }
    }
    let (pivot, wheel) = game.next_puyo();
    let next_col = col + ((game::BOARD_WIDTH + 2) as i32);
    print!("\x1b[{};{}H", row, next_col);
    render_color(wheel);
    print!("\x1b[{};{}H", row + 1, next_col);
    render_color(pivot);
}


fn main() {
    let mut game = Game::new();

    let mut ui = UI::init();

    let mut count = 61;

    while ui.frame() {
        if game.is_over() {
            print!("\x1b[2;2HGAME OVER");
            continue;
        }
        
        for button_press in ui.buttons().iter() {
            let motion = match button_press {
                ui::Button::Up => game::Direction::Up,
                ui::Button::Down => game::Direction::Down,
                ui::Button::Left => game::Direction::Left,
                ui::Button::Right => game::Direction::Right,
                ui::Button::Rotate => {
                    game.rotate();
                    continue;
                }
            };
            game.move_(motion);
        }

        if count > 60 {
            count = 0;
            game.tick();
        } else {
            count += 1;
        }

        render_at(2, 2, &game);
        game.finish_render();
    }

    ui.quit();
}
