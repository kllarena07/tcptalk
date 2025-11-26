mod cli_args;
use crate::cli_args::Args;
use clap::Parser;

mod app;
use crate::app::{App, Event};

mod events;
use crate::events::{handle_input_events, handle_server_messages, run_cursor_blink_thread};

mod input_widget;

use std::{
    io::{self, Read, Write},
    net::TcpStream,
    sync::{Arc, Mutex, mpsc},
    thread,
    time::Duration,
};

fn main() -> io::Result<()> {
    let args = Args::parse();
    let server_addr = format!("{}:2133", args.ip);

    // Connect to server
    let mut stream = match TcpStream::connect(&server_addr) {
        Ok(stream) => {
            println!("Connected to server at {}", server_addr);
            stream
        }
        Err(e) => {
            eprintln!("Failed to connect to server at {}: {}", server_addr, e);
            return Err(io::Error::new(io::ErrorKind::ConnectionRefused, e));
        }
    };

    // Handle username handshake with server
    let mut buf = [0u8; 1024];

    // Read "Enter your username: " prompt from server
    let n = stream.read(&mut buf)?;
    let _prompt = String::from_utf8_lossy(&buf[..n]);

    // Send username to server
    let username_msg = format!("{}\n", args.username);
    stream.write_all(username_msg.as_bytes())?;
    stream.flush()?;

    // Read any initial server messages (like "Username cannot be empty" or welcome)
    let mut initial_messages = Vec::new();
    loop {
        let n = stream.read(&mut buf)?;
        if n == 0 {
            break;
        }
        let response = String::from_utf8_lossy(&buf[..n]);
        if response.contains("Username cannot be empty") 
            || response.contains("Username 'System' is reserved")
            || response.contains("Username is already taken") {
            eprintln!("Server rejected username: {}", response.trim());
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                response.trim(),
            ));
        }
        initial_messages.push(response.to_string());

        // Check if there's more data available with a small timeout
        stream.set_read_timeout(Some(Duration::from_millis(100)))?;
        match stream.read(&mut buf) {
            Ok(0) | Err(_) => break,
            Ok(n) => {
                let more_response = String::from_utf8_lossy(&buf[..n]);
                initial_messages.push(more_response.to_string());
            }
        }
    }
    stream.set_read_timeout(None)?; // Remove timeout

    crossterm::execute!(std::io::stdout(), crossterm::event::EnableMouseCapture)?;

    // Create separate streams for reading and writing to avoid deadlock
    let read_stream = stream
        .try_clone()
        .expect("Failed to clone stream for reading");
    let write_stream = Arc::new(Mutex::new(stream));

    let mut app = App::new(
        args.username.clone(),
        args.ip.clone(),
        Arc::clone(&write_stream),
    );

    // Add any initial messages from server
    for msg in initial_messages {
        if !msg.trim().is_empty() {
            app.add_message("System".to_string(), msg.trim().to_string());
        }
    }

    let mut terminal = ratatui::init();

    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    let tx_to_cursor_events = event_tx.clone();
    thread::spawn(move || {
        run_cursor_blink_thread(tx_to_cursor_events);
    });

    // Start message receiver thread with separate read stream
    let rx_event_tx = event_tx.clone();
    thread::spawn(move || {
        handle_server_messages(read_stream, rx_event_tx);
    });

    let app_result = app.run(&mut terminal, event_rx, event_tx.clone());

    ratatui::restore();
    crossterm::execute!(std::io::stdout(), crossterm::event::DisableMouseCapture)?;
    app_result
}
