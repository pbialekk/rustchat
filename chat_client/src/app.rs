use crate::MAX_LENGTH;
use anyhow::Result;
use clap::Parser;
use std::io::prelude::*;

pub struct App {
    pub input: String,
    pub cursor_position: usize,
    pub messages: Vec<String>,
    pub server_socket: std::net::TcpStream,
    pub should_quit: bool,
}

#[derive(Parser)]
pub struct Server {
    #[clap(default_value = "0.0.0.0")]
    pub ip: String,
    #[clap(default_value = "8080")]
    pub port: u16,
}

impl App {
    pub fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }
    pub fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input.len() {
            self.cursor_position += 1;
        }
    }
    pub fn add_char(&mut self, c: char) {
        if self.input.len() >= MAX_LENGTH {
            return;
        }
        self.input.insert(self.cursor_position, c);
        self.move_cursor_right();
    }
    pub fn remove_char(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            self.input.remove(self.cursor_position);
        }
    }
    pub fn reset_cursor(&mut self) {
        self.cursor_position = 0;
    }
    pub fn submit_message(&mut self) -> Result<()> {
        if !self.input.is_empty() {
            self.server_socket.write_all(self.input.as_bytes())?;
        }
        Ok(())
    }
    pub fn clear_input(&mut self) {
        self.input.clear();
    }
    pub fn get_messages(&mut self) -> Result<()> {
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
