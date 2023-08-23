use std::fmt::Display;

use super::coords::*;

use chrono::{DateTime, Local};

use tui::widgets::ListState;

type LocalTime = DateTime<Local>;
#[derive(PartialEq, PartialOrd)]
pub enum InputMode {
    Normal,
    Insert,
}

#[derive(Debug, PartialEq)]
pub enum OutputType {
    Error,
    Success,
    Warn,
}

#[derive(Debug)]
pub struct Output {
    pub time: LocalTime,
    pub command: String,
    pub output: String,
    pub _type: OutputType,
}

impl Output {
    pub fn new(time: LocalTime, command: String, output: String, _type: OutputType) -> Self {
        Self {
            time,
            command,
            output,
            _type,
        }
    }
}

#[derive(Debug)]
pub struct StatefulList<T> {
    pub state: ListState,
    pub items: Vec<T>,
}

impl<T> StatefulList<T> {
    pub fn with_items(items: Vec<T>) -> StatefulList<T> {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    pub fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if self.items.is_empty() {
                    0
                } else if i + 1 >= self.items.len() {
                    i
                } else {
                    i + 1 as usize
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    pub fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(_) if self.items.is_empty() => 0,
            Some(i) => {
                if i == 0 {
                    0
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
    pub fn last(&mut self) {
        self.state.select(Some(self.items.len() - 1))
    }

    pub fn clear(&mut self) {
        self.items.clear()
    }
}

#[derive(Debug)]
pub struct AppState {
    pub(super) coords: CursorCoordsState,
    pub(super) history: StatefulList<Output>,
}
/// App holds the state of the application
pub struct App {
    pub(super) prompt: String,
    pub(super) command_prefix: String,
    /// Current value of the input box
    pub(super) input: String,
    /// Current input mode
    pub(super) input_mode: InputMode,
    pub(super) state: AppState,
}
impl App {
    pub fn new(prompt: &str, command_prefix: &str) -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            command_prefix: command_prefix.to_string(),
            prompt: prompt.to_string(),
            state: AppState {
                coords: CursorCoordsState::default(),
                history: StatefulList::with_items(vec![]),
            },
        }
    }
    pub fn push_msg<D: Display>(&mut self, command: D, msg: D, _type: OutputType) {
        let output = Output::new(
            Local::now(),
            format!("{}", command),
            format!("{}", msg),
            _type,
        );
        self.state.history.items.push(output);
    }
}
