use crate::app::App;
use crate::app::Server;
use crate::tui::Tui;
use crate::update::update;
use anyhow::Result;
use clap::Parser;
use ratatui::{backend::CrosstermBackend, Terminal};
use std::net::TcpStream;

pub fn run() -> Result<()> {
    let args = Server::parse();
    let mut app = App {
        input: String::new(),
        cursor_position: 0,
        messages: vec![],
        server_socket: TcpStream::connect(format!("{}:{}", args.ip, args.port))?,
        should_quit: false,
    };
    app.server_socket.set_nonblocking(true)?;

    let backend = CrosstermBackend::new(std::io::stderr());
    let terminal = Terminal::new(backend)?;
    let mut tui = Tui::new(terminal);

    tui.enter()?;

    while !app.should_quit {
        update(&mut app)?;

        tui.draw(&mut app)?;
    }

    tui.exit()?;
    Ok(())
}
