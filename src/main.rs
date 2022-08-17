#![feature(let_chains)]
use std::{
    env,
    {collections::HashMap, net::SocketAddr},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::broadcast,
};

#[tokio::main]
async fn main() {
    let port = env::args()
        .last()
        .expect("Please provide a valid port!")
        .parse::<u16>()
        .expect("Invalid port number provided.");

    let listener = TcpListener::bind(format!("localhost:{}", port))
        .await
        .expect("Port was occupied!");

    println!("🚀 Chat Server is running on: tcp://localhost:{}", port);

    let (sender, _) = broadcast::channel(41);
    let users: HashMap<SocketAddr, String> = HashMap::new();

    loop {
        // Wait for a new connection
        if let Ok((mut connection, addr)) = listener.accept().await {
            let mut users = users.clone();
            let sender = sender.clone();
            let mut reciever = sender.subscribe();

            // Create a new proccess
            tokio::spawn(async move {
                let (read_stream, mut write_stream) = connection.split();

                // Create buffer reader and a temporary variable
                let mut reader = BufReader::new(read_stream);
                let mut line = String::new();

                if let Err(error) = write_stream
                    .write_all(
                        "🦀 Welcome to Rust-Chat! 🦀\nSet your name with: /name your_name\n\nChat Log:\n"
                            .as_bytes(),
                    )
                    .await
                {
                    println!("Error whilst sending initial message: {error}")
                }

                loop {
                    // Spin up some concurrent branches
                    tokio::select! {
                        result = reader.read_line(&mut line) => {
                            if let Ok(0) = result {
                                users.remove(&addr);
                                break;
                            }

                            if let Err(error) = sender.send((format_message(&mut users, &line, addr), addr)) {
                                println!("Error whilst forwarding message: {error}")
                            }

                            line.clear()
                        }
                        result = reciever.recv() => {
                            if let Ok((msg, new_addr)) = result && addr != new_addr {
                                write_stream.write_all(msg.as_bytes()).await.expect("Issue when writing to stream.");
                            }
                        }
                    }
                }
            });
        }
    }
}

fn format_message(
    users: &mut HashMap<SocketAddr, String>,
    msg: &String,
    addr: SocketAddr,
) -> String {
    // Set someone's name
    // Prefix the message with a name
    if msg.starts_with("/name ") {
        let name = msg.trim().replace("/name ", "");
        users.insert(addr, name.clone());

        return format!("{name} - has just set their name, say hello!\n");
    }

    return format!(
        "{}: {}",
        if let Some(s) = users.get(&addr) {
            s
        } else {
            "Unknown"
        },
        msg
    );
}
