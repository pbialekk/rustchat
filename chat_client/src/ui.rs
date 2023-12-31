use clap::Parser;
use ratatui::{
    prelude::*,
    widgets::{block::Position, *},
};

use crate::app::App;
use crate::app::Server;
use crate::helper_fns::{gen_color, split_line};
use crate::{MAX_LENGTH, SYSTEM_MSG_PREFIX};

pub fn ui(app: &App, f: &mut Frame) {
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
