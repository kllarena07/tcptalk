use crate::input_widget::InputWidget;
use crossterm::event::{KeyCode, MouseEvent, MouseEventKind};
use ratatui::{
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Wrap},
    DefaultTerminal, Frame,
};

pub struct Message {
    pub author: String,
    pub content: String,
}

use std::{
    io::{self, Write},
    net::TcpStream,
    sync::{mpsc, Arc, Mutex},
};

pub struct App {
    pub running: bool,
    pub input_widget: InputWidget,
    pub messages: Vec<Message>,
    pub scroll_offset: usize,
    pub should_auto_scroll: bool,
    pub username: String,
    pub server_ip: String,
    pub write_stream: Arc<Mutex<TcpStream>>,
}

pub enum Event {
    Input(crossterm::event::KeyEvent),
    Mouse(crossterm::event::MouseEvent),
    CursorBlink,
    ServerMessage(String),
}

impl App {
    pub fn new(username: String, server_ip: String, write_stream: Arc<Mutex<TcpStream>>) -> Self {
        Self {
            running: true,
            input_widget: InputWidget::new(username.clone()),
            messages: Vec::new(),
            scroll_offset: 0,
            should_auto_scroll: false,
            username,
            server_ip,
            write_stream,
        }
    }

    pub fn add_message(&mut self, author: String, content: String) {
        self.messages.push(Message { author, content });
    }

    fn scroll_down(&mut self) {
        // Don't scroll past the end of messages
        // Maximum scroll offset is when we can still see at least one message
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    fn scroll_up(&mut self) {
        // Don't scroll past the beginning (can't skip more messages than we have - 1)
        if self.scroll_offset < self.messages.len().saturating_sub(1) {
            self.scroll_offset += 1;
        }
    }

    pub fn run(
        &mut self,
        terminal: &mut DefaultTerminal,
        rx: mpsc::Receiver<Event>,
        _tx: mpsc::Sender<Event>,
    ) -> io::Result<()> {
        while self.running {
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                Event::Mouse(mouse_event) => self.handle_mouse_event(mouse_event)?,
                Event::CursorBlink => {
                    self.input_widget.update_cursor_blink();
                }
                Event::ServerMessage(message) => {
                    // Parse server message and add to messages
                    let message = message.trim().to_string();
                    if !message.is_empty() {
                        // Try to parse as "username: message" format
                        if let Some(colon_pos) = message.find(':') {
                            let author = message[..colon_pos].trim().to_string();
                            let content = message[colon_pos + 1..].trim().to_string();
                            self.add_message(author, content);
                        } else {
                            // System message (join/leave notifications)
                            self.add_message("System".to_string(), message);
                        }
                        self.should_auto_scroll = true;
                    }
                }
            }

            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        const BG_PRIMARY: Color = Color::Rgb(0, 0, 0);
        const BG_SECONDARY: Color = Color::Rgb(30, 30, 30);
        const BG_SUCCESS: Color = Color::Rgb(89, 87, 86);
        const TEXT_PRIMARY: Color = Color::Rgb(255, 255, 255);
        const TEXT_SECONDARY: Color = Color::Rgb(128, 128, 128);

        let [horizontal_area] = Layout::horizontal([Constraint::Fill(1)]).areas(frame.area());
        let [main_area, info_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(horizontal_area);

        // Calculate input widget height
        let available_width = main_area.width.saturating_sub(4);
        let input_area_height = self.input_widget.calculate_height(available_width);
        let total_input_height = input_area_height + 3; // Input area + info area

        let [content_area, input_parent] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(total_input_height)])
                .areas(main_area);

        let [input_area_1, input_area_2] =
            Layout::vertical([Constraint::Length(input_area_height), Constraint::Length(3)])
                .areas(input_parent);

        let version_control = Line::from(Span::styled(
            " tcptalk v0.0.1 ",
            Style::default().fg(TEXT_PRIMARY),
        ))
        .centered()
        .bg(BG_SUCCESS);

        let conn_msg = format!(" Connected to {} ", self.server_ip);

        let conn_info = Line::from(Span::styled(conn_msg, Style::default().fg(TEXT_SECONDARY)))
            .bg(BG_SECONDARY);

        let [vc_area, conn_area] = Layout::horizontal([
            Constraint::Length(version_control.width() as u16),
            Constraint::Fill(1),
        ])
        .areas(info_area);

        // Create lines for messages with proper wrapping, starting from scroll offset
        let mut all_lines = Vec::new();
        let mut is_first_message = true;

        for message in self.messages.iter().skip(self.scroll_offset) {
            if !message.author.is_empty() {
                let content = format!("{}: {}", message.author, message.content);

                // Add spacing before message (except for first message)
                if !is_first_message {
                    all_lines.push(Line::from(""));
                }
                // Add message line (will wrap automatically)
                all_lines.push(Line::from(content));
                is_first_message = false;
            }
        }

        let messages_widget =
            Paragraph::new(all_lines)
                .wrap(Wrap { trim: true })
                .block(Block::new().padding(Padding {
                    left: 1,
                    right: 1,
                    top: 1,
                    bottom: 1,
                }));

        // Handle auto-scroll if flag is set
        if self.should_auto_scroll {
            // Calculate if messages fill the available area
            let available_height = content_area.height.saturating_sub(2) as usize; // Account for padding
            let total_lines = self
                .messages
                .iter()
                .enumerate()
                .map(|(i, msg)| {
                    if msg.author.is_empty() {
                        0
                    } else {
                        // First message: 1 line, others: 2 lines (message + spacing)
                        if i == 0 {
                            1
                        } else {
                            2
                        }
                    }
                })
                .sum::<usize>();

            if total_lines > available_height {
                // Check if we're already at the bottom (within 1 message of the end)
                let max_scroll_offset = self.messages.len().saturating_sub(available_height / 2);
                if self.scroll_offset >= max_scroll_offset.saturating_sub(2) {
                    // We're near the bottom, so auto-scroll
                    self.scroll_offset = self.messages.len().saturating_sub(available_height);
                }
            }
            self.should_auto_scroll = false;
        }

        frame.render_widget(Block::new().bg(BG_PRIMARY), main_area);
        frame.render_widget(
            messages_widget,
            Rect {
                x: content_area.x,
                y: content_area.y,
                width: content_area.width,
                height: content_area.height,
            },
        );
        // Render input widget
        self.input_widget.render(frame, input_area_1, input_area_2);
        frame.render_widget(version_control, vc_area);
        frame.render_widget(conn_info, conn_area);
    }

    fn handle_mouse_event(&mut self, mouse_event: MouseEvent) -> io::Result<()> {
        match mouse_event.kind {
            MouseEventKind::ScrollDown => {
                self.scroll_down();
            }
            MouseEventKind::ScrollUp => {
                self.scroll_up();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        let should_quit = self.input_widget.handle_key_event(key_event)?;
        if should_quit {
            self.running = false;
            return Ok(());
        }

        if key_event.code == KeyCode::Enter {
            // Send message to server if not empty
            if !self.input_widget.is_empty() {
                let message_content = self.input_widget.get_text();
                let message = format!("{}\n", message_content);

                // Add message to local UI immediately for better UX
                self.add_message(self.username.clone(), message_content.clone());
                self.should_auto_scroll = true;

                // Send to server in background
                let send_result = {
                    let lock_result = self.write_stream.lock();
                    match lock_result {
                        Ok(mut stream) => match stream.write_all(message.as_bytes()) {
                            Ok(_) => match stream.flush() {
                                Ok(_) => Ok(()),
                                Err(e) => Err(format!("Failed to send message: {}", e)),
                            },
                            Err(e) => Err(format!("Failed to write to server: {}", e)),
                        },
                        Err(e) => Err(format!("Failed to lock stream: {}", e)),
                    }
                };

                if let Err(error_msg) = send_result {
                    self.add_message("System".to_string(), error_msg);
                    self.should_auto_scroll = true;
                }

                // Clear input field
                self.input_widget.clear();
            }
        }

        Ok(())
    }
}
