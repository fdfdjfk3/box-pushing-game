use crate::map::init_tile_colors;
use pancurses::{curs_set, endwin, initscr, napms, noecho, Input};

pub mod game;
pub mod map;

pub const TOP_PADDING: i32 = 5;

#[derive(Clone, Copy)]
pub enum Direction {
    Up,
    Right,
    Down,
    Left,
}

impl Direction {
    fn get_vec2_move(&self) -> (i32, i32) {
        match self {
            Self::Up => (-1, 0),
            Self::Down => (1, 0),
            Self::Left => (0, -1),
            Self::Right => (0, 1),
        }
    }
}

fn main() {
    let window = initscr();
    if pancurses::has_colors() {
        pancurses::start_color();
        init_tile_colors();
    }
    window.keypad(true);
    noecho();
    curs_set(0);
    window.printw("Test");
    let player = game::Player {
        y: 5,
        x: 5,
        glyph: 'X',
    };

    let mut game = game::GameContext {
        player,
        map_data: None,
        map_list: map::get_maps(),
        level: 0,
    };
    game.load_current_level();

    loop {
        window.clear();
        game.draw_all(&window);
        window.refresh();
        let k = window.getch();
        match k {
            Some(Input::KeyRight) => game.player_movement(Direction::Right),
            Some(Input::KeyUp) => game.player_movement(Direction::Up),
            Some(Input::KeyLeft) => game.player_movement(Direction::Left),
            Some(Input::KeyDown) => game.player_movement(Direction::Down),
            Some(Input::Character('r')) => game.load_current_level(),
            Some(Input::Character('q')) => break,
            _ => (),
        };
        game.update_all();
        napms(20);
    }

    window.refresh();
    endwin();
}
