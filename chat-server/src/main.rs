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
