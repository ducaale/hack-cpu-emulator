use std::env;
use std::path::Path;
use std::io::{stdout, Write};
use std::time::Duration;

use tui::Terminal;
use tui::backend::CrosstermBackend;
use crossterm::event::{poll, read, Event, KeyCode};
use crossterm::{execute, terminal::{EnterAlternateScreen, LeaveAlternateScreen}};

mod assembler;
mod computer;
mod utils;
mod app;

use assembler::assemble;
use utils::lines_from_file;
use app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let args: Vec<String> = env::args().collect();
    let filename = Path::new(&args[1]).file_name().unwrap().to_string_lossy();
    let input = lines_from_file(&args[1]).expect("could not read file");
    let program = assemble(&input);
    let mut app = App::new(filename.to_string(), program);

    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;

    loop {
        terminal.draw(|mut f| app.draw(&mut f))?;

        if poll(Duration::from_millis(500))? {
            if let Event::Key(key) = read()? {
                if key.code == KeyCode::Char('q') {
                    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                    terminal.show_cursor()?;
                    break;
                }
                app.handle_input(key.code);
            }
        }
    }

    Ok(())
}
