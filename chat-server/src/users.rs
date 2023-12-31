pub fn try_to_login(msg: String) -> Option<String> {
    let mut iter = msg.split(':');
    let uname = iter.next()?;
    let passwd = iter.next()?;
    if passwd == "123456" && uname.len() < 12 {
        Some(uname.to_string())
    } else {
        None
    }
}
