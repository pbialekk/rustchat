use crate::app::App;
use crate::app::Server;
use crate::ui::ui;
use crate::update::update;
use anyhow::Result;
use clap::Parser;
use ratatui::prelude::{CrosstermBackend, Terminal};
use std::net::TcpStream;

pub fn run() -> Result<()> {
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
