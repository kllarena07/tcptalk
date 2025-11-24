use crossterm::event::KeyCode;
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::Block,
};
use std::{io, sync::mpsc, thread, time::Duration};

fn main() -> io::Result<()> {
    let mut app = App { running: true };

    let mut terminal = ratatui::init();

    let (event_tx, event_rx) = mpsc::channel::<Event>();

    let tx_to_input_events = event_tx.clone();
    thread::spawn(move || {
        handle_input_events(tx_to_input_events);
    });

    let tx_to_counter_events = event_tx.clone();
    thread::spawn(move || {
        run_tick_thread(tx_to_counter_events, 30);
    });

    let app_result = app.run(&mut terminal, event_rx);

    ratatui::restore();
    app_result
}

enum Event {
    Input(crossterm::event::KeyEvent),
    Tick(u64),
}

fn handle_input_events(tx: mpsc::Sender<Event>) {
    loop {
        match crossterm::event::read().unwrap() {
            crossterm::event::Event::Key(key_event) => tx.send(Event::Input(key_event)).unwrap(),
            _ => {}
        }
    }
}

fn run_tick_thread(tx: mpsc::Sender<Event>, fps: u64) {
    let frame_duration = Duration::from_millis(1000 / fps);
    let mut tick: u64 = 0;
    loop {
        tx.send(Event::Tick(tick)).unwrap();
        tick = tick.wrapping_add(1);
        thread::sleep(frame_duration);
    }
}

struct App {
    running: bool,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal, rx: mpsc::Receiver<Event>) -> io::Result<()> {
        while self.running {
            match rx.recv().unwrap() {
                Event::Input(key_event) => self.handle_key_event(key_event)?,
                Event::Tick(tick) => {
                    // if let Some(page) = self.pages.get_mut(self.selected_page) {
                    //     let _ = page.on_tick(tick);
                    // }
                }
            }

            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame) {
        const BG_PRIMARY: Color = Color::Rgb(10, 10, 10);
        const BG_SECONDARY: Color = Color::Rgb(20, 20, 20);
        const BG_SUCCESS: Color = Color::Rgb(30, 30, 30);
        const TEXT_PRIMARY: Color = Color::Rgb(255, 255, 255);
        const TEXT_SECONDARY: Color = Color::Rgb(128, 128, 128);

        let [horizontal_area] = Layout::horizontal([Constraint::Fill(1)]).areas(frame.area());
        let [main_area, info_area] =
            Layout::vertical([Constraint::Fill(1), Constraint::Length(1)]).areas(horizontal_area);

        let version_control = Line::from(Span::styled(
            " tailtalk v1.0.0 ",
            Style::default().fg(TEXT_PRIMARY),
        ))
        .centered()
        .bg(BG_SUCCESS);

        let conn_addr = "0.0.0.0";
        let conn_msg = format!(" Connected to {} ", conn_addr);

        let conn_info = Line::from(Span::styled(conn_msg, Style::default().fg(TEXT_SECONDARY)))
            .bg(BG_SECONDARY);

        let [vc_area, conn_area] = Layout::horizontal([
            Constraint::Length((version_control.width() as u16)),
            Constraint::Fill(1),
        ])
        .areas(info_area);

        frame.render_widget(Block::new().bg(BG_PRIMARY), main_area);
        frame.render_widget(version_control, vc_area);
        frame.render_widget(conn_info, conn_area);
    }

    fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) -> io::Result<()> {
        match key_event.code {
            KeyCode::Char('q') => {
                self.running = false;
            }
            _ => {}
        }

        Ok(())
    }
}
