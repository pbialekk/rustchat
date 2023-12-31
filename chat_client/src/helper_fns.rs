use ratatui::style::Color;

use crate::SYSTEM_MSG_PREFIX;

pub fn split_line(line: &str, width: usize, input_mode: bool) -> (String, u16) {
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

pub fn gen_color(uname: String) -> Color {
    use std::hash::{Hash, Hasher};

    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    uname.hash(&mut hasher);
    let hash = hasher.finish();
    let colors = [
        Color::LightGreen,
        Color::LightBlue,
        Color::LightMagenta,
        Color::LightCyan,
        Color::White,
        Color::Blue,
        Color::Magenta,
        Color::Cyan,
    ];
    colors[(hash % colors.len() as u64) as usize]
}
