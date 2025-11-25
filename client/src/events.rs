use crate::app::Event;
use std::{io::Read, net::TcpStream, sync::mpsc, thread, time::Duration};

pub fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => tx.send(Event::Input(key_event)).unwrap(),
            crossterm::event::Event::Mouse(mouse_event) => {
                tx.send(Event::Mouse(mouse_event)).unwrap()
            }
            _ => {}
        }
    }
}

pub fn run_cursor_blink_thread(tx: mpsc::Sender<Event>) {
    let blink_duration = Duration::from_millis(500);
    loop {
        tx.send(Event::CursorBlink).unwrap();
        thread::sleep(blink_duration);
    }
}

pub fn handle_server_messages(mut stream: TcpStream, tx: mpsc::Sender<Event>) {
    let mut buf = [0u8; 4096];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => {
                // Server disconnected
                let _ = tx.send(Event::ServerMessage("Server disconnected".to_string()));
                break;
            }
            Ok(n) => {
                let message = String::from_utf8_lossy(&buf[..n]).to_string();
                // Don't send empty messages
                if !message.trim().is_empty() {
                    let _ = tx.send(Event::ServerMessage(message));
                }
            }
            Err(e) => {
                let _ = tx.send(Event::ServerMessage(format!("Connection error: {}", e)));
                break;
            }
        }
    }
}
