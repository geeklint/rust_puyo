extern crate rand;
extern crate libc;

mod game;
mod ui;

use game::Game;
use game::color::Color::*;
use ui::UI;

fn render_color(color: game::color::Color) {
    let code = match color {
        Empty => 30,
        Garbage => 37,
        Red => 31,
        Green => 32,
        Blue => 34,
        Yellow => 33,
        Violet => 35,
    };
    print!("\x1b[{};40m\u{2687}", code);
}

const GARBAGE_DISPLAY: [(u32, char); 7] = [
    (1440, '\u{2604}'),
    (720, '\u{265b}'),
    (360, '\u{263e}'),
    (180, '\u{2605}'),
    (30, '\u{2689}'),
    (6, '\u{2687}'),
    (1, '\u{233E}'),
];

fn render_at(row: i32, col: i32, game: &Game){
    // draw garbage
    print!("\x1b[{};{}H", row, col);
    print!("\x1b[30;47m");
    let mut garbage = game.pending_garbage();
    let mut sym_index = 0;
    let mut printed = 0;
    while printed < game::BOARD_WIDTH && garbage > 0 {
        let (unit, symbol) = GARBAGE_DISPLAY[sym_index];
        if garbage >= unit {
            garbage -= unit;
            print!("{}", symbol);
            printed += 1;
        } else {
            sym_index += 1;
        }
    }
    print!("\x1b[30;40m");
    while printed < game::BOARD_WIDTH {
        print!(" ");
        printed += 1;
    }
    // draw rest below
    let row = row + 2;
    // draw next puyo
    let (pivot, wheel) = game.next_puyo();
    let next_col = col + ((game::BOARD_WIDTH + 2) as i32);
    print!("\x1b[{};{}H", row, next_col);
    render_color(wheel);
    print!("\x1b[{};{}H", row + 1, next_col);
    render_color(pivot);
    // draw board updates
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
}


fn main() {
    let mut game1 = Game::new();
    let mut game2 = Game::new();

    let mut ui = UI::init();

    while ui.frame() {
        if game1.is_over() {
            print!("\x1b[2;2H\x1b[30;47mGAME OVER");
            continue;
        } else if game2.is_over() {
            print!("\x1b[2;12H\x1b[30;47mGAME OVER");
            continue;
        }
        
        for (player, button) in ui.buttons().iter() {
            let game = match player {
                1 => &mut game1,
                2 => &mut game2,
                _ => unreachable!(),
            };
            let motion = match button {
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

        game1.tick();
        game2.tick();
        game1.add_garbage(game2.get_garbage());
        game2.add_garbage(game1.get_garbage());

        render_at(2, 2, &game1);
        game1.finish_render();
        render_at(2, 12, &game2);
        game2.finish_render();
    }

    ui.quit();
}
