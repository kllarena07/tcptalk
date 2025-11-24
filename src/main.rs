use std::{
    collections::HashMap,
    io::{self, Read, Write},
    net::{SocketAddr, TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread,
};

fn handle_client(
    mut stream: TcpStream,
    connections: Arc<Mutex<HashMap<SocketAddr, String>>>,
) -> io::Result<()> {
    let addr = stream.peer_addr()?;

    {
        let mut conn_map = connections.lock().unwrap();
        conn_map.insert(
            addr,
            format!("Connected at {:?}", std::time::SystemTime::now()),
        );
        println!("Added connection: {} (Total: {})", addr, conn_map.len());
    }

    let mut buf = [0u8; 4096];
    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            break;
        }

        std::io::stdout().write_all(&buf[..n])?;
        std::io::stdout().flush()?;
    }

    {
        let mut conn_map = connections.lock().unwrap();
        conn_map.remove(&addr);
        println!("Removed connection: {} (Total: {})", addr, conn_map.len());
    }

    Ok(())
}

fn main() -> io::Result<()> {
    let address = "0.0.0.0:2133";

    println!("Binding to port {}", address);

    let listener = TcpListener::bind(address)?;
    let connections: Arc<Mutex<HashMap<SocketAddr, String>>> = Arc::new(Mutex::new(HashMap::new()));

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
