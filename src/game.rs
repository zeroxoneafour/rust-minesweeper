// game.rs - Contains the code for the game

use std::fmt;
use rand;

pub use crossterm::event;

#[derive(Copy, Clone)]
enum TileType {
    Clear,
    Close(u8),
    Mine,
}

#[derive(Copy, Clone)]
enum TileState {
    Revealed,
    Marked,
    Hidden,
}

#[derive(Copy, Clone)]
struct Tile {
    tiletype: TileType,
    state: TileState,
}

impl fmt::Display for Tile {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let tile = match self.state {
            TileState::Revealed => {
                match self.tiletype {
                    TileType::Clear => "_".to_string(),
                    TileType::Close(x) => x.to_string(),
                    TileType::Mine => "O".to_string(),
                }
            },
            TileState::Marked => "X".to_string(),
            TileState::Hidden => "█".to_string(),
        };
        write!(f, "{}", tile)
    }
}

impl Default for Tile {
    fn default() -> Tile {
        Tile{tiletype: TileType::Clear, state: TileState::Hidden}
    }
}

pub enum Status {
    MoveCursor(u16, u16),
    End(String),
    Nothing,
}

// global state of a Game, separated because it can get complex
#[derive(Default, Clone)]
struct State {
    term_width: u16,
    term_height: u16,
    boundary_left: u32,
    boundary_top: u32,
    cursor_column: u16,
    cursor_row: u16,
}

// a parameter to a function to update a State
#[derive(Default)]
pub struct StateUpdate {
    term_width: Option<u16>,
    term_height: Option<u16>,
    cursor_column: Option<u16>,
    cursor_row: Option<u16>,
}

#[derive(Default, Clone)]
pub struct Game {
    map: Vec<Tile>,
    mines: Vec<u32>,
    pub width: u32,
    pub height: u32,
    state: State,
    not_first_mine: bool,
}

impl Game {
    pub fn new(width: u32, height: u32) -> Game {
        let mut ret = Game::default();
        (ret.width, ret.height) = (width, height);
        ret
    }
    pub fn new_map(&mut self, mine_count: u32) {
        self.mines = vec![0 as u32; mine_count as usize];
        for i in 0..mine_count as usize {
            self.mines[i] = rand::random::<u32>() % (self.width * self.height);
        }
        self.map = generate_map(self.width, self.height, self.mines.to_vec());
    }
    pub fn key_event(&mut self, event: event::KeyEvent) -> Status {
        match event.code {
            event::KeyCode::Left => self.state.update(self.clone(), StateUpdate{cursor_column: Some(self.state.cursor_column - 1), ..StateUpdate::default()}),
            event::KeyCode::Right => self.state.update(self.clone(), StateUpdate{cursor_column: Some(self.state.cursor_column + 1), ..StateUpdate::default()}),
            event::KeyCode::Up => self.state.update(self.clone(), StateUpdate{cursor_row: Some(self.state.cursor_row - 1), ..StateUpdate::default()}),
            event::KeyCode::Down => self.state.update(self.clone(), StateUpdate{cursor_row: Some(self.state.cursor_row + 1), ..StateUpdate::default()}),
            event::KeyCode::Enter => dig_tile(self),
            event::KeyCode::Char(c) => {
                if c == ' ' {
                    dig_tile(self)
                } else if c == 'x' || c == 'f' {
                    mark_tile(self);
                    Status::Nothing
                } else if c == 'p' { // for debugging
                    for i in 0..self.map.len() {
                        self.map[i].state = TileState::Revealed;
                    }
                    Status::Nothing
                } else {
                    Status::Nothing
                }
            },
            event::KeyCode::Esc => Status::End("User pressed escape".to_string()),
            _ => Status::Nothing,
        }
    }
    pub fn mouse_event(&mut self, event: event::MouseEvent) -> Status {
        if let event::MouseEventKind::Down(button) = event.kind {
            let mut survived = true;
            let ret = self.state.update(self.clone(), StateUpdate{cursor_column: Some(event.column), cursor_row: Some(event.row), ..StateUpdate::default()});
            if let event::MouseButton::Left = button {
                dig_tile(self)
            } else if let event::MouseButton::Right = button {
                mark_tile(self);
                Status::Nothing
            } else {
                Status::Nothing
            }
        } else {
            Status::Nothing
        }
    }
    pub fn resize(&mut self, columns: u16, rows: u16) -> Status {
        self.state.update(self.clone(), StateUpdate{term_width: Some(columns), term_height: Some(rows), ..StateUpdate::default()})
    }
}

impl State {
    fn update(&mut self, game: Game, update: StateUpdate) -> Status {
        if let Some(term_width) = update.term_width {
            self.term_width = term_width;
        }
        if let Some(term_height) = update.term_height {
            self.term_height = term_height;
        }

        // cursor does bounds checking to make sure it's inside the game
        if (self.cursor_column as u32) < self.boundary_left || (self.cursor_column as u32) > self.boundary_left + game.width + 1 {
            self.cursor_column = (self.boundary_left + 1) as u16;
        }
        if (self.cursor_row as u32) < self.boundary_top || (self.cursor_row as u32) > self.boundary_top + game.width + 1 {
            self.cursor_row = (self.boundary_top + 1) as u16;
        }

        if let Some(cursor_column) = update.cursor_column {
            if (cursor_column as u32) > self.boundary_left && (cursor_column as u32) < self.boundary_left + game.width + 1 {
                self.cursor_column = cursor_column;
            }
        }
        if let Some(cursor_row) = update.cursor_row {
            if (cursor_row as u32) > self.boundary_top && (cursor_row as u32) < self.boundary_top + game.height + 1 {
                self.cursor_row = cursor_row;
            }
        }
        let boundary_left: i32 = ((self.term_width as i32 - game.width as i32 - 2) as f32 / 2f32).ceil() as i32;
        if boundary_left < 0 {
            return Status::End("Game window too small! Aborting".to_string());
        } else {
            self.boundary_left = boundary_left as u32;
        }
        let boundary_top: i32 = ((self.term_height as i32 - game.height as i32 - 2) as f32 / 2f32).ceil() as i32;
        if boundary_top < 0 {
            return Status::End("Game window too small! Aborting".to_string());
        } else {
            self.boundary_top = boundary_top as u32;
        }
        return Status::MoveCursor(self.cursor_column, self.cursor_row)
    }
}

impl fmt::Display for Game {
    fn fmt (&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ret = "".to_string();
        let boundary_right = self.state.term_width as u32 - self.state.boundary_left - self.width - 2;
        let boundary_bottom = self.state.term_height as u32 - self.state.boundary_top - self.height - 2;
        // top padding
        for _ in 0..self.state.boundary_top {
            ret.push_str("\n");
        }
        // top border
        for _ in 0..self.state.boundary_left {
            ret.push_str(" ");
        }
        for _ in 0..(self.width + 2) {
            ret.push_str("#");
        }
        for _ in 0..boundary_right {
            ret.push_str(" ");
        }
        // main section
        for i in 0..self.height {
            // side padding and border
            for _ in 0..self.state.boundary_left {
                ret.push_str(" ");
            }
            ret.push_str("#");
            // actual map (most of the conversion has been hoisted off to Tile::Display)
            for j in 0..self.width {
                ret.push_str(&format!("{}", self.map[(i*self.width + j) as usize]))
            }
            // side border and padding
            ret.push_str("#");
            for _ in 0..boundary_right {
                ret.push_str(" ");
            }
        }
        // bottom border
        for _ in 0..self.state.boundary_left {
            ret.push_str(" ");
        }
        for _ in 0..(self.width + 2) {
            ret.push_str("#");
        }
        for _ in 0..boundary_right {
            ret.push_str(" ");
        }
        // bottom padding
        for _ in 0..boundary_bottom {
            ret.push_str("\n");
        }
        write!(f, "{}", ret)
    }
}

fn generate_map(width: u32, height: u32, mines: Vec<u32>) -> Vec<Tile> {
    let board_size = width * height;
    let mut ret: Vec<Tile> = vec![Tile::default(); board_size as usize];
    for i in 0..board_size {
        if mines.contains(&i) {
            ret[i as usize].tiletype = TileType::Mine;
        } else {
            let mut close_mines: u8 = 0;
            // TODO - Move this code to update surrounding squares in the mine detection code above,
            // would be much more efficient than looking in every empty square for nearby mines

            // here we check for mines in proximity. probably could be more efficient but fuck you i guess
            // i was thinking about edge cases where the mines are placed on the outskirts of blocks
            // and realized that using signed ints would solve the integer overflow, so here you go
            let column = (i as f32 / width as f32).floor() as u32;
            let row = i % width;
            if column > 0 {
                if row > 0 && mines.contains(&((i - width) - 1)) { // top left
                    close_mines += 1;
                }
                if mines.contains(&(i - width)) { // top
                    close_mines += 1;
                }
                if row < width - 1 && mines.contains(&((i - width) + 1)) { // top right
                    close_mines += 1;
                }
            }
            if row > 0 && mines.contains(&(i- 1)) { // left
                close_mines += 1;
            }
            if row < width - 1 && mines.contains(&(i + 1)) { // right
                close_mines += 1;
            }
            if column < height - 1 {
                if row > 0 && mines.contains(&((i + width) - 1)) { // bottom left
                    close_mines += 1;
                }
                if mines.contains(&(i + width)) { // bottom
                    close_mines += 1;
                }
                if row < width - 1 && mines.contains(&((i + width) + 1)) { // bottom right
                    close_mines += 1;
                }
            }
            if close_mines != 0 {
                ret[i as usize].tiletype = TileType::Close(close_mines);
            }
        }
    }
    ret
}

#[inline]
fn get_cursor_relative(game: &Game) -> (u32, u32) { // column, row (x, y)
    ((game.state.cursor_column as u32) - game.state.boundary_left - 1, (game.state.cursor_row as u32) - game.state.boundary_top - 1)
}

#[inline]
fn get_coord_vec(coord: (u32, u32), width: u32) -> usize {
    ((coord.1 * width) + coord.0) as usize
}

#[inline]
fn get_vec_coord(vec: usize, width: u32) -> (u32, u32) { // (x, y)
    (vec as u32 % width, (vec as f32 / width as f32).floor() as u32)
}

fn mark_tile(game: &mut Game) {
    let cursor = get_cursor_relative(game);
    let cursor_vec = get_coord_vec(cursor, game.width);
    match game.map[cursor_vec].state {
        TileState::Revealed => (),
        TileState::Marked => game.map[cursor_vec].state = TileState::Hidden,
        TileState::Hidden => game.map[cursor_vec].state = TileState::Marked,
    }
}

fn dig_tile(game: &mut Game) -> Status {
    let cursor = get_cursor_relative(game);
    let cursor_vec = get_coord_vec(cursor, game.width);

    if !game.not_first_mine {
        game.not_first_mine = true;
        for i in -3..4 as i32 { // column
            for j in -3..4 as i32 { // row
                let column = i + cursor.0 as i32;
                let row = j + cursor.1 as i32;
                if
                    column > -1 &&
                    column < game.width as i32 &&
                    row > -1 &&
                    row < game.height as i32
                {
                    let coord = get_coord_vec((column as u32, row as u32), game.width) as u32;
                    if game.mines.contains(&coord) {
                        let index = game.mines.iter().position(move |&x| x == coord).unwrap();
                        game.mines.remove(index);
                    }
                }
            }
        }
        game.map = generate_map(game.width, game.height, game.mines.to_vec());
    }

    if let TileState::Hidden = game.map[cursor_vec].state {
        match game.map[cursor_vec].tiletype {
            TileType::Close(_) => {game.map[cursor_vec].state = TileState::Revealed; check_map(&game.map, &game.mines)},
            TileType::Mine => Status::End("You dug a mine!".to_string()),
            TileType::Clear => {
                let clear = minesweeper(&game.map, cursor_vec, game.width, game.height);
                for &i in clear.iter() {
                    game.map[i].state = TileState::Revealed;
                }
                check_map(&game.map, &game.mines)
            },
        }
    } else {
        Status::Nothing
    }
}

// checks if everything is revealed except mines
fn check_map(map: &Vec<Tile>, mines: &Vec<u32>) -> Status {
    for i in 0..map.len() {
        if let TileState::Revealed = map[i].state {
        } else {
            if !mines.contains(&(i as u32)) {
                return Status::Nothing
            }
        }
    }
    Status::End("You win!".to_string())
}

// function that takes a map and returns a list of points on the map to reveal, given a starting point
fn minesweeper(map: &Vec<Tile>, cursor_vec: usize, width: u32, height: u32) -> Vec<usize> {
    // here is implemented a custom sweeping algorithm
    // details are in algorithm.md
    let mut ret = Vec::<usize>::default();
    let mut edge = Vec::<usize>::default();
    edge.push(cursor_vec);
    while edge.len() > 0 {
        let current_edge = edge.clone();
        while let Some(i) = edge.pop() {
            ret.push(i);
        }
        for i in current_edge {
            if let TileType::Clear = map[i].tiletype {
                let current_coord = get_vec_coord(i, width);
                for c_shift in -1..2 {
                    for r_shift in -1..2 {
                        let column = c_shift + current_coord.0 as i32;
                        let row = r_shift + current_coord.1 as i32;
                        if
                            column > -1 &&
                            column < width as i32 &&
                            row > -1 &&
                            row < height as i32 &&
                            c_shift.abs() != r_shift.abs() // no corners
                        {
                            let current_vec = get_coord_vec((column as u32, row as u32), width);
                            if !(ret.contains(&current_vec) || edge.contains(&current_vec)) {
                                edge.push(current_vec);
                            }
                        }
                    }
                }
            }
            // because numbers always border mines, anything else would be a number and wouldn't expand the edge
        }
    }
    ret
}

// tests for game
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn tile_test() {
        let clear_revealed = Tile{tiletype: TileType::Clear, state: TileState::Revealed}.to_string();
        assert!(clear_revealed.eq(&String::from("_")));
        let close_revealed = Tile{tiletype: TileType::Close(3), state: TileState::Revealed}.to_string();
        assert!(close_revealed.eq(&String::from("3")));
        let mine_revealed = Tile{tiletype: TileType::Mine, state: TileState::Revealed}.to_string();
        assert!(mine_revealed.eq(&String::from("O")));
        let hidden = Tile{tiletype: TileType::Clear, state: TileState::Hidden}.to_string();
        assert!(hidden.eq(&String::from("█")));
        let marked = Tile{tiletype: TileType::Clear, state: TileState::Marked}.to_string();
        assert!(marked.eq(&String::from("X")));
    }
}
