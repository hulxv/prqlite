use crate::repl::tui::OutputType;

use super::{App, InputMode};
use crossterm::event::{Event, KeyEvent};
use tui::{
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{
        Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget as TuiStatefulWidget,
        Widget,
    },
};

pub struct History;
impl History {
    pub fn widget(app: &App, chunk: Rect) -> impl TuiStatefulWidget<State = ListState> {
        let output: Vec<ListItem> = app
            .state
            .history
            .items
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

                [app.prompt.clone(), m.command.clone()]
                    .join(" ")
                    .chars()
                    .collect::<Vec<char>>()
                    .chunks(app.prompt.len() + chunk.width as usize + 5)
                    .map(|c| c.iter().collect::<String>())
                    .for_each(|line| {
                        content.push(Spans::from(Span::styled(
                            line,
                            Style::default().fg(Color::Blue),
                        )))
                    });

                m.output
                    .chars()
                    .collect::<Vec<char>>()
                    .chunks(chunk.width as usize + 10)
                    .map(|c| c.iter().collect::<String>())
                    .for_each(|chunk| {
                        chunk.lines().for_each(|line| {
                            content.push(Spans::from(Span::styled(
                                line.to_owned(),
                                Style::default().fg(if m._type == OutputType::Success {
                                    Color::LightCyan
                                } else {
                                    Color::Red
                                }),
                            )));
                        })
                    });
                ListItem::new(content)
            })
            .collect();

        List::new(output)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Output")
                    .style(match app.input_mode {
                        InputMode::Insert => Style::default(),
                        InputMode::Normal => Style::default().fg(Color::Yellow),
                    }),
            )
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
    }
}

pub struct Input;
impl<'a> Input {
    pub fn widget(app: &'a mut App, chunk: Rect) -> impl Widget + 'a {
        let input = app
            .input
            .chars()
            .collect::<Vec<char>>()
            .chunks(chunk.width as usize - 3)
            .map(|c| Spans::from(Span::raw(c.iter().collect::<String>())))
            .collect::<Vec<Spans>>();

        Paragraph::new(input)
            .style(match app.input_mode {
                InputMode::Normal => Style::default(),
                InputMode::Insert => Style::default().fg(Color::Yellow),
            })
            .block(Block::default().borders(Borders::ALL).title("Input"))
    }
}

pub struct HelpMessage;
impl<'a> HelpMessage {
    pub fn widget(app: &'a App) -> impl Widget + 'a {
        let (msg, style) = match app.input_mode {
            InputMode::Normal => (
                vec![
                    Span::raw("Press "),
                    Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to exit, "),
                    Span::styled("i | Tab", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to inserting commands and queries, "),
                    Span::styled("c", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to clear history."),
                ],
                Style::default().add_modifier(Modifier::RAPID_BLINK),
            ),
            InputMode::Insert => (
                vec![
                    Span::raw("Press "),
                    Span::styled("Esc | Tab", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to reading output, "),
                    Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                    Span::raw(" to execute command or query"),
                ],
                Style::default(),
            ),
        };
        let mut text = Text::from(Spans::from(msg));
        text.patch_style(style);
        Paragraph::new(text)
    }
}
