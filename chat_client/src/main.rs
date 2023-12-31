use anyhow::Result;
use clap::Parser;
use crossterm::{
    event::{self, Event::Key, KeyCode::Char},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    prelude::*,
    widgets::{block::Position, *},
};
use std::io::prelude::*;
use std::net::TcpStream;

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
    #[clap(default_value = "0.0.0.0")]
    ip: String,
    #[clap(default_value = "8080")]
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
    fn submit_message(&mut self) -> Result<()> {
        if !self.input.is_empty() {
            self.server_socket.write_all(self.input.as_bytes())?;
        }
        Ok(())
    }
    fn clear_input(&mut self) {
        self.input.clear();
    }
    fn get_messages(&mut self) -> Result<()> {
        let mut buffer = [0; MAX_LENGTH + 24]; // +24 for username and timestamp
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

fn split_line(line: &str, width: usize, input_mode: bool) -> (String, u16) {
    let mut ret = String::new();
    let mut line = line.to_string();
    if line.starts_with(SYSTEM_MSG_PREFIX) {
        line = line[SYSTEM_MSG_PREFIX.len()..].to_string();
    }
    let mut lines_used = 1;
    while line.len() > width {
        let mut i = width;
        let ibeg = i;
        while i >= 1 && !line.chars().nth(i - 1).unwrap().is_whitespace() {
            i -= 1;
        }
        if i == 0 {
            i = ibeg;
        }
        ret.push_str(&line[..i]);
        ret.push('\n');
        lines_used += 1;
        line = line[i..].to_string();
    }
    ret.push_str(&line);
    if line.len() == width && input_mode == true {
        lines_used += 1;
    }
    (ret, lines_used)
}

fn gen_color(uname: String) -> Color {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    uname.hash(&mut hasher);
    let hash = hasher.finish();
    let colors = [
        Color::LightRed,
        Color::LightGreen,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightCyan,
    ];
    colors[(hash % colors.len() as u64) as usize]
}

fn ui(app: &App, f: &mut Frame) {
    let (user_input, lines_used) = split_line(&app.input, f.size().width as usize - 2, true);
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1),
            Constraint::Min(1),
            Constraint::Length(2 + lines_used),
        ])
        .split(f.size());

    let msg = vec![
        "Press ".into(),
        "ESC".bold(),
        " to exit, ".into(),
        "ENTER".bold(),
        " to send message. ".into(),
        "Input length: ".into(),
        format!("{}/{}", app.input.len(), MAX_LENGTH).bold(),
        ". UTC time: ".into(),
        format!("{}", chrono::Utc::now().format("%H:%M:%S")).bold(),
    ];

    let help_message = Paragraph::new(Text::from(Line::from(msg)));
    f.render_widget(help_message, chunks[0]);

    let mut msgs = app.messages.clone();
    let mut sum_lengths = msgs
        .iter()
        .map(|m| split_line(m, chunks[1].width as usize - 2, false).1)
        .sum::<u16>() as usize;
    while sum_lengths + 2 < chunks[1].height as usize {
        msgs.insert(0, "".to_string());
        sum_lengths += 1;
    }
    while sum_lengths + 2 > chunks[1].height as usize {
        if msgs.len() == 0 {
            f.render_widget(
                Paragraph::new(Text::from(
                    "Your input text is too long for such small terminal height!",
                )),
                chunks[2],
            );
            return;
        }
        sum_lengths -= split_line(&msgs[0], chunks[1].width as usize - 2, false).1 as usize;
        msgs.remove(0);
    }

    let messages: Vec<ListItem> = msgs
        .iter()
        .map(|m| {
            ListItem::new(Text::from(
                split_line(m, chunks[1].width as usize - 2, false).0,
            ))
            .style(Style::default().fg(if m.starts_with(SYSTEM_MSG_PREFIX) {
                Color::LightYellow
            } else if m.len() > 0 {
                gen_color(m.split(' ').nth(1).unwrap().to_string())
            } else {
                Color::default()
            }))
        })
        .collect();

    let args = Server::parse();
    let messages = List::new(messages).block(
        Block::default()
            .borders(Borders::ALL)
            .title(format!("Chat @ {}:{}", args.ip, args.port))
            .title_position(Position::Bottom),
    );
    f.render_widget(messages, chunks[1]);

    let input = Paragraph::new(user_input.clone())
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[2]);

    let mut cursor_x = chunks[2].x + 1;
    let mut cursor_y = chunks[2].y + 1;
    let mut chars_seen = 0;
    for c in user_input.chars() {
        if c == '\n' {
            cursor_x = chunks[2].x + 1;
            cursor_y += 1;
        } else {
            if chars_seen == app.cursor_position {
                break;
            }
            chars_seen += 1;
            cursor_x += 1;
        }
    }

    f.set_cursor(cursor_x, cursor_y);
}

fn update(app: &mut App) -> Result<()> {
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

fn run() -> Result<()> {
    let mut t = Terminal::new(CrosstermBackend::new(std::io::stderr()))?;

    let args = Server::parse();
    let mut app = App {
        input: String::new(),
        cursor_position: 0,
        messages: vec![],
        server_socket: TcpStream::connect(format!("{}:{}", args.ip, args.port))?,
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

#[cfg(test)]
mod tests {
    use super::*;
    const SAMPLE_TEXT: &str =
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus. Suspendisse";

    #[test]
    fn test_invalid_cursor_moves() {
        let args = Server::parse();
        let mut app = App {
            input: SAMPLE_TEXT.to_string(),
            cursor_position: 0,
            messages: vec![],
            server_socket: TcpStream::connect(format!("{}:{}", args.ip, args.port)).unwrap(),
            should_quit: false,
        };
        app.move_cursor_left();
        assert_eq!(app.cursor_position, 0);
        for i in 0..app.input.len() {
            app.move_cursor_right();
            assert_eq!(app.cursor_position, i + 1);
        }
        app.move_cursor_right();
        assert_eq!(app.cursor_position, app.input.len());
    }

    #[test]
    fn test_inserts_and_deletions() {
        let args = Server::parse();
        let mut app = App {
            input: SAMPLE_TEXT.to_string(),
            cursor_position: 0,
            messages: vec![],
            server_socket: TcpStream::connect(format!("{}:{}", args.ip, args.port)).unwrap(),
            should_quit: false,
        };
        app.add_char('a');
        assert_eq!(app.input, format!("a{}", SAMPLE_TEXT));
        app.remove_char();
        assert_eq!(app.input, SAMPLE_TEXT);
        app.move_cursor_right();
        app.move_cursor_right();
        app.add_char('a');
        assert_eq!(app.input, format!("Loarem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus. Suspendisse"));
        app.move_cursor_left();
        app.move_cursor_left();
        app.remove_char();
        assert_eq!(app.input, format!("oarem ipsum dolor sit amet, consectetur adipiscing elit. Sed non risus. Suspendisse"));
    }

    #[test]
    fn test_split_line() {
        let (line, lines_used) = split_line(SAMPLE_TEXT, 10, false);
        assert_eq!(lines_used, 12);
        assert_eq!(
            line,
            "Lorem \nipsum \ndolor sit \namet, \nconsectetu\nr \nadipiscing\n elit. \nSed non \nrisus. \nSuspendiss\ne"
        );
    }

    #[test]
    fn test_messages() {
        let args = Server::parse();
        let mut app = App {
            input: SAMPLE_TEXT.to_string(),
            cursor_position: 0,
            messages: vec![],
            server_socket: TcpStream::connect(format!("{}:{}", args.ip, args.port)).unwrap(),
            should_quit: false,
        };
        app.get_messages().unwrap();
        assert_eq!(
            app.messages[0],
            "SYSTEM: [00:00] Please enter [username]:[password]"
        );

        app.input = "ptr:123456".to_string();
        app.submit_message().unwrap();

        app.get_messages().unwrap();
        assert_eq!(app.messages[1], "SYSTEM: [00:00] Welcome to the chat!");

        app.get_messages().unwrap();
        assert_eq!(app.messages[2], "SYSTEM: [00:00] ptr logged in");

        app.input = "Hello there!".to_string();
        app.submit_message().unwrap();

        app.get_messages().unwrap();
        assert_eq!(app.messages[3], "[00:00] ptr: Hello there!");
    }
}
