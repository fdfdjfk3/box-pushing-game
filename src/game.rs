use crate::{
    map::{Event, MapData},
    Direction, TOP_PADDING,
};
use pancurses::Window;

pub struct Player {
    pub y: i32,
    pub x: i32,
    pub glyph: char,
}

impl Player {
    pub fn draw(&self, window: &Window) {
        window.mvaddch(self.y + TOP_PADDING, self.x, self.glyph);
    }
    // Don't call this directly. Use the function in GameContext for movement logic.
    pub fn move_pos(&mut self, direction: Direction) {
        let change = direction.get_vec2_move();
        self.y += change.0;
        self.x += change.1;
    }
}

pub struct GameContext {
    pub player: Player,
    pub map_data: Option<MapData>,
    pub map_list: Vec<MapData>,
    pub level: u32,
}

impl GameContext {
    pub fn load_current_level(&mut self) {
        let map = self.map_list.get(self.level as usize);
        if map.is_none() {
            todo!("Add a back-up map in case this fails");
        }
        self.map_data = map.cloned();
        let (new_y, new_x) = self.map_data.as_ref().unwrap().player_spawn;
        self.player.y = new_y;
        self.player.x = new_x;
    }
    pub fn increment_level(&mut self) {
        self.level += 1;
        self.load_current_level();
    }
    pub fn decrement_level(&mut self) {
        self.level -= 1;
        self.load_current_level();
    }
    pub fn player_movement(&mut self, direction: Direction) {
        self.map_data
            .as_mut()
            .unwrap()
            .player_move(&mut self.player, direction);
    }
    pub fn draw_all(&self, window: &Window) {
        self.map_data.as_ref().unwrap().draw(window);
        self.player.draw(window);
        let flavor_text = &self.map_data.as_ref().unwrap().flavor_text;
        window.mvprintw(
            TOP_PADDING - 1,
            0,
            format!(
                "level {}: {}",
                self.level,
                flavor_text.as_ref().unwrap_or(&"".to_string()),
            ),
        );
    }
    pub fn collect_events(&mut self) -> Vec<Event> {
        self.map_data
            .as_mut()
            .unwrap()
            .tiles_at(self.player.y, self.player.x)
            .into_iter()
            .map(|t| t.tile_type.stood_on_event())
            .collect()
    }
    pub fn update_all(&mut self) {
        let events: Vec<Event> = self.collect_events();
        for event in events {
            if event == Event::Win {
                self.increment_level();
            }
        }
        self.map_data
            .as_mut()
            .unwrap()
            .update_button_status(&self.player);
    }
}
