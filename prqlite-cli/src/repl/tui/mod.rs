mod app;
mod ui;
pub(super) use app::*;
pub(super) use ui::*;

use anyhow::{Error, Result};
use chrono::Local;
use crossterm::{
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use prql_compiler::compile;
use std::io::stdout;
use tui::{
    backend::{Backend, CrosstermBackend},
    terminal::Terminal,
};

pub struct TuiRepl {
    prompt: String,
    command_prefix: String,
}

impl TuiRepl {
    pub fn new<T: ToString>(prompt: T, command_prefix: T) -> Self {
        Self {
            command_prefix: command_prefix.to_string(),
            prompt: prompt.to_string(),
        }
    }

    pub async fn run(&self) -> Result<()> {
        enable_raw_mode()?;

        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;

        let mut app = App::new(&self.prompt);
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        run_app(&mut terminal, &mut app)?;
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;
        Ok(())
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> Result<()> {
    let mut history_command_index = 0;
    loop {
        terminal.draw(|f| ui(f, app))?;
        match read()? {
            Event::Mouse(ev) => match ev.kind {
                MouseEventKind::ScrollUp => app.state.history.previous(),
                MouseEventKind::ScrollDown => app.state.history.next(),

                _ => {}
            },
            Event::Key(key) => {
                match key.code {
                    KeyCode::Tab => {
                        app.input_mode = match app.input_mode {
                            InputMode::Normal => InputMode::Insert,
                            InputMode::Insert => InputMode::Normal,
                        };
                    }
                    key => match app.input_mode {
                        InputMode::Normal => match key {
                            KeyCode::Char('i') => {
                                app.input_mode = InputMode::Insert;
                            }
                            KeyCode::Char('q') => {
                                return Ok(());
                            }

                            KeyCode::Up => app.state.history.previous(),
                            KeyCode::Down => app.state.history.next(),

                            _ => {}
                        },
                        InputMode::Insert => match key {
                            KeyCode::Enter => {
                                if !app.input.is_empty() {
                                    let input: String = app.input.drain(..).collect();
                                    let output: Output = Output::new(
                                        Local::now(),
                                        input.clone(),
                                        match compile(&input) {
                                            Err(e) => Err(anyhow::anyhow!(e.to_string())),
                                            Ok(sql) => Ok(sql
                                                .replace("\n", " ")
                                                .split_whitespace()
                                                .filter_map(|e| {
                                                    if e.is_empty() {
                                                        return None;
                                                    }
                                                    let mut e = e.to_string();
                                                    e.push_str(" ");
                                                    Some(e)
                                                })
                                                .collect()),
                                        },
                                    );

                                    app.state.history.items.push(output);
                                    app.state.history.last()
                                }
                            }
                            KeyCode::Char(c) => {
                                app.input.push(c);
                            }
                            KeyCode::Backspace => {
                                app.input.pop();
                            }
                            KeyCode::Up => {}
                            KeyCode::Down => {}
                            KeyCode::Right => {
                                app.state.coords.insert.right();
                            }
                            KeyCode::Left => {
                                app.state.coords.insert.left();
                            }

                            KeyCode::Esc => {
                                app.input_mode = InputMode::Normal;
                            }
                            _ => {}
                        },
                    },
                };
            }
            _ => {}
        }
    }
}
