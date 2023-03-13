use crate::{game::Player, Direction, TOP_PADDING};
use pancurses::Window;
use std::collections::HashMap;

type Id = u32;

#[derive(PartialEq, Clone, Copy)]
pub enum TileType {
    Empty,
    Wall1,
    PushBox,
    Button(Id),             // button-door id
    Door(Option<Id>, bool), // button-door id and open status
    WinPad,
}

#[derive(PartialEq)]
pub enum Event {
    Nothing,
    PressButton,
    Win,
}

impl TileType {
    pub fn glyph(self) -> char {
        match self {
            Self::Empty => ' ',
            Self::Wall1 => 'B',
            Self::PushBox => '@',
            Self::Button(..) => '^',
            Self::Door(..) => 'D',
            Self::WinPad => '#',
            _ => ' ',
        }
    }
    pub fn is_solid(self) -> bool {
        match self {
            Self::Wall1 => true,
            Self::Door(_, false) => true,
            _ => false,
        }
    }
    pub fn is_pushable(self) -> bool {
        match self {
            Self::PushBox => true,
            _ => false,
        }
    }
    pub fn stood_on_event(self) -> Event {
        match self {
            Self::WinPad => Event::Win,
            Self::Button(..) => Event::PressButton,
            _ => Event::Nothing,
        }
    }
}

pub fn init_tile_colors() {
    pancurses::init_pair(1, pancurses::COLOR_WHITE, pancurses::COLOR_BLACK);
    pancurses::init_pair(2, pancurses::COLOR_RED, pancurses::COLOR_BLACK);
    pancurses::init_pair(3, pancurses::COLOR_YELLOW, pancurses::COLOR_YELLOW);
    pancurses::init_pair(4, pancurses::COLOR_BLUE, pancurses::COLOR_BLACK);
    pancurses::init_pair(5, pancurses::COLOR_YELLOW, pancurses::COLOR_BLACK);
}

#[derive(Clone, Copy)]
pub struct Tile {
    y: i32,
    x: i32,
    pub tile_type: TileType,
}

macro_rules! tile {
    ($y: expr, $x: expr, $tile_type: expr) => {
        Tile::new($y, $x, $tile_type)
    };
}

impl Tile {
    fn new(y: i32, x: i32, tile_type: TileType) -> Self {
        Self { y, x, tile_type }
    }
    fn new_wall(
        y: i32,
        x: i32,
        tile_type: TileType,
        direction: Direction,
        len: usize,
    ) -> Vec<Tile> {
        let mut tiles = Vec::with_capacity(len);
        let (dy, dx) = direction.get_vec2_move();
        for i in 0..len {
            let tile = tile!(y + i as i32 * dy, x + i as i32 * dx, tile_type);
            tiles.push(tile);
        }
        tiles
    }
    fn move_tile(&mut self, direction: Direction) {
        let change = direction.get_vec2_move();
        self.y += change.0;
        self.x += change.1;
    }
    pub fn print_tile_plain(&self, window: &Window) {
        window.mvaddch(self.y + TOP_PADDING, self.x, self.tile_type.glyph());
    }
    pub fn print_tile_colored(&self, window: &Window) {
        match self.tile_type {
            TileType::Wall1 => {
                window.attrset(pancurses::COLOR_PAIR(1));
            }
            TileType::PushBox => {
                window.attrset(pancurses::COLOR_PAIR(5));
                window.attron(pancurses::A_BOLD);
            }
            TileType::Button(_) => {
                window.attrset(pancurses::COLOR_PAIR(2));
            }
            TileType::Door(_, false) => {
                window.attrset(pancurses::COLOR_PAIR(5));
                window.attron(pancurses::A_BOLD);
            }
            TileType::Door(_, true) => {
                window.attrset(pancurses::COLOR_PAIR(5));
                window.attron(pancurses::A_DIM);
            }
            TileType::WinPad => {
                window.attrset(pancurses::COLOR_PAIR(4));
            }
            _ => (),
        }
        self.print_tile_plain(window);
        window.attrset(pancurses::A_NORMAL);
        window.attroff(pancurses::A_ATTRIBUTES);
    }
}

#[derive(Clone)]
pub struct MapData {
    pub tile_map: Vec<Tile>,
    pub player_spawn: (i32, i32),
    pub flavor_text: Option<String>,
}

impl MapData {
    pub fn draw(&self, window: &Window) {
        for &tile in &self.tile_map {
            if pancurses::has_colors() {
                tile.print_tile_colored(window);
            } else {
                tile.print_tile_plain(window);
            }
        }
    }
    pub fn tile_count(&self) -> usize {
        self.tile_map.len()
    }
    pub fn tiles_at(&mut self, y: i32, x: i32) -> Vec<&mut Tile> {
        self.tile_map
            .iter_mut()
            .filter(|t| t.y == y && t.x == x)
            .collect()
    }
    pub fn immut_tiles_at(&self, y: i32, x: i32) -> Vec<&Tile> {
        self.tile_map
            .iter()
            .filter(|t| t.y == y && t.x == x)
            .collect()
    }
    pub fn num_solid_or_pushable_tiles_at(&self, y: i32, x: i32) -> usize {
        self.tile_map
            .iter()
            .filter(|t| {
                t.y == y && t.x == x && (t.tile_type.is_solid() || t.tile_type.is_pushable())
            })
            .count()
    }
    pub fn player_move(&mut self, player: &mut Player, direction: Direction) {
        let change = direction.get_vec2_move();
        let (new_y, new_x) = (player.y + change.0, player.x + change.1);

        let tiles_past_tile =
            self.num_solid_or_pushable_tiles_at(new_y + change.0, new_x + change.1);
        let tiles_at_new_spot = self.tiles_at(new_y, new_x);

        let mut can_move = true;

        for tile in tiles_at_new_spot {
            if tile.tile_type.is_solid() {
                // Don't move the player
                return;
            }
            if tile.tile_type.is_pushable() {
                if tiles_past_tile == 0 {
                    tile.move_tile(direction);
                } else {
                    can_move = false;
                    break;
                }
            }
        }
        if can_move {
            player.move_pos(direction);
        }
    }
    pub fn update_button_status(&mut self, player: &Player) {
        let mut ids_satiated: HashMap<Id, bool> = HashMap::new();
        {
            let buttons: Vec<&Tile> = self
                .tile_map
                .iter()
                .filter(|t| match t.tile_type {
                    TileType::Button(..) => true,
                    _ => false,
                })
                .collect();
            let push_boxes: Vec<&Tile> = self
                .tile_map
                .iter()
                .filter(|t| t.tile_type == TileType::PushBox)
                .collect();
            for button in buttons {
                if let TileType::Button(id) = button.tile_type {
                    if !ids_satiated.contains_key(&id) {
                        ids_satiated.insert(id, true);
                    }
                    let mut touched_by_box = false;
                    for pbox in &push_boxes {
                        if pbox.y == button.y && pbox.x == button.x {
                            touched_by_box = true;
                            break;
                        }
                    }
                    if (player.y != button.y || player.x != button.x) && !touched_by_box {
                        ids_satiated.insert(id, false);
                    }
                }
            }
        }
        let doors: Vec<&mut Tile> = self
            .tile_map
            .iter_mut()
            .filter(|t| match t.tile_type {
                TileType::Door(_, _) => true,
                _ => false,
            })
            .collect();

        for door in doors {
            if let TileType::Door(id, _) = door.tile_type {
                if id.is_none() {
                    continue;
                }
                if *(ids_satiated.get(&id.unwrap()).unwrap_or(&false)) {
                    door.tile_type = TileType::Door(id, true);
                }
            }
        }
    }
}

pub fn get_maps() -> Vec<MapData> {
    vec![
        // Level 1
        MapData {
            tile_map: [
                Tile::new_wall(0, 0, TileType::Wall1, Direction::Right, 30),
                Tile::new_wall(0, 0, TileType::Wall1, Direction::Down, 7),
                Tile::new_wall(6, 0, TileType::Wall1, Direction::Right, 23),
                Tile::new_wall(6, 23, TileType::Wall1, Direction::Down, 10),
                Tile::new_wall(16, 23, TileType::Wall1, Direction::Right, 7),
                Tile::new_wall(16, 29, TileType::Wall1, Direction::Up, 17),
                vec![tile!(13, 26, TileType::WinPad)],
            ]
            .concat(),
            player_spawn: (3, 3),
            flavor_text: Some("Welcome".to_string()),
        },
        // Level 2
        MapData {
            tile_map: [
                Tile::new_wall(15, 5, TileType::Wall1, Direction::Right, 30),
                Tile::new_wall(14, 34, TileType::Wall1, Direction::Up, 15),
                Tile::new_wall(0, 34, TileType::Wall1, Direction::Left, 35),
                Tile::new_wall(0, 0, TileType::Wall1, Direction::Down, 10),
                Tile::new_wall(9, 0, TileType::Wall1, Direction::Right, 31),
                Tile::new_wall(9, 32, TileType::Wall1, Direction::Right, 2),
                Tile::new_wall(14, 5, TileType::Wall1, Direction::Up, 5),
                Tile::new_wall(14, 17, TileType::Wall1, Direction::Up, 2),
                vec![tile!(12, 17, TileType::Door(Some(0), false))],
                Tile::new_wall(11, 17, TileType::Wall1, Direction::Up, 2),
                vec![tile!(10, 11, TileType::Button(0))],
                vec![tile!(7, 2, TileType::WinPad)],
                vec![tile!(9, 31, TileType::Door(Some(1), false))],
                vec![tile!(14, 33, TileType::Button(1))],
            ]
            .concat(),
            player_spawn: (14, 6),
            flavor_text: Some("Buttons? What do they do?".to_string()),
        },
        // Level 3
        MapData {
            tile_map: [
                Tile::new_wall(0, 0, TileType::Wall1, Direction::Right, 40),
                Tile::new_wall(0, 0, TileType::Wall1, Direction::Down, 6),
                Tile::new_wall(6, 0, TileType::Wall1, Direction::Right, 40),
                Tile::new_wall(0, 39, TileType::Wall1, Direction::Down, 6),
                Tile::new_wall(0, 28, TileType::Wall1, Direction::Down, 3),
                Tile::new_wall(6, 28, TileType::Wall1, Direction::Up, 3),
                vec![tile!(3, 28, TileType::Door(Some(0), false))],
                vec![tile!(2, 24, TileType::Button(0))],
                vec![tile!(4, 24, TileType::Button(0))],
                vec![tile!(3, 10, TileType::PushBox)],
                vec![tile!(3, 35, TileType::WinPad)],
            ]
            .concat(),
            player_spawn: (3, 3),
            flavor_text: Some("You must activate both buttons at once.".to_string()),
        },
    ]
}
