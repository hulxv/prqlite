use chrono::{DateTime, Local};

use anyhow::Result;
use derivative::Derivative;
use tui::widgets::{List, ListState};
type LocalTime = DateTime<Local>;

pub enum InputMode {
    Normal,
    Insert,
}

#[derive(Debug)]
pub struct Output {
    pub time: LocalTime,
    pub command: String,
    pub output: Result<String>,
}

impl Output {
    pub fn new(time: LocalTime, command: String, output: Result<String>) -> Self {
        Self {
            time,
            command,
            output,
        }
    }
}

#[derive(Derivative, Debug, Clone, Copy)]
#[derivative(Default)]
pub struct CursorCoords {
    #[derivative(Default(value = "1"))]
    pub x: u16,
    #[derivative(Default(value = "1"))]
    pub y: u16,
    #[derivative(Default(value = "None"))]
    pub min_x: Option<u16>,
    #[derivative(Default(value = "None"))]
    pub min_y: Option<u16>,
    #[derivative(Default(value = "None"))]
    pub max_x: Option<u16>,
    #[derivative(Default(value = "None"))]
    pub max_y: Option<u16>,
}

impl CursorCoords {
    pub fn set(&mut self, x: u16, y: u16) -> &mut Self {
        self.x = x;
        self.y = y;
        self
    }
    pub fn get(&self) -> (u16, u16) {
        (self.x, self.y)
    }
    pub fn set_min(&mut self, (min_x, min_y): (u16, u16)) -> &mut Self {
        self.min_x = Some(min_x);
        self.min_y = Some(min_y);
        self
    }
    pub fn set_max(&mut self, (max_x, max_y): (u16, u16)) -> &mut Self {
        self.max_x = Some(max_x);
        self.max_y = Some(max_y);
        self
    }
    pub fn up(&mut self) -> &mut Self {
        if self.y > 0 {
            match self.min_y {
                Some(min) if min < self.y => self.y -= 1,
                None => self.y -= 1,
                _ => {}
            }
        }
        self
    }
    pub fn down(&mut self) -> &mut Self {
        if let Some(max_y) = self.max_y {
            if max_y > self.y {
                self.y += 1;
            }
        }
        match self.max_y {
            Some(max) if max > self.y => self.y += 1,
            None => self.y += 1,
            _ => {}
        }
        self
    }
    pub fn right(&mut self) -> &mut Self {
        match self.max_x {
            Some(max) if max > self.x => self.x += 1,
            None => self.x += 1,
            _ => {}
        }
        self
    }
    pub fn left(&mut self) -> &mut Self {
        if self.x > 0 {
            match self.min_x {
                Some(min) if min < self.x => self.x -= 1,
                None => self.x -= 1,
                _ => {}
            }
        }
        self
    }
}

impl From<(u16, u16)> for CursorCoords {
    fn from((x, y): (u16, u16)) -> Self {
        Self {
            x,
            y,
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct CursorCoordsState {
    pub normal: CursorCoords,
    pub insert: CursorCoords,
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
}

#[derive(Debug)]
pub struct State {
    pub(super) coords: CursorCoordsState,
    pub(super) history: StatefulList<Output>,
}
/// App holds the state of the application
pub struct App {
    pub(super) prompt: String,
    /// Current value of the input box
    pub(super) input: String,
    /// Current input mode
    pub(super) input_mode: InputMode,
    pub(super) state: State,
}
impl App {
    pub fn new(prompt: &str) -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            prompt: prompt.to_string(),
            state: State {
                coords: CursorCoordsState::default(),
                history: StatefulList::with_items(vec![]),
            },
        }
    }
}
