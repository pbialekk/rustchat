use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event::Key, KeyCode::Char},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{prelude::*, widgets::*};
use std::io::prelude::*;

fn startup() -> Result<()> {
    enable_raw_mode()?;
    execute!(std::io::stderr(), EnterAlternateScreen)?;
    Ok(())
}

fn shutdown() -> Result<()> {
    execute!(std::io::stderr(), LeaveAlternateScreen)?;
    disable_raw_mode()?;
    Ok(())
}

struct App {
    input: String,
    cursor_position: usize,
    messages: Vec<String>,
    server_socket: std::net::TcpStream,
    should_quit: bool,
}

const MAX_LENGTH: usize = 1000;
const SYSTEM_MSG_PREFIX: &str = "SYSTEM: ";

#[derive(Parser)]
struct Server {
    ip: String,
    port: u16,
}

impl App {
    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }
    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }
    fn add_char(&mut self, c: char) {
        if self.input.len() >= MAX_LENGTH {
            return;
        }
        self.input.insert(self.cursor_position, c);
        self.move_cursor_right();
    }
    fn remove_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input.remove(self.cursor_position);
        }
    }
    fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }
    fn submit_message(&mut self) {
        if !self.input.is_empty() {
            self.server_socket
                .write_all(self.input.as_bytes())
                .expect("Failed to send message");
        }
    }
    fn clear_input(&mut self) {
        self.input.clear();
    }
    fn get_messages(&mut self) -> Result<()> {
        let mut buffer = [0; MAX_LENGTH];
        match self.server_socket.read(&mut buffer) {
            Ok(n) => {
                for message in String::from_utf8_lossy(&buffer[..n]).split('\n') {
                    if !message.is_empty() {
                        self.messages.push(message.to_string());
                    }
                }
                assert!(n == 0 || buffer[n - 1] == b'\n');
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {}
            Err(e) => Result::Err(e)?,
        }
        Ok(())
    }
}

fn split_line(line: &str, width: usize) -> (String, u16) {
    let mut ret = String::new();
    let mut line = line.to_string();
    if line.starts_with(SYSTEM_MSG_PREFIX) {
        line = line[SYSTEM_MSG_PREFIX.len()..].to_string();
    }
    let mut lines_used = 1;
    while line.len() > width {
        let mut i = width;
        while !line.is_char_boundary(i) {
            i -= 1;
        }
        ret.push_str(&line[..i]);
        ret.push('\n');
        lines_used += 1;
        line = line[i..].to_string();
    }
    ret.push_str(&line);
    if line.len() == width {
        lines_used += 1;
    }
    (ret, lines_used)
}

fn ui(app: &App, f: &mut Frame) {
    let (user_input, lines_used) = split_line(&app.input, f.size().width as usize - 2);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Length(2 + lines_used),
            Constraint::Min(1),
        ])
        .split(f.size());

    let args = Server::parse();
    let msg = vec![
        "Press ".into(),
        "ESC".bold(),
        " to exit, ".into(),
        "ENTER".bold(),
        " to send message. ".into(),
        "System messages are in ".into(),
        "yellow"
            .set_style(Style::default().fg(Color::LightYellow))
            .bold(),
        ". Input length: ".into(),
        format!("{}/{}", app.input.len(), MAX_LENGTH).bold(),
        ". UTC time: ".into(),
        format!("{}", chrono::Utc::now().format("%H:%M:%S")).bold(),
        " Connected to ".into(),
        format!("{}:{}", args.ip, args.port).bold(),
    ];

    let help_message = Paragraph::new(Text::from(Line::from(msg)));
    f.render_widget(help_message, chunks[0]);

    let input =
        Paragraph::new(user_input).block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[1]);

    let cursor_x = chunks[1].x + (app.cursor_position as u16) % (chunks[1].width - 2) as u16;
    let cursor_y = chunks[1].y + (app.cursor_position as u16) / (chunks[1].width - 2) as u16;

    f.set_cursor(cursor_x + 1, cursor_y + 1);

    let messages: Vec<ListItem> = app
        .messages
        .iter()
        .rev()
        .map(|m| {
            ListItem::new(Text::from(split_line(m, chunks[2].width as usize).0)).style(
                Style::default().fg(if m.starts_with(SYSTEM_MSG_PREFIX) {
                    Color::LightYellow
                } else {
                    Color::default()
                }),
            )
        })
        .collect();
    let messages = List::new(messages).block(Block::default().borders(Borders::ALL).title("Chat"));
    f.render_widget(messages, chunks[2]);
}

fn update(app: &mut App) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                match key.code {
                    Char(c) => app.add_char(c),
                    event::KeyCode::Backspace => app.remove_char(),
                    event::KeyCode::Enter => {
                        app.submit_message();
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

fn run() -> Result<()> {
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    let args = Server::parse();
    let mut app = App {
        input: String::new(),
        cursor_position: 0,
        messages: vec![],
        server_socket: std::net::TcpStream::connect(format!("{}:{}", args.ip, args.port))?,
        should_quit: false,
    };
    app.server_socket.set_nonblocking(true)?;

    loop {
        update(&mut app)?;

        t.draw(|f| {
            ui(&app, f);
        })?;

        if app.should_quit {
            break;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    let _ = Server::parse(); // We check for valid arguments here, so user doesn't see garbage on the screen due to RAW mode being enabled

    startup()?;

    let result = run();

    shutdown()?;

    result?;

    Ok(())
}
