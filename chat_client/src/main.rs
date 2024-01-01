pub mod app;
pub mod helper_fns;
pub mod run;
pub mod tui;
pub mod ui;
pub mod update;

use crate::app::Server;
use crate::run::run;
use anyhow::Result;
use clap::Parser;

const MAX_LENGTH: usize = 1000;
const SYSTEM_MSG_PREFIX: &str = "SYSTEM: ";

fn main() -> Result<()> {
    let result = run();

    result?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::App;
    use std::net::TcpStream;
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
        let (line, lines_used) = crate::helper_fns::split_line(SAMPLE_TEXT, 10, false);
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
