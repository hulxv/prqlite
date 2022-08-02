use anyhow::Result;
use chrono::{DateTime, Local};
use crossterm::{
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use derivative::Derivative;
use prql_compiler::compile;
use std::io::stdout;
use tokio::time::{sleep, Duration};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};
use unicode_width::UnicodeWidthStr;

type LocalTime = DateTime<Local>;

enum InputMode {
    Normal,
    Insert,
}

struct Output {
    pub time: LocalTime,
    pub command: String,
    pub output: String,
}

impl Output {
    pub fn new(time: LocalTime, command: String, output: String) -> Self {
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

#[derive(Default)]
pub struct CursorCoordsState {
    pub normal: CursorCoords,
    pub insert: CursorCoords,
}

/// App holds the state of the application
struct App {
    pub prompt: String,
    /// Current value of the input box
    input: String,
    /// Current input mode
    input_mode: InputMode,
    /// History of recorded messages
    outputs: Vec<Output>,
    coords_state: CursorCoordsState,
}
impl App {
    fn new(prompt: &str) -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            outputs: Vec::new(),
            prompt: prompt.to_string(),
            coords_state: CursorCoordsState::default(),
        }
    }
}

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
    loop {
        terminal.draw(|f| ui(f, app))?;

        if let Event::Key(key) = read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('i') => {
                        app.input_mode = InputMode::Insert;
                    }
                    KeyCode::Char('q') => {
                        return Ok(());
                    }

                    KeyCode::Up => {
                        app.coords_state.normal.up();
                    }
                    KeyCode::Down => {
                        app.coords_state.normal.down();
                    }

                    _ => {}
                },
                InputMode::Insert => match key.code {
                    KeyCode::Enter => {
                        if !app.input.is_empty() {
                            let input: String = app.input.drain(..).collect();
                            app.outputs.push(Output::new(
                                Local::now(),
                                input.clone(),
                                match compile(&input) {
                                    Err(e) => format!("Error: {e}"),
                                    Ok(sql) => sql
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
                                        .collect(),
                                },
                            ));
                        }
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
                    }
                    KeyCode::Right => {
                        app.coords_state.insert.right();
                    }
                    KeyCode::Left => {
                        app.coords_state.insert.left();
                    }
                    KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints(
            [
                Constraint::Percentage(5),
                Constraint::Percentage(75),
                Constraint::Percentage(15),
            ]
            .as_ref(),
        )
        .split(f.size());

    let (msg, style) = match app.input_mode {
        InputMode::Normal => (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("i", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to inserting commands and queries."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        ),
        InputMode::Insert => (
            vec![
                Span::raw("Press "),
                Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to reading output, "),
                Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to execute command or query"),
            ],
            Style::default(),
        ),
    };
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    // List of output messages
    let output: Vec<ListItem> = app
        .outputs
        .iter()
        .enumerate()
        .map(|(i, m)| {
            let mut content = vec![Spans::from(Span::styled(
                m.time.to_rfc2822(),
                Style::default().fg(Color::Yellow),
            ))];
            if i != 0 {
                content.insert(0, Spans::from(Span::raw("")))
            }
            vec![
                app.prompt.clone() + " " + &m.command.clone(),
                m.output.clone(),
            ]
            .iter()
            .enumerate()
            .for_each(|(i, t)| {
                t.chars()
                    .collect::<Vec<char>>()
                    .chunks(app.prompt.len() + chunks[1].width as usize + 5)
                    .map(|c| c.iter().collect::<String>())
                    .for_each(|chunk| {
                        chunk.split("\n").for_each(|line| {
                            content.push(Spans::from(Span::styled(
                                line.to_owned(),
                                Style::default().fg(if i == 0 {
                                    Color::Blue
                                } else {
                                    Color::LightCyan
                                }),
                            )));
                        });
                    });
            });

            ListItem::new(content)
        })
        .collect();

    // Output widget
    let output_list = List::new(output).block(
        Block::default()
            .borders(Borders::ALL)
            .title("Output")
            .style(match app.input_mode {
                InputMode::Insert => Style::default(),
                InputMode::Normal => Style::default().fg(Color::Yellow),
            }),
    );
    f.render_widget(output_list, chunks[1]);

    // Input widget

    let input = Paragraph::new(vec![Spans::from(Span::raw(app.input.as_str()))])
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Insert => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"))
        .wrap(Wrap { trim: true });
    f.render_widget(input, chunks[2]);

    match app.input_mode {
        InputMode::Normal => {
            let coords = &mut app.coords_state.normal;

            coords.set_max((chunks[1].width, chunks[1].height));
            coords.set_min((chunks[1].x, chunks[1].y));

            if coords.get() == CursorCoords::default().get() {
                coords.set(chunks[1].x + 1, chunks[1].y + 1);
            }

            f.set_cursor(coords.x, coords.y)
        }

        InputMode::Insert => {
            let coords = &mut app.coords_state.insert;
            let lines = app.input.width() as u16 / chunks[2].width;

            coords.set_max((chunks[2].width, chunks[2].height));
            coords.set_min((chunks[1].x, chunks[1].y));

            coords.set(
                chunks[2].x + ((app.input.width() as u16 + lines) % chunks[2].width) + 1,
                chunks[2].y + lines + 1,
            );
            f.set_cursor(coords.x, coords.y)
        }
    }
}
