use crate::users::try_to_login;
use anyhow::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::broadcast::Sender,
};

const MAX_LINE_LENGTH: usize = 1024;
const LOGIN_PROMPT: &str = "Please enter [username]:[password]\n";
const WELCOME_MSG: &str = "Welcome to the chat!\n";
const BAD_LOGIN_MSG: &str = "Wrong username or password\n";
const SYSTEM_MSG_PREF: &str = "SYSTEM:";

fn get_time() -> String {
    chrono::Utc::now().format("%H:%M").to_string()
}

fn system_msg(msg: &str) -> String {
    format!("{SYSTEM_MSG_PREF} [{}] {msg}\n", get_time())
}
fn normal_msg(uname: &str, msg: &str) -> String {
    format!("[{}] {uname}: {msg}\n", get_time())
}

pub async fn handler(mut socket: TcpStream, tx: Sender<String>) -> Result<()> {
    let mut buf = [0u8; MAX_LINE_LENGTH];

    // Phase 1: Verification
    socket
        .write_all(system_msg(LOGIN_PROMPT).as_bytes())
        .await?;
    let uname: String;
    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            return Ok(());
        }
        let msg = String::from_utf8_lossy(&buf[..n]).trim().to_string();

        if let Some(u) = try_to_login(msg) {
            uname = u;
            socket.write_all(system_msg(WELCOME_MSG).as_bytes()).await?;
            break;
        } else {
            socket
                .write_all(system_msg(BAD_LOGIN_MSG).as_bytes())
                .await?;
        }
    }

    // Phase 2: Proxy messages to other clients
    let mut rx = tx.subscribe();
    tx.send(system_msg(format!("{uname} logged in\n").as_str()))?;

    loop {
        tokio::select! {
            Ok(msg) = rx.recv() => {
                socket.write_all(msg.as_bytes()).await?
            }
            Ok(n) = socket.read(&mut buf) => {
                if n == 0 {
                    tx.send(system_msg(format!("{uname} logged out\n").as_str()))?;
                    return Ok(());
                }
                let msg = String::from_utf8_lossy(&buf[..n]).trim().to_string();
                tx.send(normal_msg(&uname, &msg))?;
            }
        }
    }
}
