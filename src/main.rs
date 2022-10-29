use std::io::{Write, stdout};

use crossterm::{
    terminal,
    event,
    cursor,
    self,
    ExecutableCommand,
};

mod game;
use game::*;

fn main() {
    let mut minesweeper = Game::new(10, 5);
    let initial_cursor = minesweeper.resize(terminal::size().unwrap().0, terminal::size().unwrap().1);
    let mine_count: u32 = ((minesweeper.height * minesweeper.width) as f64 / 5.0).floor() as u32;
    println!("generating map with {} mines", mine_count);
    minesweeper.new_map(mine_count);
    terminal::enable_raw_mode().unwrap();
    stdout().execute(terminal::EnterAlternateScreen).unwrap();
    stdout().execute(terminal::SetTitle("Minesweeper")).unwrap();
    stdout().execute(event::EnableMouseCapture).unwrap();
    stdout().execute(cursor::MoveTo(0,0)).unwrap();
    write!(stdout(), "{}", minesweeper).unwrap();
    stdout().flush().unwrap();
    if let Status::MoveCursor(column, row) = initial_cursor {
        stdout().execute(cursor::MoveTo(column, row)).unwrap();
    }
    let result = 'main: loop {
        stdout().execute(cursor::SavePosition).unwrap();
        stdout().execute(cursor::MoveTo(0,0)).unwrap();
        write!(stdout(), "{}", minesweeper).unwrap();
        stdout().flush().unwrap();
        stdout().execute(cursor::RestorePosition).unwrap();
        let status: Status = match event::read().unwrap() {
            event::Event::Key(event) => minesweeper.key_event(event),
            event::Event::Mouse(event) => minesweeper.mouse_event(event),
            event::Event::Resize(columns, rows) => minesweeper.resize(columns, rows),
            _ => Status::Nothing,
        };
        match status {
            Status::MoveCursor(column, row) => { stdout().execute(cursor::MoveTo(column, row)).unwrap(); () },
            Status::End(reason) => break 'main reason,
            Status::Nothing => (),
        };
    };
    stdout().execute(terminal::LeaveAlternateScreen).unwrap();
    stdout().execute(event::DisableMouseCapture).unwrap();
    terminal::disable_raw_mode().unwrap();
    println!("{}", result);
}
