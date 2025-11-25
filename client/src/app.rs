use crossterm::event::{KeyCode, MouseEvent, MouseEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap},
};

pub struct Message {
    pub author: String,
    pub content: String,
}

use std::{
    io::{self, Write},
    net::TcpStream,
    sync::{Arc, Mutex, mpsc},
};

pub struct App {
    pub running: bool,
    pub input_text: String,
    pub cursor_position: usize,
    pub cursor_visible: bool,
    pub last_input_time: std::time::Instant,
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

    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.input_text.len() {
            self.cursor_position += 1;
        }
    }

    fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.input_text.len();
    }

    fn move_cursor_word_left(&mut self) {
        if self.cursor_position == 0 {
            return;
        }

        // Find the start of the previous word, treating punctuation as boundaries
        let text_before_cursor = &self.input_text[..self.cursor_position];
        let trimmed = text_before_cursor.trim_end_matches(|c: char| {
            c.is_whitespace()
                || c == '\''
                || c == '"'
                || c == ';'
                || c == ','
                || c == '.'
                || c == '!'
                || c == '?'
        });

        if let Some(last_boundary_pos) = trimmed.rfind(|c: char| {
            c.is_whitespace()
                || c == '\''
                || c == '"'
                || c == ';'
                || c == ','
                || c == '.'
                || c == '!'
                || c == '?'
        }) {
            self.cursor_position = last_boundary_pos + 1;
        } else {
            self.cursor_position = 0;
        }
    }

    fn move_cursor_word_right(&mut self) {
        if self.cursor_position >= self.input_text.len() {
            return;
        }

        // Find the start of the next word, treating punctuation as boundaries
        let text_after_cursor = &self.input_text[self.cursor_position..];
        if let Some(next_boundary_pos) = text_after_cursor.find(|c: char| {
            c.is_whitespace()
                || c == '\''
                || c == '"'
                || c == ';'
                || c == ','
                || c == '.'
                || c == '!'
                || c == '?'
        }) {
            let new_pos = self.cursor_position + next_boundary_pos;
            // Skip any boundaries to get to the next word
            let remaining = &self.input_text[new_pos..];
            if let Some(non_boundary_pos) = remaining.find(|c: char| {
                !(c.is_whitespace()
                    || c == '\''
                    || c == '"'
                    || c == ';'
                    || c == ','
                    || c == '.'
                    || c == '!'
                    || c == '?')
            }) {
                self.cursor_position = new_pos + non_boundary_pos;
            } else {
                self.cursor_position = self.input_text.len();
            }
        } else {
            self.cursor_position = self.input_text.len();
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
                    // Only blink cursor if user hasn't typed in the last 1 second
                    if self.last_input_time.elapsed().as_secs() >= 1 {
                        self.cursor_visible = !self.cursor_visible;
                    } else {
                        // Keep cursor visible when typing
                        self.cursor_visible = true;
                    }
                }
                Event::ServerMessage(message) => {
                    // Parse server message and add to messages
                    let message = message.trim().to_string();
                    if !message.is_empty() {
                        // Try to parse as "username: message" format
                        if let Some(colon_pos) = message.find(':') {
                            let author = message[..colon_pos].trim().to_string();
                            let content = message[colon_pos + 1..].trim().to_string();
                            self.messages.push(Message { author, content });
                        } else {
                            // System message (join/leave notifications)
                            self.messages.push(Message {
                                author: "System".to_string(),
                                content: message,
                            });
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

        let before_cursor = &self.input_text[..self.cursor_position];

        // Create styled spans for cursor with different color
        let mut input_spans = vec![Span::from(before_cursor.to_string())];

        // Add cursor - replace the character at cursor position if there is one
        if self.cursor_position < self.input_text.len() {
            // Replace the character at cursor position with cursor
            let char_at_cursor = &self.input_text[self.cursor_position..self.cursor_position + 1];
            if self.cursor_visible {
                input_spans.push(Span::styled(
                    char_at_cursor,
                    Style::default().fg(Color::Cyan).bg(Color::Rgb(0, 100, 100)),
                ));
            } else {
                input_spans.push(Span::from(char_at_cursor));
            }
            input_spans.push(Span::from(&self.input_text[self.cursor_position + 1..]));
        } else {
            // Cursor at end of line
            if self.cursor_visible {
                input_spans.push(Span::styled("█", Style::default().fg(Color::Cyan)));
            }
        }

        // Estimate lines needed (account for padding and borders)
        let available_width = main_area.width.saturating_sub(4);
        let text_width = self.input_text.len() as u16 + 1; // +1 for cursor
        let lines_needed = std::cmp::max(1, (text_width + available_width - 1) / available_width);
        let input_area_height = std::cmp::max(3, lines_needed); // Minimum 3 lines for input area
        let total_input_height = input_area_height + 3; // Input area + info area

        let [content_area, input_parent] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(total_input_height)])
                .areas(main_area);

        let [input_area_1, input_area_2] =
            Layout::vertical([Constraint::Length(input_area_height), Constraint::Length(3)])
                .areas(input_parent);

        let version_control = Line::from(Span::styled(
            " tailtalk v0.0.1 ",
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

        let input_paragraph = Paragraph::new(vec![Line::from(input_spans)])
            .block(
                Block::new()
                    .borders(Borders::LEFT)
                    .border_type(BorderType::Thick)
                    .padding(Padding {
                        left: 1,
                        right: 0,
                        top: 0,
                        bottom: 0,
                    }),
            )
            .wrap(Wrap { trim: true });

        let input_info = Paragraph::new(vec![
            Line::from(""),
            Line::from(
                Span::from(format!("Sending message as {}", self.username))
                    .style(Style::default().bold()),
            ),
            Line::from(""),
        ])
        .block(Block::new().padding(Padding {
            left: 1,
            right: 0,
            top: 0,
            bottom: 0,
        }));

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
                        if i == 0 { 1 } else { 2 }
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
        frame.render_widget(
            input_paragraph,
            Rect {
                x: input_area_1.x + 1,
                y: input_area_1.y,
                width: input_area_1.width.saturating_sub(1),
                height: input_area_1.height,
            },
        );
        frame.render_widget(input_info, input_area_2);
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
        use crossterm::event::KeyModifiers;

        match key_event.code {
            KeyCode::Char(c) => {
                // Handle Ctrl+C for quit
                if c == 'c' && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    self.running = false;
                    return Ok(());
                }
                // Handle Ctrl+U for clear line (Win+Delete in your case)
                if c == 'u' && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    // Delete everything before cursor
                    self.input_text.drain(0..self.cursor_position);
                    self.cursor_position = 0;
                }
                // Handle Ctrl+Left (a) and Ctrl+Right (e) for line navigation
                else if c == 'a' && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_cursor_to_start();
                } else if c == 'e' && key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_cursor_to_end();
                }
                // Handle Alt+Left (f) and Alt+Right (b) for word navigation
                else if c == 'f' && key_event.modifiers.contains(KeyModifiers::ALT) {
                    self.move_cursor_word_right();
                } else if c == 'b' && key_event.modifiers.contains(KeyModifiers::ALT) {
                    self.move_cursor_word_left();
                } else {
                    self.input_text.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                }
                self.last_input_time = std::time::Instant::now();
            }
            KeyCode::Backspace => {
                // Handle Alt+Backspace for delete word
                if key_event.modifiers.contains(KeyModifiers::ALT) {
                    // Delete word logic - find previous boundary and delete from there
                    if self.cursor_position > 0 {
                        let text_before_cursor = &self.input_text[..self.cursor_position];
                        let trimmed = text_before_cursor.trim_end_matches(|c: char| {
                            c.is_whitespace()
                                || c == '\''
                                || c == '"'
                                || c == ';'
                                || c == ','
                                || c == '.'
                                || c == '!'
                                || c == '?'
                        });

                        if let Some(last_boundary_pos) = trimmed.rfind(|c: char| {
                            c.is_whitespace()
                                || c == '\''
                                || c == '"'
                                || c == ';'
                                || c == ','
                                || c == '.'
                                || c == '!'
                                || c == '?'
                        }) {
                            self.input_text
                                .drain(last_boundary_pos + 1..self.cursor_position);
                            self.cursor_position = last_boundary_pos + 1;
                        } else {
                            self.input_text.drain(0..self.cursor_position);
                            self.cursor_position = 0;
                        }
                    }
                } else {
                    // Regular backspace - delete previous character
                    if self.cursor_position > 0 {
                        self.input_text.remove(self.cursor_position - 1);
                        self.cursor_position -= 1;
                    }
                }
                self.last_input_time = std::time::Instant::now();
            }
            KeyCode::Delete => {
                // Delete character at cursor position
                if self.cursor_position < self.input_text.len() {
                    self.input_text.remove(self.cursor_position);
                }
                self.last_input_time = std::time::Instant::now();
            }
            KeyCode::Left => {
                if key_event.modifiers.contains(KeyModifiers::ALT) {
                    self.move_cursor_word_left();
                } else if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_cursor_to_start();
                } else {
                    self.move_cursor_left();
                }
                self.last_input_time = std::time::Instant::now();
            }
            KeyCode::Right => {
                if key_event.modifiers.contains(KeyModifiers::ALT) {
                    self.move_cursor_word_right();
                } else if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                    self.move_cursor_to_end();
                } else {
                    self.move_cursor_right();
                }
                self.last_input_time = std::time::Instant::now();
            }
            KeyCode::Enter => {
                // Send message to server if not empty
                if !self.input_text.trim().is_empty() {
                    let message_content = self.input_text.clone();
                    let message = format!("{}\n", self.input_text);

                    // Add message to local UI immediately for better UX
                    self.messages.push(Message {
                        author: self.username.clone(),
                        content: message_content.clone(),
                    });
                    self.should_auto_scroll = true;

                    // Send to server in background
                    match self.write_stream.lock() {
                        Ok(mut stream) => {
                            match stream.write_all(message.as_bytes()) {
                                Ok(_) => {
                                    match stream.flush() {
                                        Ok(_) => {
                                            // Message sent successfully
                                        }
                                        Err(e) => {
                                            self.messages.push(Message {
                                                author: "System".to_string(),
                                                content: format!("Failed to send message: {}", e),
                                            });
                                            self.should_auto_scroll = true;
                                        }
                                    }
                                }
                                Err(e) => {
                                    self.messages.push(Message {
                                        author: "System".to_string(),
                                        content: format!("Failed to write to server: {}", e),
                                    });
                                    self.should_auto_scroll = true;
                                }
                            }
                        }
                        Err(e) => {
                            self.messages.push(Message {
                                author: "System".to_string(),
                                content: format!("Failed to lock stream: {}", e),
                            });
                            self.should_auto_scroll = true;
                        }
                    }

                    // Clear input field and reset cursor
                    self.input_text.clear();
                    self.cursor_position = 0;
                }
                self.last_input_time = std::time::Instant::now();
            }
            _ => {}
        }

        Ok(())
    }
}
