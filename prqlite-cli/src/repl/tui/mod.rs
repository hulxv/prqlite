use anyhow::Result;
use chrono::{DateTime, Local};
use crossterm::{
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use derivative::Derivative;
use std::io::stdout;
use tokio::time::{sleep, Duration};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
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
struct CursorCoords {
    #[derivative(Default(value = "1"))]
    pub x: u16,
    #[derivative(Default(value = "1"))]
    pub y: u16,
}

impl CursorCoords {
    pub fn set(&mut self, x: u16, y: u16) -> &mut Self {
        self.x = x;
        self.y = y;
        self
    }
    pub fn set_x(&mut self, x: u16) -> &mut Self {
        self.x = x;
        self
    }
    pub fn set_y(&mut self, y: u16) -> &mut Self {
        self.y = y;
        self
    }
    pub fn up(&mut self) -> &mut Self {
        if self.y > 0 {
            self.y -= 1;
        }
        self
    }
    pub fn down(&mut self) -> &mut Self {
        self.y += 1;
        self
    }
    pub fn right(&mut self) -> &mut Self {
        self.x += 1;
        self
    }
    pub fn left(&mut self) -> &mut Self {
        if self.x > 0 {
            self.x -= 1;
        }
        self
    }
}

impl From<(u16, u16)> for CursorCoords {
    fn from((x, y): (u16, u16)) -> Self {
        Self { x, y }
    }
}

pub struct CursorCoordsStateMode {
    normal: CursorCoords,
    insert: CursorCoords,
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
}
impl App {
    fn new(prompt: &str) -> Self {
        Self {
            input: String::new(),
            input_mode: InputMode::Normal,
            outputs: Vec::new(),
            prompt: prompt.to_string(),
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
    let mut cursor_coords = CursorCoords::from((6, 0));
    loop {
        terminal.draw(|f| ui(f, app, cursor_coords))?;

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
                        cursor_coords.up();
                    }
                    KeyCode::Down => {
                        cursor_coords.down();
                    }

                    _ => {}
                },
                InputMode::Insert => match key.code {
                    KeyCode::Enter => {
                        if !app.input.is_empty() {
                            app.outputs.push(Output::new(
                                Local::now(),
                                app.input.drain(..).collect(),
                                String::from("Output"),
                            ));
                        }
                    }
                    KeyCode::Char(c) => {
                        app.input.push(c);
                    }
                    KeyCode::Backspace => {
                        app.input.pop();
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

fn ui<B: Backend>(f: &mut Frame<B>, app: &mut App, mut cursor_coords: CursorCoords) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Min(3),
                Constraint::Length(3),
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
            (app.prompt.clone() + " " + &m.command.clone())
                .chars()
                .collect::<Vec<char>>()
                .chunks(app.prompt.len() + chunks[1].width as usize + 5)
                .map(|c| c.iter().collect::<String>())
                .for_each(|line| {
                    content.push(Spans::from(Span::styled(
                        line,
                        Style::default().fg(Color::Blue),
                    )))
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
    let input = Paragraph::new(app.input.as_ref())
        .style(match app.input_mode {
            InputMode::Normal => Style::default(),
            InputMode::Insert => Style::default().fg(Color::Yellow),
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[2]);
    match app.input_mode {
        InputMode::Normal => {
            cursor_coords.set(chunks[1].x + 1, chunks[1].y + 1);
        }

        InputMode::Insert => {
            cursor_coords.set(chunks[2].x + app.input.width() as u16 + 1, chunks[2].y + 1);
        }
    }
    f.set_cursor(cursor_coords.x, cursor_coords.y)
}
