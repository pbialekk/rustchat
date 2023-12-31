use crate::app::App;
use anyhow::Result;
use crossterm::event::{self, Event::Key, KeyCode::Char};

pub fn update(app: &mut App) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    Char(c) => app.add_char(c),
                    event::KeyCode::Backspace => app.remove_char(),
                    event::KeyCode::Enter => {
                        app.submit_message()?;
                        app.reset_cursor();
                        app.clear_input();
                    }
                    event::KeyCode::Left => app.move_cursor_left(),
                    event::KeyCode::Right => app.move_cursor_right(),
                    event::KeyCode::Esc => app.should_quit = true,
                    _ => {}
                }
            }
        }
    }
    app.get_messages()?;
    Ok(())
}
