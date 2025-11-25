use crossterm::event::KeyCode;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Wrap},
};
use std::{io, sync::mpsc, thread, time::Duration};

fn main() -> io::Result<()> {
    let mut app = App { 
        running: true,
        input_text: String::new(),
        cursor_visible: true,
        last_input_time: std::time::Instant::now(),
    };

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

    let app_result = app.run(&mut terminal, event_rx);

    ratatui::restore();
    app_result
}

enum Event {
    Input(crossterm::event::KeyEvent),
    CursorBlink,
}

fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => tx.send(Event::Input(key_event)).unwrap(),
            _ => {}
        }
    }
}

fn run_cursor_blink_thread(tx: mpsc::Sender<Event>) {
    let blink_duration = Duration::from_millis(500);
    loop {
        tx.send(Event::CursorBlink).unwrap();
        thread::sleep(blink_duration);
    }
}

struct App {
    running: bool,
    input_text: String,
    cursor_visible: bool,
    last_input_time: std::time::Instant,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<Event>) -> io::Result<()> {
        while self.running {
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                Event::CursorBlink => {
                    // Only blink cursor if user hasn't typed in the last 1 second
                    if self.last_input_time.elapsed().as_secs() >= 1 {
                        self.cursor_visible = !self.cursor_visible;
                    } else {
                        // Keep cursor visible when typing
                        self.cursor_visible = true;
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

        let [content_area, input_parent] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(6)]).areas(main_area);

        let [input_area_1, input_area_2] =
            Layout::vertical([Constraint::Length(3), Constraint::Length(3)]).areas(input_parent);

        let version_control = Line::from(Span::styled(
            " tailtalk v0.0.1 ",
            Style::default().fg(TEXT_PRIMARY),
        ))
        .centered()
        .bg(BG_SUCCESS);

        let conn_addr = "0.0.0.0";
        let conn_msg = format!(" Connected to {} ", conn_addr);

        let conn_info = Line::from(Span::styled(conn_msg, Style::default().fg(TEXT_SECONDARY)))
            .bg(BG_SECONDARY);

        let [vc_area, conn_area] = Layout::horizontal([
            Constraint::Length(version_control.width() as u16),
            Constraint::Fill(1),
        ])
        .areas(info_area);

        let cursor_char = if self.cursor_visible { "█" } else { " " };
        let input_with_cursor = format!("{}{}", self.input_text, cursor_char);
        
        let input_paragraph = Paragraph::new(vec![
            Line::from(""),
            Line::from(Span::styled(input_with_cursor, Style::default().fg(TEXT_PRIMARY))),
            Line::from(""),
        ])
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
            Line::from(Span::from("Sending message as krayon").style(Style::default().bold())),
            Line::from(""),
        ])
        .block(Block::new().padding(Padding {
            left: 1,
            right: 0,
            top: 0,
            bottom: 0,
        }));

        let message_1 = Paragraph::new(vec![
            Line::from(vec![
                Span::styled("krayon: ", Style::default().bold()),
                Span::from("hello world"),
            ]),
            Line::from(""),
            Line::from(vec![
                Span::styled("krayon: ", Style::default().bold()),
                Span::from("hello world"),
            ]),
        ])
        .block(Block::new().padding(Padding {
            left: 1,
            right: 1,
            top: 1,
            bottom: 1,
        }));

        frame.render_widget(Block::new().bg(BG_PRIMARY), main_area);
        frame.render_widget(
            message_1,
            Rect {
                x: content_area.x,
                y: content_area.y,
                width: content_area.width,
                height: content_area.height - 1,
            },
        );
        frame.render_widget(
            input_paragraph,
            Rect {
                x: input_area_1.x + 1,
                y: input_area_1.y,
                width: input_area_1.width,
                height: input_area_1.height,
            },
        );
        frame.render_widget(input_info, input_area_2);
        frame.render_widget(version_control, vc_area);
        frame.render_widget(conn_info, conn_area);
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        match key_event.code {
            KeyCode::Char('q') => {
                self.running = false;
            }
            KeyCode::Char(c) => {
                self.input_text.push(c);
                self.last_input_time = std::time::Instant::now();
            }
            KeyCode::Backspace => {
                self.input_text.pop();
                self.last_input_time = std::time::Instant::now();
            }
            _ => {}
        }

        Ok(())
    }
}
