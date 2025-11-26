use crossterm::event::{KeyCode, KeyModifiers};
use ratatui::{
    Frame,
    layout::Rect,
    prelude::Stylize,
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap},
};
use std::io;

pub struct InputWidget {
    pub text: String,
    pub cursor_position: usize,
    pub cursor_visible: bool,
    pub last_input_time: std::time::Instant,
    pub username: String,
}

impl InputWidget {
    pub fn new(username: String) -> Self {
        Self {
            text: String::new(),
            cursor_position: 0,
            cursor_visible: true,
            last_input_time: std::time::Instant::now(),
            username,
        }
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor_position = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty()
    }

    pub fn get_text(&self) -> String {
        self.text.clone()
    }

    fn move_cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.cursor_position < self.text.len() {
            self.cursor_position += 1;
        }
    }

    fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
    }

    fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.text.len();
    }

    fn move_cursor_word_left(&mut self) {
        if self.cursor_position == 0 {
            return;
        }

        let text_before_cursor = &self.text[..self.cursor_position];
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
        if self.cursor_position >= self.text.len() {
            return;
        }

        let text_after_cursor = &self.text[self.cursor_position..];
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
            let remaining = &self.text[new_pos..];
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
                self.cursor_position = self.text.len();
            }
        } else {
            self.cursor_position = self.text.len();
        }
    }

    pub fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<bool> {
        match key_event.code {
            KeyCode::Char(c) => {
                let ctrl_c_quit = c == 'c' && key_event.modifiers.contains(KeyModifiers::CONTROL);
                let win_del = c == 'u' && key_event.modifiers.contains(KeyModifiers::CONTROL);
                let ctrl_lft = c == 'a' && key_event.modifiers.contains(KeyModifiers::CONTROL);
                let ctrl_rht = c == 'e' && key_event.modifiers.contains(KeyModifiers::CONTROL);
                let alt_lft = c == 'f' && key_event.modifiers.contains(KeyModifiers::ALT);
                let alt_rht = c == 'b' && key_event.modifiers.contains(KeyModifiers::ALT);

                if ctrl_c_quit {
                    return Ok(true); // Signal to quit
                } else if win_del {
                    self.text.drain(0..self.cursor_position);
                    self.cursor_position = 0;
                } else if ctrl_lft {
                    self.move_cursor_to_start();
                } else if ctrl_rht {
                    self.move_cursor_to_end();
                } else if alt_lft {
                    self.move_cursor_word_right();
                } else if alt_rht {
                    self.move_cursor_word_left();
                } else {
                    self.text.insert(self.cursor_position, c);
                    self.cursor_position += 1;
                }

                self.last_input_time = std::time::Instant::now();
            }
            KeyCode::Backspace => {
                if key_event.modifiers.contains(KeyModifiers::ALT) {
                    if self.cursor_position > 0 {
                        let text_before_cursor = &self.text[..self.cursor_position];
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
                            self.text.drain(last_boundary_pos + 1..self.cursor_position);
                            self.cursor_position = last_boundary_pos + 1;
                        } else {
                            self.text.drain(0..self.cursor_position);
                            self.cursor_position = 0;
                        }
                    }
                } else {
                    if self.cursor_position > 0 {
                        self.text.remove(self.cursor_position - 1);
                        self.cursor_position -= 1;
                    }
                }
                self.last_input_time = std::time::Instant::now();
            }
            KeyCode::Delete => {
                if self.cursor_position < self.text.len() {
                    self.text.remove(self.cursor_position);
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
                self.last_input_time = std::time::Instant::now();
            }
            _ => {}
        }

        Ok(false) // Don't quit by default
    }

    pub fn update_cursor_blink(&mut self) {
        if self.last_input_time.elapsed().as_secs() >= 1 {
            self.cursor_visible = !self.cursor_visible;
        } else {
            self.cursor_visible = true;
        }
    }

    pub fn render(&self, frame: &mut Frame, input_area: Rect, info_area: Rect) {
        let before_cursor = &self.text[..self.cursor_position];

        let mut input_spans = vec![Span::from(before_cursor.to_string())];

        if self.cursor_position < self.text.len() {
            let char_at_cursor = &self.text[self.cursor_position..self.cursor_position + 1];
            if self.cursor_visible {
                input_spans.push(Span::styled(
                    char_at_cursor,
                    Style::default().fg(Color::Cyan).bg(Color::Rgb(0, 100, 100)),
                ));
            } else {
                input_spans.push(Span::from(char_at_cursor));
            }
            input_spans.push(Span::from(&self.text[self.cursor_position + 1..]));
        } else {
            if self.cursor_visible {
                input_spans.push(Span::styled("█", Style::default().fg(Color::Cyan)));
            }
        }

        let input_paragraph = Paragraph::new(vec![Line::from(input_spans)])
            .block(
                Block::new()
                    .borders(Borders::LEFT)
                    .border_type(BorderType::Thick)
                    .padding(Padding {
                        left: 1,
                        right: 3,
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

        frame.render_widget(
            input_paragraph,
            Rect {
                x: input_area.x + 1,
                y: input_area.y,
                width: input_area.width.saturating_sub(1),
                height: input_area.height,
            },
        );
        frame.render_widget(input_info, info_area);
    }

    pub fn calculate_height(&self, available_width: u16) -> u16 {
        let text_width = self.text.len() as u16 + 1;
        let lines_needed = std::cmp::max(1, (text_width + available_width - 1) / available_width);
        std::cmp::max(3, lines_needed)
    }
}
