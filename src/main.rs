use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

struct Client {
    stream: TcpStream,
    username: String,
}

fn broadcast_message(
    message: &[u8],
    sender_addr: SocketAddr,
    connections: &Arc<Mutex<HashMap<SocketAddr, Client>>>,
    include_sender: bool,
) -> io::Result<()> {
    let mut conn_map = connections.lock().unwrap();
    let mut to_remove = Vec::new();

    for (addr, client) in conn_map.iter_mut() {
        if include_sender || *addr != sender_addr {
            match client.stream.write_all(message) {
                Ok(_) => match client.stream.flush() {
                    Ok(_) => {}
                    Err(_) => to_remove.push(*addr),
                },
                Err(_) => to_remove.push(*addr),
            }
        }
    }

    for addr in to_remove {
        conn_map.remove(&addr);
        println!(
            "Removed dead connection: {} (Total: {})",
            addr,
            conn_map.len()
        );
    }

    Ok(())
}

fn get_username(mut stream: &TcpStream) -> io::Result<String> {
    loop {
        stream.write_all(b"Enter your username: ")?;
        stream.flush()?;

        let mut buf = [0u8; 32];
        let n = stream.read(&mut buf)?;

        let username = String::from_utf8_lossy(&buf[..n]).trim().to_string();

        if !username.is_empty() {
            return Ok(username);
        }

        stream.write_all(b"Username cannot be empty. Please try again.\n")?;
        stream.flush()?;
    }
}

fn handle_client(
    mut stream: TcpStream,
    connections: Arc<Mutex<HashMap<SocketAddr, Client>>>,
) -> io::Result<()> {
    let addr = stream.peer_addr()?;

    let username = get_username(&stream)?;

    let mut conn_map = connections.lock().unwrap();
    conn_map.insert(
        addr,
        Client {
            stream: stream.try_clone()?,
            username: username.clone(),
        },
    );
    let total = conn_map.len();
    drop(conn_map);
    println!("{} connected from {} (Total: {})", username, addr, total);

    let join_message = format!("{} has joined the chat\n", username);
    broadcast_message(join_message.as_bytes(), addr, &connections, true)?;

    let mut buf = [0u8; 4096];
    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            break;
        }

        let message = String::from_utf8_lossy(&buf[..n]);
        let formatted_msg = format!("{}: {}", username, message);

        std::io::stdout().write_all(formatted_msg.as_bytes())?;
        std::io::stdout().flush()?;

        broadcast_message(formatted_msg.as_bytes(), addr, &connections, false)?;
    }

    let leave_message = format!("{} has left the chat\n", username);
    broadcast_message(leave_message.as_bytes(), addr, &connections, false)?;

    let mut conn_map = connections.lock().unwrap();
    conn_map.remove(&addr);
    let total = conn_map.len();
    drop(conn_map);
    println!("{} disconnected from {} (Total: {})", username, addr, total);

    Ok(())
}

fn main() -> io::Result<()> {
    let address = "0.0.0.0:2133";

    println!("Binding to port {}", address);

    let listener = TcpListener::bind(address)?;
    let connections: Arc<Mutex<HashMap<SocketAddr, Client>>> = Arc::new(Mutex::new(HashMap::new()));

    for connection in listener.incoming() {
        match connection {
            Ok(stream) => {
                let connections_clone = Arc::clone(&connections);
                thread::spawn(move || {
                    if let Err(err) = handle_client(stream, connections_clone) {
                        eprintln!("Client handler error: {}", err);
                    }
                });
            }
            Err(err) => eprintln!("Accept error: {}", err),
        }
    }

    Ok(())
}
