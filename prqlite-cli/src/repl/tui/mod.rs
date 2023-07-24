mod app;
mod coords;
mod ui;

pub(super) use app::*;
pub(super) use ui::*;

use anyhow::Result;
use chrono::Local;
use crossterm::{
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, MouseEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{fmt::format, io::stdout};
use tui::{
    backend::{Backend, CrosstermBackend},
    terminal::Terminal,
};

use crate::{ReplInputEvent, ReplState};

pub struct TuiRepl<'a> {
    prompt: String,
    command_prefix: String,
    state: &'a ReplState,
}

impl<'a> TuiRepl<'a> {
    pub fn new<T: ToString>(prompt: T, command_prefix: T, state: &'a ReplState) -> Self {
        Self {
            command_prefix: command_prefix.to_string(),
            prompt: prompt.to_string(),
            state,
        }
    }

    pub async fn run(&self) -> Result<()> {
        enable_raw_mode()?;

        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture,)?;

        let mut app = App::new(&self.prompt, &self.command_prefix);
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        run_app(&mut terminal, &mut app, self)?;
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

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App, repl: &TuiRepl) -> Result<()> {
    loop {
        terminal.draw(|f| ui(f, app))?;
        match app.input_mode {
            InputMode::Normal => terminal.hide_cursor(),
            InputMode::Insert => terminal.show_cursor(),
        }
        .unwrap();
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

                            KeyCode::Char('c') => {
                                app.state.history.clear();
                            }

                            KeyCode::Up => app.state.history.previous(),
                            KeyCode::Down => app.state.history.next(),

                            _ => {}
                        },
                        InputMode::Insert => match key {
                            KeyCode::Enter => {
                                if !app.input.is_empty() {
                                    let input: String = app.input.drain(..).collect();

                                    let repl_input_event = ReplInputEvent::new(&repl.state);
                                    let exec_output =
                                        match input.trim().starts_with(&app.command_prefix) {
                                            true => repl_input_event.on_command(&input),
                                            false => repl_input_event.on_regular_input(&input),
                                        };

                                    match exec_output {
                                        Ok(out) => {
                                            app.push_msg(input.clone(), out, OutputType::Success)
                                        }
                                        Err(err) => app.push_msg(
                                            input.clone(),
                                            err.to_string(),
                                            OutputType::Error,
                                        ),
                                    }
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
