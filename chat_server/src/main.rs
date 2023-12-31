use tokio::{net::TcpListener, sync::broadcast};

mod handler;
mod users;

const PORT: u16 = 8080;

#[tokio::main]
async fn main() {
    let (tx, mut rx) = broadcast::channel::<String>(16);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", PORT))
        .await
        .unwrap();

    println!("Listening on {}", listener.local_addr().unwrap());

    #[cfg(debug_assertions)]
    tokio::spawn(async move {
        loop {
            let msg = rx.recv().await.unwrap().trim().to_string();
            println!("[BROADCAST]: {}", msg);
        }
    });

    loop {
        let (socket, addr) = listener.accept().await.unwrap();

        let tx = tx.clone();

        tokio::spawn(async move {
            println!("{} connected", addr);
            match handler::handler(socket, tx).await {
                Ok(_) => println!("{} disconnected", addr),
                Err(e) => eprintln!("{} error: {:?}", addr, e),
            }
        });
    }
}

// we run the server with `cargo run --bin chat-server`
// and then we test it with `cargo test --bin chat-server`

#[cfg(test)]
mod tests {
    use crate::handler::*;
    use serial_test::serial;

    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;

    fn get_socket() -> TcpStream {
        TcpStream::connect(format!("0.0.0.0:{}", PORT)).unwrap()
    }
    fn read_data(socket: &mut TcpStream) -> String {
        let mut buf = [0; handler::MAX_LINE_LENGTH];
        let n = socket.read(&mut buf).unwrap();
        String::from_utf8_lossy(&buf[..n]).to_string()
    }
    fn send_data(socket: &mut TcpStream, data: &str) {
        socket.write_all(data.as_bytes()).unwrap();
    }

    #[test]
    #[serial]
    fn test_login_msg() {
        let mut socket = get_socket();
        assert_eq!(read_data(&mut socket), system_msg(LOGIN_PROMPT));
        socket.shutdown(std::net::Shutdown::Both).unwrap();
    }

    #[test]
    #[serial]
    fn wrong_login() {
        let mut socket = get_socket();
        read_data(&mut socket); // skip login prompt
        send_data(&mut socket, "wrong:123:543\n");
        assert_eq!(read_data(&mut socket), system_msg(BAD_LOGIN_MSG));
        send_data(&mut socket, "aaaaaaaaaaaaaaaaaaaaaaaaaaa:123456\n");
        assert_eq!(read_data(&mut socket), system_msg(BAD_LOGIN_MSG));
        send_data(&mut socket, "aa:a:123456\n");
        assert_eq!(read_data(&mut socket), system_msg(BAD_LOGIN_MSG));
        socket.shutdown(std::net::Shutdown::Both).unwrap();
    }

    #[test]
    #[serial]
    fn correct_login() {
        let mut socket = get_socket();
        read_data(&mut socket); // skip login prompt
        send_data(&mut socket, "piotrek:123456\n");
        assert_eq!(read_data(&mut socket), system_msg(WELCOME_MSG));
        assert_eq!(
            read_data(&mut socket),
            system_msg(format!("piotrek logged in\n").as_str())
        );
        socket.shutdown(std::net::Shutdown::Both).unwrap();
    }

    #[test]
    #[serial]
    fn test_broadcast() {
        let mut socket_1 = get_socket();
        read_data(&mut socket_1); // skip login prompt
        send_data(&mut socket_1, "piotrek:123456\n");
        read_data(&mut socket_1); // skip welcome msg
        read_data(&mut socket_1); // skip login msg
        let mut socket_2 = get_socket();
        read_data(&mut socket_2); // skip login prompt
        send_data(&mut socket_2, "kasia:123456\n");
        read_data(&mut socket_2); // skip welcome msg
        read_data(&mut socket_2); // skip login msg
        assert_eq!(
            read_data(&mut socket_1),
            system_msg(format!("kasia logged in\n").as_str()).as_str()
        );

        send_data(&mut socket_1, "Hello!\n");
        assert_eq!(
            read_data(&mut socket_1),
            normal_msg("piotrek", "Hello!").as_str()
        );
        assert_eq!(
            read_data(&mut socket_2),
            normal_msg("piotrek", "Hello!").as_str()
        );
        send_data(&mut socket_2, "Hi!\n");
        assert_eq!(
            read_data(&mut socket_1),
            normal_msg("kasia", "Hi!").as_str()
        );
        assert_eq!(
            read_data(&mut socket_2),
            normal_msg("kasia", "Hi!").as_str()
        );

        socket_1.shutdown(std::net::Shutdown::Both).unwrap();
        assert_eq!(
            read_data(&mut socket_2),
            system_msg(format!("piotrek logged out\n").as_str()).as_str()
        );
        socket_2.shutdown(std::net::Shutdown::Both).unwrap();
    }
}
